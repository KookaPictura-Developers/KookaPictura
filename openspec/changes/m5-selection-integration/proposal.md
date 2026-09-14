## Why

M5 selections can be computed in `pictura-select` but cannot leave the process:
there is no way to persist a selection as a PSD alpha channel or to make a
selection constrain an edit in the app. This change wires the existing selection
engine into the document model, the PSD codec, and the app so selections become
durable and usable.

## What Changes

- `pictura-core`: add `Document.channels: Vec<Channel>` for document-level extra
  channels (saved selections / spot channels), distinct from per-layer channels.
- `pictura-codec`: read and write those extra channels — bump the header channel
  count and append the extra planes after the color planes in the image-data
  section — so a saved selection round-trips without regressing composite or
  layer handling. Reject extra-channel length mismatches.
- `pictura-select`: add `Selection::to_channel(id) -> Channel` and
  `Selection::from_channel(&Channel, width, height) -> Result<Selection>`.
- `pictura-app`: add `PictureView` selection state (`select_all()`,
  `deselect()`, `magic_wand(x, y, tolerance)`, active-selection querying) and
  make `add_adjustment(kind)` attach a raster mask built from the active
  selection so the adjustment is confined to selected pixels; no selection stays
  full-frame. Add Select all / Magic wand (center) / Deselect buttons.
- `--self-test`: wand-select one quadrant of a layered PSD, add an Invert
  adjustment, and assert the inverted pixels are confined to that quadrant.

## Capabilities

### New Capabilities
- `selection-channels`: document-level alpha channels in `pictura-core`, their
  PSD round-trip in `pictura-codec`, and `Selection` ↔ `Channel` conversion in
  `pictura-select`.
- `selection-masked-edits`: app-side selection state, selection-derived layer
  masks on adjustment layers, the selection UI buttons, and the headless
  masked-adjustment self-test.

### Modified Capabilities

## Impact

- Code: `crates/pictura-core/src/lib.rs`, `crates/pictura-codec/src/lib.rs`
  (plus `tests/oracle.rs`), `crates/pictura-select/src/lib.rs`,
  `crates/pictura-app/src/cxxqt_object.rs`, `crates/pictura-app/cpp/main.cpp`.
- Data model: `Document` gains a channel list; PSD header channel count and
  image-data section gain extra planes.
- No new dependencies. No changes to `docs/`.
- Out of scope: Refine Edge, quick-selection brush, marquee tool,
  channel-thumbnail UI.
