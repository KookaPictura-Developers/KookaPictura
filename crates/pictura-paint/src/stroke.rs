//! Stroke engine: dab coverage accumulation and per-pixel compositing.

use crate::spacing::DabPlacer;
use crate::{tip_coverage, PaintMode, Rgba, StrokeConfig, StrokeSample};
use pictura_core::{layer_pixel_locked, layer_transparency_locked, Document, Layer, PsdRect};

/// Deterministic stroke seed; Dissolve randomness must not vary between runs.
const STROKE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug)]
pub enum PaintError {
    EmptyDocument,
    NoRasterLayer,
    /// The target layer's pixel or transparency lock refuses the stroke.
    Locked,
}

pub struct StrokeOutcome {
    pub document: Document,
    pub dirty: PsdRect,
}

/// A live stroke. `base` is the untouched document, `working` accumulates paint.
pub struct Stroke {
    base: Document,
    working: Document,
    cfg: StrokeConfig,
    placer: DabPlacer,
    scratch: Vec<u8>,
    applied: Vec<u8>,
    dirty: Option<PsdRect>,
    layer_path: Vec<usize>,
    rect: PsdRect,
    paint: Rgba,
    rng: u64,
    started: bool,
    painted: bool,
}

impl Stroke {
    pub fn begin(doc: &Document, cfg: StrokeConfig) -> Result<Stroke, PaintError> {
        if doc.layers.is_empty() {
            return Err(PaintError::EmptyDocument);
        }
        let path =
            find_topmost_raster(&doc.layers, &mut Vec::new()).ok_or(PaintError::NoRasterLayer)?;
        let cfg = cfg.sanitized();
        let target = layer_at(doc, &path).expect("path returned by find_topmost_raster");
        if layer_pixel_locked(target) {
            return Err(PaintError::Locked);
        }
        if layer_transparency_locked(target) && (cfg.mode == PaintMode::Clear || cfg.auto_erase) {
            return Err(PaintError::Locked);
        }
        let rect = target.rect;
        let w = rect.width().max(0) as usize;
        let h = rect.height().max(0) as usize;
        // ponytail: layer-sized coverage buffer; move to tiled/sparse coverage only if huge layers matter.
        let scratch = vec![0u8; w * h];
        let applied = vec![0u8; w * h];
        Ok(Stroke {
            base: doc.clone(),
            working: doc.clone(),
            cfg,
            placer: DabPlacer::new(cfg.spacing, cfg.diameter as f32),
            scratch,
            applied,
            dirty: None,
            layer_path: path,
            rect,
            paint: cfg.color,
            rng: STROKE_SEED,
            started: false,
            painted: false,
        })
    }

