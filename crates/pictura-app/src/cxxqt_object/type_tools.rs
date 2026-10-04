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
use pictura_core::{
    AntiAlias, CharacterAttrs, CharacterOverrides, Composer, Justify, KerningMode, Leading,
    ParagraphAttrs, ParagraphOverrides, StyleOverrides, TypeSpec,
};
use pictura_select::Selection;

#[cxx_qt::bridge]
pub mod ffi {
    /// Where and how the Type tools place the text: the click `(x, y)` in
    /// document pixels — the first baseline's start, or the first column's top
    /// centre for vertical type — the linear part `xx, xy, yx, yy` a
    /// transformed type layer reopens with (identity for new type), and the
    /// orientation. Placement rides separately from the character and paragraph
    /// attribute sets.
    #[namespace = "pictura"]
    struct TypeSetting {
        vertical: bool,
        x: f64,
        y: f64,
        xx: f64,
        xy: f64,
        yx: f64,
        yy: f64,
    }

    /// The character attribute set as the bridge carries it: enums as `i32`
    /// (`leading_mode` 0 Auto / 1 Fixed, `kerning_mode` 0 Metrics / 2 Manual —
    /// the legacy `1` also maps to Metrics because the model has no Optical,
    /// `anti_alias` 0 None … 4 Smooth), `color` as `0xAARRGGBB`, the toggles as
    /// `bool`, and the model's units for sizes, scales, and spacing. The font
    /// family and PostScript style ride separately: `font_style` here, the
    /// family on the bridge functions' dedicated `font` argument.
    #[namespace = "pictura"]
    struct CharacterSetting {
        font_style: QString,
        size: f64,
        leading_mode: i32,
        leading_value: f64,
        kerning_mode: i32,
        kerning_value: i32,
        tracking: f64,
        horizontal_scale: f64,
        vertical_scale: f64,
        baseline_shift: f64,
        anti_alias: i32,
        color: u32,
        all_caps: bool,
        small_caps: bool,
        superscript: bool,
        subscript: bool,
        underline: bool,
        strikethrough: bool,
        fractional_widths: bool,
    }

