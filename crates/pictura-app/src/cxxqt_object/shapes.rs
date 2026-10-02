//! The shape tools (Rectangle, Rounded Rectangle, Ellipse, Polygon): the live
//! outline and the three ways a drag lands, CS6's Shape / Path / Fill Pixels
//! modes. Free functions over a [`PictureView`] (their own bridge, so the
//! `PictureView` declaration list does not grow).
//!
//! Every mode draws [`pictura_core::shape::outline`], the same call the
//! preview makes, and records one `"<Tool> Tool"` state (e.g.
//! `"Rectangle Tool"`).
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::rgba_from_argb;
use super::paint_tools::fills::{apply, selection};
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::path::Subpath;
use pictura_core::shape::{self, ShapeKind, ShapeOptions};
use pictura_paint::bucket::{self, BucketPaint};
use pictura_paint::PaintMode;

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
        /// The closed outline shape tool `kind` (0 Rectangle, 1 Rounded Rectangle, 2 Ellipse, 3 Polygon) draws for a drag `[x0, y0, x1, y1]` (document space): `radius` is the Rounded Rectangle's corner radius, `sides` the Polygon's; `shift` squares off (or snaps the Polygon's turn to 15°) and `alt` grows from the centre. Six numbers per anchor `[ax, ay, ix, iy, ox, oy]`, a missing handle at its anchor; empty when the drag encloses nothing.
        fn shape_outline(
            kind: i32,
            drag: &[f64],
            shift: bool,
            alt: bool,
            radius: f64,
            sides: i32,
        ) -> Vec<f64>;

        /// Shape mode: a new shape layer above the active layer (a `foreground` (`0xAARRGGBB`) color fill cut to the outline by a vector mask), named after the tool. Returns its path; empty (no state) without a document or for an empty outline.
        fn shape_add_layer(
            view: Pin<&mut PictureView>,
            kind: i32,
            drag: &[f64],
            shift: bool,
            alt: bool,
            radius: f64,
            sides: i32,
            foreground: u32,
        ) -> QString;

        /// Path mode: add the outline to the Work Path as a new closed component. False (no state) without a document or for an empty outline.
        fn shape_add_path(
            view: Pin<&mut PictureView>,
            kind: i32,
            drag: &[f64],
            shift: bool,
            alt: bool,
            radius: f64,
            sides: i32,
        ) -> bool;

        /// Fill Pixels mode: paint the anti-aliased outline in `foreground` (`0xAARRGGBB`) onto the active pixel layer, inside the selection when there is one. False (no state) as for `draw_gradient`, or for an empty outline.
        fn shape_fill_pixels(
            view: Pin<&mut PictureView>,
            kind: i32,
            drag: &[f64],
            shift: bool,
            alt: bool,
            radius: f64,
            sides: i32,
            foreground: u32,
        ) -> bool;
    }
}

/// The tool's kind and the outline it draws for `drag`.
fn outline(
    kind: i32,
    drag: &[f64],
    shift: bool,
    alt: bool,
    radius: f64,
    sides: i32,
) -> Option<(ShapeKind, Subpath)> {
    let kind = ShapeKind::from_i32(kind)?;
    let &[x0, y0, x1, y1] = drag else {
        return None;
    };
    let options = ShapeOptions {
        kind,
        radius,
        sides: sides.max(0) as u32,
    };
    let subpath = shape::outline(options, (x0, y0), (x1, y1), shift, alt)?;
    Some((kind, subpath))
}

fn label(kind: ShapeKind) -> String {
    format!("{} Tool", kind.layer_name())
}

fn shape_outline(
    kind: i32,
    drag: &[f64],
    shift: bool,
    alt: bool,
    radius: f64,
    sides: i32,
) -> Vec<f64> {
    let Some((_, subpath)) = outline(kind, drag, shift, alt, radius, sides) else {
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

fn shape_add_layer(
    mut view: Pin<&mut PictureView>,
    kind: i32,
    drag: &[f64],
    shift: bool,
    alt: bool,
    radius: f64,
    sides: i32,
    foreground: u32,
) -> QString {
    let Some((kind, subpath)) = outline(kind, drag, shift, alt, radius, sides) else {
        return QString::default();
    };
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

fn shape_add_path(
    mut view: Pin<&mut PictureView>,
    kind: i32,
    drag: &[f64],
    shift: bool,
    alt: bool,
    radius: f64,
    sides: i32,
) -> bool {
    let Some((kind, subpath)) = outline(kind, drag, shift, alt, radius, sides) else {
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

fn shape_fill_pixels(
    view: Pin<&mut PictureView>,
    kind: i32,
    drag: &[f64],
    shift: bool,
    alt: bool,
    radius: f64,
    sides: i32,
    foreground: u32,
) -> bool {
    let Some((kind, subpath)) = outline(kind, drag, shift, alt, radius, sides) else {
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
