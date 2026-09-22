## Why

The reader decodes ZIP (2) and ZIP-with-prediction (3), but the writer always
emits PackBits RLE, so opening a ZIP-compressed PSD and re-saving silently
changes its compression and inflates the file. Closing roadmap gap G12 makes an
open→save keep the source compression, and gives the writer ZIP output that
`psd-tools` and Photoshop already read.

## What Changes

- `Document` records the composite compression kind and the layer-channel
  compression kind observed on read, defaulting to RLE for a constructed
  document, so existing output is unchanged.
- `write_psd` emits the recorded kind: RLE (as today), ZIP (zlib-wrapped
  deflate), or ZIP-with-prediction (the reversible per-row delta then deflate)
  for the composite color planes, document extra channels, layer color channels,
  and the raster mask.
- Preserved verbatim channels (`Layer.raw_channels`) stay byte-for-byte, as
  today.
- Ceiling: a document whose channels mix compression kinds normalizes each
  category (composite / layer channels) to the first kind seen, marked
  `ponytail:`.
- No new dependency: `flate2` (already used by the reader and the pattern
  decoder) provides the deflate encoder.

## Capabilities

### New Capabilities
- (none)

### Modified Capabilities
- `psd-codec`: the writer no longer hardcodes PackBits RLE — it emits the
  document's recorded compression (RLE by default, ZIP/ZIP-with-prediction when
  the source used them) for the composite and for engine-encoded layer channels.

## Impact

- `crates/pictura-core`: a `Compression` enum and two `Document` fields
  (`composite_compression`, `layer_compression`), default RLE.
- `crates/pictura-codec`: `read.rs` records the observed kinds; `write.rs` gains
  a ZIP encoder and a forward predictor and dispatches on the kind. `flate2` is
  already a dependency.
- Tests/oracles: a psd-tools ZIP/ZIP-prediction write oracle; the RLE golden and
  constructed-document byte tests stay green because a constructed document
  defaults to RLE.