    /// The paragraph attribute set as the bridge carries it: `justify` 0 Left,
    /// 1 Right, 2 Center, 3 Justify Last Left, 4 Justify Last Right, 5 Justify
    /// Last Center, 6 Justify All; word, letter, and glyph spacing as min /
    /// desired / max in the model's units; `direction` is inert (kept for the
    /// generated C++ header); `composer` 0 SingleLine / 1 EveryLine.
    #[namespace = "pictura"]
    struct ParagraphSetting {
        justify: i32,
        word_spacing_min: f64,
        word_spacing_desired: f64,
        word_spacing_max: f64,
        letter_spacing_min: f64,
        letter_spacing_desired: f64,
        letter_spacing_max: f64,
        glyph_spacing_min: f64,
        glyph_spacing_desired: f64,
        glyph_spacing_max: f64,
        start_indent: f64,
        end_indent: f64,
        first_line_indent: f64,
        space_before: f64,
        space_after: f64,
        hanging: bool,
        hyphenate: bool,
        direction: i32,
        composer: i32,
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
        fn type_preview_rect(
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> Vec<i32>;

        /// The default character attribute set for new type, matching the model.
        fn type_default_character_setting() -> CharacterSetting;

        /// The default paragraph attribute set for new type, matching the model.
        fn type_default_paragraph_setting() -> ParagraphSetting;

        /// Register `bytes` (an sfnt Qt resolved for `family`) so type set in `family` renders in it; false for bytes that do not parse.
        fn type_register_font(family: &QString, bytes: &[u8]) -> bool;

        /// The caret at every UTF-16 boundary of `text` (`len + 1` segments), flat `[x0, y0, x1, y1, …]` in document pixels; empty for a non-positive size.
        fn type_caret_stops(
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> Vec<f64>;

        /// `text` rendered over its preview rect as packed straight RGBA; empty when nothing renders.
        fn type_preview_rgba(
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> Vec<u8>;

        /// Add `text` as a type layer above the active layer, make it the active layer, and record one "Horizontal Type" / "Vertical Type" state. Returns the new path; empty (no state) for blank text or a document that is not 8-bit RGB.
        fn type_commit_layer(
            view: Pin<&mut PictureView>,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> QString;

        /// The topmost visible type layer under `(x, y)`; empty when none.
        fn type_layer_at(view: &PictureView, x: f64, y: f64) -> QString;

        /// The text of the type layer at `path`, lines separated by `\r`; empty for a non-type layer.
        fn type_layer_text(view: &PictureView, path: &QString) -> QString;

        /// The font family the type layer at `path` names; empty for a non-type layer.
        fn type_layer_font(view: &PictureView, path: &QString) -> QString;

        /// Where the type layer at `path` sits, its origin following the layer; identity for a non-type layer.
        fn type_layer_setting(view: &PictureView, path: &QString) -> TypeSetting;

        /// The type layer at `path`'s character attribute set; the model defaults for a non-type layer.
        fn type_layer_character_setting(view: &PictureView, path: &QString) -> CharacterSetting;

        /// The type layer at `path`'s paragraph attribute set; the model defaults for a non-type layer.
        fn type_layer_paragraph_setting(view: &PictureView, path: &QString) -> ParagraphSetting;

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
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> bool;

        /// Re-set the type layer at `path` from `text` without reopening it (the options bar changed while it is selected) and record one "Edit Type Layer" state; false (no state) for blank text or a non-type layer.
        fn type_update_layer(
            view: Pin<&mut PictureView>,
            path: &QString,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> bool;

        /// Apply the saved character (`paragraph` false) or paragraph (`paragraph` true) style named `name` to the type layer at `path`, resolve it, and record one "Apply Type Style" state; false (no state) for a missing style or non-type layer.
        fn type_apply_style(
            view: Pin<&mut PictureView>,
            path: &QString,
            name: &QString,
            paragraph: bool,
        ) -> bool;

        /// Create the named character style from `character` and record one "New Character Style" state; false (no state) for a duplicate name.
        fn type_create_character_style(
            view: Pin<&mut PictureView>,
            name: &QString,
            font: &QString,
            character: &CharacterSetting,
        ) -> bool;

        /// Edit the named character style to `character`, re-resolve every applying type layer, and record one "Edit Character Style" state; false (no state) for a missing style.
        fn type_edit_character_style(
            view: Pin<&mut PictureView>,
            name: &QString,
            font: &QString,
            character: &CharacterSetting,
        ) -> bool;

        /// Delete the named character style and record one "Delete Character Style" state; false (no state) for a missing style.
        fn type_delete_character_style(view: Pin<&mut PictureView>, name: &QString) -> bool;

        /// Create the named paragraph style and record one "New Paragraph Style" state; false (no state) for a duplicate name.
        fn type_create_paragraph_style(
            view: Pin<&mut PictureView>,
            name: &QString,
            font: &QString,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> bool;

        /// Edit the named paragraph style, re-resolve every applying type layer, and record one "Edit Paragraph Style" state; false (no state) for a missing style.
        fn type_edit_paragraph_style(
            view: Pin<&mut PictureView>,
            name: &QString,
            font: &QString,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
        ) -> bool;

        /// Delete the named paragraph style and record one "Delete Paragraph Style" state; false (no state) for a missing or protected style.
        fn type_delete_paragraph_style(view: Pin<&mut PictureView>, name: &QString) -> bool;

        /// Merge `text`'s coverage into the selection with `mode` ("new", "add", "subtract", "intersect") and record one "Horizontal Type Mask" / "Vertical Type Mask" state. False (no state) for blank text or without a document.
        fn type_commit_mask(
            view: Pin<&mut PictureView>,
            text: &QString,
            font: &QString,
            setting: &TypeSetting,
            character: &CharacterSetting,
            paragraph: &ParagraphSetting,
            mode: &QString,
        ) -> bool;
    }
}

use ffi::{CharacterSetting, ParagraphSetting, TypeSetting};

fn rgba_color(color: u32) -> [f64; 4] {
    let [a, r, g, b] = color.to_be_bytes();
    [
        f64::from(a) / 255.0,
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    ]
}

fn character_attrs(setting: &CharacterSetting) -> CharacterAttrs {
    CharacterAttrs {
        font_style: setting.font_style.to_string(),
        size: setting.size,
        leading: if setting.leading_mode == 1 {
            Leading::Fixed(setting.leading_value)
        } else {
            Leading::Auto
        },
        kerning: match setting.kerning_mode {
            1 => KerningMode::Metrics,
            2 => KerningMode::Manual(setting.kerning_value),
            _ => KerningMode::Metrics,
        },
        tracking: setting.tracking,
        horizontal_scale: setting.horizontal_scale,
        vertical_scale: setting.vertical_scale,
        baseline_shift: setting.baseline_shift,
        anti_alias: AntiAlias::from_index(setting.anti_alias),
        fill_color: rgba_color(setting.color),
        all_caps: setting.all_caps,
        small_caps: setting.small_caps,
        superscript: setting.superscript,
        subscript: setting.subscript,
        underline: setting.underline,
        strikethrough: setting.strikethrough,
        fractional_widths: setting.fractional_widths,
        ..CharacterAttrs::default()
    }
}

fn paragraph_attrs(setting: &ParagraphSetting) -> ParagraphAttrs {
    ParagraphAttrs {
        justify: Justify::from_index(setting.justify.clamp(0, 6) as u8).unwrap_or_default(),
        word_spacing: [
            setting.word_spacing_min,
            setting.word_spacing_desired,
            setting.word_spacing_max,
        ],
        letter_spacing: [
            setting.letter_spacing_min,
            setting.letter_spacing_desired,
            setting.letter_spacing_max,
        ],
        glyph_spacing: [
            setting.glyph_spacing_min,
            setting.glyph_spacing_desired,
            setting.glyph_spacing_max,
        ],
        start_indent: setting.start_indent,
        end_indent: setting.end_indent,
        first_line_indent: setting.first_line_indent,
        space_before: setting.space_before,
        space_after: setting.space_after,
        hanging: setting.hanging,
        hyphenate: setting.hyphenate,
        composer: if setting.composer == 1 {
            Composer::EveryLine
        } else {
            Composer::SingleLine
        },
    }
}

fn character_setting(attrs: &CharacterAttrs) -> CharacterSetting {
    let [r, g, b, a] = attrs.rgba();
    CharacterSetting {
        font_style: QString::from(attrs.font_style.as_str()),
        size: attrs.size,
        leading_mode: match attrs.leading {
            Leading::Auto => 0,
            Leading::Fixed(_) => 1,
        },
        leading_value: match attrs.leading {
            Leading::Fixed(value) => value,
            Leading::Auto => 0.0,
        },
        kerning_mode: match attrs.kerning {
            KerningMode::Metrics => 0,
            KerningMode::Manual(_) => 2,
        },
        kerning_value: match attrs.kerning {
            KerningMode::Manual(value) => value,
            _ => 0,
        },
        tracking: attrs.tracking,
        horizontal_scale: attrs.horizontal_scale,
        vertical_scale: attrs.vertical_scale,
        baseline_shift: attrs.baseline_shift,
        anti_alias: attrs.anti_alias.index(),
        color: u32::from_be_bytes([a, r, g, b]),
        all_caps: attrs.all_caps,
        small_caps: attrs.small_caps,
        superscript: attrs.superscript,
        subscript: attrs.subscript,
        underline: attrs.underline,
        strikethrough: attrs.strikethrough,
        fractional_widths: attrs.fractional_widths,
    }
}

fn paragraph_setting(attrs: &ParagraphAttrs) -> ParagraphSetting {
    ParagraphSetting {
        justify: i32::from(attrs.justify.index()),
        word_spacing_min: attrs.word_spacing[0],
        word_spacing_desired: attrs.word_spacing[1],
        word_spacing_max: attrs.word_spacing[2],
        letter_spacing_min: attrs.letter_spacing[0],
        letter_spacing_desired: attrs.letter_spacing[1],
        letter_spacing_max: attrs.letter_spacing[2],
        glyph_spacing_min: attrs.glyph_spacing[0],
        glyph_spacing_desired: attrs.glyph_spacing[1],
        glyph_spacing_max: attrs.glyph_spacing[2],
        start_indent: attrs.start_indent,
        end_indent: attrs.end_indent,
        first_line_indent: attrs.first_line_indent,
        space_before: attrs.space_before,
        space_after: attrs.space_after,
        hanging: attrs.hanging,
        hyphenate: attrs.hyphenate,
        // Kept on the bridge struct so the generated C++ header is unchanged;
        // the model no longer carries a paragraph direction.
        direction: 0,
        composer: match attrs.composer {
            Composer::SingleLine => 0,
            Composer::EveryLine => 1,
        },
    }
}

fn type_default_character_setting() -> CharacterSetting {
    character_setting(&CharacterAttrs::default())
}

fn type_default_paragraph_setting() -> ParagraphSetting {
    paragraph_setting(&ParagraphAttrs::default())
}

fn spec(
    text: &QString,
    font: &QString,
    placement: &TypeSetting,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> TypeSpec {
    let mut spec = TypeSpec::new(text.to_string(), font.to_string(), character.size);
    spec.vertical = placement.vertical;
    spec.origin = (placement.x, placement.y);
    spec.matrix = [placement.xx, placement.xy, placement.yx, placement.yy];
    spec.character = character_attrs(character);
    spec.character.font_family = font.to_string();
    spec.paragraph = paragraph_attrs(paragraph);
    spec
}

fn type_preview_rect(
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> Vec<i32> {
    pictura_render::type_placement(&spec(text, font, setting, character, paragraph))
        .map_or_else(Vec::new, |p| {
            vec![p.rect.left, p.rect.top, p.rect.width(), p.rect.height()]
        })
}

fn type_register_font(family: &QString, bytes: &[u8]) -> bool {
    pictura_render::register_font(&family.to_string(), bytes.to_vec())
}

fn type_caret_stops(
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> Vec<f64> {
    pictura_render::type_caret_stops(&spec(text, font, setting, character, paragraph))
        .into_iter()
        .flatten()
        .collect()
}

fn type_preview_rgba(
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> Vec<u8> {
    let Some((_, buffer)) =
        pictura_render::render_type(&spec(text, font, setting, character, paragraph))
    else {
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
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> QString {
    let spec = spec(text, font, setting, character, paragraph);
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
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
    mode: &QString,
) -> bool {
    let spec = spec(text, font, setting, character, paragraph);
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
    layer_spec(view, path).map_or_else(QString::default, |s| {
        QString::from(s.character.font_family.as_str())
    })
}

fn placement_setting(spec: &TypeSpec) -> TypeSetting {
    TypeSetting {
        vertical: spec.vertical,
        x: spec.origin.0,
        y: spec.origin.1,
        xx: spec.matrix[0],
        xy: spec.matrix[1],
        yx: spec.matrix[2],
        yy: spec.matrix[3],
    }
}

fn type_layer_setting(view: &PictureView, path: &QString) -> TypeSetting {
    layer_spec(view, path).map_or(
        TypeSetting {
            vertical: false,
            x: 0.0,
            y: 0.0,
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
        },
        |spec| placement_setting(&spec),
    )
}

fn type_layer_character_setting(view: &PictureView, path: &QString) -> CharacterSetting {
    layer_spec(view, path).map_or_else(type_default_character_setting, |s| {
        character_setting(&s.character)
    })
}

fn type_layer_paragraph_setting(view: &PictureView, path: &QString) -> ParagraphSetting {
    layer_spec(view, path).map_or_else(type_default_paragraph_setting, |s| {
        paragraph_setting(&s.paragraph)
    })
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
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> bool {
    if !replace(
        view.as_mut(),
        path,
        text,
        font,
        setting,
        character,
        paragraph,
    ) {
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
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> bool {
    let replaced = replace(
        view.as_mut(),
        path,
        text,
        font,
        setting,
        character,
        paragraph,
    );
    if replaced {
        commit_edit(view);
    }
    replaced
}

fn type_apply_style(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    name: &QString,
    paragraph: bool,
) -> bool {
    let applied = view.as_mut().rust_mut().doc.as_mut().is_some_and(|doc| {
        pictura_render::apply_type_style(doc, &path.to_string(), &name.to_string(), paragraph)
    });
    if applied {
        view.as_mut().clear_link_sets();
        view.as_mut().recomposite();
        view.as_mut().record("Apply Type Style");
        view.as_mut().changed();
    }
    applied
}

fn character_overrides(attrs: &CharacterAttrs) -> CharacterOverrides {
    CharacterOverrides {
        font_family: Some(attrs.font_family.clone()),
        font_style: Some(attrs.font_style.clone()),
        size: Some(attrs.size),
        leading: Some(attrs.leading),
        kerning: Some(attrs.kerning),
        tracking: Some(attrs.tracking),
        horizontal_scale: Some(attrs.horizontal_scale),
        vertical_scale: Some(attrs.vertical_scale),
        baseline_shift: Some(attrs.baseline_shift),
        anti_alias: Some(attrs.anti_alias),
        fill_color: Some(attrs.fill_color),
        all_caps: Some(attrs.all_caps),
        small_caps: Some(attrs.small_caps),
        superscript: Some(attrs.superscript),
        subscript: Some(attrs.subscript),
        underline: Some(attrs.underline),
        strikethrough: Some(attrs.strikethrough),
        fractional_widths: Some(attrs.fractional_widths),
    }
}

fn paragraph_overrides(attrs: &ParagraphAttrs) -> ParagraphOverrides {
    ParagraphOverrides {
        justify: Some(attrs.justify),
        word_spacing: Some(attrs.word_spacing),
        letter_spacing: Some(attrs.letter_spacing),
        glyph_spacing: Some(attrs.glyph_spacing),
        start_indent: Some(attrs.start_indent),
        end_indent: Some(attrs.end_indent),
        first_line_indent: Some(attrs.first_line_indent),
        space_before: Some(attrs.space_before),
        space_after: Some(attrs.space_after),
        hanging: Some(attrs.hanging),
        hyphenate: Some(attrs.hyphenate),
        composer: Some(attrs.composer),
    }
}

/// The character style's sparse overrides, with the bridge's separate family.
fn style_character_overrides(font: &QString, character: &CharacterSetting) -> CharacterOverrides {
    let mut attrs = character_attrs(character);
    attrs.font_family = font.to_string();
    character_overrides(&attrs)
}

/// Run a style operation on the document and record one history state when it
/// changed anything.
fn commit_style(
    mut view: Pin<&mut PictureView>,
    label: &str,
    apply: impl FnOnce(&mut pictura_core::Document) -> bool,
) -> bool {
    let changed = view.as_mut().rust_mut().doc.as_mut().is_some_and(apply);
    if changed {
        view.as_mut().clear_link_sets();
        view.as_mut().recomposite();
        view.as_mut().record(label);
        view.as_mut().changed();
    }
    changed
}

fn type_create_character_style(
    view: Pin<&mut PictureView>,
    name: &QString,
    font: &QString,
    character: &CharacterSetting,
) -> bool {
    let name = name.to_string();
    let attrs = style_character_overrides(font, character);
    commit_style(view, "New Character Style", |doc| {
        crate::history::create_character_style(doc, &name, attrs).is_ok()
    })
}

fn type_edit_character_style(
    view: Pin<&mut PictureView>,
    name: &QString,
    font: &QString,
    character: &CharacterSetting,
) -> bool {
    let name = name.to_string();
    let attrs = style_character_overrides(font, character);
    commit_style(view, "Edit Character Style", |doc| {
        crate::history::edit_character_style(doc, &name, attrs).is_ok()
    })
}

fn type_delete_character_style(view: Pin<&mut PictureView>, name: &QString) -> bool {
    let name = name.to_string();
    commit_style(view, "Delete Character Style", |doc| {
        crate::history::delete_character_style(doc, &name).is_ok()
    })
}

fn type_create_paragraph_style(
    view: Pin<&mut PictureView>,
    name: &QString,
    font: &QString,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> bool {
    let name = name.to_string();
    let character = style_character_overrides(font, character);
    let paragraph = paragraph_overrides(&paragraph_attrs(paragraph));
    commit_style(view, "New Paragraph Style", |doc| {
        crate::history::create_paragraph_style(doc, &name, character, paragraph).is_ok()
    })
}

fn type_edit_paragraph_style(
    view: Pin<&mut PictureView>,
    name: &QString,
    font: &QString,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> bool {
    let name = name.to_string();
    let character = style_character_overrides(font, character);
    let paragraph = paragraph_overrides(&paragraph_attrs(paragraph));
    commit_style(view, "Edit Paragraph Style", |doc| {
        crate::history::edit_paragraph_style(doc, &name, character, paragraph).is_ok()
    })
}

fn type_delete_paragraph_style(view: Pin<&mut PictureView>, name: &QString) -> bool {
    let name = name.to_string();
    commit_style(view, "Delete Paragraph Style", |doc| {
        crate::history::delete_paragraph_style(doc, &name).is_ok()
    })
}

/// Record `after` as a manual override for one field only when it differs from
/// the layer's `before`; an unchanged field stays inherited from the styles.
fn changed<T: Clone + PartialEq>(current: &mut Option<T>, before: &T, after: &T) {
    if after != before {
        *current = Some(after.clone());
    }
}

/// The run's manual overrides after a bridge edit: the layer's existing
/// overrides plus every attribute that changed from its previous effective
/// value. Unchanged fields stay unset, so a later style application still
/// updates them.
fn edited_overrides(previous: &TypeSpec, next: &TypeSpec) -> StyleOverrides {
    let mut overrides = previous.overrides.clone();
    let (before, after) = (&previous.character, &next.character);
    let c = &mut overrides.character;
    changed(&mut c.font_family, &before.font_family, &after.font_family);
    changed(&mut c.font_style, &before.font_style, &after.font_style);
    changed(&mut c.size, &before.size, &after.size);
    changed(&mut c.leading, &before.leading, &after.leading);
    changed(&mut c.kerning, &before.kerning, &after.kerning);
    changed(&mut c.tracking, &before.tracking, &after.tracking);
    changed(
        &mut c.horizontal_scale,
        &before.horizontal_scale,
        &after.horizontal_scale,
    );
    changed(
        &mut c.vertical_scale,
        &before.vertical_scale,
        &after.vertical_scale,
    );
    changed(
        &mut c.baseline_shift,
        &before.baseline_shift,
        &after.baseline_shift,
    );
    changed(&mut c.anti_alias, &before.anti_alias, &after.anti_alias);
    changed(&mut c.fill_color, &before.fill_color, &after.fill_color);
    changed(&mut c.all_caps, &before.all_caps, &after.all_caps);
    changed(&mut c.small_caps, &before.small_caps, &after.small_caps);
    changed(&mut c.superscript, &before.superscript, &after.superscript);
    changed(&mut c.subscript, &before.subscript, &after.subscript);
    changed(&mut c.underline, &before.underline, &after.underline);
    changed(
        &mut c.strikethrough,
        &before.strikethrough,
        &after.strikethrough,
    );
    changed(
        &mut c.fractional_widths,
        &before.fractional_widths,
        &after.fractional_widths,
    );

    let (before, after) = (&previous.paragraph, &next.paragraph);
    let p = &mut overrides.paragraph;
    changed(&mut p.justify, &before.justify, &after.justify);
    changed(
        &mut p.word_spacing,
        &before.word_spacing,
        &after.word_spacing,
    );
    changed(
        &mut p.letter_spacing,
        &before.letter_spacing,
        &after.letter_spacing,
    );
    changed(
        &mut p.glyph_spacing,
        &before.glyph_spacing,
        &after.glyph_spacing,
    );
    changed(
        &mut p.start_indent,
        &before.start_indent,
        &after.start_indent,
    );
    changed(&mut p.end_indent, &before.end_indent, &after.end_indent);
    changed(
        &mut p.first_line_indent,
        &before.first_line_indent,
        &after.first_line_indent,
    );
    changed(
        &mut p.space_before,
        &before.space_before,
        &after.space_before,
    );
    changed(&mut p.space_after, &before.space_after, &after.space_after);
    changed(&mut p.hanging, &before.hanging, &after.hanging);
    changed(&mut p.hyphenate, &before.hyphenate, &after.hyphenate);
    changed(&mut p.composer, &before.composer, &after.composer);
    overrides
}

fn replace(
    mut view: Pin<&mut PictureView>,
    path: &QString,
    text: &QString,
    font: &QString,
    setting: &TypeSetting,
    character: &CharacterSetting,
    paragraph: &ParagraphSetting,
) -> bool {
    let previous = layer_spec(view.as_ref().get_ref(), path);
    let mut spec = spec(text, font, setting, character, paragraph);
    if let Some(previous) = previous {
        spec.overrides = edited_overrides(&previous, &spec);
    }
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
