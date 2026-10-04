//! The Filter-menu bridges: apply a filter with runtime parameters, render a
//! non-committing preview, and keep the last filter. Free functions over a
//! [`PictureView`] (their own bridge, so the `PictureView` declaration list does
//! not grow). Ported from perfecto25/photorust's filter commit path.
//!
//! Source: https://github.com/perfecto25/photorust
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::{active_layer_visible, active_pixel_layer};
use super::impl_filters::{apply_filter_active, apply_filter_active_region, cancel_filter_preview};
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString};
use pictura_core::layer_pixel_locked;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;

        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// Commit a filter `kind` with an ordered `params` slot list. When a
        /// preview is open the layer already holds the result, so only one
        /// `"Filter"` history state is recorded; otherwise the filter is applied
        /// fresh. Stores the filter for Last Filter. Returns false on an unknown
        /// kind, a bad arity, or no editable pixel layer.
        fn apply_filter_params(
            view: Pin<&mut PictureView>,
            kind: &QString,
            params: &QList_f64,
        ) -> bool;

        /// Preview a filter `kind` with `params` on the canvas without recording
        /// history. Each call re-filters the pre-preview pixels, so parameter
        /// changes do not compound. Returns false when the active layer is not
        /// an unlocked normal pixel layer or the parameters are invalid.
        fn filter_preview(view: Pin<&mut PictureView>, kind: &QString, params: &QList_f64) -> bool;

        /// Preview `kind` restricted to the document rect `(x, y, w, h)` (the
        /// visible viewport), expanded by the filter's support. Cheaper than a
        /// full-layer preview while exact across the visible area. Positional
        /// kinds (see [`filter_preview_needs_whole_layer`]) ignore the rect and
        /// preview the whole layer, because a crop changes their result.
        fn filter_preview_section(
            view: Pin<&mut PictureView>,
            kind: &QString,
            params: &QList_f64,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
        ) -> bool;

        /// Discard an open preview, restoring the pre-preview pixels
        /// bit-identically. Returns false when no preview is open.
        fn filter_preview_cancel(view: Pin<&mut PictureView>) -> bool;

        /// Whether a filter has been committed this session.
        fn filter_has_last(view: Pin<&mut PictureView>) -> bool;

        /// The last committed filter's kind, or an empty string.
        fn filter_last_kind(view: Pin<&mut PictureView>) -> QString;

        /// The last committed filter's parameter slot values, or an empty list.
        fn filter_last_params(view: Pin<&mut PictureView>) -> QList_f64;

        /// Whether the active layer can take a destructive filter (an unlocked
        /// normal pixel layer in an open document).
        fn filter_target_ready(view: Pin<&mut PictureView>) -> bool;

        /// Whether `kind` is a filter the parameterised mapper understands.
        fn filter_kind_supported(kind: &QString) -> bool;

        /// The number of parameter slots `kind` expects, or -1 for an unknown
        /// kind.
        fn filter_param_arity(kind: &QString) -> i32;

        /// The reason the most recent apply/preview was refused, or an empty
        /// string when it succeeded.
        fn filter_last_error(view: Pin<&mut PictureView>) -> QString;
    }
}

fn apply_filter_params(
    mut view: Pin<&mut PictureView>,
    kind: &QString,
    params: &QList<f64>,
) -> bool {
    let kind_s = kind.to_string();
    let params_v: Vec<f64> = params.into_iter().copied().collect();
    let region = {
        let mut rust = view.as_mut().rust_mut();
        apply_filter_active(&mut rust, &kind_s, &params_v, true)
    };
    let Some(region) = region else {
        let reason = view.rust().filter_error.clone().unwrap_or_default();
        eprintln!("pictura: filter '{kind_s}' refused: {reason}");
        return false;
    };
    match region {
        Some(rect) => view.as_mut().refresh_region(rect),
        None => view.as_mut().recomposite(),
    }
    view.as_mut().record("Filter");
    true
}

