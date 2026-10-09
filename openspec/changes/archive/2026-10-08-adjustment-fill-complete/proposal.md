# Proposal

## Why

The `Layer > New Adjustment Layer` submenu and the `adjustment_layer` bridge
already cover all sixteen CS6 adjustment kinds, but the Layers panel's
bottom-strip **New Fill / Adjustment** button lists only Solid Color, Gradient,
and five of the sixteen kinds, and labels them with raw engine ids. The
`Layer > Layer Content Options…` entry is a greyed stub even though the
Properties panel can already edit the active adjustment. The two creation
surfaces therefore disagree and the panel path is incomplete.

## What Changes

- The Layers panel's New Fill / Adjustment menu lists the two implemented fill
  kinds (Solid Color…, Gradient…), a deferred Pattern… entry, and all sixteen
  CS6 adjustment kinds with their proper display names, each wired to
  `add_adjustment`.
- The shared adjustment-kind table moves out of `frame_menus_adjust.cpp` into a
  small header so the Layer menu and the panel strip cannot drift.
- `Layer > Layer Content Options…` is enabled for the current fill or
  adjustment layer and opens that layer's Properties panel page; it is disabled
  for every other layer kind.
- **Deferred:** Pattern-fill authoring (`Layer > New Fill Layer > Pattern…` and
  the panel Pattern… entry). The engine can render and rasterize a `PtFl`
  pattern fill, but authoring needs a pattern preset/library and a picker that
  do not exist. The entries stay disabled with the `— not implemented yet`
  convention. No new capability is added for it.
- No breaking changes; no new dependency.

## Capabilities

### New Capabilities

<!-- None. Pattern-fill authoring is deferred; see design.md Open Questions. -->

### Modified Capabilities

- `imaging/adjustment-layers`: the creation-surface requirement broadens from
  "the New Adjustment Layer menu" to both creation surfaces (the `Layer` menu
  submenu and the Layers panel New Fill / Adjustment menu) offering all
  sixteen kinds.
- `ui/layers-panel`: adds the Fill / Adjustment strip-menu contents requirement
  and the `Layer Content Options…` behavior that opens the Properties panel for
  the current fill/adjustment layer.

## Impact

- `crates/pictura-app/cpp/layer_adjustments.h` (new): the shared
  `LayerAdjustment` table and its count.
- `crates/pictura-app/cpp/frame_menus_adjust.cpp`: drop the local table, include
  the shared header.
- `crates/pictura-app/cpp/panels/layers_panel.cpp`: build the full strip menu
  from the shared table, keep the fill entries, add the disabled Pattern… entry.
- `crates/pictura-app/cpp/commands.h` and `crates/pictura-app/cpp/command_tree.cpp`:
  a frozen `LayerContentOptions` id, implemented.
- `crates/pictura-app/cpp/frame_menus.cpp`: wire `Layer Content Options…` to show
  the Properties panel for the current adjustment/fill layer.
- `crates/pictura-app/cpp/tests/tst_layers_panel.cpp` and
  `crates/pictura-app/cpp/tests/tst_command_tree.cpp`: Qt Test coverage for the
  strip menu (all sixteen kinds, Pattern disabled) and for
  `Layer Content Options…`.
- `CMakeLists.txt`: register the new header.
- No engine/Rust change; no new dependency.