    /// Feed one sample. Returns `true` when any pixel's accumulated coverage changed.
    pub fn sample(&mut self, s: StrokeSample) -> bool {
        let cfg = self.cfg;
        let w = self.rect.width();
        let h = self.rect.height();
        if w <= 0 || h <= 0 {
            return false;
        }
        let (w, h) = (w as usize, h as usize);
        let sx = s.x - self.rect.left as f32;
        let sy = s.y - self.rect.top as f32;

        if !self.started {
            self.started = true;
            if cfg.auto_erase && self.pixel_rgb_matches(sx, sy, cfg.color) {
                self.paint = cfg.background;
            }
        }

        let mut dabs = Vec::new();
        self.placer.feed(sx, sy, &mut dabs);

        let radius = cfg.diameter as f32 * 0.5;
        let flow = cfg.flow as f32 / 100.0;
        let mut changed = false;

        for (dab_x, dab_y) in dabs {
            let x0 = ((dab_x - radius).floor() as i32).max(0);
            let x1 = ((dab_x + radius).ceil() as i32).min(w as i32 - 1);
            let y0 = ((dab_y - radius).floor() as i32).max(0);
            let y1 = ((dab_y + radius).ceil() as i32).min(h as i32 - 1);
            for ly in y0..=y1 {
                for lx in x0..=x1 {
                    let dx = (lx as f32 + 0.5) - dab_x;
                    let dy = (ly as f32 + 0.5) - dab_y;
                    let tip = tip_coverage(&cfg, dx, dy);
                    if tip <= 0.0 {
                        continue;
                    }
                    let i = ly as usize * w + lx as usize;
                    let acc = 1.0 - (1.0 - self.scratch[i] as f32 / 255.0) * (1.0 - flow * tip);
                    let v = (acc.clamp(0.0, 1.0) * 255.0).round() as u8;
                    self.scratch[i] = v;
                    if v == self.applied[i] {
                        continue;
                    }
                    self.applied[i] = v;
                    self.composite_pixel(i);
                    self.expand_dirty(lx, ly);
                    self.painted = true;
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn document(&self) -> &Document {
        &self.working
    }

    pub fn dirty(&self) -> Option<PsdRect> {
        self.dirty.map(|d| self.dirty_doc(d))
    }

    pub fn finish(self) -> Option<StrokeOutcome> {
        if !self.painted {
            return None;
        }
        let local = self.dirty.expect("painted implies dirty");
        let dirty = self.dirty_doc(local);
        Some(StrokeOutcome {
            document: self.working,
            dirty,
        })
    }

    fn dirty_doc(&self, local: PsdRect) -> PsdRect {
        PsdRect {
            top: self.rect.top + local.top,
            left: self.rect.left + local.left,
            bottom: self.rect.top + local.bottom,
            right: self.rect.left + local.right,
        }
    }

    fn expand_dirty(&mut self, lx: i32, ly: i32) {
        self.dirty = Some(match self.dirty {
            None => PsdRect {
                top: ly,
                left: lx,
                bottom: ly + 1,
                right: lx + 1,
            },
            Some(d) => PsdRect {
                top: d.top.min(ly),
                left: d.left.min(lx),
                bottom: d.bottom.max(ly + 1),
                right: d.right.max(lx + 1),
            },
        });
    }

    fn pixel_rgb_matches(&self, lx: f32, ly: f32, color: Rgba) -> bool {
        let (w, h) = (self.rect.width(), self.rect.height());
        let px = lx.floor() as i32;
        let py = ly.floor() as i32;
        if px < 0 || py < 0 || px >= w || py >= h {
            return false;
        }
        let i = py as usize * w as usize + px as usize;
        let Some(layer) = layer_at(&self.working, &self.layer_path) else {
            return false;
        };
        let (r, g, b, _) = read_pixel(layer, i);
        r == color.r && g == color.g && b == color.b
    }

    fn composite_pixel(&mut self, i: usize) {
        let cov = self.scratch[i] as f32 / 255.0;
        let a = (cov * self.cfg.opacity as f32 / 100.0).clamp(0.0, 1.0);
        let Some(base_layer) = layer_at(&self.base, &self.layer_path) else {
            return;
        };
        let (dr, dg, db, da) = read_pixel(base_layer, i);
        let s = self.paint;
        let mode = self.cfg.mode;
        let write = match mode {
            PaintMode::Normal => Some(normal_pixel(dr, dg, db, da, &s, a)),
            PaintMode::Dissolve => {
                if self.next_f32() < a {
                    Some((s.r, s.g, s.b, s.a))
                } else {
                    None
                }
            }
            PaintMode::Behind => {
                if da == 255 {
                    None
                } else {
                    Some(behind_pixel(dr, dg, db, da, &s, a))
                }
            }
            PaintMode::Clear => {
                let out_a = (da as f32 / 255.0) * (1.0 - a);
                Some((dr, dg, db, to_u8(out_a * 255.0)))
            }
        };
        if let Some(rgba) = write {
            if let Some(layer) = layer_at_mut(&mut self.working, &self.layer_path) {
                write_pixel(layer, i, mode, rgba);
            }
        }
    }

    fn next_f32(&mut self) -> f32 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 40) as f32 / 16_777_216.0
    }
}

/// Feed a whole sample list into a destructive stroke and commit on success.
pub fn paint_stroke(
    doc: &mut Document,
    cfg: &StrokeConfig,
    samples: &[StrokeSample],
) -> Option<PsdRect> {
    let mut stroke = Stroke::begin(doc, *cfg).ok()?;
    for &s in samples {
        stroke.sample(s);
    }
    let outcome = stroke.finish()?;
    *doc = outcome.document;
    Some(outcome.dirty)
}

fn normal_pixel(dr: u8, dg: u8, db: u8, da: u8, s: &Rgba, a: f32) -> (u8, u8, u8, u8) {
    let sa = s.a as f32 / 255.0;
    let da_f = da as f32 / 255.0;
    let src_a = sa * a;
    let out_a = src_a + da_f * (1.0 - src_a);
    if out_a <= 0.0 {
        return (0, 0, 0, 0);
    }
    let mix = |src: u8, dst: u8| {
        ((src as f32 * src_a + dst as f32 * da_f * (1.0 - src_a)) / out_a).round() as u8
    };
    (
        mix(s.r, dr),
        mix(s.g, dg),
        mix(s.b, db),
        to_u8(out_a * 255.0),
    )
}

fn behind_pixel(dr: u8, dg: u8, db: u8, da: u8, s: &Rgba, a: f32) -> (u8, u8, u8, u8) {
    let sa = s.a as f32 / 255.0;
    let da_f = da as f32 / 255.0;
    let src_a = sa * a;
    let out_a = da_f + src_a * (1.0 - da_f);
    if out_a <= 0.0 {
        return (0, 0, 0, 0);
    }
    let mix = |src: u8, dst: u8| {
        ((dst as f32 * da_f + src as f32 * src_a * (1.0 - da_f)) / out_a).round() as u8
    };
    (
        mix(s.r, dr),
        mix(s.g, dg),
        mix(s.b, db),
        to_u8(out_a * 255.0),
    )
}

fn to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

fn read_pixel(layer: &Layer, i: usize) -> (u8, u8, u8, u8) {
    let g = |id| {
        channel_data(layer, id)
            .and_then(|d| d.get(i))
            .copied()
            .unwrap_or(0)
    };
    (g(0), g(1), g(2), g(-1))
}

fn write_pixel(layer: &mut Layer, i: usize, mode: PaintMode, rgba: (u8, u8, u8, u8)) {
    let (r, g, b, a) = rgba;
    if !matches!(mode, PaintMode::Clear) {
        if let Some(d) = channel_data_mut(layer, 0) {
            d[i] = r;
        }
        if let Some(d) = channel_data_mut(layer, 1) {
            d[i] = g;
        }
        if let Some(d) = channel_data_mut(layer, 2) {
            d[i] = b;
        }
    }
    if let Some(d) = channel_data_mut(layer, -1) {
        d[i] = a;
    }
}

fn channel_data(layer: &Layer, id: i16) -> Option<&[u8]> {
    layer
        .channels
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.data.as_slice())
}

fn channel_data_mut(layer: &mut Layer, id: i16) -> Option<&mut [u8]> {
    layer
        .channels
        .iter_mut()
        .find(|c| c.id == id)
        .map(|c| c.data.as_mut_slice())
}

fn layer_at<'a>(doc: &'a Document, path: &[usize]) -> Option<&'a Layer> {
    let mut node = doc.layers.get(*path.first()?)?;
    for &idx in &path[1..] {
        node = node.children.get(idx)?;
    }
    Some(node)
}

