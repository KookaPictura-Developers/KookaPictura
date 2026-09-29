//! The paint tool bridges beyond the Brush: Color Replacement, Mixer Brush,
//! and the stamps (Clone Stamp, Pattern Stamp, History Brush). Free functions
//! over a [`PictureView`] (their own bridge, so the `PictureView` declaration
//! list does not grow). Each only *begins* a stroke — of its [`StrokeKind`], or
//! one whose colour comes from a [`StampSource`]; the live stroke then runs
//! through the Brush's `paint_dab` / `end_paint` / `cancel_paint`, so the
//! preview and the one history state per stroke (`"Color Replacement Tool"`,
//! `"Mixer Brush Tool"`, `"Clone Stamp"`, `"Pattern Stamp"`, `"History
//! Brush"`) are shared.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::{active_layer_visible, active_pixel_layer, paint_mode_from, rgba_from_argb};
use super::qobject::PictureView;
use super::PictureViewRust;
use crate::history::BrushSource;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::Document;
use pictura_paint::mixer::MixerOptions;
use pictura_paint::pattern::{self, PATTERN_NAMES};
use pictura_paint::replace::{Limits, ReplaceMode, ReplaceOptions, Sampling};
use pictura_paint::stamp::{
    layer_surface, sample_scope, surface_from_composite, tiled, CloneSampling, StampSource,
};
use pictura_paint::{Stroke, StrokeConfig, StrokeKind};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// Begin a Color Replacement stroke on the active pixel layer, painting
        /// `foreground` (`0xAARRGGBB`); `background` is the Background Swatch
        /// reference. `mode` 0 Hue … 3 Luminosity, `sampling` 0 Continuous /
        /// 1 Once / 2 Background Swatch, `limits` 0 Discontiguous /
        /// 1 Contiguous / 2 Find Edges, `tolerance` 0–100 %. False without a
        /// visible lone pixel layer, when its pixels are locked, or mid-stroke.
        fn begin_color_replacement(
            view: Pin<&mut PictureView>,
            foreground: u32,
            background: u32,
            diameter: i32,
            hardness: i32,
            mode: i32,
            sampling: i32,
            limits: i32,
            tolerance: i32,
            antialias: bool,
        ) -> bool;

        /// Begin a Mixer Brush stroke with `reservoir` (`0xAARRGGBB`; alpha 0
        /// is a clean brush) on the brush. `wet`, `load`, `mix`, and `flow` are
        /// 0–100 %. False as for `begin_color_replacement`.
        fn begin_mixer_brush(
            view: Pin<&mut PictureView>,
            reservoir: u32,
            diameter: i32,
            hardness: i32,
            wet: i32,
            load: i32,
            mix: i32,
            flow: i32,
        ) -> bool;

        /// The paint on the brush of the active Mixer Brush stroke
        /// (`0xAARRGGBB`), or 0 without one. Read before `end_paint` to carry
        /// the brush into the next stroke.
        fn mixer_reservoir(view: &PictureView) -> u32;

        /// Begin a Clone Stamp stroke painting what lies at `(offset_x,
        /// offset_y)` from each pixel, snapshotted now. `sampling` 0 Current
        /// Layer / 1 Current And Below / 2 All Layers; `ignore_adjustments`
        /// applies to All Layers. `mode` is a Brush mode (`"normal"`, …). False
        /// without a visible lone pixel layer, when its pixels are locked, for
        /// a zero offset, or mid-stroke.
        fn begin_clone_stamp(
            view: Pin<&mut PictureView>,
            diameter: i32,
            hardness: i32,
            opacity: i32,
            flow: i32,
            mode: &QString,
            offset_x: i32,
            offset_y: i32,
            sampling: i32,
            ignore_adjustments: bool,
        ) -> bool;

        /// Begin a Pattern Stamp stroke painting built-in pattern `pattern`
        /// tiled from document point `(origin_x, origin_y)`. False as for
        /// `begin_clone_stamp`, or for an unknown pattern.
        fn begin_pattern_stamp(
            view: Pin<&mut PictureView>,
            diameter: i32,
            hardness: i32,
            opacity: i32,
            flow: i32,
            mode: &QString,
            pattern: i32,
            origin_x: i32,
            origin_y: i32,
        ) -> bool;

        /// Begin a History Brush stroke painting the active layer as it was in
        /// the brush source state. False as for `begin_clone_stamp`, or when
        /// the source state has no pixel layer at the active path.
        fn begin_history_brush(
            view: Pin<&mut PictureView>,
            diameter: i32,
            hardness: i32,
            opacity: i32,
            flow: i32,
            mode: &QString,
        ) -> bool;

        /// The number of built-in patterns.
        fn stamp_pattern_count() -> i32;

        /// Built-in pattern `index`'s name, or empty out of range.
        fn stamp_pattern_name(index: i32) -> QString;

        /// Built-in pattern `index` as packed RGBA8888, `stamp_pattern_side()`
        /// square; empty out of range.
        fn stamp_pattern_tile(index: i32) -> Vec<u8>;

        /// The side of every built-in pattern tile, in pixels.
        fn stamp_pattern_side() -> i32;

        /// The History panel row of the brush source: the state index (the
        /// oldest state by default), or -1 when the source is a snapshot or a
        /// state the stack has dropped.
        fn history_brush_source_state(view: &PictureView) -> i32;

        /// The snapshot index of the brush source, or -1.
        fn history_brush_source_snapshot(view: &PictureView) -> i32;

        /// Point the History Brush at state (or, with `snapshot`, named
        /// snapshot) `index`. False when it does not exist.
        fn set_history_brush_source(
            view: Pin<&mut PictureView>,
            snapshot: bool,
            index: i32,
        ) -> bool;
    }
}

