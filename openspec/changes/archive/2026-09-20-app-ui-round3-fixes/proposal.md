## Why

Verifying the second round of reported fixes found most were already shipped
(tab extension, scrollbars-always-visible, Ctrl-click thumbnail selection,
active-layer edit routing, `%` suffixes, Feather, space-pan, shortcut-guide bar,
transparent-pixel lock in the paint engine). Four genuine gaps remain:

- Dragging a layer into a group is refused or corrupts the tree when the dragged
  layer sits above the group (`move_path_to_dest` mixes pre- and post-removal
  indices).
- The brush cursor/refusal guard resolves the **topmost** layer's lock and
  visibility, not the active layer's, so the refusal cursor and message can be
  wrong.
- Panel width-mode persistence has an untested primary iconic→normal path where
  the icon-strip width can be saved as the normal width.
- Large-document painting is not interactive: each dab round-trips the GPU
  (instantiation + blocking readback; the 4000² profile shows the readback
  `convert` dominating), re-runs the command registry and title updates on the
  GUI thread, and can detach the full-resolution canvas image.

Krita's documented approach (Optimizing tips / strokes queue) is CPU tile-based
strokes that touch only dirty regions, avoid per-event object churn, and do not
GPU-sync per dab; its known hot spots are thumbnails, histogram, and brush
outline. This change applies the same levers to the shipped paint path.

## What Changes

- **Layer reparent**: resolve the destination parent in post-removal coordinates
  so dropping a layer into a group above/below it works and never mis-nests.
- **Active-layer cursor**: the brush blank/forbidden cursor and refusal message
  resolve the active layer's lock and visibility (mirroring `active_pixel_layer`).
- **Panel persistence**: persist the remembered normal width across an
  iconic→normal flip and cover the primary column's iconic→normal→restart path.
- **Paint responsiveness**: no per-dab command-registry/title refresh; the
  present cache is patched rather than rebuilt; no per-dab full-image detach; the
  in-stroke backend keeps whichever region composite is faster (measured: GPU).
  Add an ignored 4000² live-dab benchmark.
- **Lock coverage**: add a bridge-level transparent-pixel-lock self-test and a
  panel-driven multi-selection no-edit self-test.

Deferred: the artboard/frame "prevent auto-nesting" lock — this engine has no
artboards/frames and the Move tool never reparents, so there is nothing to gate;
it stays documented as a ceiling.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `layers-panel`: reparent into/out of a group resolves indices correctly.
- `tool-framework`: the refusal cursor/message follows the active layer.
- `workspace-persistence`: width mode/width survive a restart for every column.
- `paint-engine`: a live stroke stays interactive on large documents.
- `layer-locks`: transparent-pixel lock proven across the bridge.

## Impact

- **Engine** (`crates/pictura-render/src/document_ops/layer_ops/properties.rs`):
  the index math for `move_path_to`.
- **Rust bridge** (`crates/pictura-app/src/cxxqt_object/`): the paint profile and
  cursor/refusal helper.
- **C++ app** (`crates/pictura-app/cpp/`): `tools_marquee.cpp` active-layer
  lock/visibility, `frame.cpp`/`frame_columns.cpp` per-dab fan-out and
  persistence, `image_view.cpp` blit path, panel multi-select test.
- **Tests**: Rust unit tests + C++ self-tests from the next free codes; one
  `#[ignore]`d 4000² latency benchmark.
- **No document-format change, no new dependency, no `docs/` edit.**