fn filter_preview(mut view: Pin<&mut PictureView>, kind: &QString, params: &QList<f64>) -> bool {
    let kind_s = kind.to_string();
    let params_v: Vec<f64> = params.into_iter().copied().collect();
    let region = {
        let mut rust = view.as_mut().rust_mut();
        apply_filter_active(&mut rust, &kind_s, &params_v, false)
    };
    let Some(region) = region else {
        return false;
    };
    match region {
        Some(rect) => view.as_mut().refresh_region(rect),
        None => view.as_mut().recomposite(),
    }
    true
}

/// Filters that resolve pixel position or a global statistic against the whole
/// layer, so a cropped section preview would not match the commit: Diffuse
/// hashes the pixel's absolute coordinates, Lighting resolves its light span
/// against the crop, and HDR Toning computes a global pivot.
/// ponytail: `lens-flare` places its centre as a fraction of the buffer, a
/// pre-existing case of the same class, left out to keep this fix scoped.
pub(crate) fn filter_preview_needs_whole_layer(kind: &str) -> bool {
    matches!(kind, "diffuse" | "lighting-effects" | "hdr-toning")
}

fn filter_preview_section(
    mut view: Pin<&mut PictureView>,
    kind: &QString,
    params: &QList<f64>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> bool {
    let kind_s = kind.to_string();
    let params_v: Vec<f64> = params.into_iter().copied().collect();
    let visible = pictura_core::PsdRect {
        top: y,
        left: x,
        bottom: y + h.max(0),
        right: x + w.max(0),
    };
    let section = (!filter_preview_needs_whole_layer(&kind_s)).then_some(visible);
    let region = {
        let mut rust = view.as_mut().rust_mut();
        apply_filter_active_region(&mut rust, &kind_s, &params_v, false, section)
    };
    let Some(region) = region else {
        return false;
    };
    match region {
        Some(rect) => view.as_mut().refresh_region(rect),
        None => view.as_mut().recomposite(),
    }
    true
}

fn filter_preview_cancel(mut view: Pin<&mut PictureView>) -> bool {
    let region = {
        let mut rust = view.as_mut().rust_mut();
        cancel_filter_preview(&mut rust)
    };
    let Some(region) = region else {
        return false;
    };
    match region {
        Some(rect) => view.as_mut().refresh_region(rect),
        None => view.as_mut().recomposite(),
    }
    true
}

fn filter_has_last(view: Pin<&mut PictureView>) -> bool {
    view.rust().last_filter.is_some()
}

fn filter_last_kind(view: Pin<&mut PictureView>) -> QString {
    view.rust()
        .last_filter
        .as_ref()
        .map(|(kind, _)| QString::from(kind.as_str()))
        .unwrap_or_default()
}

fn filter_last_params(view: Pin<&mut PictureView>) -> QList<f64> {
    view.rust()
        .last_filter
        .as_ref()
        .map(|(_, params)| params.iter().copied().collect())
        .unwrap_or_default()
}

fn filter_target_ready(view: Pin<&mut PictureView>) -> bool {
    let rust = view.rust();
    let Some(doc) = rust.doc.as_ref() else {
        return false;
    };
    let active = rust.active_layer.as_deref();
    active_layer_visible(doc, active)
        && active_pixel_layer(doc, active).is_some_and(|layer| !layer_pixel_locked(layer))
}

fn filter_kind_supported(kind: &QString) -> bool {
    super::filter_map::filter_param_arity(&kind.to_string()).is_some()
}

fn filter_param_arity(kind: &QString) -> i32 {
    super::filter_map::filter_param_arity(&kind.to_string())
        .map(|n| n as i32)
        .unwrap_or(-1)
}

fn filter_last_error(view: Pin<&mut PictureView>) -> QString {
    view.rust()
        .filter_error
        .as_deref()
        .map(QString::from)
        .unwrap_or_default()
}
