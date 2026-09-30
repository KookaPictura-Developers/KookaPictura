//! Stroke engine: dab coverage accumulation and per-pixel compositing.

use crate::art_history::{layer_local, ArtHistoryBrush, ArtHistoryOptions};
use crate::healing::RgbaImage;
use crate::mixer::{MixerBrush, MixerOptions};
use crate::replace::{ColorReplacer, ReplaceOptions};
use crate::spacing::DabPlacer;
use crate::stamp::StampSource;
use crate::{tip_coverage, PaintMode, Rgba, StrokeConfig, StrokeSample};
use pictura_core::{
    layer_pixel_locked, layer_transparency_locked, BitDepth, Document, Layer, PsdRect,
};

/// Deterministic stroke seed; Dissolve randomness must not vary between runs.
const STROKE_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug)]
pub enum PaintError {
    EmptyDocument,
    NoRasterLayer,
    /// The target layer's pixel or transparency lock refuses the stroke.
    Locked,
    /// The tool works on 8-bit documents only.
    UnsupportedDepth,
}

/// What a stroke's dabs do. `Paint` accumulates coverage and composites the
/// paint colour (Brush, Pencil); the others read the layer at every dab and
/// write into it directly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokeKind {
    Paint,
    /// Color Replacement: `cfg.color` replaces what matches the sample;
    /// `cfg.background` is the Background Swatch reference.
    Replace(ReplaceOptions),
    /// Mixer Brush, starting with `reservoir` on the brush.
    Mixer {
        options: MixerOptions,
        reservoir: Rgba,
    },
}

enum DabEngine {
    Replace(ColorReplacer),
    Mixer(MixerBrush),
    ArtHistory(ArtHistoryBrush),
}

/// A per-dab engine and the layer pixels it edits in place.
struct PerDab {
    engine: DabEngine,
    pixels: RgbaImage,
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
    dab_dirty: Option<PsdRect>,
    layer_path: Vec<usize>,
    rect: PsdRect,
    paint: Rgba,
    rng: u64,
    started: bool,
    painted: bool,
    per_dab: Option<PerDab>,
    source: Option<StampSource>,
}

impl Stroke {
    pub fn begin_at(doc: &Document, path: &str, cfg: StrokeConfig) -> Result<Stroke, PaintError> {
        Stroke::begin_kind(doc, path, cfg, StrokeKind::Paint)
    }

    /// Start a stroke whose dabs behave as `kind`.
    pub fn begin_kind(
        doc: &Document,
        path: &str,
        cfg: StrokeConfig,
        kind: StrokeKind,
    ) -> Result<Stroke, PaintError> {
        if doc.layers.is_empty() {
            return Err(PaintError::EmptyDocument);
        }
        let indices = parse_layer_path(path).ok_or(PaintError::NoRasterLayer)?;
        let cfg = cfg.sanitized();
        let target = layer_at(doc, &indices).ok_or(PaintError::NoRasterLayer)?;
        if target.is_group || target.adjustment.is_some() {
            return Err(PaintError::NoRasterLayer);
        }
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
        let per_dab =
            match kind {
                StrokeKind::Paint => None,
                StrokeKind::Replace(options) => Some(DabEngine::Replace(ColorReplacer::new(
                    w * h,
                    options,
                    cfg.background,
                ))),
                StrokeKind::Mixer { options, reservoir } => Some(DabEngine::Mixer(
                    MixerBrush::new(options, reservoir, layer_transparency_locked(target)),
                )),
            }
            .map(|engine| PerDab {
                engine,
                pixels: layer_rgba(target),
            });
        Ok(Stroke {
            base: doc.clone(),
            working: doc.clone(),
            cfg,
            placer: DabPlacer::new(cfg.spacing, cfg.diameter as f32),
            scratch,
            applied,
            dirty: None,
            dab_dirty: None,
            layer_path: indices,
            rect,
            paint: cfg.color,
            rng: STROKE_SEED,
            started: false,
            painted: false,
            per_dab,
            source: None,
        })
    }

    /// Start a Paint stroke whose colour at each pixel comes from `source`
    /// (Clone Stamp, Pattern Stamp, History Brush) instead of `cfg.color`.
    pub fn begin_source(
        doc: &Document,
        path: &str,
        cfg: StrokeConfig,
        source: StampSource,
    ) -> Result<Stroke, PaintError> {
        let mut stroke = Stroke::begin_kind(doc, path, cfg, StrokeKind::Paint)?;
        stroke.source = Some(source);
        Ok(stroke)
    }

