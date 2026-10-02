//! The shape tools (Rectangle, Rounded Rectangle, Ellipse, Polygon): the live
//! outline and the three ways a shape lands, CS6's Shape / Path / Fill Pixels
//! modes, plus the Layers panel's view of a shape layer. Free functions over a
//! [`PictureView`] (their own bridge, so the `PictureView` declaration list
//! does not grow).
//!
//! Every mode draws one [`ShapeSpec`]: a drag, or (`boxed`) the box a Create
//! dialog placed. The preview makes the same call, and each landing records
//! one `"<Tool> Tool"` state (e.g. `"Rectangle Tool"`). A Rectangle, Rounded
//! Rectangle, or Ellipse drawn in Shape mode is a live shape (a CC feature):
//! the layer keeps its parametric origin until an edit turns it into a regular
//! path (`paths.rs`).
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::rgba_from_argb;
use super::helpers_composite::rgba_image;
use super::paint_tools::fills::{apply, selection};
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};
use ffi::ShapeSpec;
use pictura_codec::LiveShape;
use pictura_core::path::{Subpath, VectorPath};
use pictura_core::shape::{self, ShapeKind, ShapeOptions};
use pictura_core::{Document, Layer};
use pictura_paint::bucket::{self, BucketPaint};
use pictura_paint::PaintMode;

#[cxx_qt::bridge]
pub mod ffi {
    /// One shape: tool `kind` (0 Rectangle, 1 Rounded Rectangle, 2 Ellipse,
    /// 3 Polygon) and either a drag from `(x0, y0)` to `(x1, y1)` (`shift`
    /// squares off or snaps the Polygon's turn to 15°, `alt` grows from the
    /// press) or, when `boxed`, the box with those corners. The Rounded
    /// Rectangle's corner radii `r_tl`, `r_tr`, `r_br`, `r_bl` (px); the
    /// Polygon's `sides`, `star` with `indent` (% of the radius), and smooth
    /// corners / indents.
    #[namespace = "pictura"]
    struct ShapeSpec {
        kind: i32,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        shift: bool,
        alt: bool,
        boxed: bool,
        r_tl: f64,
        r_tr: f64,
        r_br: f64,
        r_bl: f64,
        sides: i32,
        star: bool,
        indent: f64,
        smooth_corners: bool,
        smooth_indents: bool,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;

        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// The closed outline `spec` draws. Six numbers per anchor `[ax, ay, ix, iy, ox, oy]`, a missing handle at its anchor; empty when the shape encloses nothing.
        fn shape_outline(spec: &ShapeSpec) -> Vec<f64>;

        /// Shape mode: a new shape layer above the active layer (a `foreground` (`0xAARRGGBB`) color fill cut to the outline by a vector mask), named after the tool, live unless a Polygon. Returns its path; empty (no state) without a document or for an empty outline.
        fn shape_add_layer(
            view: Pin<&mut PictureView>,
            spec: &ShapeSpec,
            foreground: u32,
        ) -> QString;

        /// Path mode: add the outline to the Work Path as a new closed component. False (no state) without a document or for an empty outline.
        fn shape_add_path(view: Pin<&mut PictureView>, spec: &ShapeSpec) -> bool;

        /// Fill Pixels mode: paint the anti-aliased outline in `foreground` (`0xAARRGGBB`) onto the active pixel layer, inside the selection when there is one. False (no state) as for `draw_gradient`, or for an empty outline.
        fn shape_fill_pixels(
            view: Pin<&mut PictureView>,
            spec: &ShapeSpec,
            foreground: u32,
        ) -> bool;

        /// Whether Layers row `i` is a shape layer (a fill cut by a vector mask).
        fn shape_row_is_shape(view: &PictureView, i: i32) -> bool;

        /// Whether Layers row `i` is a live shape.
        fn shape_row_is_live(view: &PictureView, i: i32) -> bool;

        /// Shape layer row `i` drawn into a `size` square laid out like the whole document (fill color where the outline covers, transparent elsewhere); null when the row is not a shape layer.
        fn shape_row_thumbnail(view: &PictureView, i: i32, size: i32) -> QImage;
    }
}

/// The tool's kind and the outline `spec` draws.
fn outline(spec: &ShapeSpec) -> Option<(ShapeKind, Subpath)> {
    let kind = ShapeKind::from_i32(spec.kind)?;
    let options = ShapeOptions {
        kind,
        radii: [spec.r_tl, spec.r_tr, spec.r_br, spec.r_bl],
        sides: spec.sides.max(0) as u32,
        star: spec.star.then_some(spec.indent),
        smooth_corners: spec.smooth_corners,
        smooth_indents: spec.smooth_indents,
    };
    let subpath = if spec.boxed {
        let (x, y) = (spec.x0.min(spec.x1), spec.y0.min(spec.y1));
        let rect = (x, y, (spec.x1 - spec.x0).abs(), (spec.y1 - spec.y0).abs());
        shape::outline_in_box(options, rect)?
    } else {
        shape::outline(
            options,
            (spec.x0, spec.y0),
            (spec.x1, spec.y1),
            spec.shift,
            spec.alt,
        )?
    };
    Some((kind, subpath))
}

