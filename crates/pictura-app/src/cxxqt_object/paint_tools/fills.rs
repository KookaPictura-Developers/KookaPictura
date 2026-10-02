//! The fill tool bridges: the Gradient and the Paint Bucket. Free functions
//! over a [`PictureView`], as in the parent module. One gesture is one edit,
//! so each applies it through the selection, recomposites, and records its own
//! history state (`"Gradient"`, `"Paint Bucket"`). The shape tools' Fill
//! Pixels mode shares [`apply`].
//!
//! [`PictureView`]: super::super::qobject::PictureView

use super::super::helpers::{
    active_layer_visible, active_pixel_layer, paint_mode_from, rgba_from_argb,
};
use super::super::helpers_composite::current_buffer;
use super::super::qobject::PictureView;
use super::super::PictureViewRust;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::{Document, PsdRect};
use pictura_paint::bucket::{self, BucketPaint};
use pictura_paint::eraser::antialias_mask;
use pictura_paint::gradient::{self, GradientOptions, GradientStyle, PRESET_NAMES};
use pictura_paint::pattern;
use pictura_paint::stamp::{layer_surface, surface_from_composite};

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
        /// The number of built-in gradients.
        fn gradient_preset_count() -> i32;

        /// Built-in gradient `index`'s name, or empty out of range.
        fn gradient_preset_name(index: i32) -> QString;

        /// Built-in gradient `index` between `foreground` and `background`
        /// (`0xAARRGGBB`) as a horizontal strip, packed straight RGBA8888
        /// `width` × `height`; empty out of range.
        fn gradient_preset_strip(
            index: i32,
            foreground: u32,
            background: u32,
            width: i32,
            height: i32,
        ) -> Vec<u8>;

        /// Gradient tool: draw built-in gradient `preset` over the active
        /// pixel layer along the drag from `(x0, y0)` to `(x1, y1)` (document
        /// space), inside the selection when there is one. `style` 0 Linear /
        /// 1 Radial / 2 Angle / 3 Reflected / 4 Diamond; `mode` is a Brush mode
        /// (`"normal"`, …); `opacity` 0–100 %; `transparency` honours the
        /// gradient's own alpha. One "Gradient" state; false when nothing
        /// changed (a drag with no length), without a visible lone pixel
        /// layer, when its pixels are locked, or mid-stroke.
        fn draw_gradient(
            view: Pin<&mut PictureView>,
            preset: i32,
            foreground: u32,
            background: u32,
            style: i32,
            mode: &QString,
            opacity: i32,
            reverse: bool,
            dither: bool,
            transparency: bool,
            x0: f64,
            y0: f64,
            x1: f64,
            y1: f64,
        ) -> bool;

        /// Paint Bucket: fill the pixels that match the one at document point
        /// `(x, y)` — `tolerance` 0–255 per channel, `contiguous`, read from
        /// the active layer or with `all_layers` the composite — with
        /// `foreground` (`0xAARRGGBB`), or built-in pattern `pattern` when it
        /// is not negative, at `opacity` 0–100 % in Brush mode `mode`, with a
        /// softened edge when `antialias`, inside the selection when there is
        /// one. One "Paint Bucket" state; false as for `draw_gradient`, for a
        /// point off the canvas, or for an unknown pattern.
        fn bucket_fill_at(
            view: Pin<&mut PictureView>,
            x: i32,
            y: i32,
            foreground: u32,
            pattern: i32,
            mode: &QString,
            opacity: i32,
            tolerance: i32,
            antialias: bool,
            contiguous: bool,
            all_layers: bool,
        ) -> bool;
    }
}

fn gradient_preset_count() -> i32 {
    PRESET_NAMES.len() as i32
}

fn gradient_preset_name(index: i32) -> QString {
    usize::try_from(index)
        .ok()
        .and_then(|i| PRESET_NAMES.get(i))
        .map_or_else(QString::default, |name| QString::from(*name))
}

