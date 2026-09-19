## Context

Before this change `write_psd` (`crates/pictura-codec/src/write.rs:529`) emitted
only a version-1 container: it wrote `VERSION_PSD` and `u32` length fields and
rejected any dimension over `MAX_DIM_PSD` (30 000) at `write.rs:553`. The reader
already understands version 2 (PSB): `read_psd` accepts `VERSION_PSB`, sets
`is_psb`, and selects `MAX_DIM_PSB` (300 000) for the dimension check
(`read.rs:18-22`, `read.rs:31`). Every on-disk field the PSB variant widens is
already decoded by the reader, so the writer must mirror those widths exactly.
The codec's determinism promise and the `Layer.raw_channels` byte-for-byte rule
(`write.rs:20-23`, `write.rs:107-109`) must survive the refactor.

**Revision: tagged-block big keys and the container-preserving version rule.**
The writer previously always emitted a 4-byte tagged-block length, and the reader
always decoded 4 bytes (`read.rs:514`), as did `smart_object.rs`
(`collect_linked_records`/`remove_linked_source`). That is wrong for PSB big
keys: an authored or preserved `lnk2` is framed as `u32` and psd-tools cannot
parse the file. The fix adds a `Document.is_psb: bool` field (true when read
from a version-2 PSB; false for PSD and new documents) so the writer can select
the container and the big-key width from the source rather than from the
dimensions alone. `Document.is_psb` participates in the derived `PartialEq`, so
tests comparing a written-then-read PSB set it explicitly.

**Revision: the two tagged-block framing rules.** psd-tools reads the two
tagged-block families differently. Per-layer blocks use
`TaggedBlocks.read(fp, version, padding=1)` (`layer_and_mask.py:647`): the
declared length is exact-or-even and there is no external pad, so a compliant
writer declares an even length with the pad byte inside it. Document-level
(global) blocks use `TaggedBlocks.read(fp, version, padding=4)`
(`layer_and_mask.py:176-177`): the declared length is the exact data length and
the block is padded externally to a 4-byte boundary. `read_length_block`
(`psd/bin_utils.py:73-91`, `read_padding` at 157-168) reads exactly the declared
length and then the external pad. The writer implements both (`write_tag` for
per-layer, `write_tag_document` for document-level), and a preserved
document-level block is re-framed to the destination container's length width
(`reframe_document_extra`).

Exact reader anchors this design mirrors (all in `crates/pictura-codec/src/read.rs`):

| On-disk field | PSD width | PSB width | Reader anchor |
|---|---|---|---|
| header version word | `u16` = `VERSION_PSD` (1) | `u16` = `VERSION_PSB` (2) | `read.rs:17-22` |
| dimension limit | `MAX_DIM_PSD` 30 000 | `MAX_DIM_PSB` 300 000 | `read.rs:31` |
| layer-and-mask section length | `u32` | `u64` | `read.rs:281-286` |
| layer-info length | `u32` | `u64` | `read.rs:298-302` |
| layer record per-channel data length | `u32` | `u64` | `read.rs:427-431` |
| RLE scanline byte-count entry (composite) | `u16` | `u32` | `read.rs:210-214`, `read.rs:644-648` |
| RLE scanline byte-count entry (layer channel / mask) | `u16` | `u32` | `read.rs:695-712` (same `read_rle_count`) |
| global layer-mask info length | `u32` | `u32` | `read.rs:317` |
| additional-layer-info (tagged) block length, big keys (per-layer and document-level) | `u32` | `u64` | `read.rs:514-518` |

The header height/width remain `u32` in both versions (`read.rs:26-27`), so only
the version word changes in the header. The tagged-block width is key-dependent:
psd-tools' `TaggedBlock._length_format` (`psd/tagged_blocks.py:319-321`) returns
`"Q"` when `version == 2 and key in _BIG_KEYS`, `"I"` otherwise. `_BIG_KEYS`
holds `Alph`, `FELS`, `FEid`, `FMsk`, `FXid`, `LMsk`, `Layr`, `Lr16`, `Lr32`,
`Mt16`, `Mt32`, `Mtrn`, `PxSD`, `artd`, `cinf`, `extd`, `extn`, `lnk2`, `lnk3`,
`lnkE`, `pths`; the writer must mirror that exact set (`common::is_psb_big_key`).
`lnkD` is **not** a big key (it is `LINKED_LAYER1`). Without the widening a
written PSB whose `lnk2` block is framed as `u32` mis-frames on read and makes
psd-tools raise `MemoryError`.

