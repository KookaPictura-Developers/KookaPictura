//! The Eyedropper group's Color Sampler, Note, and Ruler commands. Free
//! functions over a [`PictureView`] (their own bridge, so the `PictureView`
//! declaration list does not grow). `kind` is a [`MarkerKind`] integer: 0 color
//! sampler, 1 note. Every committed sampler/note edit records one history state
//! and emits `changed`; a live drag step records nothing. The ruler is view
//! state: it is neither recorded nor saved.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::{Annotations, MarkerKind, Ruler};

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
        /// Number of markers of `kind`; 0 without a document or for an unknown kind.
        fn marker_count(view: &PictureView, kind: i32) -> i32;

        /// Marker `index` of `kind` as `[x, y]` in document pixels; empty when out of range.
        fn marker_at(view: &PictureView, kind: i32, index: i32) -> Vec<i32>;

        /// The marker of `kind` nearest `(x, y)` within `radius` document pixels, or -1.
        fn marker_near(view: &PictureView, kind: i32, x: f64, y: f64, radius: f64) -> i32;

        /// Place a marker and record one "Color Sampler" / "New Note" state; its index, or -1 (no state) when the four samplers are placed or without a document.
        fn add_marker(view: Pin<&mut PictureView>, kind: i32, x: i32, y: i32) -> i32;

        /// Move marker `index`. With `commit` false this is a live drag step (no history, no `changed`); with `commit` true it records one "Move Color Sampler" / "Move Note" state. False for a bad index or no document.
        fn move_marker(
            view: Pin<&mut PictureView>,
            kind: i32,
            index: i32,
            x: i32,
            y: i32,
            commit: bool,
        ) -> bool;

        /// Delete marker `index` and record one "Delete Color Sampler" / "Delete Note" state; false for a bad index or no document.
        fn remove_marker(view: Pin<&mut PictureView>, kind: i32, index: i32) -> bool;

        /// Delete every marker of `kind` and record one "Clear Color Samplers" / "Delete All Notes" state; false (no state) when there were none.
        fn clear_markers(view: Pin<&mut PictureView>, kind: i32) -> bool;

        /// Note `index`'s text; empty when out of range.
        fn note_text(view: &PictureView, index: i32) -> QString;

        /// Replace note `index`'s text and record one "Edit Note" state; false (no state) for a bad index or unchanged text.
        fn set_note_text(view: Pin<&mut PictureView>, index: i32, text: &QString) -> bool;

        /// Set the Ruler's line from `(ax, ay)` to `(bx, by)` in document pixels, snapped to 45° when `constrain`. No history.
        fn set_ruler(
            view: Pin<&mut PictureView>,
            ax: f64,
            ay: f64,
            bx: f64,
            by: f64,
            constrain: bool,
        );

        /// Remove the Ruler's line.
        fn clear_ruler(view: Pin<&mut PictureView>);

        /// The Ruler's line as `[ax, ay, bx, by]`; empty when there is none.
        fn ruler_line(view: &PictureView) -> Vec<f64>;

        /// The Ruler readout `[X, Y, W, H, A (degrees), D1]`; empty when there is no line.
        fn ruler_measurement(view: &PictureView) -> Vec<f64>;
    }
}

fn annotations(view: &PictureView) -> Option<&Annotations> {
    view.rust().doc.as_ref().map(|doc| &doc.annotations)
}

/// Apply `edit` to the document's annotations; `None` without a document.
fn edit<T>(view: Pin<&mut PictureView>, edit: impl FnOnce(&mut Annotations) -> T) -> Option<T> {
    let mut rust = view.rust_mut();
    rust.doc.as_mut().map(|doc| edit(&mut doc.annotations))
}

fn commit(mut view: Pin<&mut PictureView>, label: &str) {
    view.as_mut().record(label);
    view.as_mut().changed();
}

fn label(kind: MarkerKind, sampler: &'static str, note: &'static str) -> &'static str {
    match kind {
        MarkerKind::ColorSampler => sampler,
        MarkerKind::Note => note,
    }
}

fn kind_and_index(kind: i32, index: i32) -> Option<(MarkerKind, usize)> {
    Some((MarkerKind::from_i32(kind)?, usize::try_from(index).ok()?))
}

