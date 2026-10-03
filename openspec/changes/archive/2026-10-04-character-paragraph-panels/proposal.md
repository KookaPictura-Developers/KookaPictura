# Proposal

## Why

The Character and Paragraph panels are CS6 Core (`docs/02-ui-ux/panels/character-and-paragraph.md`,
`PAN-022`). The type model and its bridge now exist (change `type-style-model`),
but there are no panels: `Window > Panels > Character`/`Paragraph` and
`Type > Panels > Character`/`Paragraph` are disabled stubs
(`command_tree.cpp:798,821,489,490`), and nothing lets a user edit a type
layer's attributes. Issues #73 (Character) and #74 (Paragraph).

## What Changes

- Add a **Character panel**: font family, size, leading (auto or fixed), kerning,
  tracking, horizontal/vertical scale, baseline shift, anti-aliasing, colour, and
  the All Caps / Small Caps / Superscript / Subscript / Underline / Strikethrough
  toggles — bound to the active type layer.
- Add a **Paragraph panel**: alignment/justification, left/right/first-line
  indents, space before/after, hanging punctuation, hyphenation, and composer.
- Enable `Window > Panels > Character`/`Paragraph` and
  `Type > Panels > Character`/`Paragraph` as checkable toggles that show/hide the
  panels in the panel column and keep their checked state in sync.
- Show the active type layer's values and edit it live (one history state per
  committed change); when no type layer is active, show the model defaults and
  disable editing.

## Capabilities

### New Capabilities

- `ui/character-paragraph-panels`: the Character and Paragraph panel widgets and
  their menu/toggle wiring, as attribute editors over the type model.

### Modified Capabilities

- None.

## Impact

- `crates/pictura-app/cpp`: new `panels/character_panel.{h,cpp}` and
  `panels/paragraph_panel.{h,cpp}`; `commands.h` panel ids;
  `command_tree.cpp` (un-stub the Window/Type entries); `frame_build.cpp`
  (build, register, group); `frame_menus.cpp` (toggle handlers/checked
  providers); `frame.h`; `CMakeLists.txt`; a Qt Test suite.
- No new engine or bridge work: the panels consume the existing
  `type_layer_character_setting` / `type_layer_paragraph_setting` /
  `type_layer_text` / `type_layer_font` / `type_layer_setting` /
  `type_update_layer` bridge functions.
- No new dependencies. No `docs/` change.