Two padding rules apply, and the writer must not conflate them: a **per-layer**
block declares an even length with the pad inside it (`write_tag`), while a
**document-level** block declares its exact length and is padded externally to a
4-byte boundary (`write_tag_document`). The `iOpa` fill-opacity block is a 4-byte
`B3x` value in both containers (`layer_and_mask.py`), so it is written as
`[fill, 0, 0, 0]`.

## Goals / Non-Goals

**Goals:**

- `write_psd` keeps its current byte output for in-limit PSD documents and
  selects a PSB container when the source was a PSB (`Document.is_psb`) or either
  dimension exceeds 30 000.
- Add `write_psb` that always emits a PSB; both share one container writer.
- Mirror the reader's PSB field widths exactly (`u64` lengths, `u32` RLE counts,
  `u32` global-mask length, `u64` tagged-block length for PSB big keys).
- Preserve determinism and the verbatim `Layer.raw_channels` guarantee.
- Round-trip a >30 000 px document equal to the input.
- Keep a read PSB in the PSB container on re-save so preserved big-key blocks
  keep the `u64` width their bytes already use.

**Non-Goals:**

- ZIP / ZIP-with-prediction write (still read-only).
- App UI changes (extension auto-selection, >30 000 probe budget).
- Read-path changes beyond mirroring the PSB tagged-block big-key length
  (`read_layer_record` and `smart_object.rs`); no decoding or model change.
- Changing the inner linked-record list framing (its record lengths are always
  `u64`, independent of the container).

## Decisions

### D1. One `write_container(doc, psb: bool)`, two public entry points

`write_psd` becomes a thin dispatcher:

```rust
pub fn write_psd(doc: &Document) -> Result<Vec<u8>, PsdError> {
    let psb = doc.is_psb || doc.width > MAX_DIM_PSD || doc.height > MAX_DIM_PSD;
    write_container(doc, psb)
}

pub fn write_psb(doc: &Document) -> Result<Vec<u8>, PsdError> {
    write_container(doc, true)
}
```

The dimension limit inside `write_container` is `if psb { MAX_DIM_PSB } else {
MAX_DIM_PSD }`, mirroring `read.rs:31`. A dimension above the selected limit
still returns `PsdError::Unsupported` (the existing message shape at
`write.rs:451-456` is kept, with the max dimension variable).

Alternative considered: a public `write_versioned(doc, psb)` only. Rejected — the
spec and callers want the two named entry points, and `write_psd`'s
auto-selection is required so the app's existing `PictureView::save` path saves a
huge document without a call-site change.

### D2. Thread `psb` through the writer, not the data model

Every helper that writes a widened field gains a `psb: bool` parameter:

- `write_layer_info(doc, psb)` → `write_record(..., psb)` and `rle_channel(..., psb)`.
- `OutChannel::declared_len()` returns `u64`; `write_record` writes it as `u64`
  under PSB and `u32` under PSD.
- `encode_scanlines(planes, width, height, psb)` writes a `u32` count entry
  under PSB and a `u16` entry under PSD; the per-row guard becomes `u32::MAX`
  under PSB and `u16::MAX` under PSD (the `ponytail:` comment at
  `write.rs:494-495` is updated accordingly).
- `rle_channel(width, height, plane, psb)` forwards `psb` to `encode_scanlines`.
- `write_tag(out, key, data, psb)` is the **per-layer** writer: it declares an
  **even** length `data.len() + (data.len() & 1)` — the pad byte is written
  inside the declared length, never outside it — and writes that length as `u64`
  when `psb && common::is_psb_big_key(key)`, else `u32`. `write_extra(..., psb)`
  threads `psb` through every per-layer `write_tag` call (including the preserved
  `extra_blocks` and the authored `SoLd`).
- `write_tag_document(out, key, data, psb)` is the **document-level** writer:
  psd-tools reads global tagged blocks with `TaggedBlocks.read(..., padding=4)`,
  so it declares the **exact** `data.len()` (same big-key width rule) and then
  pads externally to a 4-byte boundary.