fn marker_count(view: &PictureView, kind: i32) -> i32 {
    match (annotations(view), MarkerKind::from_i32(kind)) {
        (Some(a), Some(kind)) => a.markers(kind).len() as i32,
        _ => 0,
    }
}

fn marker_at(view: &PictureView, kind: i32, index: i32) -> Vec<i32> {
    kind_and_index(kind, index)
        .and_then(|(kind, index)| annotations(view)?.marker(kind, index))
        .map_or_else(Vec::new, |m| vec![m.x, m.y])
}

fn marker_near(view: &PictureView, kind: i32, x: f64, y: f64, radius: f64) -> i32 {
    MarkerKind::from_i32(kind)
        .and_then(|kind| annotations(view)?.marker_at(kind, x, y, radius.max(0.0)))
        .map_or(-1, |i| i as i32)
}

fn add_marker(mut view: Pin<&mut PictureView>, kind: i32, x: i32, y: i32) -> i32 {
    let Some(kind) = MarkerKind::from_i32(kind) else {
        return -1;
    };
    let Some(index) = edit(view.as_mut(), |a| a.add(kind, x, y)).flatten() else {
        return -1;
    };
    commit(view, label(kind, "Color Sampler", "New Note"));
    index as i32
}

fn move_marker(
    mut view: Pin<&mut PictureView>,
    kind: i32,
    index: i32,
    x: i32,
    y: i32,
    commit_move: bool,
) -> bool {
    let Some((kind, index)) = kind_and_index(kind, index) else {
        return false;
    };
    let moved = edit(view.as_mut(), |a| a.move_marker(kind, index, x, y)) == Some(true);
    if moved && commit_move {
        commit(view, label(kind, "Move Color Sampler", "Move Note"));
    }
    moved
}

fn remove_marker(mut view: Pin<&mut PictureView>, kind: i32, index: i32) -> bool {
    let Some((kind, index)) = kind_and_index(kind, index) else {
        return false;
    };
    let removed = edit(view.as_mut(), |a| a.remove(kind, index)) == Some(true);
    if removed {
        commit(view, label(kind, "Delete Color Sampler", "Delete Note"));
    }
    removed
}

fn clear_markers(mut view: Pin<&mut PictureView>, kind: i32) -> bool {
    let Some(kind) = MarkerKind::from_i32(kind) else {
        return false;
    };
    let cleared = edit(view.as_mut(), |a| a.clear(kind)) == Some(true);
    if cleared {
        commit(
            view,
            label(kind, "Clear Color Samplers", "Delete All Notes"),
        );
    }
    cleared
}

fn note_text(view: &PictureView, index: i32) -> QString {
    usize::try_from(index)
        .ok()
        .and_then(|i| annotations(view)?.marker(MarkerKind::Note, i))
        .map_or_else(QString::default, |m| QString::from(m.text.as_str()))
}

fn set_note_text(mut view: Pin<&mut PictureView>, index: i32, text: &QString) -> bool {
    let Ok(index) = usize::try_from(index) else {
        return false;
    };
    let text = text.to_string();
    let set = edit(view.as_mut(), |a| {
        let unchanged = a
            .marker(MarkerKind::Note, index)
            .is_some_and(|m| m.text == text);
        !unchanged && a.set_note_text(index, text)
    }) == Some(true);
    if set {
        commit(view, "Edit Note");
    }
    set
}

fn set_ruler(view: Pin<&mut PictureView>, ax: f64, ay: f64, bx: f64, by: f64, constrain: bool) {
    let ruler = if constrain {
        Ruler::constrained((ax, ay), (bx, by))
    } else {
        Ruler {
            a: (ax, ay),
            b: (bx, by),
        }
    };
    view.rust_mut().ruler = Some(ruler);
}

fn clear_ruler(view: Pin<&mut PictureView>) {
    view.rust_mut().ruler = None;
}

fn ruler_line(view: &PictureView) -> Vec<f64> {
    view.rust()
        .ruler
        .map_or_else(Vec::new, |r| vec![r.a.0, r.a.1, r.b.0, r.b.1])
}

fn ruler_measurement(view: &PictureView) -> Vec<f64> {
    view.rust().ruler.map_or_else(Vec::new, |r| {
        let m = r.measure();
        vec![m.x, m.y, m.width, m.height, m.angle, m.distance]
    })
}
