## Context

M0 landed a working PSD read/write path for a single composite image and a
minimal `Document` (`width`, `height`, `mode`, `depth`, `composite`). M1 must add
the layer/channel/mask model and layer-section I/O without disturbing that path.
The PSD byte layout is fixed by Adobe's File Formats Specification; the model
contract was written down in `crates/pictura-core/MODEL.md` (M1-A) before the
codec work (M1-B) started.

The key constraint is trust: a codec that only round-trips through its own writer
can share a bug with itself. The independent `psd-tools` Python library authors
the fixtures and inspects codec output, so the oracle is a second implementation.

## Goals / Non-Goals

**Goals:**

- A bottom-first layer tree that survives read and write without reordering.
- Pixel layers and groups (open/closed), with the property set the PSD layer
  record carries: name, signed bounds, opacity, blend mode, clipping, visibility.
- Per-layer channels with PSD IDs (`0…`, `-1` transparency, `-2` mask, `-3` real
  mask) and raster layer masks whose bounds can differ from the layer's.
- A `BlendMode` that maps all 27 CS6 layer keys plus the group-only `pass`.
- `'Layr'` read/write: layer records, raw and PackBits RLE channel data, global
  layer mask info, `'luni'` names, `'lsct'` group markers.
- Malformed input produces `PsdError`, never a panic.
- Differential verification against `psd-tools`.

**Non-Goals:**

- Layer styles (`lrFX`), smart objects, text, vector masks, 3D/video.
- Adjustment-layer *semantics*: adjustment blocks are read and written as opaque
  key+bytes only; interpreting them is M4 (`pictura-adjust`/`pictura-render`).
- 16/32-bit layer data (`Lr16`/`Lr32`) and ZIP/ZIP-prediction channel
  compression on read or write.
- PSB *write* (PSB read is supported by the same reader).
- Blend-mode math; M1 round-trips the mode enum only.
- Layer protection/lock flags and mask density/feather: the layer flags byte is
  consumed only for visibility, and the mask flags byte is preserved but density
  and feather are not modeled.
- Non-RGB/grayscale modes.

## Decisions

### Layer ordering is bottom-first and never reversed

The model stores layers in the same order as the PSD `'Layr'` records
(bottom-to-top). Reversing for a top-first UI is a view concern. This removes an
entire class of off-by-one bugs at the codec boundary and makes round-trip
equality the direct check.

*Alternative considered:* store top-first to match the Layers panel. Rejected
because every codec read/write would need a reverse pass, and the PSD order is the
authoritative one.

### Channel data is planar per layer, decompressed in the model

A `Channel` is one `Vec<u8>` plane, matching PSD's per-channel block and per-row
RLE. The model stores decompressed bytes; the compression code and scanline
counts live in the codec. `data.len()` must equal the owning rectangle's pixel
count, which the writer validates before emitting.

*Alternative considered:* store compressed bytes and decompress lazily. Rejected
for M1: it adds a parse-on-access API for a compression set (raw, RLE) small
enough to decode up front.

### Compression: read raw + PackBits, write raw only, reject ZIP

Read supports compression `0` (raw) and `1` (PackBits RLE); write always emits
raw. RLE is lossless, so the two agree byte-for-byte, which the tests verify.
ZIP codes `2`/`3` return `Unsupported`. This keeps the first layer I/O surface
small and dependency-free while still reading the most common Photoshop RLE
layer files.

*Alternative considered:* depend on a deflate crate and support ZIP on read in
M1. Rejected as unrequested scope; the reader already has a clean place to add
it when a real need appears.

### Group blend mode is taken from the `'lsct'` block when present

psd-tools and Photoshop write `pass` inside the `'lsct'` section-divider block
while the folder record's own key is usually `norm`. The reader prefers the
`'lsct'` key, so a Pass Through group loads as `PassThrough` rather than being
silently downgraded to Normal. The writer emits the key in `'lsct'` for the same
reason.

### Mask channels are sized by the mask rect

The `-2` channel is sized by `LayerMask.rect`, which may differ from the layer
rectangle. Reading all channels by the layer rect would misread masks that extend
past or sit inside the layer's bounds. Channel ID `-3` (real user mask) is
consumed but not modeled, because vector masks are out of M1 scope.

### Visibility and clipping come from separate fields

`visible` is `!(flags & 0x02)`. `clipping` is the layer record's dedicated
clipping byte read after opacity. The `0x08` flag psd-tools sets on ordinary
layers is *not* clipping; using it would mislabel every layer. The raw flags
byte is otherwise not retained, which is the one place M1's model is narrower
than the PSD record.

### Opaque additional-layer blocks are preserved

Any tagged block that is not `'luni'` or `'lsct'` and carries a recognized
adjustment key is stored as `AdjustmentData { key, data }` and written back
verbatim. This gives lossless round-trip for adjustment layers without pulling
adjustment math into core. Truly unknown keys are currently skipped, not
preserved (a known limitation).

### The reader is a bounds-checked cursor

Every primitive read goes through `Reader::take`, which returns `Truncated`
instead of indexing. Section lengths are validated with `checked_add` against the
file length. This is what makes the error-not-panic requirement hold under
fuzzing/adversarial input.

## Risks / Trade-offs

- **Model narrower than the format (protection/lock flags, density/feather)** →
  Documented as a non-goal; revisit when locked-layer UI or mask density
  round-trip is required. Round-trip equality is only claimed for the modeled
  subset.
- **Unknown tagged blocks are skipped, not preserved** → The `luni`/`lsct`/
  adjustment set is handled; other keys are dropped, so a save may lose
  unrecognized metadata. Tracked for a later preservation pass.
- **`'lsct'` blend-key preference is a heuristic** → It matches psd-tools and
  Photoshop output. If a real file disagreed, the folder-record key is the
  fallback already in the code.
- **PSB is read-only** → PSB write is out of M1 scope; the reader shares the PSD
  path with 4-byte/8-byte length switches.
- **Write emits raw channel data** → Files are valid but larger than Photoshop's
  RLE output. RLE write is a later optimization, not a correctness gap.
- **Global layer mask info is skipped** → Its length is honored so parsing stays
  aligned, but its contents are not modeled. Acceptable because the capability
  covers per-layer masks, not the document-level mask.
- **Endianness assumptions are structural** → All reads/writes are explicit
  `to_be_bytes`/`from_be_bytes`; there is no native-endian shortcut, so the same
  bytes are produced on every platform.

## Open Questions

- Should unknown additional-layer blocks be preserved verbatim (as `AdjustmentData`
  does) rather than skipped?
- When locked-layer editing and mask density/feather arrive, do the raw layer and
  mask flags bytes get modeled, or are they parsed into typed fields?
- The layer count can be negative (merged transparency); the reader uses its
  magnitude but does not yet expose the merged-transparency channel specially.
  Is that needed before PSB write?
