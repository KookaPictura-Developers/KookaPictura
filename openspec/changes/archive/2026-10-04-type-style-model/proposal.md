# Proposal

## Why

Kooka's type authoring model is a fixed five-attribute shape. `TypeSpec` (the
write side) carries only text, font, size, colour, justification, orientation,
anti-aliasing, origin, and the transform matrix; `TextStyle` (the read side)
carries only font, size, fill colour, tracking, and justification. Every other
character and paragraph attribute lives as a hardcoded constant in
`pictura-codec/src/type_write.rs` — tracking `0`, leading `size × 1.2`,
horizontal/vertical scale `1.0`, baseline shift `0`, kerning `0`, first-line and
margin indents `0`, space before/after `0`, `AutoHyphenate true`.

Because `pictura-render::type_layer::replace_type_layer` rebuilds the whole `TySh`
from scratch through `type_content` → `author_type_tool`, and `type_layer_spec`
reads back only font/size/colour/justification, opening a PSD type layer and
editing it silently resets every attribute the model does not carry. The
Character and Paragraph panels (issues #73/#74) cannot ship on top of this
without first widening the model, and the CS6 Character Styles / Paragraph Styles
panels (#76) have no model at all. This is the first change of the type-panels
program under umbrella issue #65.

## What Changes

- Add a character-attribute model: font family, font style, size, leading (auto
  or fixed), kerning (Metrics / Optical / manual), tracking, horizontal and
  vertical scale, baseline shift, anti-aliasing method (None / Sharp / Crisp /
  Strong / Smooth), fill colour, the All Caps, Small Caps, Superscript,
  Subscript, Underline, and Strikethrough toggles, and layer-wide Fractional
  Widths. Faux Bold/Italic and Asian/type-language attributes are deferred (their
  PSD keys and preferences are ungrounded; see Non-Goals).
- Add a paragraph-attribute model: horizontal/vertical alignment, justification,
  min/desired/max word spacing, letter spacing, glyph scaling, left/right/
  first-line indents, space before/after, hanging punctuation, hyphenation
  (default **off**), direction, and the single-line/every-line composer.
- Add character and paragraph **styles** with a `TextStyleSheet` holding CS6's
  **manual override > character style > paragraph style** resolution and the
  non-renamable **Basic Paragraph** default, stored on the document and undoable
  through the existing history snapshot.
- Drive `style_sheet`/`paragraph_sheet` authoring from the model instead of
  constants, and **preserve the unmodeled `EngineData` keys of the existing
  block** on a re-author by merging the parsed tree rather than regenerating it.
  Multi-run, multi-paragraph text is a documented ceiling.
- Extend the read side so the full attribute set is decoded and resolved against
  the sheet and paragraph defaults.
- Make the TySh `AntA` descriptor enum the canonical anti-aliasing method and
  replace the anti-aliasing `bool` with the method enum.
- Carry the attributes across the app bridge and record one history state per
  committed edit.

## Capabilities

### New Capabilities

- `document/type-style-model`: the in-memory character and paragraph attribute
  sets, the character/paragraph style definitions, the style hierarchy that
  resolves an effective value, and the app edit that applies them under one
  history state.

### Modified Capabilities

- `codec/psd-type-tool`: `style_sheet`/`paragraph_sheet` authoring is driven by
  the model, the anti-aliasing method is authored to both the `AntA` descriptor
  and the EngineData flag, and a re-author preserves the unmodeled EngineData
  keys of the parsed block rather than resetting them to constants.
- `tools/type-engine-data`: the first-run style decode exposes the full character
  and paragraph attribute set, resolved through the same hierarchy.

## Impact

- `crates/pictura-core`: new attribute/style types; `TypeSpec` and `TextStyle`
  gain fields; the anti-aliasing field changes type (`bool` → `AntiAlias`); a
  `TextStyleSheet` field on the document.
- `crates/pictura-codec`: `type_write.rs` authoring rewrite (from-model, merge
  over the parsed tree, `AntA` + `/AntiAlias`); a new `encode_engine_data`;
  `engine_data.rs` extraction extension; `type_tool.rs` view.
- `crates/pictura-render`: `type_layer.rs` (`type_content`,
  `replace_type_layer`, `type_layer_spec`, `render_type`) must thread the
  existing descriptor through the author and read the full model back; this is
  the crate where the destructive rebuild actually happens.
- `crates/pictura-app`: the type bridge carries a placement setting plus new
  character/paragraph settings; `type_tools.rs`, `tool_type.cpp`, the type
  options bar, and `tool_context.h` read/write them.
- No new crate: the panel spec's proposed `pictura-text` crate is not
  introduced; the model stays in `pictura-core` with serialization in
  `pictura-codec`, matching the current layout. No new dependencies.
- PSD round-trip of the styles themselves stays opaque (serialization is
  unresolved in the contract); only the per-run/per-paragraph attributes are
  modelled.

## Non-Goals (deferred, named so they are not silently dropped)

- Faux Bold/Italic, Asian options (tsume), type language, vertical underline
  variants, and paragraph direction. Their PSD keys or a preference surface do
  not exist yet; they belong with a preferences/type-tools follow-up.
- Serializing character/paragraph style definitions to PSD. The style block's
  location is unresolved; any existing block is preserved verbatim.
- Panels #73/#74/#75/#76 (separate changes that consume this model).