fn layer_at_mut<'a>(doc: &'a mut Document, path: &[usize]) -> Option<&'a mut Layer> {
    let mut node = doc.layers.get_mut(*path.first()?)?;
    for &idx in &path[1..] {
        node = node.children.get_mut(idx)?;
    }
    Some(node)
}

/// Topmost (last) non-group, non-adjustment layer, as an index path.
fn find_topmost_raster(layers: &[Layer], prefix: &mut Vec<usize>) -> Option<Vec<usize>> {
    let mut found = None;
    for (i, layer) in layers.iter().enumerate() {
        prefix.push(i);
        if layer.is_group {
            if let Some(p) = find_topmost_raster(&layer.children, prefix) {
                found = Some(p);
            }
        } else if layer.adjustment.is_none() {
            found = Some(prefix.clone());
        }
        prefix.pop();
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spacing::SpacingMode;
    use pictura_core::{BitDepth, BlendMode, Channel, ColorLabel, ColorMode, LockFlags};

    const BASE: (u8, u8, u8, u8) = (100, 120, 140, 255);

    fn layer_doc(w: u32, h: u32, rgba: (u8, u8, u8, u8)) -> Document {
        let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        let n = (w * h) as usize;
        let (r, g, b, a) = rgba;
        doc.layers.push(Layer {
            name: "px".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: h as i32,
                right: w as i32,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![r; n],
                },
                Channel {
                    id: 1,
                    data: vec![g; n],
                },
                Channel {
                    id: 2,
                    data: vec![b; n],
                },
                Channel {
                    id: -1,
                    data: vec![a; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        });
        doc
    }

    fn transparent_doc(w: u32, h: u32) -> Document {
        layer_doc(w, h, (0, 0, 0, 0))
    }

    fn chan(doc: &Document, id: i16, i: usize) -> u8 {
        channel_data(&doc.layers[0], id).expect("channel")[i]
    }

    fn max_alpha(doc: &Document) -> u8 {
        *channel_data(&doc.layers[0], -1)
            .expect("alpha channel")
            .iter()
            .max()
            .unwrap()
    }

    fn sample(x: f32, y: f32) -> StrokeSample {
        StrokeSample {
            x,
            y,
            pressure: 1.0,
        }
    }

    fn red() -> Rgba {
        Rgba {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    #[test]
    fn horizontal_stroke_changes_pixels_and_reports_dirty() {
        let mut doc = layer_doc(64, 32, BASE);
        let cfg = StrokeConfig {
            color: red(),
            diameter: 8,
            hardness: 100,
            spacing: SpacingMode::Fixed(25),
            ..StrokeConfig::default()
        };
        let dirty = paint_stroke(&mut doc, &cfg, &[sample(2.0, 16.0), sample(40.0, 16.0)])
            .expect("painted");
        assert!(dirty.width() > 0 && dirty.height() > 0);
        assert!(dirty.top >= 0 && dirty.left >= 0 && dirty.bottom <= 32 && dirty.right <= 64);
        assert!(channel_data(&doc.layers[0], 0)
            .unwrap()
            .iter()
            .any(|&v| v != BASE.0));
        assert!(chan(&doc, 0, 16 * 64 + 20) > BASE.0);
    }

    #[test]
    fn pixel_locked_layer_refuses_stroke_and_writes_nothing() {
        let mut doc = layer_doc(16, 16, BASE);
        doc.layers[0].lock = LockFlags::default().with(LockFlags::PIXELS, true);
        let before = doc.clone();
        let cfg = StrokeConfig {
            color: red(),
            diameter: 6,
            hardness: 100,
            spacing: SpacingMode::Fixed(25),
            ..StrokeConfig::default()
        };
        let begun = Stroke::begin(&doc, cfg);
        assert!(matches!(begun, Err(PaintError::Locked)));
        assert!(paint_stroke(&mut doc, &cfg, &[sample(4.0, 4.0), sample(12.0, 4.0)]).is_none());
        assert_eq!(doc, before, "a refused stroke writes no pixels");
    }

    #[test]
    fn transparency_lock_refuses_clear_but_allows_normal() {
        let mut doc = layer_doc(16, 16, BASE);
        doc.layers[0].lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
        let clear = StrokeConfig {
            color: red(),
            diameter: 6,
            hardness: 100,
            spacing: SpacingMode::Fixed(25),
            mode: PaintMode::Clear,
            ..StrokeConfig::default()
        };
        let begun = Stroke::begin(&doc, clear);
        assert!(matches!(begun, Err(PaintError::Locked)));
        let normal = StrokeConfig {
            mode: PaintMode::Normal,
            ..clear
        };
        assert!(Stroke::begin(&doc, normal).is_ok());
    }

    #[test]
    fn dense_spacing_is_continuous_where_sparse_leaves_a_gap() {
        let samples = [sample(10.5, 16.0), sample(30.5, 16.0)];
        let base = |spacing| StrokeConfig {
            color: red(),
            diameter: 20,
            hardness: 100,
            spacing,
            ..StrokeConfig::default()
        };

        let mut dense = transparent_doc(64, 32);
        paint_stroke(&mut dense, &base(SpacingMode::Fixed(10)), &samples).expect("dense");
        for lx in 11..=30 {
            assert!(
                chan(&dense, -1, 16 * 64 + lx) > 0,
                "dense left a gap at {lx}"
            );
        }

        let mut sparse = transparent_doc(64, 32);
        paint_stroke(&mut sparse, &base(SpacingMode::Fixed(100)), &samples).expect("sparse");
        assert_eq!(
            chan(&sparse, -1, 16 * 64 + 20),
            0,
            "expected a gap at the midpoint"
        );
        assert!(chan(&sparse, -1, 16 * 64 + 19) > 0);
        assert!(chan(&sparse, -1, 16 * 64 + 21) > 0);
    }

    #[test]
    fn opacity_caps_a_single_stroke() {
        let cfg = StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 100,
            spacing: SpacingMode::Fixed(10),
            opacity: 33,
            flow: 100,
            ..StrokeConfig::default()
        };
        let path = [sample(5.0, 16.0), sample(55.0, 16.0), sample(5.0, 16.0)];

        let mut doc = transparent_doc(64, 32);
        paint_stroke(&mut doc, &cfg, &path).expect("first");
        let first = max_alpha(&doc);
        assert!(
            first as f32 / 255.0 <= 0.33 + 1.0 / 255.0,
            "single stroke exceeded the opacity cap: {first}"
        );

        paint_stroke(&mut doc, &cfg, &path).expect("second");
        let second = max_alpha(&doc);
        assert!(
            second > first,
            "second stroke must raise coverage: {first} -> {second}"
        );
    }

    #[test]
    fn higher_flow_reaches_more_coverage() {
        let base = |flow| StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 100,
            spacing: SpacingMode::Fixed(10),
            opacity: 100,
            flow,
            ..StrokeConfig::default()
        };
        let path = [sample(5.0, 16.0), sample(55.0, 16.0)];

        let mut lo = transparent_doc(64, 32);
        paint_stroke(&mut lo, &base(20), &path).expect("lo");
        let mut hi = transparent_doc(64, 32);
        paint_stroke(&mut hi, &base(80), &path).expect("hi");

        let sum = |d: &Document| -> u32 {
            channel_data(&d.layers[0], -1)
                .unwrap()
                .iter()
                .map(|&v| v as u32)
                .sum()
        };
        assert!(
            sum(&lo) < sum(&hi),
            "flow 20 {} vs flow 80 {}",
            sum(&lo),
            sum(&hi)
        );
    }

    #[test]
    fn pencil_is_binary_brush_has_soft_edges() {
        let path = [sample(2.0, 16.0), sample(60.0, 16.0)];
        let cfg = |aliased| StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 50,
            spacing: SpacingMode::Fixed(25),
            opacity: 100,
            flow: 100,
            aliased,
            ..StrokeConfig::default()
        };

        let mut pencil = transparent_doc(64, 32);
        paint_stroke(&mut pencil, &cfg(true), &path).expect("pencil");
        let alpha = channel_data(&pencil.layers[0], -1).unwrap();
        assert!(alpha.contains(&255), "pencil painted nothing");
        assert!(
            alpha.iter().all(|&a| a == 0 || a == 255),
            "pencil produced a partial pixel"
        );

        let mut brush = transparent_doc(64, 32);
        paint_stroke(&mut brush, &cfg(false), &path).expect("brush");
        let alpha = channel_data(&brush.layers[0], -1).unwrap();
        assert!(
            alpha.iter().any(|&a| a > 0 && a < 255),
            "brush produced no soft edge"
        );
    }

    #[test]
    fn clear_drives_opaque_alpha_to_zero() {
        let mut doc = layer_doc(64, 32, BASE);
        let cfg = StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 100,
            spacing: SpacingMode::Fixed(10),
            opacity: 100,
            flow: 100,
            mode: PaintMode::Clear,
            ..StrokeConfig::default()
        };
        paint_stroke(&mut doc, &cfg, &[sample(5.0, 16.0), sample(55.0, 16.0)]).expect("clear");
        assert_eq!(chan(&doc, -1, 16 * 64 + 30), 0);
    }

    #[test]
    fn behind_leaves_opaque_rgb_unchanged() {
        let mut doc = layer_doc(64, 32, BASE);
        let cfg = StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 100,
            spacing: SpacingMode::Fixed(10),
            opacity: 100,
            flow: 100,
            mode: PaintMode::Behind,
            ..StrokeConfig::default()
        };
        paint_stroke(&mut doc, &cfg, &[sample(5.0, 16.0), sample(55.0, 16.0)]).expect("behind");
        let i = 16 * 64 + 30;
        assert_eq!(
            (chan(&doc, 0, i), chan(&doc, 1, i), chan(&doc, 2, i)),
            (BASE.0, BASE.1, BASE.2)
        );
    }

    #[test]
    fn dissolve_is_deterministic() {
        let cfg = StrokeConfig {
            color: red(),
            diameter: 12,
            hardness: 100,
            spacing: SpacingMode::Fixed(10),
            opacity: 100,
            flow: 100,
            mode: PaintMode::Dissolve,
            ..StrokeConfig::default()
        };
        let path = [sample(5.0, 16.0), sample(55.0, 16.0)];

        let mut a = transparent_doc(64, 32);
        paint_stroke(&mut a, &cfg, &path).expect("a");
        let mut b = transparent_doc(64, 32);
        paint_stroke(&mut b, &cfg, &path).expect("b");

        for id in [0, 1, 2, -1] {
            assert_eq!(
                channel_data(&a.layers[0], id).unwrap(),
                channel_data(&b.layers[0], id).unwrap(),
                "channel {id} differed between runs"
            );
        }
    }

    #[test]
    fn empty_samples_leave_document_unchanged() {
        let mut doc = layer_doc(16, 16, BASE);
        let before = doc.clone();
        let cfg = StrokeConfig {
            color: red(),
            diameter: 8,
            ..StrokeConfig::default()
        };
        assert!(paint_stroke(&mut doc, &cfg, &[]).is_none());
        assert_eq!(doc, before);
    }

    #[test]
    fn group_only_document_has_no_raster_layer() {
        let mut doc = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
        doc.layers.push(Layer {
            name: "group".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 16,
                right: 16,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: Vec::new(),
            children: Vec::new(),
            is_group: true,
            background: false,
            ..Default::default()
        });
        assert!(matches!(
            Stroke::begin(&doc, StrokeConfig::default()),
            Err(PaintError::NoRasterLayer)
        ));

        let empty = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
        assert!(matches!(
            Stroke::begin(&empty, StrokeConfig::default()),
            Err(PaintError::EmptyDocument)
        ));
    }

    #[test]
    fn default_brush_paints_center_with_configured_color() {
        let mut doc = transparent_doc(64, 32);
        let cfg = StrokeConfig {
            color: red(),
            diameter: 20,
            hardness: 50,
            spacing: SpacingMode::Fixed(25),
            opacity: 100,
            flow: 100,
            ..StrokeConfig::default()
        };
        paint_stroke(&mut doc, &cfg, &[sample(10.0, 16.0), sample(50.0, 16.0)]).expect("paint");
        let i = 16 * 64 + 30;
        assert_eq!(
            (
                chan(&doc, 0, i),
                chan(&doc, 1, i),
                chan(&doc, 2, i),
                chan(&doc, -1, i)
            ),
            (255, 0, 0, 255)
        );
    }
}