- `reframe_document_extra(bytes, src_psb, dst_psb)` walks preserved document-level
  `8BIM`/`8B64` blocks (source width, skipping each block's external pad to 4)
  and re-emits them with `write_tag_document` for the target width; it returns
  the input unchanged when `src_psb == dst_psb` and copies the tail verbatim on a
  parse failure. `write_container` calls it instead of cloning
  `layer_section_extra`.
- `smart_writer::author_lnk2_bytes(sos, psb)` frames its document-level `lnk2`
  block through `write_tag_document`.
- `smart_object::remove_linked_source` re-emits a matched document-level block
  through `write_tag_document` and skips each surviving block's external pad.

The composite and layer paths already funnel through `encode_scanlines` /
`rle_channel`, so this is a signature change plus two width branches, no new
control flow.

Alternative considered: a `Version` enum with `u64`-vs-`u32` methods. Rejected —
one `bool` already exists as `is_psb` on the read side (`read.rs:18`), and a
two-variant enum is the interface-with-one-implementation smell for a single
fixed pair of formats.

### D3. Length fields widen; bytes do not

For a preserved `OutChannel::Verbatim`, the stream (compression word + count
table + rows) is emitted unchanged; only the record's declared length field
widens (`u32` → `u64`). The same holds for the global layer-mask info: its
length stays `u32` in both versions (`read.rs:317`) and its bytes are untouched.
`global_layer_mask` bytes are copied verbatim; `layer_section_extra` block data
is copied verbatim but its per-block length field is re-framed to the target
container when the container changes (`reframe_document_extra`); between same-
container writes the bytes are untouched. The composite image-data section has no
outer length prefix, so only its count table width changes.

### D4. `write_psd` in-limit output is byte-identical

For a document within 30 000 px, `psb` is `false`, the header version stays
`VERSION_PSD`, and every width branch takes the PSD arm. The output bytes are
therefore identical to today's, so
`crates/pictura-codec/tests/fixtures/default_before.psd` and the
`default_document_matches_rle_golden` test (`tests.rs:670-679`) are unchanged.
This is the guard that the refactor did not churn the version-1 path.

### D5. PSB field widths mirror the reader (the correctness core)

PSB emits: version word `2`; `u64` layer-and-mask section length and layer-info
length; `u64` per-layer-channel declared length; `u32` RLE count entries in both
the composite count table and each layer channel/mask stream; `u32` global
layer-mask info length; and a `u64` length for every additional-layer-information
block whose key is a PSB big key (`common::is_psb_big_key`, the exact psd-tools
`_BIG_KEYS` set). The two tagged-block families frame differently and the writer
mirrors both: a per-layer block declares an even length with the pad inside
(`TaggedBlocks.read(..., padding=1)`), while a document-level block declares its
exact length and is padded externally to 4 (`TaggedBlocks.read(..., padding=4)`).
The read side is fixed in lockstep: `read_layer_record` reads the big-key `u64`
at `read.rs:514` and keeps the odd-length skip for legacy per-layer files, while
`collect_linked_records` / `remove_linked_source` / `reframe_document_extra` read
the document-level exact length and skip the external 4-byte pad. `is_psb`
selects the big-key width everywhere. This is exactly what psd-tools decodes, so
a codec-written PSB round-trips through `read_psd` and opens in psd-tools.

### D6. The container follows the source document or the dimension

`Document.is_psb` records the source container (`read_psd` sets it from the
header version); new/blank documents are `false`. `write_psd` selects a PSB when
`doc.is_psb` is true or either dimension exceeds 30 000, so a small PSB read from
disk re-saves as a PSB. Keeping the container is preferred because a preserved
PSB big-key block's bytes already carry a `u64` length. When the container does
change (`write_psb` on a PSD-sourced document, or a huge new document),
`reframe_document_extra` rewrites preserved document-level block lengths to the
destination width and re-pads each block externally to 4, so the `u32` → `u64`
widening cannot mis-frame the stream.
`write_psb` still forces the PSB container. The old "small PSB re-saves as PSD"
ceiling is removed. `ponytail: version follows Document.is_psb or the dimension;
add a per-document format override only if a caller needs to force PSD for a
small PSB.`

### D7. App needs no change for correctness

`PictureView::save` is `pictura_codec::write_psd(doc)`
(`crates/pictura-app/src/cxxqt_object/impl_core.rs:196-218`), so a >30 000
document now serializes as a PSB through the existing path. Both Save As dialogs
already use the filter `Photoshop files (*.psd *.psb)`
(`crates/pictura-app/cpp/frame_menus.cpp:87-88`,
`crates/pictura-app/cpp/frame.cpp:443-444`); when the user types no suffix the
app appends `.psd` (`frame_menus.cpp:93`, `frame.cpp:449`). That suffix choice is
cosmetic: the bytes are a PSB regardless. Auto-choosing the `.psb` suffix for
huge documents is an optional follow-up, not required for the contract.

Separately, the app's import budget still caps `max_dimension` at 30 000
(`crates/pictura-codec/src/probe.rs:43-50` is used with its default in the app),
so opening a >30 000 PSB through the app is a distinct gap. It is out of scope
here and is recorded as a follow-up so the codec contract is not overstated as
end-to-end app parity.

### D8. Tests

- Unit: `write_psb` of a small document emits version word `2` and reads back
  equal (with `is_psb` set on the expected document); the section/info/channel
  lengths are 8 bytes.
- Unit: `write_psd` of a 30 001×1 document emits version word `2` (auto-select)
  and round-trips equal.
- Unit: a `Layer.raw_channels` stream round-trips byte-for-byte under PSB with
  only its declared length widened.
- Unit: a preserved `Lr16` big-key `extra_blocks` entry round-trips under PSB and
  its length field is 8 bytes (counting the inside pad).
- Unit: a small document read from a PSB (`is_psb` true) re-saves with
  `write_psd` as a version-2 PSB (container preserved).
- Unit: a dimension above 300 000 returns `PsdError::Unsupported`.
- Unit: `write_tag` (per-layer) declares an even length with the pad inside it,
  and `write_tag_document` (document-level) declares the exact length and pads
  externally to 4 (raw byte assertions, non-big and big keys).
- Unit: a PSB composite uses 4-byte RLE count entries and a PSD uses 2-byte
  (raw byte assertion, not only via round-trip).
- Unit: repeated `write_psb` calls are byte-identical.
- Unit: a non-big-key block in a PSB still declares a 4-byte length.
- Unit: `iOpa` is written as a 4-byte value.
- Unit: a preserved PSD-framed `lnk2` is re-framed to `u64` by `write_psb`, and is
  left untouched by `write_psd`.
- Existing golden test (`tests.rs:670`) asserts the version-1 path is unchanged.
- Oracle: psd-tools opens a codec-written PSB and decodes its composite to the
  same pixels (self-skips without `python3` + `psd-tools`, matching
  `oracle.rs:383-471`).
- Oracle (critical repro): psd-tools `PSD.read`/`PSDImage.open` parses a
  `write_psb` document carrying an authored embedded smart object (its authored
  `lnk2` big-key block must be `u64`-framed) without error.
- Oracle: an odd-length and a non-4-multiple per-layer block followed by the
  authored `SoLd` still lets psd-tools find the smart object (pad-inside rule).
- Oracle: a PSD-sourced document with a preserved `lnk2` re-saved with `write_psb`
  is parsed by psd-tools (document-level re-framing).
- Oracle: a preserved document-level block whose length is not a multiple of 4 is
  re-emitted and still lets psd-tools read the later `lnk2` (external-pad rule).

## Risks / Trade-offs

- **A missed width branch desynchronizes the stream.** → D8's round-trip tests
  cover every widened field (section, info, channel length, composite count,
  layer count, tagged-block big keys), and the psd-tools oracle is an
  independent decoder; the critical authored-`lnk2` repro guards the exact
  framing bug.
- **A big-key set that drifts from psd-tools.** → `common::is_psb_big_key` is a
  `const` copy of `TaggedBlock._BIG_KEYS` (`psd/tagged_blocks.py:235-258`,
  `_length_format` at 319-321); the psd-tools oracle test fails if it drifts.
- **An odd per-layer tagged-block length mis-frames the next block for
  psd-tools.** → `write_tag` declares the even length and puts the pad inside it;
  the odd-block oracle test proves the following `SoLd` is still found. An
  in-memory odd payload gains a trailing zero on read-back (that is the pad);
  legacy external-pad per-layer files keep reading through the reader fallback.
  The PSD golden is unchanged because the default document emits no odd block.
- **A document-level block uses the per-layer rule.** → psd-tools reads global
  blocks with `padding=4`; `write_tag_document` declares the exact length and pads
  externally, and the doc-level oracle test proves a non-4-multiple block still
  lets psd-tools read the later `lnk2`.
- **A container switch keeps preserved big-key blocks u32-framed.** →
  `reframe_document_extra` rewrites preserved document-level block lengths to the
  destination width and re-pads externally to 4; same-container writes return the
  bytes unchanged, so the PSD golden and normal PSB saves are byte-identical.
- **Preserving the container for a small PSB** is now required (D6) so preserved
  big-key bytes stay `u64`-framed.
- **The app cannot open a >30 000 PSB yet (probe budget).** → Recorded as a
  follow-up; the codec-level contract (`read_psd`/`write_psb`) is verified by the
  round-trip and oracle tests.
- **`u32` count-table guard is now reachable in theory for a huge PSB row.** →
  Keeping the guard means an over-limit row errors rather than truncating; with
  a 300 000 px max width the PackBits worst case is well under `u32::MAX`.
- **An `8B64` document-level signature is normalized to `8BIM` on re-frame.** →
  Low severity: `reframe_document_extra` accepts both signatures but
  `write_tag_document` always emits `8BIM`, so a preserved `8B64` block switches
  signature on a container-changing write. `8B64` is a rare big-document
  variant and the signature carries no length-width meaning in the writer
  (the width is already key- and container-derived), so the block decodes
  identically. Preserving the source signature is a follow-up if a real `8B64`
  file ever needs byte-exact re-save.