fn begin_color_replacement(
    view: Pin<&mut PictureView>,
    foreground: u32,
    background: u32,
    diameter: i32,
    hardness: i32,
    mode: i32,
    sampling: i32,
    limits: i32,
    tolerance: i32,
    antialias: bool,
) -> bool {
    let cfg = StrokeConfig {
        color: rgba_from_argb(foreground),
        background: rgba_from_argb(background),
        ..brush(diameter, hardness)
    };
    let options = ReplaceOptions {
        mode: ReplaceMode::from_i32(mode).unwrap_or_default(),
        sampling: Sampling::from_i32(sampling).unwrap_or_default(),
        limits: Limits::from_i32(limits).unwrap_or_default(),
        // The bar shows a percentage; the match is per channel in 0–255.
        tolerance: (tolerance.clamp(0, 100) as f32 * 2.55).round() as u8,
        antialias,
    };
    begin(view, "Color Replacement Tool", |doc, path, _| {
        Stroke::begin_kind(doc, path, cfg, StrokeKind::Replace(options)).ok()
    })
}

fn begin_mixer_brush(
    view: Pin<&mut PictureView>,
    reservoir: u32,
    diameter: i32,
    hardness: i32,
    wet: i32,
    load: i32,
    mix: i32,
    flow: i32,
) -> bool {
    let unit = |v: i32| v.clamp(0, 100) as f32 / 100.0;
    let kind = StrokeKind::Mixer {
        options: MixerOptions {
            wet: unit(wet),
            load: unit(load),
            mix: unit(mix),
            flow: unit(flow),
        },
        reservoir: rgba_from_argb(reservoir),
    };
    let cfg = brush(diameter, hardness);
    begin(view, "Mixer Brush Tool", |doc, path, _| {
        Stroke::begin_kind(doc, path, cfg, kind).ok()
    })
}

