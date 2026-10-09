# Proposal

## Why

Smart Filters exist end-to-end for display and enable toggles, but the stack is
read-only: a user cannot add, delete, reorder, or clear filters the way CS6
allows, and the filter-mask pixels never reach the compositor. Finishing the
authoring half of Smart Filters (issue #166) is required before the feature can
be described as implemented rather than imported.

## What Changes

- Add codec descriptor edits that delete, reorder, and clear the entries of a
  preserved `SoLd`/`SoLE` `filterFX` list and keep the typed `smart_filters` view
  in step, so an edited stack round-trips through save and re-read.
- Add engine document operations `add_smart_filter`, `delete_smart_filter`,
  `reorder_smart_filters`, and `clear_smart_filters` over a resolved layer path.
- Add a `Layer > Smart Filter > Clear Smart Filters` command (`layer.smartFilters.clear`)
  that clears the current smart object's filters and records exactly one undo
  state.
- Record the deferred filter-mask pixel work: the mask is not decoded and no
  mask menu leaf is wired until a reference PSD carrying `FEid`/`FXid`/`FMsk`
  exists.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `codec/psd-smart-filters` — the preserved `filterFX` list can now be edited
  (delete / reorder / clear) as well as toggled; the requirement change is new
  behavior added to the capability.
- `compositing/smart-object-layer-actions` — new smart-filter document
  operations and the Clear Smart Filters command.
- `ui/layers-panel` — the Smart Filter layer-menu surface gains the
  Clear Smart Filters command.

## Impact

- `crates/pictura-codec/src/smart_filter.rs` gains delete/reorder/clear.
- `crates/pictura-render/src/document_ops/smart_filters.rs` (new) exposes the
  engine operations; `lib.rs` re-exports them.
- `crates/pictura-app/src/cxxqt_object/layers_smart_filters.rs` gains the
  delete / clear / reorder bridge rows.
- `crates/pictura-app/cpp/commands.h`, `command_tree.cpp`, `frame_menus.cpp`
  gain the Clear Smart Filters command.
- No dependencies, no `docs/` changes, no new self-test exit codes.
- Deferred (not in this change): decoding filter-mask pixels from
  `FEid`/`FXid`/`FMsk`, routing arbitrary Filter-menu filters into the smart
  stack, and panel drag-reorder / mask thumbnail. See `design.md`.
