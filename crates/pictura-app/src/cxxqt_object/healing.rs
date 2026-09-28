//! The healing bridges: Spot Healing Brush, Healing Brush, and Patch. Free functions
//! over a [`PictureView`] (their own bridge, so the `PictureView` declaration
//! list does not grow). A gesture accumulates a coverage mask
//! ([`HealStroke`]); `healing_commit` runs the solve on the layer, recomposites,
//! and records one `"Spot Healing Brush"` / `"Healing Brush"` history state.
//! `patch_selection` heals through the selection instead of a stroke mask and
//! records one `"Patch Tool"` state.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use pictura_paint::healing::{patch_layer, HealMode, HealStroke, PatchOptions, Transfer};

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// Start a healing gesture on the active pixel layer at `diameter` /
        /// `hardness`. False without a lone active raster layer or when its
        /// pixels are locked.
        fn healing_begin(view: Pin<&mut PictureView>, diameter: i32, hardness: i32) -> bool;

        /// Add a dab at document-space `(x, y)`. False while no gesture is
        /// active.
        fn healing_dab(view: Pin<&mut PictureView>, x: f64, y: f64) -> bool;

        /// Run the heal on release and commit it. `mode` is 0/1/2 for the Spot
        /// Healing types; for the Healing Brush pass the sample offset in
        /// `source_x`/`source_y` and `use_source` true. Records one history
        /// state; false when nothing was healed.
        fn healing_commit(
            view: Pin<&mut PictureView>,
            mode: i32,
            use_source: bool,
            source_x: i32,
            source_y: i32,
            texture_only: bool,
        ) -> bool;

        /// Drop an active gesture without committing (Escape, tool switch).
        fn healing_cancel(view: Pin<&mut PictureView>);

        /// Whether a gesture is active.
        fn healing_active(view: &PictureView) -> bool;

        /// Patch the active pixel layer through the selection after a drag of
        /// `(dx, dy)`: Source repairs the selection from the dragged-to area,
        /// Destination applies the selection there, Content-Aware ignores the
        /// drag and rebuilds the selection from its surroundings. Records one
        /// "Patch Tool" state; false (no state) without a selection, on a zero
        /// drag outside Content-Aware, or on a locked / non-raster layer.
        fn patch_selection(
            view: Pin<&mut PictureView>,
            dx: i32,
            dy: i32,
            content_aware: bool,
            destination: bool,
            transparent: bool,
        ) -> bool;
    }
}

fn healing_begin(mut view: Pin<&mut PictureView>, diameter: i32, hardness: i32) -> bool {
    let begun = {
        let rust = view.rust();
        let Some(doc) = rust.doc.as_ref() else {
            return false;
        };
        let Some(path) = rust.active_layer.as_deref() else {
            return false;
        };
        HealStroke::begin(
            doc,
            path,
            diameter.max(1) as u32,
            hardness.clamp(0, 100) as u8,
        )
    };
    match begun {
        Ok(stroke) => {
            let mut rust = view.as_mut().rust_mut();
            rust.heal_stroke = Some(stroke);
            true
        }
        Err(_) => false,
    }
}

fn healing_dab(mut view: Pin<&mut PictureView>, x: f64, y: f64) -> bool {
    let mut rust = view.as_mut().rust_mut();
    let Some(stroke) = rust.heal_stroke.as_mut() else {
        return false;
    };
    // A dab only grows the mask; the pixels change on commit.
    stroke.dab(x as f32, y as f32);
    true
}

fn healing_commit(
    mut view: Pin<&mut PictureView>,
    mode: i32,
    use_source: bool,
    source_x: i32,
    source_y: i32,
    texture_only: bool,
) -> bool {
    let source = use_source.then_some((source_x, source_y));
    let transfer = if texture_only {
        Transfer::TextureOnly
    } else {
        Transfer::Full
    };
    let heal_mode = HealMode::from_i32(mode).unwrap_or_default();
    let outcome = {
        let stroke = view.as_mut().rust_mut().heal_stroke.take();
        let Some(stroke) = stroke else {
            return false;
        };
        let mut rust = view.as_mut().rust_mut();
        let Some(doc) = rust.doc.as_mut() else {
            return false;
        };
        match stroke.commit(doc, heal_mode, source, transfer) {
            Ok(outcome) => outcome,
            Err(_) => return false,
        }
    };
    let Some(outcome) = outcome else {
        return false;
    };
    let label = if source.is_some() {
        "Healing Brush"
    } else {
        "Spot Healing Brush"
    };
    {
        let mut rust = view.as_mut().rust_mut();
        rust.doc = Some(outcome.document);
    }
    view.as_mut().refresh_region(outcome.dirty);
    view.as_mut().record(label);
    view.as_mut().changed();
    true
}

fn healing_cancel(mut view: Pin<&mut PictureView>) {
    view.as_mut().rust_mut().heal_stroke = None;
}

fn healing_active(view: &PictureView) -> bool {
    view.rust().heal_stroke.is_some()
}

fn patch_selection(
    mut view: Pin<&mut PictureView>,
    dx: i32,
    dy: i32,
    content_aware: bool,
    destination: bool,
    transparent: bool,
) -> bool {
    let options = PatchOptions {
        dx,
        dy,
        content_aware,
        destination,
        transparent,
    };
    let dirty = {
        let mut rust = view.as_mut().rust_mut();
        let rust = &mut *rust;
        let (Some(doc), Some(selection), Some(path)) = (
            rust.doc.as_mut(),
            rust.selection.as_ref(),
            rust.active_layer.as_deref(),
        ) else {
            return false;
        };
        match patch_layer(doc, path, &selection.data, options) {
            Ok(Some(dirty)) => dirty,
            _ => return false,
        }
    };
    view.as_mut().refresh_region(dirty);
    view.as_mut().record("Patch Tool");
    view.as_mut().changed();
    true
}
