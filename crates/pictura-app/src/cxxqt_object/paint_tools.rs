//! The per-dab paint tool bridges: Color Replacement and Mixer Brush. Free
//! functions over a [`PictureView`] (their own bridge, so the `PictureView`
//! declaration list does not grow). Each only *begins* a stroke of its
//! [`StrokeKind`]; the live stroke then runs through the Brush's `paint_dab` /
//! `end_paint` / `cancel_paint`, so the preview and the one history state per
//! stroke (`"Color Replacement Tool"`, `"Mixer Brush Tool"`) are shared.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::{active_layer_visible, active_pixel_layer, rgba_from_argb};
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use pictura_paint::mixer::MixerOptions;
use pictura_paint::replace::{Limits, ReplaceMode, ReplaceOptions, Sampling};
use pictura_paint::{Stroke, StrokeConfig, StrokeKind};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
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
    begin(
        view,
        cfg,
        StrokeKind::Replace(options),
        "Color Replacement Tool",
    )
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
    begin(view, brush(diameter, hardness), kind, "Mixer Brush Tool")
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

fn begin(
    mut view: Pin<&mut PictureView>,
    cfg: StrokeConfig,
    kind: StrokeKind,
    label: &str,
) -> bool {
    let begun = {
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
        Stroke::begin_kind(doc, path, cfg, kind)
    };
    let Ok(stroke) = begun else {
        return false;
    };
    let mut rust = view.as_mut().rust_mut();
    rust.stroke = Some(stroke);
    rust.stroke_label = label.to_string();
    true
}
