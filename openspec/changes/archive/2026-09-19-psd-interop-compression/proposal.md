## Why

The codec refuses PSDs whose composite or layer channels use ZIP compression
(codes 2/3), aborts a whole file on an unrecognized blend key, and does not
tolerate a missing merged composite. Photoshop writes ZIP-with-prediction for
layer channels by default in many saves, so a large share of real PSDs currently
fail to open with `PsdError::Unsupported`. Closing those three gaps is the first
step of the PSD support roadmap (`docs/dev/psd-support-roadmap.md`, phase P1).

## What Changes

- Decode composite image data with compression `2` (ZIP) and `3`
  (ZIP-with-prediction), in addition to `0` (raw) and `1` (RLE).
- Decode layer channel image data with compression `2` and `3` likewise.
- Apply the ZIP-with-prediction delta inversion byte-wise per scanline, exactly
  as Photoshop and `psd-tools` do.
- Degrade an unrecognized layer blend key to `Normal` instead of failing the
  whole file. (The raw key is not preserved yet; opaque preservation is roadmap
  phase P2.)
- Tolerate a missing merged composite ("Maximize Compatibility" off): when the
  image-data section is absent, return a document with a zero-filled composite
  and the parsed layers instead of a truncation error.
- Add `flate2` (miniz_oxide backend, pure Rust) to `pictura-codec` — the only
  new dependency, needed because deflate has no standard-library decoder.
- **BREAKING**: none. Supported inputs keep the same result; previously
  `Unsupported` inputs now open.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `psd-codec`: the composite-read requirement extends to ZIP/ZIP-with-prediction,
  and a new requirement covers the absent merged composite.
- `psd-layer-io`: the channel-image-data requirement extends to ZIP/
  ZIP-with-prediction, and a new requirement covers unknown-blend-key degrade.

## Impact

- `crates/pictura-codec/Cargo.toml`: add `flate2`.
- `crates/pictura-codec/src/common.rs`: ZIP compression constants.
- `crates/pictura-codec/src/read.rs`: ZIP decode helper, composite and layer
  channel dispatch, blend-key degrade, absent-composite path.
- `crates/pictura-codec/src/lib.rs`: scope doc update.
- `crates/pictura-codec/src/tests.rs` (+ possibly a new test module): ZIP/
  prediction/absent-composite/unknown-blend tests.
- `crates/pictura-codec/tests/oracle.rs`: a differential test where `psd-tools`
  decodes a hand-built ZIP/ZIP-with-prediction stream and its bytes must match
  `read_psd` (psd-tools' writer only emits raw/RLE, so it is used as the decoder).
- No model, render, or app change.
