## Context

`write_psd` serializes the composite image-data section with compression `0`
(raw) and every engine layer channel with raw as well (`OutChannel::Encoded`
writes `COMPRESSION_RAW` + the plane). The reader already understands
compression `1` (PackBits RLE) for both the composite (`read.rs::read_rle`) and
layer channels (`read_rle_count` + `decode_rle_channel`), and `common.rs`
already defines `COMPRESSION_RLE = 1`. The only missing piece is an encoder and
using it on write.

The on-disk layout for RLE (both sections) is fixed by the reader: a 2-byte
compression word, then a byte-count table sized by scanline length and whether
the file is PSD or PSB, then the PackBits payloads in the same order. PSD (the
only version `write_psd` emits) uses 2-byte counts; PSB write is P5.

## Goals / Non-Goals

**Goals:**

- Deterministic PackBits encoding of composite color planes, document extra
  channels, layer color channels, and the raster mask, using the layout
  `read_psd` already decodes.
- Lossless read→write→read: decoding our own RLE reproduces the exact pixels.
- Preserved channels (`Layer.raw_channels`) and preserved blocks stay
  byte-for-byte.
- Error rather than truncate when a scanline exceeds the `u16` count limit.

**Non-Goals:**

- ZIP or ZIP-with-prediction writing (G1 is read-only today).
- PSB/4-byte counts, PSB write (G9/P5).
- Changing any read path or `psd-layer-io` decode contract.
- Verified byte parity with Photoshop's encoder; only standard-reader
  acceptance (psd-tools) is required, since Adobe's exact run-splitting choices
  are not an oracle.

## Decisions

### D1. `OutChannel` carries the complete RLE stream

`OutChannel::Encoded` currently holds a raw plane and `write` prepends
`COMPRESSION_RAW`. Redesign it to hold the complete on-disk stream (compression
word + count table + rows) so `declared_len` is the stream length and `write`
emits it unchanged. `OutChannel::Verbatim` keeps its current meaning: an
already-complete preserved stream emitted byte-for-byte. A single
`rle_channel(width, height, plane) -> Result<Vec<u8>, PsdError>` builder produces
the stream for any engine channel; the composite and layer paths call it.

Alternative considered: keep `Encoded` raw and have the writer compress at emit
time. Rejected — the layer channel's declared length must be known when the
record header is written, before the channel data block, and the composite
section shares one compression word across all planes, so the encoded bytes
have to exist before either section is laid out.

### D2. One compression word, all planes encoded

The image-data section has a single compression word for every header channel
(composite colors plus document extras). All of them are encoded with `rle_channel`;
leaving any plane raw would desynchronize `read_rle`. Rows are emitted
channel-major, row-major: for each channel plane, its `height` rows in order,
each preceded by its own 2-byte count. This mirrors `read_rle` exactly.

### D3. Deterministic PackBits

The encoder scans each row and emits: a run of three or more equal bytes as a
repeat packet (control `257 - n`, then the byte, `n` in 3..=128); otherwise a
literal packet of up to 128 bytes (control `n - 1`, then the bytes). This is
deterministic and never worse than raw. Note the spec permits any valid
run-splitting; this choice is fixed so repeated writes are byte-identical.

### D4. `u16` guard

Each row's packed length is checked against `u16::MAX` before its count is
written. Over-limit returns `PsdError::Invalid` naming the RLE scanline; nothing
is truncated. For the PSD maximum width (30 000) the PackBits worst case is
about 30 235 bytes, so the guard is unreachable in practice but keeps the
count-table cast honest.

### D5. The P2 byte-faithful guarantee is unaffected

Only channels the engine owns change compression. `Layer.raw_channels`
(preserved unknown compressed streams), unmodeled tagged blocks, the global
layer mask, mask extras, blending ranges, and section extras are still emitted
unchanged. The P2 promise — open→save preserves what the engine does not model —
holds; what changes is that the engine's *own* composite and channel bytes are
now RLE.

### D6. The golden fixture is refreshed

`default_document_bytes_are_unchanged` compares `write_psd(&default_document())`
to `crates/pictura-codec/tests/fixtures/default_before.psd`. That fixture
encodes the raw composite and is expected to change: the P2 roadmap states "No
write-format change (the byte-layout golden is unchanged); write compression
moves to P3", and P3 is this change. The fixture is regenerated from the RLE
output and the test's assertion message updated; the test still guards against
unintended serialization drift. This is the one sanctioned golden change and is
called out in the task result.

### D7. Tests

- Encoder unit test: encode a plane, decode it with the existing
  `decode_rle_channel`/`decode_packbits`, assert the bytes match.
- Composite declares compression 1: a default document's image-data compression
  word (offset 38 for the layerless layout) is `1`.
- Size: a compressible image (solid fill) writes smaller than the raw size.
- Existing randomized round-trip and `psd-tools` oracle tests must still pass.

## Risks / Trade-offs

- **Output bytes change for every document.** Any consumer comparing exact
  bytes must refresh its baseline. → Only the `default_before.psd` fixture is
  affected in-tree; it is refreshed here (D6).
- **Incompressible data can grow slightly.** PackBits never expands by more than
  one control byte per 128-byte literal run, so worst-case growth is under 1%.
  → Acceptable; Photoshop's default is RLE too.
- **Oracle coverage.** psd-tools cannot author RLE, but it decodes it, so the
  existing "psd-tools opens codec output" tests become the independent decoder
  check for our encoder. → Keep them; no new dependency.