fn gradient_preset_strip(
    index: i32,
    foreground: u32,
    background: u32,
    width: i32,
    height: i32,
) -> Vec<u8> {
    let (fg, bg) = (rgba_from_argb(foreground), rgba_from_argb(background));
    usize::try_from(index)
        .ok()
        .and_then(|i| gradient::preset(i, fg, bg))
        .map_or_else(Vec::new, |ramp| {
            ramp.preview(width.clamp(1, 4096), height.clamp(1, 4096))
                .data
                .concat()
        })
}

fn draw_gradient(
    view: Pin<&mut PictureView>,
    preset: i32,
    foreground: u32,
    background: u32,
    style: i32,
    mode: &QString,
    opacity: i32,
    reverse: bool,
    dither: bool,
    transparency: bool,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> bool {
    // Built per drag: "Foreground to Background" means the colours as they
    // are now.
    let Some(ramp) = usize::try_from(preset)
        .ok()
        .and_then(|i| gradient::preset(i, rgba_from_argb(foreground), rgba_from_argb(background)))
    else {
        return false;
    };
    let options = GradientOptions {
        style: GradientStyle::from_i32(style).unwrap_or_default(),
        mode: paint_mode_from(&mode.to_string()),
        opacity: opacity.clamp(0, 100) as f32 / 100.0,
        reverse,
        dither,
        transparency,
    };
    let (start, end) = ((x0 as f32, y0 as f32), (x1 as f32, y1 as f32));
    apply(view, "Gradient", |doc, path, rust| {
        gradient::draw(doc, path, &ramp, &options, start, end, selection(rust))
    })
}

fn bucket_fill_at(
    view: Pin<&mut PictureView>,
    x: i32,
    y: i32,
    foreground: u32,
    pattern: i32,
    mode: &QString,
    opacity: i32,
    tolerance: i32,
    antialias: bool,
    contiguous: bool,
    all_layers: bool,
) -> bool {
    let tile = match usize::try_from(pattern) {
        Ok(index) => match pattern::tile(index) {
            Some(tile) => Some(tile),
            None => return false,
        },
        Err(_) => None,
    };
    let paint = match &tile {
        Some(tile) => BucketPaint::Pattern(tile),
        None => BucketPaint::Foreground(rgba_from_argb(foreground)),
    };
    let mode = paint_mode_from(&mode.to_string());
    let opacity = opacity.clamp(0, 100) as f32 / 100.0;
    let tolerance = tolerance.clamp(0, 255) as u8;
    apply(view, "Paint Bucket", |doc, path, rust| {
        let surface = if all_layers {
            surface_from_composite(&current_buffer(doc, rust.gpu_compute))
        } else {
            layer_surface(doc, path)?
        };
        let flood = bucket::flood(&surface, (x, y), tolerance, contiguous)?;
        let mask = if antialias {
            antialias_mask(&flood, surface.width as usize)
        } else {
            flood
        };
        bucket::fill(doc, path, &mask, paint, mode, opacity, selection(rust))
    })
}

pub(crate) fn selection(rust: &PictureViewRust) -> Option<&[u8]> {
    rust.selection.as_ref().map(|s| s.data.as_slice())
}

/// Run `edit` on a copy of the document and its visible lone active pixel
/// layer; when it changed something, adopt the copy and record it as `label`.
/// False (no state) mid-stroke, without such a layer, or when nothing changed.
pub(crate) fn apply(
    mut view: Pin<&mut PictureView>,
    label: &str,
    edit: impl FnOnce(&mut Document, &str, &PictureViewRust) -> Option<PsdRect>,
) -> bool {
    let edited = {
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
        let mut edited = doc.clone();
        edit(&mut edited, path, rust).map(|_| edited)
    };
    let Some(doc) = edited else {
        return false;
    };
    view.as_mut().rust_mut().doc = Some(doc);
    view.as_mut().recomposite();
    view.as_mut().record(label);
    true
}
