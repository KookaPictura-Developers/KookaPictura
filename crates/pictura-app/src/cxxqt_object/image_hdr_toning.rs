//! Image > Adjustments > HDR Toning: the Local Adaptation neighborhood operator.
//!
//! A free-function bridge over [`PictureView`]: the nine Local Adaptation
//! controls preview on the canvas through the Filter menu's preview machinery
//! (re-applied from the pre-preview pixels, restricted to the visible section)
//! and apply as one "HDR Toning" state. The kernel lives in
//! `pictura_filters::hdr_toning`.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::impl_filters::apply_filter_obj_active_region;
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::PsdRect;
use pictura_filters::{Filter, HdrToningParams};

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
        /// Preview HDR Toning's Local Adaptation on the active layer over the
        /// document rect `(x, y, w, h)` (the whole layer when `w` is not
        /// positive), re-applied from the pre-preview pixels; no history.
        /// False when refused.
        #[allow(clippy::too_many_arguments)]
        fn hdr_toning_preview(
            view: Pin<&mut PictureView>,
            radius: f64,
            strength: f64,
            gamma: f64,
            exposure: f64,
            detail: f64,
            shadow: f64,
            highlight: f64,
            vibrance: f64,
            saturation: f64,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
        ) -> bool;

        /// Apply HDR Toning's Local Adaptation to the whole active layer as one
        /// `label` state (an open preview is replaced). False when refused.
        #[allow(clippy::too_many_arguments)]
        fn hdr_toning_apply(
            view: Pin<&mut PictureView>,
            radius: f64,
            strength: f64,
            gamma: f64,
            exposure: f64,
            detail: f64,
            shadow: f64,
            highlight: f64,
            vibrance: f64,
            saturation: f64,
            label: &QString,
        ) -> bool;
    }
}

#[allow(clippy::too_many_arguments)]
fn params(
    radius: f64,
    strength: f64,
    gamma: f64,
    exposure: f64,
    detail: f64,
    shadow: f64,
    highlight: f64,
    vibrance: f64,
    saturation: f64,
) -> HdrToningParams {
    HdrToningParams {
        radius,
        strength,
        gamma,
        exposure,
        detail,
        shadow,
        highlight,
        vibrance,
        saturation,
    }
}

/// Run HDR Toning (a preview over `preview` when given) and refresh the canvas;
/// records `label` on a commit.
fn run(
    mut view: Pin<&mut PictureView>,
    params: HdrToningParams,
    preview: Option<PsdRect>,
    label: Option<&str>,
) -> bool {
    let commit = label.is_some();
    let region = {
        let mut rust = view.as_mut().rust_mut();
        apply_filter_obj_active_region(&mut rust, Filter::HdrToning(params), commit, preview)
    };
    let Some(region) = region else {
        return false;
    };
    match region {
        Some(rect) => view.as_mut().refresh_region(rect),
        None => view.as_mut().recomposite(),
    }
    if let Some(label) = label {
        view.as_mut().record(label);
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn hdr_toning_preview(
    view: Pin<&mut PictureView>,
    radius: f64,
    strength: f64,
    gamma: f64,
    exposure: f64,
    detail: f64,
    shadow: f64,
    highlight: f64,
    vibrance: f64,
    saturation: f64,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> bool {
    let section = (w > 0 && h > 0).then_some(PsdRect {
        top: y,
        left: x,
        bottom: y + h,
        right: x + w,
    });
    run(
        view,
        params(
            radius, strength, gamma, exposure, detail, shadow, highlight, vibrance, saturation,
        ),
        section,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn hdr_toning_apply(
    view: Pin<&mut PictureView>,
    radius: f64,
    strength: f64,
    gamma: f64,
    exposure: f64,
    detail: f64,
    shadow: f64,
    highlight: f64,
    vibrance: f64,
    saturation: f64,
    label: &QString,
) -> bool {
    run(
        view,
        params(
            radius, strength, gamma, exposure, detail, shadow, highlight, vibrance, saturation,
        ),
        None,
        Some(&label.to_string()),
    )
}
