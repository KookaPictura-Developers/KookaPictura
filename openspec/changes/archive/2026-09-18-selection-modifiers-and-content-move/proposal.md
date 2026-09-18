## Why

The selection tools accept keyboard modifiers in CS6 but not here: Shift/Alt do
not add or subtract mid-gesture, the marquee has no square/from-centre
constraints, the Polygonal Lasso's rubber band is hidden until commit, and the
Move tool still only slides a whole layer even when a selection is active. The
engine primitives to move selection pixels already exist (`layer_via_copy` /
`layer_via_cut`), so the gap is modifier handling, preview rendering, a hover
cursor, and one content-move operation.

## What Changes

- **Combine quick keys**: while a selection tool drags, Shift = Add, Alt =
  Subtract, and Shift+Alt = Intersect, decided at the first press when a
  selection already exists (no selection means New) and locked for the whole
  gesture until it commits. The options-bar mode stays the default when no
  modifier is held.
- **Marquee geometry**: Shift constrains the Rectangular/Elliptical Marquee drag
  to a square/circle; Alt draws from the press point as a centre (pivot) and
  mirrors the region around it. A floating size readout (`W × H`) follows the
  cursor during the drag for both tools.
- **Polygonal Lasso preview**: the in-progress path is a solid polyline from the
  first vertex through each clicked vertex to the live cursor position (a
  rubber band); the phantom closing edge still appears only at commit.
- **Hover cursor**: the move-selection cursor appears as the pointer enters the
  selected area and reverts when it leaves, without needing a click.
- **Move selected pixels**: the Move tool drags the active selection's pixels
  (cut from the source layer); Alt duplicates them to a new layer instead. A
  selection tool with Ctrl performs the same move, and Ctrl+Alt duplicates.
  Each gesture records one undo state.
- **BREAKING**: none. Without a modifier the marquee/lasso keep their existing
  combine mode; a Move-tool drag with no selection still moves the whole layer.

## Capabilities

### New Capabilities

- `selection-content-move`: dragging selected pixels with the Move tool or a
  selection tool (Ctrl), duplicating them to a new layer (Move+Alt /
  Ctrl+Alt), the one-undo-state contract, and the bridge operation behind them.

### Modified Capabilities

- `shape-selection-tools`: the combine-mode requirement gains the Shift/Alt/
  Shift+Alt quick keys and per-gesture locking; a new requirement covers the
  marquee square/from-centre constraints and the size readout; the Polygonal
  Lasso preview becomes a solid cursor-tracking rubber band; a new requirement
  covers the hover cursor.

## Impact

- Engine reuse: `pictura-render::layer_ops::{layer_via_copy,layer_via_cut,
  merge_scope}` already implement the pixel copy/cut; a thin
  `move_selection_content` wrapper adds the translate + merge-down and a test.
- `crates/pictura-render/src/document_ops/layer_ops/`: new `move_content` module
  and re-export.
- `crates/pictura-app/src/cxxqt_object/impl_selection.rs` +
  `cxxqt_object.rs`: one new `move_selection_content(dx, dy, duplicate)` bridge
  method (the declaration file is near its size cap and is trimmed to fit).
- `crates/pictura-app/cpp/image_view.{h,cpp}`: mouse tracking, a solid-preview
  flag, and the floating drag-size readout.
- `crates/pictura-app/cpp/tools.{h,cpp}` + `tools_selection_move.cpp`: quick
  modifier mapping, geometry constraints, polygonal rubber band, hover cursor,
  and the content-move state machine.
- `crates/pictura-app/cpp/selftest_tools_selection.cpp`: self-tests for the new
  logic. No new dependencies.
