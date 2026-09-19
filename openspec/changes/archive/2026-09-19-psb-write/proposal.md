## Why

Roadmap P5 gap G9: PSB **read** is shipped but **write** is missing. `write_psd`
always emits version 1 and rejects any dimension over 30 000, so a document
larger than the PSD limit cannot be saved at all. The file-format contract
requires that "a PSB with dimensions above 30,000 px, open/save round-trips
without truncation" (`docs/01-architecture/file-formats.md:263`); today the save
half of that contract is unreachable.

## What Changes

- `write_psd` SHALL keep emitting a version-1 PSD by default. Its byte output for
  existing (in-limit, PSD-sourced) documents is unchanged; it SHALL emit a
  version-2 PSB when the source document was a PSB (`Document.is_psb`) or when
  `doc.width` or `doc.height` exceeds 30 000 (`MAX_DIM_PSD`), because a PSD can
  hold neither those dimensions nor their length fields.
- Add `pub fn write_psb(doc: &Document) -> Result<Vec<u8>, PsdError>` that always
  emits a version-2 PSB. Both public functions SHALL share one
  `write_container(doc, psb: bool)`.
- PSB container/field widths SHALL mirror the reader exactly: header version word
  `2`; layer-and-mask section length and layer-info length as `u64`; each layer
  record's per-channel data length as `u64`; RLE scanline byte-count table
  entries as `u32` in the composite image-data section and in every layer
  channel and mask stream. The global layer-mask info length SHALL stay `u32` in
  both containers. Dimensions are accepted up to 300 000 (`MAX_DIM_PSB`); a
  dimension above that SHALL return `PsdError::Unsupported`.
- Additional-layer-information (tagged) blocks SHALL follow the two psd-tools
  framing rules: a per-layer block declares an even length with the pad byte
  inside it, while a document-level block declares its exact length and is padded
  externally to a 4-byte boundary. In a PSB, a big-key block (`_BIG_KEYS`,
  including `lnk2`/`lnk3`/`lnkE`) declares an 8-byte length. A preserved
  document-level block SHALL be re-framed to the output container's width. The
  `iOpa` fill-opacity block SHALL be 4 bytes.
- Determinism and the preserved-stream rule are unchanged: a stream on
  `Layer.raw_channels` SHALL be re-emitted byte-for-byte with its own compression
  header; only its declared length field widens. RLE stays the compression
  (ZIP write remains out of scope).
- Container preservation: a document read from a PSB (`Document.is_psb`) re-saves
  as a PSB even when small, so its preserved big-key blocks keep their `u64`
  lengths; a PSD-sourced document stays a PSD unless a dimension exceeds 30 000.
  This is content-lossless and is required for big-key framing.
- App: no code change is required for correctness. `PictureView::save` already
  calls `write_psd`, and both Save As dialogs already offer `*.psd *.psb`.
  Choosing a `.psb` suffix automatically for huge documents is an optional
  follow-up.

## Capabilities

### New Capabilities

<!-- None. The PSB write path is owned by psd-codec. -->

### Modified Capabilities

- `psd-codec`: an ADDED requirement "PSB write" specifies the version-selection
  rule, the `write_psb` entry point, the `u64`/`u32` PSB field widths, the
  300 000 dimension ceiling, and the preserved-verbatim guarantee. The existing
  "Composite PSD write and round-trip" and "Engine-encoded PSD channels are
  written with PackBits RLE" requirements keep their wording: they describe the
  version-1 path, which stays byte-for-byte unchanged.

A separate `psb-write` capability is not added: `psd-codec` already owns the
aggregate "write a Document to PSD/PSB" contract, and splitting the version-2
container into its own capability would duplicate the write contract.

## Impact

- `crates/pictura-codec/src/write.rs`: thread a `psb: bool` through the writer —
  `write_container`, `write_layer_info`, `write_record`, `OutChannel::declared_len`
  (`u64` in PSB), `encode_scanlines` (4-byte count entries in PSB), `rle_channel`;
  widen the section/info lengths in the container and select the dimension limit.
  Two tagged-block writers: `write_tag` (per-layer, even length with the pad
  inside) and `write_tag_document` (document-level, exact length, external pad to
  4). `reframe_document_extra` re-frames preserved document-level blocks to the
  target container width. `iOpa` is written as 4 bytes.
- `crates/pictura-codec/src/smart_writer.rs` / `src/smart_object.rs`: author and
  re-emit the document-level `lnk2` block with `write_tag_document`, and skip the
  external 4-byte pad when reading document-level blocks.
- `crates/pictura-codec/src/lib.rs`: export `write_psb`.
- `crates/pictura-codec/src/tests/psb.rs`: PSB field-width unit tests, auto-selection
  test, large-dimension round-trip, preserved-channel length widening, and the
  over-300 000 error.
- `crates/pictura-codec/tests/oracle.rs`: a psd-tools oracle test that opens a
  codec-written PSB and decodes its composite.
- `docs/dev/psd-support-roadmap.md` and `docs/dev/STATE.md`: move G9/P5 to
  shipped. Docs changes commit separately under `TASK-ALLOWS-DOCS`.
- No new dependency: the width changes are `std` integer encodings.
