# Design

## Context

See `proposal.md` for motivation. Relevant current state:

- Panels are content widgets hosted by `PanelColumn`/`PanelGroup`
  (`frame_build.cpp`): constructed, `registerPanel(panel, area)`, added to a
  group with `addPanel(group, panel, title, icon)`, and toggled by objectName
  through `PanelColumn::showPanel`/`isPanelVisible`.
- `Window > Panels > …` and `Type > Panels > …` are `CommandSpec`s built in
  `command_tree.cpp`; the live ones are wired by the `kPanelToggles` table in
  `frame_menus.cpp:1063` (handler + checked provider). `Character`/`Paragraph`
  are disabled `leaf` stubs today.
- The bridge already exposes everything the panels need:
  `type_layer_text`, `type_layer_font`, `type_layer_setting`,
  `type_layer_character_setting`, `type_layer_paragraph_setting`,
  `type_update_layer`, and `type_default_character_setting` /
  `type_default_paragraph_setting` (`cxxqt_object/type_tools.rs`).
- Contract: `docs/02-ui-ux/panels/character-and-paragraph.md`.

## Goals / Non-Goals

**Goals:**

- Character and Paragraph panel widgets that display and edit the active type
  layer, and menu + options-bar toggles to show/hide them.

**Non-Goals:**

- Glyphs (#75), Paragraph/Character **Styles** panels (#76), and the type-styles
  UI.
- Run/selection-level editing — the panels edit the whole active type layer
  (the model is single-run; a run selection is a later change).
- Editing the defaults for new type while no type layer is active (the panels
  show the defaults and are disabled).
- Font enumeration UI beyond a family combo; style/faux pickers.

## Decisions

### Panels are PanelColumn content widgets

Add `CharacterPanel` and `ParagraphPanel` (`panels/character_panel.{h,cpp}`,
`panels/paragraph_panel.{h,cpp}`), each a `QWidget` with
`setView(PictureView*)` and `refresh()`, object names `characterPanel` /
`paragraphPanel`. Register them and place them in a hidden-by-default panel
group (CS6 opens them from `Type > Panels`/`Window > Character`). Editing is
disabled unless the active layer is a type layer (`type_layer_setting` size > 0);
otherwise the controls show `type_default_*_setting()`.

### Commit through `type_update_layer`

A shared `apply()` reads the active path, `type_layer_text`, `type_layer_font`,
and `type_layer_setting`, fills the edited `CharacterSetting`/`ParagraphSetting`,
and calls `type_update_layer`, which records one `"Edit Type Layer"` state.
`refresh()` wraps control updates in `QSignalBlocker` so showing a layer's
values never echoes an edit. Controls commit on `editingFinished` /
`currentIndexChanged` / toggled.

Changing the font family must register the resolved face first, exactly as the
type tool does (`type_register_font` from the sfnt Qt resolves for the family);
factor/reuse that small helper rather than duplicating the `QRawFont` extraction.

### Menus and options bar

Add `window.panels.character` / `window.panels.paragraph` ids in `commands.h`.
Replace the Window `leaf` stubs and the Type `leaf` stubs with enabled, checkable
`CommandSpec`s (Window entries use the `WindowPanels*` ids; Type entries either
reuse the same ids or add `type.panels.*` mirrored by a checked provider — prefer
reusing the Window ids so both menu paths share one toggle). Append both to
`kPanelToggles`. The type options bar's Panel button already emits
`panelToggleRequested`; wire a Panel button to toggle both panels.

## Risks / Trade-offs

- [A refresh that emits a control signal could record a spurious history state] →
  block signals during `refresh`; commit only user-driven changes.
- [Changing the font family without registering the face renders a fallback] →
  reuse the type tool's font-registration helper.
- [History spam while a spinner is scrolled] → numeric fields commit on
  `editingFinished`, not on every `valueChanged`.
- [New icon ids could trip the provenance guard] → reuse an existing mapped icon
  id for the panel tabs; do not add icon files.
