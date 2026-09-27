//! The Crop tool group's Perspective Crop and Slice commands. Free functions
//! over a [`PictureView`] (their own bridge, so the `PictureView` declaration
//! list does not grow). A Perspective Crop records one "Perspective Crop" state
//! and a new slice one "Slice" state; every refusal records nothing.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::PsdRect;

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
        /// Perspective-crop to `quad` (8 values: TL, TR, BR, BL x/y in document pixels): warp every layer so the quad becomes the canvas, drop the selection, recomposite, and record one "Perspective Crop" state. False, changing nothing, without a document, on a refusal, or for a degenerate quad.
        fn perspective_crop_commit(view: Pin<&mut PictureView>, quad: &[f64]) -> bool;

        /// Why the document cannot be perspective-cropped (live type, smart objects, vector masks, 16/32-bit), or empty when it can.
        fn perspective_crop_refusal_reason(view: &PictureView) -> QString;

        /// Number of resolved slices (user and auto); 0 without a document.
        fn slice_count(view: &PictureView) -> i32;

        /// Resolved slice `index` as `[x, y, width, height, number, userIndex]` (`userIndex` -1 for an auto slice); empty when out of range.
        fn slice_at(view: &PictureView, index: i32) -> Vec<i32>;

        /// Add a user slice and record one "Slice" state; its index, or -1 (no state) for an empty rect or without a document.
        fn add_user_slice(
            view: Pin<&mut PictureView>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> i32;
    }
}

fn perspective_crop_commit(mut view: Pin<&mut PictureView>, quad: &[f64]) -> bool {
    let [x0, y0, x1, y1, x2, y2, x3, y3] = *quad else {
        return false;
    };
    let corners = [(x0, y0), (x1, y1), (x2, y2), (x3, y3)];
    let cropped = {
        let mut rust = view.as_mut().rust_mut();
        let Some(doc) = rust.doc.as_mut() else {
            return false;
        };
        if !pictura_render::perspective_crop(doc, corners) {
            return false;
        }
        // Canvas dimensions changed, so the old selection no longer maps.
        rust.selection = None;
        true
    };
    if cropped {
        view.as_mut().recomposite();
        view.as_mut().record("Perspective Crop");
    }
    cropped
}

fn perspective_crop_refusal_reason(view: &PictureView) -> QString {
    view.rust()
        .doc
        .as_ref()
        .and_then(pictura_render::perspective_crop_refusal)
        .map(QString::from)
        .unwrap_or_default()
}

fn slices(view: &PictureView) -> Vec<pictura_render::Slice> {
    view.rust()
        .doc
        .as_ref()
        .map(pictura_render::resolve_slices)
        .unwrap_or_default()
}

fn slice_count(view: &PictureView) -> i32 {
    slices(view).len() as i32
}

fn slice_at(view: &PictureView, index: i32) -> Vec<i32> {
    let slices = slices(view);
    let Some(slice) = usize::try_from(index).ok().and_then(|i| slices.get(i)) else {
        return Vec::new();
    };
    vec![
        slice.rect.left,
        slice.rect.top,
        slice.rect.width(),
        slice.rect.height(),
        slice.number as i32,
        slice.user_index.map_or(-1, |i| i as i32),
    ]
}

fn add_user_slice(mut view: Pin<&mut PictureView>, x: i32, y: i32, width: i32, height: i32) -> i32 {
    let rect = PsdRect {
        top: y,
        left: x,
        bottom: y.saturating_add(height.max(0)),
        right: x.saturating_add(width.max(0)),
    };
    let added = match view.as_mut().rust_mut().doc.as_mut() {
        Some(doc) => pictura_render::add_slice(doc, rect),
        None => None,
    };
    let Some(index) = added else {
        return -1;
    };
    view.as_mut().record("Slice");
    view.as_mut().changed();
    index as i32
}
