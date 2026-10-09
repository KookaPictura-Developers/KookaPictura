//! The Layer Style dialog and the Layer > Layer Style commands. Free functions
//! over a [`PictureView`] (their own bridge, so the `PictureView` declaration
//! list does not grow).
//!
//! The dialog edits live: each control writes one `"<effect>.<field>"` value
//! and recomposites without a history state; OK records the visit as one state
//! and Cancel re-applies the current state, dropping every live edit. The
//! copied style is app-wide, as CS6's is, so it pastes across documents.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};
use std::sync::Mutex;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;

        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// Whether the layer at `path` can carry a style (not a group or the Background).
        fn layer_style_can_edit(view: &PictureView, path: &QString) -> bool;

        /// Whether the layer at `path` carries any effect, on or off.
        fn layer_style_has(view: &PictureView, path: &QString) -> bool;

        /// Whether the projection row at `i` carries any effect, on or off.
        fn layer_row_has_style(view: &PictureView, i: i32) -> bool;

        /// The ten effect dialog keys, in CS6 list order.
        fn layer_style_effect_names() -> QStringList;

        /// The value of `key` (`"<effect>.<field>"`, `"<effect>.on"`, `"fx.visible"` or `"blending.<option>"`) on the layer at `path`; colours pack as 0xRRGGBB, blend modes and choices are indices. NaN for an unknown key or path.
        fn layer_style_value(view: &PictureView, path: &QString, key: &QString) -> f64;

        /// Set `key` on the layer at `path` and recomposite, without a history state. False when refused or unchanged.
        fn layer_style_set(
            view: Pin<&mut PictureView>,
            path: &QString,
            key: &QString,
            value: f64,
        ) -> bool;

        /// The built-in patterns a Pattern Overlay offers, in `patternOverlay.pattern` order.
        fn layer_style_pattern_names() -> QStringList;

        /// Record the live edits as one state named `label`.
        fn layer_style_commit(view: Pin<&mut PictureView>, label: &QString);

        /// Drop every live edit by re-applying the current history state.
        fn layer_style_cancel(view: Pin<&mut PictureView>);

        /// Copy the style of the layer at `path` to the app-wide style clipboard. False when it cannot carry one.
        fn layer_style_copy(view: &PictureView, path: &QString) -> bool;

        /// Whether a copied style is waiting to be pasted.
        fn layer_style_can_paste() -> bool;

        /// Replace the style of every layer in `paths` with the copied one. Returns how many changed.
        fn layer_style_paste(view: Pin<&mut PictureView>, paths: &QStringList) -> i32;

        /// Remove every effect from `paths`. Returns how many changed.
        fn layer_style_clear(view: Pin<&mut PictureView>, paths: &QStringList) -> i32;

        /// Scale the pixel-sized effect parameters on `paths` by `percent`. Returns how many changed.
        fn layer_style_scale(view: Pin<&mut PictureView>, paths: &QStringList, percent: f64)
            -> i32;

        /// Whether any layer's effects are currently shown (`visible`) or hidden by the master switch.
        fn layer_style_any_visible(view: &PictureView, visible: bool) -> bool;

        /// Show or hide every layer's effects. Returns how many layers changed.
        fn layer_style_set_all_visible(view: Pin<&mut PictureView>, visible: bool) -> i32;
    }
}

/// The app-wide copied style (Copy Layer Style / Paste Layer Style).
static COPIED: Mutex<Option<pictura_render::LayerStyle>> = Mutex::new(None);

fn with_layer<T>(
    view: &PictureView,
    path: &QString,
    f: impl FnOnce(&pictura_core::Layer) -> T,
) -> Option<T> {
    let doc = view.rust().doc.as_ref()?;
    pictura_render::resolve_path(doc, &path.to_string()).map(f)
}

fn layer_style_can_edit(view: &PictureView, path: &QString) -> bool {
    with_layer(view, path, pictura_render::copy_layer_style)
        .flatten()
        .is_some()
}

fn layer_style_has(view: &PictureView, path: &QString) -> bool {
    with_layer(view, path, pictura_render::has_layer_style).unwrap_or(false)
}

fn layer_row_has_style(view: &PictureView, i: i32) -> bool {
    let path = view.layer_row_path(i);
    !path.is_empty() && layer_style_has(view, &path)
}

fn layer_style_effect_names() -> QStringList {
    let mut names = QStringList::default();
    for name in pictura_render::layer_style_effect_names() {
        names.append(QString::from(name));
    }
    names
}

fn layer_style_value(view: &PictureView, path: &QString, key: &QString) -> f64 {
    let key = key.to_string();
    with_layer(view, path, |layer| {
        pictura_render::layer_style_value(layer, &key)
    })
    .flatten()
    .unwrap_or(f64::NAN)
}

fn layer_style_set(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    key: &QString,
    value: f64,
) -> bool {
    let (path, key) = (path.to_string(), key.to_string());
    let changed =
        view.as_mut().rust_mut().doc.as_mut().is_some_and(|doc| {
            pictura_render::set_document_layer_style_value(doc, &path, &key, value)
        });
    if changed {
        view.as_mut().recomposite();
        let mut rust = view.as_mut().rust_mut();
        rust.content_revision = rust.content_revision.wrapping_add(1);
    }
    changed
}

fn layer_style_pattern_names() -> QStringList {
    let mut names = QStringList::default();
    for name in pictura_render::layer_style_pattern_names() {
        names.append(QString::from(name));
    }
    names
}

fn layer_style_commit(view: Pin<&mut PictureView>, label: &QString) {
    view.record(&label.to_string());
}

fn layer_style_cancel(mut view: Pin<&mut PictureView>) {
    let current = view.history_index();
    view.as_mut().history_jump(current);
}

fn copied() -> std::sync::MutexGuard<'static, Option<pictura_render::LayerStyle>> {
    COPIED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn layer_style_copy(view: &PictureView, path: &QString) -> bool {
    let Some(style) = with_layer(view, path, pictura_render::copy_layer_style).flatten() else {
        return false;
    };
    *copied() = Some(style);
    true
}

fn layer_style_can_paste() -> bool {
    copied().is_some()
}

fn layer_style_paste(view: Pin<&mut PictureView>, paths: &QStringList) -> i32 {
    let Some(style) = copied().clone() else {
        return 0;
    };
    view.batch_changed(paths, "Paste Layer Style", |doc, paths| {
        pictura_render::paste_layer_style(doc, paths, &style)
    })
}

fn layer_style_clear(view: Pin<&mut PictureView>, paths: &QStringList) -> i32 {
    view.batch_changed(
        paths,
        "Clear Layer Style",
        pictura_render::clear_layer_style,
    )
}

fn layer_style_scale(view: Pin<&mut PictureView>, paths: &QStringList, percent: f64) -> i32 {
    view.batch_changed(paths, "Scale Effects", |doc, paths| {
        pictura_render::scale_layer_effects(doc, paths, percent)
    })
}

fn layer_style_any_visible(view: &PictureView, visible: bool) -> bool {
    view.rust()
        .doc
        .as_ref()
        .is_some_and(|doc| pictura_render::any_effects_visible(doc, visible))
}

fn layer_style_set_all_visible(mut view: Pin<&mut PictureView>, visible: bool) -> i32 {
    let changed = view.as_mut().rust_mut().doc.as_mut().map_or(0, |doc| {
        pictura_render::set_all_effects_visible(doc, visible)
    });
    if changed > 0 {
        view.as_mut().recomposite();
        let label = if visible {
            "Show All Effects"
        } else {
            "Hide All Effects"
        };
        view.as_mut().record(label);
    }
    changed as i32
}