    /// Start an Art History Brush stroke painting stylized strokes coloured
    /// from `source`, a document-space image of the source state's layer.
    /// 8-bit documents only.
    pub fn begin_art_history(
        doc: &Document,
        path: &str,
        cfg: StrokeConfig,
        source: RgbaImage,
        options: ArtHistoryOptions,
    ) -> Result<Stroke, PaintError> {
        if doc.depth != BitDepth::Eight {
            return Err(PaintError::UnsupportedDepth);
        }
        let mut stroke = Stroke::begin_kind(doc, path, cfg, StrokeKind::Paint)?;
        let target = layer_at(doc, &stroke.layer_path).ok_or(PaintError::NoRasterLayer)?;
        let brush = ArtHistoryBrush::new(
            options,
            layer_local(&source, stroke.rect),
            layer_transparency_locked(target),
        );
        stroke.per_dab = Some(PerDab {
            engine: DabEngine::ArtHistory(brush),
            pixels: layer_rgba(target),
        });
        Ok(stroke)
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
        if self.per_dab.is_some() {
            return self.apply_per_dab(&dabs);
        }

        let flow = cfg.flow as f32 / 100.0;
        let mut changed = false;
        for (dab_x, dab_y) in dabs {
            for _ in 0..cfg.count.max(1) {
                let (x, y, tip) = self.jittered(dab_x, dab_y);
                changed |= self.lay_dab(x, y, &tip, flow, w, h);
            }
        }
        changed
    }

    /// One dab of a step, scattered and jittered per the dynamics; the step's
    /// own position and tip when there are none.
    fn jittered(&mut self, x: f32, y: f32) -> (f32, f32, StrokeConfig) {
        let cfg = self.cfg;
        let mut tip = cfg;
        let (mut x, mut y) = (x, y);
        if cfg.scatter > 0 {
            let reach = cfg.diameter as f32 * cfg.scatter as f32 / 100.0;
            x += (self.next_f32() * 2.0 - 1.0) * reach;
            y += (self.next_f32() * 2.0 - 1.0) * reach;
        }
        if cfg.size_jitter > 0 {
            // Jitter only ever shrinks, as CS6's does.
            let shrink = (1.0 - cfg.size_jitter as f32 / 100.0 * self.next_f32()).max(0.05);
            tip.diameter = ((cfg.diameter as f32 * shrink).round() as u32).max(1);
        }
        if cfg.angle_jitter > 0 {
            let turn = (self.next_f32() * 2.0 - 1.0) * cfg.angle_jitter as f32;
            tip.angle_deg = (cfg.angle_deg as f32 + turn).round() as i32;
        }
        if cfg.roundness_jitter > 0 {
            let flatten = cfg.roundness_jitter as f32 * self.next_f32();
            tip.roundness = (cfg.roundness as f32 - flatten).clamp(5.0, 100.0) as u8;
        }
        (x, y, tip)
    }

