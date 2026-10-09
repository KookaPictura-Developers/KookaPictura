# Design

## Context

See proposal.md — Why. Today `LayersPanel::populateRowMenu`
(`layers_panel_menu.cpp:70`) builds a fixed list; it reads `kind` only to gate
the two Export rows. `showContextMenu` (`:44`) already passes `kind` from the
`KindRole` model data, and the panel menu / strip already use a
`— not implemented yet` disabled-row convention (`panel_group_menu.cpp:277`,
`layers_panel.cpp:251`). `LayersPanel::performPanelMenuAction` (`:141`) is the
existing id-to-handler dispatch pattern to mirror.

## Goals / Non-Goals

- **Goal:** one declarative table per layer kind, walked by `populateRowMenu`,
  dispatched by a single row-action handler.
- **Goal:** unimplemented-but-applicable rows are disabled with the tooltip.
- **Non-Goal:** unifying the panel menu, strip, and row menu into one registry
  (later, if it earns its keep). The menu-bar `Layer` tree keeps its own
  `commands.cpp` mechanism.
- **Non-Goal:** any engine or bridge change.

## Decisions

### D1 — Static per-kind table, not a switch

A `struct RowSpec { const char* id; const char* label; KindMask kinds;
bool implemented; }` array plus a `KindMask forKind(const QString&)` helper.
`populateRowMenu` filters the array by the row's kind, in array order, and adds
separators at explicit id boundaries. Chosen over a per-kind `if (kind == …)`
ladder because each upcoming feature then adds one array entry instead of a
branch — the whole point of landing this first.

### D2 — One dispatch function

`performRowAction(const QString& id, const QString& path)` mirrors
`performPanelMenuAction`: a chain of `if (id == …)` calls into existing
`view_` operations and panel helpers (`duplicateSelection`, `addLayerAt`, …).
`populateRowMenu` connects every action to it with the row's path. Keeps the
table data-only.

### D3 — `implemented` is a table column, not a runtime predicate

Rows that need context (Merge, Export, mask-on-absent-mask) still gate at
dispatch time; the table's `implemented` flag only distinguishes "shipped" from
"planned", matching the existing convention. Richer enable predicates are added
per feature when a row needs them.

### D4 — Rename, Color Label, Export stay first-class

Rename needs the model index and color label is a submenu; both keep bespoke
code outside the table. Export becomes a kind-masked table row pair. Layout is
preserved: Rename, separator, creation/ordering rows, kind-specific rows,
separator, Color Label.

## Risks / Trade-offs

- **Menu length grows as rows are added** → group kind-specific rows under
  existing CS6 submenus (Layer Mask, Layer Style, Smart Objects) exactly as the
  menu bar does, rather than flattening.
- **Test seam churn** → `rowMenuTextsForTest` (`layers_panel_test.cpp:148`)
  currently enumerates the whole menu for one kind; extend it to take a kind.

## Migration Plan

None — internal to the panel; no persisted state.

## Open Questions

None blocking. Exact per-kind row membership beyond the currently-shipped set
is decided by each downstream change, which flips its rows' `implemented` flag.
