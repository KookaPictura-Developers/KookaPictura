# M20 — Panels (Layers, History, Navigator, Color/Swatches, Info/Histogram)

Goal: replace the M16 debug dock with real panels backed by the document and
history models: a Layers panel with property editing, a labeled History panel
with jump/snapshots, a Navigator, Color + Swatches, and Info + Histogram.
OpenSpec change `m20-panels` (new capabilities `layers-panel`, `history-panel`,
`navigator-panel`, `color-swatches-panel`, `info-histogram-panel`).

## Scope

- Rust (`history.rs`, bridge): labels on history states, `count`/`index`/
  `label(i)`/`jump(i)`/`add_snapshot`; layer blend/opacity/name setters,
  `move_layer`, `layer_thumbnail`.
- `cpp/panels/`: `layers_panel`, `history_panel`, `navigator_panel`,
  `color_panel` (+`ColorState`), `swatches_panel`, `info_panel`,
  `histogram_panel`.
- `image_view`: public arbitrary-zoom setter for the Navigator.
- Frame: register the panels as docks, bind them to the active document, own the
  `ColorState`, feed the Eyedropper into it, and expose `Window > Panels`
  toggles.
- Self-test: layer property/thumbnail, history label/jump, panel presence/toggle.

## Out of scope (later milestones)

- Group-tree expansion, drag-reorder, layer lock flags, clipping/link/labels,
  filter/search row.
- Swatch library file I/O, Info colour samplers, Histogram source/cache states,
  History branching beyond the bounded stack.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Sub-agents own all code, in waves: (1a) history + layer bridge,
(1b) Navigator/Color/Swatches/Info/Histogram panels, (2) Layers + History panels,
(3) frame integration and commands, (4) self-test.

## Verification

- `cmake -S . -B build && cmake --build build`
- fixture and no-argument self-tests exit 0 with new panel checks (exit codes
  from 50)
- `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
