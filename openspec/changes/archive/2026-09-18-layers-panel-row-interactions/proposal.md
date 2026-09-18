## Why

A second UI pass on the Layers panel: the Opacity/Fill labels do not scrub the
value and the value omits `%`; the five padlock icons are indistinguishable
except by position; the Kind filter buttons have no icons; the visibility toggle
is a painted shape with no eye art and sits too far left; the group disclosure
chevron is barely visible; and rows still cannot be dragged, so a drag starts a
rubber-band multi-selection instead of a reorder / drop-on-button gesture.

## What Changes

- **Percent fields.** The Opacity and Fill label is part of the field and drags
  to scrub the value like the field itself; the value shows a `%` suffix.
- **Semantic lock icons.** Only **Lock All** is a padlock. The others are
  semantic glyphs: alpha = transparency checkerboard, paint = brush, position =
  move cross, nesting = nested squares.
- **Kind filter icons.** The Kind toggle buttons carry icons (pixel,
  adjustment, group, background) with the names as tooltips.
- **Eye icon.** The visibility toggle is a proper eye SVG (`layers.eyeOn`/
  `layers.eyeOff`), drawn slightly to the right of the row's left edge.
- **Group chevron.** A group row shows a clear disclosure icon
  (`layers.disclosureRight`/`layers.disclosureDown`), right when collapsed and
  down when expanded.
- **Drag and drop.** Rows are draggable. Dragging does **not** extend the
  selection (no rubber-band), the drag reorders/reparents the row (drop above,
  below, or onto a group), and dropping a row on a bottom-strip button applies
  that button's action to the dragged row(s): Delete deletes, New Layer
  duplicates, New Group groups. **BREAKING** (engine): a new
  `move_layer_to(path, target, mode)` bridge op and `move_path_to` engine op
  reparent a node (refusing the Background, fully/nesting-locked nodes, and a
  node dropped into its own descendant).

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `layers-panel`: the percent-field label/scrub/`%`, the semantic lock icons,
  the eye icon and offset, the group disclosure chevron, and the row drag-and-
  drop reorder plus drop-on-strip behavior.
- `layers-filtering-search`: the Kind toggle buttons carry icons.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/properties.rs` —
  `move_path_to` + tests.
- `crates/pictura-app/src/cxxqt_object/impl_layers.rs` — `move_layer_to`.
- `crates/pictura-app/cpp/panels/{percent_field,layers_panel,layers_panel_internal,layers_filter_bar}.{h,cpp}`
  and `layers_panel_test.cpp` — the widgets, icons, and drag/drop plumbing.
- `crates/pictura-app/cpp/selftest_layers_controls.cpp` and assets/qrc — self-
  tests and icons.
- No new dependencies.
