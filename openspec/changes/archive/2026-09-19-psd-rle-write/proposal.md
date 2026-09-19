## Why

Roadmap P3 gap G12: `write_psd` always emits the image-data section with
compression `0` (raw), so a saved PSD is roughly the uncompressed size of the
document. Photoshop writes PackBits RLE (compression `1`) by default, and
`pictura-codec` already *decodes* RLE (`read.rs::read_rle`,
`decode_rle_channel`, `read_rle_count`) — there is no encoder. Adding one and
making RLE the write default closes G12.

## What Changes

- Add a deterministic PackBits encoder to `crates/pictura-codec/src/write.rs`.
- `write_psd` SHALL encode every channel the engine owns with compression `1`:
  the composite color planes and any document extra channels in the image-data
  section, and every engine-encoded layer channel — the layer's color channels
  and its raster mask (`-2`).
- Channels preserved byte-for-byte on `Layer.raw_channels` (unknown compressed
  streams) SHALL be re-emitted unchanged, never re-encoded. Unmodeled blocks,
  global-layer-mask bytes, and section extras are untouched.
- A scanline whose PackBits encoding would exceed the `u16` byte-count limit
  returns a `PsdError`; it is never truncated. For the PSD maximum width this is
  unreachable.
- Update the `default_document_bytes_are_unchanged` golden fixture to the new
  RLE serialization. This baseline refresh is explicitly sanctioned by the P3
  roadmap ("write compression moves to P3"); the P2 byte-faithful guarantee is
  unaffected because only engine-encoded channels change compression.
- **BREAKING**: `write_psd` output bytes change for every document. Decoded
  documents are unchanged, so round-trip equality holds.
- No new dependency: the PackBits encoder is a few lines over `std`.

## Capabilities

### New Capabilities

<!-- None. The write path is owned by psd-codec. -->

### Modified Capabilities

- `psd-codec`: the requirement "Composite PSD write and round-trip" states the
  image-data section is raw; it now states compression `1` and the golden
  scenario changes. A new requirement specifies the RLE layout, the
  preserved-verbatim guarantee, and the `u16` guard.

A separate `psd-write-compression` capability is not added: `psd-codec` already
owns the aggregate "write a Document to PSD" requirement, this is one codec
change, and splitting would duplicate the write contract across capabilities.
`psd-layer-io`'s "Channel image data supports raw and PackBits RLE" requirement
is decode-focused and keeps its exact wording; layer-channel *writing* is
specified here alongside the composite it shares a section with.

## Impact

- `crates/pictura-codec/src/write.rs`: add the encoder; redesign `OutChannel` so
  engine channels carry their complete RLE stream (compression word + count
  table + rows) and preserved channels stay verbatim; encode the composite and
  layer channels/mask.
- `crates/pictura-codec/src/tests.rs`: encoder round-trip unit test, a
  composite-declares-compression-1 test, a compressible-size test, and the
  refreshed `default_document_bytes_are_unchanged` golden.
- `crates/pictura-codec/tests/oracle.rs`: the psd-tools/ImageMagick oracle must
  still open and decode the RLE output.
- `docs/dev/psd-support-roadmap.md` and `docs/dev/STATE.md`: G12 moves to
  shipped.
