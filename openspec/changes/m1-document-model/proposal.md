## Why

M0 shipped a document that is only a single composite image, so there is nothing
for compositing, selection, or adjustment work to hang off. M1 adds the
layer/channel/mask spine and native PSD layer read/write, and proves it does not
merely agree with its own writer by diffing against the independent `psd-tools`
implementation.

## What Changes

- `Document` gains a `layers` tree (bottom-first, matching PSD `'Layr'` on-disk
  z-order) of pixel layers and groups.
- `pictura-core` gains `Layer`, `LayerMask`, `Channel`, `PsdRect`, and a
  `BlendMode` covering all 27 CS6 layer keys plus the group-only `pass`.
- `read_psd`/`write_psd` read and write the PSD Layer and Mask Information
  section: layer records, raw + PackBits RLE channel image data, global layer
  mask info, `'luni'` Unicode names, and `'lsct'` group markers.
- Differential tests read `psd-tools`-authored fixtures (`two_layers`, `group`,
  `masked`, `gray`, `adjustment`) and one test opens a codec-written PSD in
  `psd-tools`; `scripts/validate_output.py` inspects an arbitrary codec output.
- Malformed or truncated layer sections return `PsdError`, never panic.
- **BREAKING**: `Document` and `Layer` gain public fields, so struct literals in
  dependents must be updated (`layers`, `channels`, …).

## Capabilities

### New Capabilities

- `document-model`: the `Document`/`Layer`/`Channel`/`LayerMask`/`PsdRect`/
  `BlendMode` data model — the bottom-first layer tree, layer properties, the
  PSD channel-ID scheme, raster layer masks, the 27-key blend-mode mapping, and
  the big-endian/signed-coordinate serialization contract.
- `psd-layer-io`: PSD `'Layr'` section read and write — layer records, channel
  image data (raw + PackBits RLE), global layer mask info, `'luni'` and `'lsct'`
  tagged blocks, model→write→read→model round-trip, `psd-tools` oracle checks,
  and error-not-panic behavior on malformed input.

### Modified Capabilities

_None. This is the first spec for these capabilities._

## Impact

- `crates/pictura-core/src/lib.rs` — new model types; `Document` gains
  `layers` and document-level `channels`; `BlendMode` key table.
- `crates/pictura-codec/src/lib.rs` — `read_psd` parses the layer section;
  `write_psd` emits it; a bounds-checked `Reader` makes malformed input an error.
- `crates/pictura-codec/tests/oracle.rs`, `crates/pictura-codec/tests/fixtures/`,
  and `scripts/generate-fixtures.py` / `scripts/validate_output.py` — the
  `psd-tools` oracle.
- Out of scope for M1: layer styles, adjustment-layer *semantics* (their blocks
  are carried opaquely only), smart objects, vector masks, 16/32-bit layers,
  ZIP channel compression, protection/lock flags, mask density/feather, PSB
  write, and blend-mode math.
