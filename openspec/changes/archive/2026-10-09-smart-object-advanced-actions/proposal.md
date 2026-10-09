# Proposal

## Why

Two Smart Object layer actions from the CS6 command set (#114) and the
New Smart Object via Copy item of #106 are still missing: **Reset Transform**,
**Convert to Layers**, and an independent **New Smart Object via Copy**. The
engine already embeds, places, renders, edits, replaces, and rasterizes smart
objects, so these three complete the `Layer > Smart Objects` program without new
file-format work.

## What Changes

- **Reset Transform** — restore the smart object's placement/rotation/scale to
  the embedded source's native transform and re-render the object from that
  source. The native transform is the embedded source document's size at the
  origin; no transform history is stored.
- **Convert to Layers** — replace the smart object with the layers of its
  embedded source, scaled and positioned into the object's `rect`. An embedded
  source with no layers (a raster payload) becomes one raster layer.
- **New Smart Object via Copy** — duplicate a smart object so the copy carries
  an independent embedded source; editing one no longer affects the other. A
  plain duplicate deep-copies the payload but keeps a preserved link record, so
  the copy also drops its preserved `SoLd`/`SoLE` link and re-authors its own.
- Wire the three actions as `Layer > Smart Objects` menu commands and as
  type-aware smart-object rows in the Layers panel context menu.
- Each successful command records exactly one undo state.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `compositing/smart-object-layer-actions`: add Reset Transform, Convert to
  Layers, and New Smart Object via Copy engine operations, their eligibility and
  refusal contracts, and their one-undo-state app commands.
- `ui/layers-panel`: the row context menu offers the three new smart-object
  rows, enabled, for a smart-object row.

## Impact

- `crates/pictura-render/src/document_ops/layer_ops/smart_object.rs` (three new
  operations + predicates), `.../layer_ops/mod.rs` (re-exports), and
  `crates/pictura-render/src/tests/smart_object.rs` (unit tests).
- `crates/pictura-app/src/cxxqt_object/impl_layers_smart_object.rs` (bridge
  invokables), keeping `cxxqt_object.rs` at its ceiling.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, `frame_menus.cpp`
  (menu wiring), and `panels/layers_panel_menu.cpp` (type-aware rows).
- `crates/pictura-app/cpp/tests/tst_command_tree.cpp` and a new
  `tst_smart_object_actions.cpp` (Qt Test), plus `tests/CMakeLists.txt`.
- No new dependencies. No `docs/` change.