fn mixer_reservoir(view: &PictureView) -> u32 {
    view.rust()
        .stroke
        .as_ref()
        .and_then(Stroke::mixer_reservoir)
        .map_or(0, |c| {
            (c.a as u32) << 24 | (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
        })
}

fn brush(diameter: i32, hardness: i32) -> StrokeConfig {
    StrokeConfig {
        diameter: diameter.max(0) as u32,
        hardness: hardness.clamp(0, 100) as u8,
        ..StrokeConfig::default()
    }
}

fn begin_clone_stamp(
    view: Pin<&mut PictureView>,
    diameter: i32,
    hardness: i32,
    opacity: i32,
    flow: i32,
    mode: &QString,
    offset_x: i32,
    offset_y: i32,
    sampling: i32,
    ignore_adjustments: bool,
) -> bool {
    // Sampling where it paints would copy each pixel onto itself.
    if (offset_x, offset_y) == (0, 0) {
        return false;
    }
    let sampling = CloneSampling::from_i32(sampling).unwrap_or_default();
    let cfg = stamp_brush(diameter, hardness, opacity, flow, mode);
    begin(view, "Clone Stamp", |doc, path, _| {
        let image = match sample_scope(doc, path, sampling, ignore_adjustments) {
            Some(scope) => surface_from_composite(&pictura_render::composite_rgba(&scope)),
            None => layer_surface(doc, path)?,
        };
        Stroke::begin_source(
            doc,
            path,
            cfg,
            StampSource::new(image, (offset_x, offset_y)),
        )
        .ok()
    })
}

fn begin_pattern_stamp(
    view: Pin<&mut PictureView>,
    diameter: i32,
    hardness: i32,
    opacity: i32,
    flow: i32,
    mode: &QString,
    pattern: i32,
    origin_x: i32,
    origin_y: i32,
) -> bool {
    let Some(tile) = usize::try_from(pattern).ok().and_then(pattern::tile) else {
        return false;
    };
    let cfg = stamp_brush(diameter, hardness, opacity, flow, mode);
    begin(view, "Pattern Stamp", |doc, path, _| {
        let image = tiled(&tile, doc.width, doc.height, (origin_x, origin_y))?;
        Stroke::begin_source(doc, path, cfg, StampSource::new(image, (0, 0))).ok()
    })
}

fn begin_history_brush(
    view: Pin<&mut PictureView>,
    diameter: i32,
    hardness: i32,
    opacity: i32,
    flow: i32,
    mode: &QString,
) -> bool {
    let cfg = stamp_brush(diameter, hardness, opacity, flow, mode);
    begin(view, "History Brush", |doc, path, view| {
        // ponytail: the source layer is matched by panel path (layers carry no
        // stable id), so a reordered stack paints from whatever now sits there.
        let past = view.history.brush_source_doc()?;
        let source = StampSource::new(layer_surface(past, path)?, (0, 0));
        Stroke::begin_source(doc, path, cfg, source).ok()
    })
}

fn stamp_pattern_count() -> i32 {
    PATTERN_NAMES.len() as i32
}

fn stamp_pattern_name(index: i32) -> QString {
    usize::try_from(index)
        .ok()
        .and_then(|i| PATTERN_NAMES.get(i))
        .map_or_else(QString::default, |name| QString::from(*name))
}

fn stamp_pattern_tile(index: i32) -> Vec<u8> {
    usize::try_from(index)
        .ok()
        .and_then(pattern::tile)
        .map_or_else(Vec::new, |tile| tile.data.concat())
}

fn stamp_pattern_side() -> i32 {
    pattern::TILE
}

fn history_brush_source_state(view: &PictureView) -> i32 {
    match view.rust().history.brush_source() {
        BrushSource::Oldest => 0,
        BrushSource::State(i) => i as i32,
        _ => -1,
    }
}

fn history_brush_source_snapshot(view: &PictureView) -> i32 {
    match view.rust().history.brush_source() {
        BrushSource::Snapshot(i) => i as i32,
        _ => -1,
    }
}

fn set_history_brush_source(mut view: Pin<&mut PictureView>, snapshot: bool, index: i32) -> bool {
    let Ok(index) = usize::try_from(index) else {
        return false;
    };
    let source = if snapshot {
        BrushSource::Snapshot(index)
    } else {
        BrushSource::State(index)
    };
    view.as_mut().rust_mut().history.set_brush_source(source)
}

fn stamp_brush(
    diameter: i32,
    hardness: i32,
    opacity: i32,
    flow: i32,
    mode: &QString,
) -> StrokeConfig {
    StrokeConfig {
        diameter: diameter.max(0) as u32,
        hardness: hardness.clamp(0, 100) as u8,
        opacity: opacity.clamp(0, 100) as u8,
        flow: flow.clamp(0, 100) as u8,
        mode: paint_mode_from(&mode.to_string()),
        ..StrokeConfig::default()
    }
}

/// Begin a stroke on the visible lone active pixel layer; `start` builds it
/// from the document, the active path, and the view. False mid-stroke or when
/// `start` refuses.
fn begin(
    mut view: Pin<&mut PictureView>,
    label: &str,
    start: impl FnOnce(&Document, &str, &PictureViewRust) -> Option<Stroke>,
) -> bool {
    let stroke = {
        let rust = view.rust();
        let (Some(doc), Some(path)) = (rust.doc.as_ref(), rust.active_layer.as_deref()) else {
            return false;
        };
        if rust.stroke.is_some()
            || active_pixel_layer(doc, Some(path)).is_none()
            || !active_layer_visible(doc, Some(path))
        {
            return false;
        }
        start(doc, path, rust)
    };
    let Some(stroke) = stroke else {
        return false;
    };
    let mut rust = view.as_mut().rust_mut();
    rust.stroke = Some(stroke);
    rust.stroke_label = label.to_string();
    true
}
