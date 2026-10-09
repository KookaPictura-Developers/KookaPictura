# Design

## Context

The Layers panel is a pure view over the document: the bridge projects the layer
tree into flat rows, `LayersModel` stores one `LayerRow` per node, and
`LayerRowDelegate` paints from `LayerRole`s. Panel Options are opaque session
state (`SessionState`, `state.json`). The `Layer` menu is a declarative command
table (`command_tree.cpp`) wired by `frame_menus*.cpp`.

## Colour-label chip

CS6 renders a non-`None` color label as a small colour **bar next to the layer
thumbnail** (Adobe: "The color appears as a bar next to the layer or group
thumbnail"). The existing delegate only tints the eye toggle
(`lpr_label_tint`), so this change keeps that tint and adds a chip.

- Add `int labelChipAdvance(const QModelIndex&)` to `LayerRowDelegate`: the chip
  width plus gap when the row's `ColorRole` resolves to a valid label, else `0`.
- Insert the advance after `contentLeft()` in `paint`, `thumbRect`, and
  `nameRect`, so the chip sits between the disclosure/clipping glyph and the
  thumbnail and every downstream rect stays aligned.
- In `paint`, fill a `chipRect` (a few px wide, row-height) with the opaque label
  colour before drawing the thumbnail.
- Expose `labelChipRect(itemRect, index)` for the Qt test, mirroring the paint
  math.

The chip is a paint-only affordance; it is not clickable, so no hit-testing
changes beyond rect alignment.

## Smart-object badge

`layer_row_kind` does not distinguish smart objects (an embedded smart object is
kind `"pixel"`), and `layer_row_placed` is only true for `External`/`Alias`.
Add a dedicated predicate.

- New secondary cxx-qt bridge file
  `src/cxxqt_object/impl_layers/layers_surface.rs` (its own `.cxxqt.h`,
  registered in `build.rs`; nested under `impl_layers` so no declaration is
  added to `cxxqt_object.rs`, which stays at 1227) exposing
  `layer_row_is_smart_object(view, i) -> bool` — true when the row's layer has
  `smart_object.is_some()`.
- `LayerRow` gains `bool smartObject`; a `SmartObjectRole` is added to
  `LayerRole`; `refresh()` populates it from the bridge.
- The delegate paints `layers.kindSmartObject` on the thumbnail's lower-right
  corner, mirroring the existing shape-layer badge (`LayerRowShapeRole`).

## Panel Options

Two options, both default on, persisted through `SessionState` as
`layersAddCopyOnDuplicate` and `layersUseDefaultMasksOnFill` (plus `load`/`save`
keys). The dialog and `setOptionsForTest` gain the two checkboxes.

Because the options gate bridge behaviour but the bridge methods are called from
many C++ sites (menu, panel, selftests) without an options argument, the flags
live on the view: `PictureViewRust` gains `add_copy: bool` and
`use_default_masks: bool` (both default `true`), set through a
`set_layers_panel_options(view, add_copy, use_default_masks)` free function in
`layers_surface.rs`. `LayersPanel::setView` and `persistOptions`/
`openPanelOptions` push the current values, so every view the panel binds gains
the settings without touching the call sites.

- **Duplicate naming.** `duplicate_layer` / `duplicate_layers` read
  `add_copy`; when false, each created layer's name has the engine's `" copy"`
  suffix stripped (the engine always appends it), restoring the original name.
  No engine signature changes.
- **Default masks.** `add_adjustment` uses the flag to decide whether to derive
  a mask from the active selection (it previously always did). `add_solid_fill`
  and `add_gradient_fill` derive the same `selection_to_mask` and assign it to
  the created layer when the flag is on; the engine signatures are unchanged.

## Tab / Shift+Tab rename navigation

The spec already defines this behaviour (`m39_rename`); only the wiring is
missing. Rename editors are built by `QStyledItemDelegate`, and the view installs
the delegate itself as the editor's event filter. That filter's base
implementation commits `Tab`/`Shift+Tab` and closes the editor **without moving**
(`closeEditor(NoHint)`), so a plain panel-side filter never sees the key.

`LayerRowDelegate` overrides `eventFilter`: on `Tab`/`Backtab` it commits the
editor and closes it with `EditNextItem` / `EditPreviousItem` (which the view
turns into `moveCursor` + reopen) when `QTreeView::indexBelow` / `indexAbove`
reports a visible neighbour, and with `NoHint` at either end so there is no wrap.

## Lock commands

- `commands.h`: `LayerLockLayersAll`, `LayerLockLayersTransparency`,
  `LayerLockLayersImage`, `LayerLockLayersPosition`, `LayerLockAllInGroup`.
- `command_tree.cpp`: a `Layer > Lock Layers` submenu with the four lock types,
  and convert the greyed `Lock All Layers In Group…` leaf to the frozen id.
- `frame_menus_layer_ops.cpp` (or a sibling) wires the four to the existing
  `set_layers_lock(paths, flag, true)` over the panel selection; each is one
  undo state. `Lock All Layers In Group…` calls a new bridge free function
  `lock_group_layers_run(view, group_path)` that resolves the group's descendant
  paths and applies `"all"` in one `set_lock_paths` batch. The group is the
  current row when it is a group, else the current row's parent group.

## Deferred (dependencies, not omissions)

- **Blend-If badge** and **`Alt`-click FX show/hide-all**: both need the
  layer-effects authoring model (an editable effect list per layer). The panel
  cannot show or toggle what the document cannot yet represent. *Resolves with:*
  the layer-effects authoring change.
- **Vector-mask thumbnail**: needs the layer's vector-mask data surfaced on the
  row, which the vector-mask change owns. *Resolves with:* the vector-mask
  change.
- **Filter Effect dimension**: the filter bar's Effect dimension filters by
  applied layer effects; with no effect model there is nothing to filter on.
  *Resolves with:* the layer-effects authoring change.

## Open Questions

- The exact CS6 `Lock All Layers in Group…` dialog (which lock types it offers)
  is not in the corpus. This change applies the full `"all"` lock set without a
  dialog, consistent with the existing "one state" command contract; a dialog
  can replace it once the screenshot/text source is available.
- The colour chip's exact width/padding is a design choice (CS6 gives no UI
  metrics); the implementation uses a small fixed bar aligned with the
  thumbnail.