    /// Accumulate one dab of `tip` at layer-local `(dab_x, dab_y)`; true when
    /// any pixel's coverage changed.
    fn lay_dab(
        &mut self,
        dab_x: f32,
        dab_y: f32,
        tip: &StrokeConfig,
        flow: f32,
        w: usize,
        h: usize,
    ) -> bool {
        let radius = tip.diameter as f32 * 0.5;
        let x0 = ((dab_x - radius).floor() as i32).max(0);
        let x1 = ((dab_x + radius).ceil() as i32).min(w as i32 - 1);
        let y0 = ((dab_y - radius).floor() as i32).max(0);
        let y1 = ((dab_y + radius).ceil() as i32).min(h as i32 - 1);
        let mut changed = false;
        for ly in y0..=y1 {
            for lx in x0..=x1 {
                let dx = (lx as f32 + 0.5) - dab_x;
                let dy = (ly as f32 + 0.5) - dab_y;
                let cov = tip_coverage(tip, dx, dy);
                if cov <= 0.0 {
                    continue;
                }
                let i = ly as usize * w + lx as usize;
                let acc = 1.0 - (1.0 - self.scratch[i] as f32 / 255.0) * (1.0 - flow * cov);
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
        changed
    }

    pub fn document(&self) -> &Document {
        &self.working
    }

    /// The paint on a Mixer Brush after the dabs so far; `None` for other kinds.
    pub fn mixer_reservoir(&self) -> Option<Rgba> {
        match &self.per_dab {
            Some(PerDab {
                engine: DabEngine::Mixer(mixer),
                ..
            }) => Some(mixer.reservoir()),
            _ => None,
        }
    }

    fn apply_per_dab(&mut self, dabs: &[(f32, f32)]) -> bool {
        let Some(per) = self.per_dab.as_mut() else {
            return false;
        };
        let cfg = self.cfg;
        let mut changed = None;
        for &(x, y) in dabs {
            let dirty = match &mut per.engine {
                DabEngine::Replace(replacer) => {
                    replacer.dab(&mut per.pixels, &cfg, x, y, cfg.color)
                }
                DabEngine::Mixer(mixer) => mixer.dab(&mut per.pixels, &cfg, x, y),
                DabEngine::ArtHistory(brush) => brush.dab(&mut per.pixels, &cfg, x, y),
            };
            if let Some(d) = dirty {
                changed = Some(changed.map_or(d, |c: PsdRect| union_rect(c, d)));
            }
        }
        let Some(rect) = changed else {
            return false;
        };
        let Some(layer) = layer_at_mut(&mut self.working, &self.layer_path) else {
            return false;
        };
        let w = per.pixels.width;
        for y in rect.top..rect.bottom {
            for x in rect.left..rect.right {
                let i = (y * w + x) as usize;
                let [r, g, b, a] = per.pixels.data[i];
                write_pixel(layer, i, PaintMode::Normal, (r, g, b, a));
            }
        }
        self.expand_dirty(rect.left, rect.top);
        self.expand_dirty(rect.right - 1, rect.bottom - 1);
        self.painted = true;
        true
    }

    /// The document-space rectangle the dabs placed since the last call changed,
    /// then clears it. The next call reports only newer dabs, so the per-dab
    /// refresh cost stays dab-sized instead of growing with the stroke.
    pub fn take_dirty(&mut self) -> Option<PsdRect> {
        self.dab_dirty.take().map(|d| self.dirty_doc(d))
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
        let grow = |d: Option<PsdRect>| {
            Some(match d {
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
            })
        };
        self.dirty = grow(self.dirty);
        self.dab_dirty = grow(self.dab_dirty);
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
        let transparency_locked = layer_transparency_locked(base_layer);
        let s = match &self.source {
            None => self.paint,
            Some(source) => {
                let w = self.rect.width() as usize;
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                match source.at(self.rect.left + x, self.rect.top + y) {
                    Some(color) => color,
                    None => return,
                }
            }
        };
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
        let Some((r, g, b, alpha)) = write else {
            return;
        };
        // A transparency lock preserves each pixel's alpha: fully transparent
        // pixels stay untouched, everything else keeps its pre-stroke alpha.
        // (Clear/auto-erase are refused for the whole stroke in `begin_at`.)
        if transparency_locked {
            if da == 0 {
                return;
            }
            if let Some(layer) = layer_at_mut(&mut self.working, &self.layer_path) {
                write_pixel(layer, i, mode, (r, g, b, da));
            }
        } else if let Some(layer) = layer_at_mut(&mut self.working, &self.layer_path) {
            write_pixel(layer, i, mode, (r, g, b, alpha));
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
    path: &str,
    cfg: &StrokeConfig,
    samples: &[StrokeSample],
) -> Option<PsdRect> {
    let mut stroke = Stroke::begin_at(doc, path, *cfg).ok()?;
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

pub(crate) fn union_rect(a: PsdRect, b: PsdRect) -> PsdRect {
    PsdRect {
        top: a.top.min(b.top),
        left: a.left.min(b.left),
        bottom: a.bottom.max(b.bottom),
        right: a.right.max(b.right),
    }
}

/// The layer's pixels as straight RGBA; a layer without alpha is opaque.
pub(crate) fn layer_rgba(layer: &Layer) -> RgbaImage {
    let (w, h) = (layer.rect.width().max(0), layer.rect.height().max(0));
    RgbaImage {
        width: w,
        height: h,
        data: (0..(w * h) as usize)
            .map(|i| {
                let (r, g, b, a) = read_pixel(layer, i);
                [r, g, b, a]
            })
            .collect(),
    }
}

fn to_u8(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

/// A layer without an alpha channel (a Background) is opaque.
fn read_pixel(layer: &Layer, i: usize) -> (u8, u8, u8, u8) {
    let g =
        |id, missing| channel_data(layer, id).map_or(missing, |d| d.get(i).copied().unwrap_or(0));
    (g(0, 0), g(1, 0), g(2, 0), g(-1, 255))
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

pub(crate) fn layer_at<'a>(doc: &'a Document, path: &[usize]) -> Option<&'a Layer> {
    let mut node = doc.layers.get(*path.first()?)?;
    for &idx in &path[1..] {
        node = node.children.get(idx)?;
    }
    Some(node)
}

pub(crate) fn layer_at_mut<'a>(doc: &'a mut Document, path: &[usize]) -> Option<&'a mut Layer> {
    let mut node = doc.layers.get_mut(*path.first()?)?;
    for &idx in &path[1..] {
        node = node.children.get_mut(idx)?;
    }
    Some(node)
}

/// Resolve a panel path (`"0"`, `"0/1"`) to layer indices; `None` when malformed.
pub(crate) fn parse_layer_path(path: &str) -> Option<Vec<usize>> {
    path.split('/')
        .map(|part| part.parse::<usize>().ok())
        .collect()
}

#[cfg(test)]
mod tests;
