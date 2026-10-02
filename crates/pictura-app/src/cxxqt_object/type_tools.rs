//! The Type tools' commands: Horizontal / Vertical Type and their Type Mask
//! twins. Free functions over a [`PictureView`] (their own bridge, so the
//! `PictureView` declaration list does not grow).
//!
//! While text is being typed the canvas shows `type_preview_*` — the engine's
//! own render, so the preview is the commit — and nothing touches the document
//! or history. `type_commit_layer` adds one type layer and records one
//! "Horizontal Type" / "Vertical Type" state; `type_commit_mask` merges the
//! type's coverage into the selection and records one "Horizontal Type Mask" /
//! "Vertical Type Mask" state. Clicking an existing type layer reopens it:
//! `type_edit_begin` hides it (no state) while its text is retyped over it,
//! `type_commit_edit` re-sets it in place as one "Edit Type Layer" state, and
//! `type_edit_cancel` shows it again unchanged.
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers::combine_mode_from;
use super::qobject::PictureView;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::TypeSpec;
use pictura_select::Selection;

#[cxx_qt::bridge]
pub mod ffi {
    /// How the Type tools set the text: `size` in pixels, `color` as
    /// `0xAARRGGBB`, `justification` 0 left / top, 1 right / bottom, 2 centre,
    /// the click `(x, y)` in document pixels — the first baseline's start, or
    /// the first column's top centre for vertical type — and the linear part
    /// `xx, xy, yx, yy` a transformed type layer reopens with (identity for
    /// new type).
    #[namespace = "pictura"]
    struct TypeSetting {
        size: f64,
        color: u32,
        justification: i32,
        vertical: bool,
        antialias: bool,
        x: f64,
        y: f64,
        xx: f64,
        xy: f64,
        yx: f64,
        yy: f64,
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// The rect `text` occupies as `[left, top, width, height]` in document pixels; empty when nothing would render.
        fn type_preview_rect(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<i32>;

        /// Register `bytes` (an sfnt Qt resolved for `family`) so type set in `family` renders in it; false for bytes that do not parse.
        fn type_register_font(family: &QString, bytes: &[u8]) -> bool;

        /// The caret at every UTF-16 boundary of `text` (`len + 1` segments), flat `[x0, y0, x1, y1, …]` in document pixels; empty for a non-positive size.
        fn type_caret_stops(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<f64>;

        /// `text` rendered over its preview rect as packed straight RGBA; empty when nothing renders.
        fn type_preview_rgba(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<u8>;

        /// Add `text` as a type layer above the active layer, make it the active layer, and record one "Horizontal Type" / "Vertical Type" state. Returns the new path; empty (no state) for blank text or a document that is not 8-bit RGB.
        fn type_commit_layer(
            view: Pin<&mut PictureView>,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
        ) -> QString;

        /// The topmost visible type layer under `(x, y)`; empty when none.
        fn type_layer_at(view: &PictureView, x: f64, y: f64) -> QString;

        /// The text of the type layer at `path`, lines separated by `\r`; empty for a non-type layer.
        fn type_layer_text(view: &PictureView, path: &QString) -> QString;

        /// The font family the type layer at `path` names; empty for a non-type layer.
        fn type_layer_font(view: &PictureView, path: &QString) -> QString;

        /// How the type layer at `path` is set, its origin following the layer; `size` 0 for a non-type layer.
        fn type_layer_setting(view: &PictureView, path: &QString) -> TypeSetting;

        /// Hide the type layer at `path` while it is retyped. No history; false for a non-type layer.
        fn type_edit_begin(view: Pin<&mut PictureView>, path: &QString) -> bool;

        /// Show the type layer at `path` again, unchanged. No history.
        fn type_edit_cancel(view: Pin<&mut PictureView>, path: &QString);

        /// Re-set the type layer at `path` from `text` and record one "Edit Type Layer" state. Blank text or a failed re-set shows the layer again unchanged and returns false (no state).
        fn type_commit_edit(
            view: Pin<&mut PictureView>,
            path: &QString,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
        ) -> bool;

        /// Re-set the type layer at `path` from `text` without reopening it (the options bar changed while it is selected) and record one "Edit Type Layer" state; false (no state) for blank text or a non-type layer.
        fn type_update_layer(
            view: Pin<&mut PictureView>,
            path: &QString,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
        ) -> bool;

        /// Merge `text`'s coverage into the selection with `mode` ("new", "add", "subtract", "intersect") and record one "Horizontal Type Mask" / "Vertical Type Mask" state. False (no state) for blank text or without a document.
        fn type_commit_mask(
            view: Pin<&mut PictureView>,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            mode: &QString,
        ) -> bool;
    }
}

use ffi::TypeSetting;

fn spec(text: &QString, font: &QString, setting: &TypeSetting) -> TypeSpec {
    let [a, r, g, b] = setting.color.to_be_bytes();
    TypeSpec {
        text: text.to_string(),
        font: font.to_string(),
        size: setting.size,
        color: [r, g, b, a],
        justification: setting.justification.clamp(0, 2) as u8,
        vertical: setting.vertical,
        antialias: setting.antialias,
        origin: (setting.x, setting.y),
        matrix: [setting.xx, setting.xy, setting.yx, setting.yy],
    }
}

fn type_preview_rect(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<i32> {
    pictura_render::type_placement(&spec(text, font, setting)).map_or_else(Vec::new, |p| {
        vec![p.rect.left, p.rect.top, p.rect.width(), p.rect.height()]
    })
}

fn type_register_font(family: &QString, bytes: &[u8]) -> bool {
    pictura_render::register_font(&family.to_string(), bytes.to_vec())
}

fn type_caret_stops(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<f64> {
    pictura_render::type_caret_stops(&spec(text, font, setting))
        .into_iter()
        .flatten()
        .collect()
}

fn type_preview_rgba(text: &QString, font: &QString, setting: &TypeSetting) -> Vec<u8> {
    let Some((_, buffer)) = pictura_render::render_type(&spec(text, font, setting)) else {
        return Vec::new();
    };
    let plane = buffer.pixel_count();
    (0..plane)
        .flat_map(|i| (0..4).map(move |c| c * plane + i))
        .map(|i| buffer.data[i])
        .collect()
}

fn type_commit_layer(
    mut view: Pin<&mut PictureView>,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
) -> QString {
    let spec = spec(text, font, setting);
    let created = {
        let mut rust = view.as_mut().rust_mut();
        let above = rust.active_layer.clone().unwrap_or_default();
        let created = match rust.doc.as_mut() {
            Some(doc) => pictura_render::add_type_layer(doc, &above, &spec),
            None => String::new(),
        };
        if !created.is_empty() {
            rust.active_layer = Some(created.clone());
        }
        created
    };
    if !created.is_empty() {
        view.as_mut().clear_link_sets();
        view.as_mut().recomposite();
        view.as_mut().record(if spec.vertical {
            "Vertical Type"
        } else {
            "Horizontal Type"
        });
        view.as_mut().changed();
    }
    QString::from(created.as_str())
}

fn type_commit_mask(
    mut view: Pin<&mut PictureView>,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
    mode: &QString,
) -> bool {
    let spec = spec(text, font, setting);
    let shape = {
        let rust = view.rust();
        let Some(doc) = rust.doc.as_ref() else {
            return false;
        };
        let Some(data) = pictura_render::type_mask(doc.width, doc.height, &spec) else {
            return false;
        };
        Selection {
            width: doc.width,
            height: doc.height,
            data,
        }
    };
    let label = if spec.vertical {
        "Vertical Type Mask"
    } else {
        "Horizontal Type Mask"
    };
    view.as_mut()
        .apply_selection_labeled(shape, combine_mode_from(&mode.to_string()), label)
}

fn layer_spec(view: &PictureView, path: &QString) -> Option<TypeSpec> {
    let doc = view.rust().doc.as_ref()?;
    pictura_render::type_layer_spec(pictura_render::resolve_path(doc, &path.to_string())?)
}

fn type_layer_at(view: &PictureView, x: f64, y: f64) -> QString {
    view.rust()
        .doc
        .as_ref()
        .and_then(|doc| pictura_render::type_layer_at(doc, x, y))
        .map_or_else(QString::default, |path| QString::from(path.as_str()))
}

fn type_layer_text(view: &PictureView, path: &QString) -> QString {
    layer_spec(view, path).map_or_else(QString::default, |s| QString::from(s.text.as_str()))
}

fn type_layer_font(view: &PictureView, path: &QString) -> QString {
    layer_spec(view, path).map_or_else(QString::default, |s| QString::from(s.font.as_str()))
}

fn type_layer_setting(view: &PictureView, path: &QString) -> TypeSetting {
    let Some(spec) = layer_spec(view, path) else {
        return TypeSetting {
            size: 0.0,
            color: 0,
            justification: 0,
            vertical: false,
            antialias: true,
            x: 0.0,
            y: 0.0,
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
        };
    };
    let [r, g, b, a] = spec.color;
    TypeSetting {
        size: spec.size,
        color: u32::from_be_bytes([a, r, g, b]),
        justification: i32::from(spec.justification),
        vertical: spec.vertical,
        antialias: spec.antialias,
        x: spec.origin.0,
        y: spec.origin.1,
        xx: spec.matrix[0],
        xy: spec.matrix[1],
        yx: spec.matrix[2],
        yy: spec.matrix[3],
    }
}

fn set_visible(mut view: Pin<&mut PictureView>, path: &QString, visible: bool) -> bool {
    let changed = view
        .as_mut()
        .rust_mut()
        .doc
        .as_mut()
        .and_then(|doc| pictura_render::resolve_path_mut(doc, &path.to_string()))
        .filter(|layer| layer.type_tool.is_some())
        .map(|layer| layer.visible = visible)
        .is_some();
    if changed {
        view.as_mut().recomposite();
    }
    changed
}

fn type_edit_begin(view: Pin<&mut PictureView>, path: &QString) -> bool {
    set_visible(view, path, false)
}

fn type_edit_cancel(view: Pin<&mut PictureView>, path: &QString) {
    set_visible(view, path, true);
}

fn type_commit_edit(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
) -> bool {
    if !replace(view.as_mut(), path, text, font, setting) {
        type_edit_cancel(view, path);
        return false;
    }
    set_shown(view.as_mut(), path);
    commit_edit(view);
    true
}

fn type_update_layer(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
) -> bool {
    let replaced = replace(view.as_mut(), path, text, font, setting);
    if replaced {
        commit_edit(view);
    }
    replaced
}

fn replace(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
) -> bool {
    let spec = spec(text, font, setting);
    !spec.text.trim().is_empty()
        && view
            .as_mut()
            .rust_mut()
            .doc
            .as_mut()
            .is_some_and(|doc| pictura_render::replace_type_layer(doc, &path.to_string(), &spec))
}

fn commit_edit(mut view: Pin<&mut PictureView>) {
    view.as_mut().clear_link_sets();
    view.as_mut().recomposite();
    view.as_mut().record("Edit Type Layer");
    view.as_mut().changed();
}

/// Show the layer at `path` again after `type_edit_begin` hid it; no composite.
fn set_shown(mut view: Pin<&mut PictureView>, path: &QString) {
    if let Some(layer) = view
        .as_mut()
        .rust_mut()
        .doc
        .as_mut()
        .and_then(|doc| pictura_render::resolve_path_mut(doc, &path.to_string()))
    {
        layer.visible = true;
    }
}
