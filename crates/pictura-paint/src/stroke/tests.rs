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

/// Paint into the single layer at path `"0"`.
fn paint(doc: &mut Document, cfg: &StrokeConfig, samples: &[StrokeSample]) -> Option<PsdRect> {
    paint_stroke(doc, "0", cfg, samples)
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
fn take_dirty_reports_each_dab_not_the_stroke_union() {
    let doc = layer_doc(128, 32, BASE);
    let cfg = StrokeConfig {
        color: red(),
        diameter: 8,
        hardness: 100,
        spacing: SpacingMode::Fixed(150),
        opacity: 100,
        flow: 100,
        ..StrokeConfig::default()
    };
    // Diameter 8 at 150% spacing steps 12 px, so each sample places one dab.
    let mut stroke = Stroke::begin_at(&doc, "0", cfg).expect("begin");
    assert!(stroke.sample(sample(50.0, 16.0)));
    let first = stroke.take_dirty().expect("first dab");
    assert!(stroke.sample(sample(62.0, 16.0)));
    let second = stroke.take_dirty().expect("second dab");
    assert!(
        second.left >= first.right,
        "second rect {second:?} overlaps the first {first:?}"
    );
    assert!(
        second.width() <= 12,
        "second rect grew to {}",
        second.width()
    );
    assert!(stroke.take_dirty().is_none(), "take_dirty must clear");
    // The committed union still covers the whole stroke.
    let outcome = stroke.finish().expect("painted");
    assert!(outcome.dirty.left <= first.left);
    assert!(outcome.dirty.right >= second.right);
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
    let dirty = paint(&mut doc, &cfg, &[sample(2.0, 16.0), sample(40.0, 16.0)]).expect("painted");
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
    let begun = Stroke::begin_at(&doc, "0", cfg);
    assert!(matches!(begun, Err(PaintError::Locked)));
    assert!(paint(&mut doc, &cfg, &[sample(4.0, 4.0), sample(12.0, 4.0)]).is_none());
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
    let begun = Stroke::begin_at(&doc, "0", clear);
    assert!(matches!(begun, Err(PaintError::Locked)));
    let normal = StrokeConfig {
        mode: PaintMode::Normal,
        ..clear
    };
    assert!(Stroke::begin_at(&doc, "0", normal).is_ok());
}

#[test]
fn transparency_lock_preserves_alpha_per_pixel() {
    let mut doc = layer_doc(2, 1, (10, 20, 30, 180));
    channel_data_mut(&mut doc.layers[0], -1).expect("alpha")[1] = 0;
    doc.layers[0].lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
    let cfg = StrokeConfig {
        color: red(),
        diameter: 3,
        hardness: 100,
        spacing: SpacingMode::Fixed(25),
        opacity: 100,
        flow: 100,
        ..StrokeConfig::default()
    };
    paint(&mut doc, &cfg, &[sample(0.5, 0.5)]).expect("painted");
    assert_eq!(chan(&doc, -1, 0), 180, "semi-transparent pixel keeps alpha");
    assert_ne!(
        (chan(&doc, 0, 0), chan(&doc, 1, 0), chan(&doc, 2, 0)),
        (10, 20, 30),
        "semi-transparent pixel keeps its colour edit"
    );
    assert_eq!(
        (
            chan(&doc, 0, 1),
            chan(&doc, 1, 1),
            chan(&doc, 2, 1),
            chan(&doc, -1, 1)
        ),
        (10, 20, 30, 0),
        "fully transparent pixel is untouched"
    );
}

#[test]
fn a_locked_background_without_alpha_takes_paint_as_opaque() {
    let mut doc = layer_doc(32, 32, (255, 255, 255, 255));
    doc.layers[0].channels.retain(|c| c.id != -1);
    doc.layers[0].lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
    let cfg = StrokeConfig {
        color: red(),
        diameter: 8,
        hardness: 0,
        ..StrokeConfig::default()
    };
    paint(&mut doc, &cfg, &[sample(16.0, 16.0)]).expect("painted");
    assert!(
        chan(&doc, 1, 16 * 32 + 16) < 64,
        "the centre was not painted"
    );
    // A soft edge blends with the white beneath rather than replacing it.
    let edge = 16 * 32 + 19;
    assert!(chan(&doc, 1, edge) > 0 && chan(&doc, 1, edge) < 255);
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
    paint(&mut dense, &base(SpacingMode::Fixed(10)), &samples).expect("dense");
    for lx in 11..=30 {
        assert!(
            chan(&dense, -1, 16 * 64 + lx) > 0,
            "dense left a gap at {lx}"
        );
    }

    let mut sparse = transparent_doc(64, 32);
    paint(&mut sparse, &base(SpacingMode::Fixed(100)), &samples).expect("sparse");
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
    paint(&mut doc, &cfg, &path).expect("first");
    let first = max_alpha(&doc);
    assert!(
        first as f32 / 255.0 <= 0.33 + 1.0 / 255.0,
        "single stroke exceeded the opacity cap: {first}"
    );

    paint(&mut doc, &cfg, &path).expect("second");
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
    paint(&mut lo, &base(20), &path).expect("lo");
    let mut hi = transparent_doc(64, 32);
    paint(&mut hi, &base(80), &path).expect("hi");

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
    paint(&mut pencil, &cfg(true), &path).expect("pencil");
    let alpha = channel_data(&pencil.layers[0], -1).unwrap();
    assert!(alpha.contains(&255), "pencil painted nothing");
    assert!(
        alpha.iter().all(|&a| a == 0 || a == 255),
        "pencil produced a partial pixel"
    );

    let mut brush = transparent_doc(64, 32);
    paint(&mut brush, &cfg(false), &path).expect("brush");
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
    paint(&mut doc, &cfg, &[sample(5.0, 16.0), sample(55.0, 16.0)]).expect("clear");
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
    paint(&mut doc, &cfg, &[sample(5.0, 16.0), sample(55.0, 16.0)]).expect("behind");
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
    paint(&mut a, &cfg, &path).expect("a");
    let mut b = transparent_doc(64, 32);
    paint(&mut b, &cfg, &path).expect("b");

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
    assert!(paint(&mut doc, &cfg, &[]).is_none());
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
        Stroke::begin_at(&doc, "0", StrokeConfig::default()),
        Err(PaintError::NoRasterLayer)
    ));

    let empty = Document::new(16, 16, ColorMode::Rgb, BitDepth::Eight);
    assert!(matches!(
        Stroke::begin_at(&empty, "0", StrokeConfig::default()),
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
    paint(&mut doc, &cfg, &[sample(10.0, 16.0), sample(50.0, 16.0)]).expect("paint");
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

fn run(doc: &Document, cfg: StrokeConfig, kind: StrokeKind, xs: &[f32]) -> Stroke {
    let mut stroke = Stroke::begin_kind(doc, "0", cfg, kind).expect("begin");
    for &x in xs {
        stroke.sample(sample(x, 16.0));
    }
    stroke
}

#[test]
fn a_replace_stroke_recolours_the_layer_live_and_commits_once() {
    let doc = layer_doc(32, 32, (120, 120, 120, 255));
    let cfg = StrokeConfig {
        color: Rgba {
            r: 220,
            g: 30,
            b: 30,
            a: 255,
        },
        diameter: 10,
        ..StrokeConfig::default()
    };
    let kind = StrokeKind::Replace(ReplaceOptions::default());
    let mut stroke = run(&doc, cfg, kind, &[8.0, 24.0]);
    let i = 16 * 32 + 16;
    let live = (chan(stroke.document(), 0, i), chan(stroke.document(), 1, i));
    assert!(
        live.0 > live.1 + 20,
        "the grey was not recoloured: {live:?}"
    );
    assert!(stroke.take_dirty().is_some());
    assert_eq!(stroke.mixer_reservoir(), None);
    let outcome = stroke.finish().expect("painted");
    assert_eq!(chan(&outcome.document, 0, i), live.0);
    assert_eq!(chan(&outcome.document, -1, i), 255);
    assert_eq!(chan(&doc, 0, i), 120, "the base document changed");
}

#[test]
fn a_mixer_stroke_smears_and_reports_its_reservoir() {
    let mut doc = layer_doc(64, 32, (255, 255, 255, 255));
    for y in 0..32 {
        for x in 0..20 {
            let i = y * 64 + x;
            for id in 0..3 {
                channel_data_mut(&mut doc.layers[0], id).unwrap()[i] = 0;
            }
        }
    }
    let cfg = StrokeConfig {
        diameter: 10,
        spacing: SpacingMode::Fixed(25),
        ..StrokeConfig::default()
    };
    let kind = StrokeKind::Mixer {
        options: MixerOptions {
            wet: 1.0,
            load: 1.0,
            mix: 1.0,
            flow: 1.0,
        },
        reservoir: Rgba {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
    };
    let stroke = run(&doc, cfg, kind, &[10.0, 40.0]);
    assert!(
        chan(stroke.document(), 0, 16 * 64 + 26) < 250,
        "nothing was dragged"
    );
    let carried = stroke.mixer_reservoir().expect("a mixer reservoir");
    assert!(carried.r < 255, "the reservoir picked nothing up");
}

/// The red pixels of a one-dab stroke at the centre of a 64² transparent layer.
fn inked(cfg: &StrokeConfig) -> Vec<usize> {
    let mut doc = transparent_doc(64, 64);
    paint(&mut doc, cfg, &[sample(32.0, 32.0)]);
    (0..64 * 64).filter(|&i| chan(&doc, -1, i) > 0).collect()
}

fn extent(pixels: &[usize]) -> usize {
    let xs = pixels.iter().map(|i| i % 64);
    xs.clone().max().unwrap_or(0) - xs.min().unwrap_or(0)
}

fn dab(diameter: u32) -> StrokeConfig {
    StrokeConfig {
        color: red(),
        diameter,
        ..StrokeConfig::default()
    }
}

#[test]
fn scatter_and_count_spread_a_cluster_off_the_path() {
    let plain = inked(&dab(6));
    let cluster = inked(&StrokeConfig {
        scatter: 300,
        count: 12,
        ..dab(6)
    });
    assert!(
        extent(&cluster) > extent(&plain) * 2,
        "scatter did not spread the dabs"
    );
    assert!(
        cluster.len() > plain.len() * 3,
        "a higher count laid no extra ink"
    );
}

#[test]
fn size_jitter_only_ever_shrinks() {
    let jittered = inked(&StrokeConfig {
        size_jitter: 90,
        ..dab(20)
    });
    assert!(extent(&jittered) <= extent(&inked(&dab(20))));
}

#[test]
fn dynamics_replay_exactly() {
    let cfg = StrokeConfig {
        scatter: 100,
        count: 6,
        size_jitter: 60,
        angle_jitter: 90,
        roundness: 40,
        roundness_jitter: 30,
        ..dab(10)
    };
    assert_eq!(inked(&cfg), inked(&cfg));
}