/// The live origin of a Shape-mode `kind` drawn as `outline`: its box and
/// clamped radii. Polygons were not live shapes in CC 2015, so they have none.
fn live_shape(spec: &ShapeSpec, kind: ShapeKind, outline: &Subpath) -> Option<LiveShape> {
    let origin_type = match kind {
        ShapeKind::Rectangle => pictura_codec::ORIGIN_RECTANGLE,
        ShapeKind::RoundedRectangle => pictura_codec::ORIGIN_ROUNDED_RECTANGLE,
        ShapeKind::Ellipse => pictura_codec::ORIGIN_ELLIPSE,
        ShapeKind::Polygon => return None,
    };
    let mut path = VectorPath::default();
    path.add_subpath(outline.clone());
    let bounds = path.subpath_bounds(0)?;
    let cap = (bounds.2 - bounds.0).min(bounds.3 - bounds.1) / 2.0;
    let radii = if kind == ShapeKind::RoundedRectangle {
        [spec.r_tl, spec.r_tr, spec.r_br, spec.r_bl].map(|r| r.clamp(0.0, cap))
    } else {
        [0.0; 4]
    };
    Some(LiveShape {
        origin_type,
        bounds,
        radii,
    })
}

fn label(kind: ShapeKind) -> String {
    format!("{} Tool", kind.layer_name())
}

fn shape_outline(spec: &ShapeSpec) -> Vec<f64> {
    let Some((_, subpath)) = outline(spec) else {
        return Vec::new();
    };
    subpath
        .points
        .iter()
        .flat_map(|p| {
            let (ix, iy) = p.in_handle.unwrap_or(p.anchor);
            let (ox, oy) = p.out_handle.unwrap_or(p.anchor);
            [p.anchor.0, p.anchor.1, ix, iy, ox, oy]
        })
        .collect()
}

fn shape_add_layer(mut view: Pin<&mut PictureView>, spec: &ShapeSpec, foreground: u32) -> QString {
    let Some((kind, subpath)) = outline(spec) else {
        return QString::default();
    };
    let live = live_shape(spec, kind, &subpath);
    let c = rgba_from_argb(foreground);
    let created = {
        let mut rust = view.as_mut().rust_mut();
        let above = rust.active_layer.clone().unwrap_or_default();
        match rust.doc.as_mut() {
            Some(doc) => pictura_render::add_shape_layer(
                doc,
                &above,
                [c.r, c.g, c.b, 255],
                kind.layer_name(),
                &subpath,
                live.as_ref(),
            ),
            None => String::new(),
        }
    };
    if !created.is_empty() {
        view.as_mut().clear_link_sets();
        view.as_mut().recomposite();
        view.as_mut().record(&label(kind));
    }
    QString::from(created.as_str())
}

fn shape_add_path(mut view: Pin<&mut PictureView>, spec: &ShapeSpec) -> bool {
    let Some((kind, subpath)) = outline(spec) else {
        return false;
    };
    let added = match view.as_mut().rust_mut().doc.as_mut() {
        Some(doc) => {
            doc.work_path.add_subpath(subpath);
            true
        }
        None => false,
    };
    if added {
        view.as_mut().record(&label(kind));
        view.as_mut().changed();
    }
    added
}

fn shape_fill_pixels(view: Pin<&mut PictureView>, spec: &ShapeSpec, foreground: u32) -> bool {
    let Some((kind, subpath)) = outline(spec) else {
        return false;
    };
    let paint = BucketPaint::Foreground(rgba_from_argb(foreground));
    apply(view, &label(kind), |doc, path, rust| {
        let mask = shape::coverage(&subpath, doc.width, doc.height);
        bucket::fill(
            doc,
            path,
            &mask,
            paint,
            PaintMode::Normal,
            1.0,
            selection(rust),
        )
    })
}

/// Layers row `i`'s document and layer, when it is a shape layer.
fn shape_row(view: &PictureView, i: i32) -> Option<(&Document, &Layer)> {
    let doc = view.rust().doc.as_ref()?;
    let (path, _) = pictura_render::flatten_rows(doc)
        .into_iter()
        .nth(usize::try_from(i).ok()?)?;
    let layer = pictura_render::resolve_path(doc, &path)?;
    pictura_render::is_shape_layer(layer).then_some((doc, layer))
}

fn shape_row_is_shape(view: &PictureView, i: i32) -> bool {
    shape_row(view, i).is_some()
}

fn shape_row_is_live(view: &PictureView, i: i32) -> bool {
    shape_row(view, i).is_some_and(|(_, layer)| pictura_render::layer_live_shape(layer).is_some())
}

fn shape_row_thumbnail(view: &PictureView, i: i32, size: i32) -> QImage {
    let Some((doc, layer)) = shape_row(view, i) else {
        return QImage::default();
    };
    let Some([r, g, b, _]) = pictura_render::shape_fill_color(layer) else {
        return QImage::default();
    };
    if size <= 0 || doc.width == 0 || doc.height == 0 {
        return QImage::default();
    }
    let side = size as u32;
    let scale = doc.width.max(doc.height) as f64 / side as f64;
    let mut rgba = vec![0u8; (side * side * 4) as usize];
    for ty in 0..side {
        for tx in 0..side {
            let x = ((tx as f64 + 0.5) * scale) as i32;
            let y = ((ty as f64 + 0.5) * scale) as i32;
            if x >= doc.width as i32 || y >= doc.height as i32 {
                continue;
            }
            let a = pictura_render::shape_coverage(layer, x, y);
            let o = ((ty * side + tx) * 4) as usize;
            rgba[o..o + 4].copy_from_slice(&[r, g, b, a]);
        }
    }
    rgba_image(rgba, size, size)
}
