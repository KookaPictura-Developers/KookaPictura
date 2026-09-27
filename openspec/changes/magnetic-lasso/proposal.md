# Proposal: magnetic-lasso

## Why

The Magnetic Lasso (`ToolId::MagneticLasso`) was catalogued but disabled
("no edge map or fastening-point tracker"), issue #2. photorust
(perfecto25/photorust) ships a working live-wire engine and shell interaction;
this change ports them onto Kooka's selection engine, tool framework, and lasso
commit path, against the behaviour in `docs/03-tools/lasso-selection.md`.

## What Changes

- Add `pictura_select::EdgeMap`: a Sobel edge-cost field from the composite
  (Contrast threshold) and a corridor-bounded Dijkstra `trace` (Width), falling
  back to a straight line when there is nothing to snap to.
- Add a second cxx-qt bridge, `cxxqt_object/magnetic.rs`
  (`magnetic_begin/trace/end`), caching the field per gesture on the
  `PictureView` state; the size-capped `cxxqt_object.rs` does not grow. A
  32-bit document refuses.
- Add `tool_magneticlasso.cpp`: click to start and fasten, a live wire on hover,
  automatic fastening by Frequency, close on the first point / double-click /
  Enter, Delete peels fastening points back, Escape cancels. The outline commits
  through `begin_lasso`/`end_lasso`, so feather, combine modes, and history are
  shared with the other lassos.
- Options bar: Width (1–256 px), Contrast (1–100 %), Frequency (0–100), a
  disabled Stylus Pressure box; `[` / `]` step Width by 1 px.
- Enable the catalog entry, the hint row, and selection-tool routing; a press
  inside a live selection extends an open outline instead of starting a
  selection move.
- The options bar now sizes to the active tool's page only (hidden pages no
  longer set the window's minimum width).
- C++ self-test `magnetic_lasso` (code 530); the unimplemented-tool guard
  (code 98) now probes Perspective Crop.

## Capabilities

### New Capabilities

- `tools/magnetic-lasso`: the Magnetic Lasso engine, options, and interaction.

### Modified Capabilities

- `tools/select-menu`: drops the "Magnetic Lasso SHALL remain disabled" clause.
- `tools/shape-selection-tools`: removes "Deferred selection tools stay visible
  and disabled" (its only subject is now implemented).

## Impact

- New `crates/pictura-select/src/magnetic.rs`; `pictura_select::EdgeMap`.
- App: `build.rs`, `cxxqt_object.rs` (`mod magnetic;`), `cxxqt_object/state.rs`,
  new `cxxqt_object/magnetic.rs`, new `tool_magneticlasso.cpp`,
  `tool_handler.h`, `tool_context.h`, `tools.{h,cpp}`, `tools_selection_move.cpp`,
  `tool_catalog.cpp`, `options_bar.{h,cpp}`, `frame.cpp`, `frame_build.cpp`,
  new `selftest_magnetic_lasso.{h,cpp}`, `selftest_layers_controls.cpp`,
  `selftest.cpp`, `CMakeLists.txt`.
- No new dependency.
