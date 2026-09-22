## Context

`clrL` is the last whitelisted adjustment key without a decoder
(`crates/pictura-render/src/composite.rs` `decode_adjustment`). Read preserves
the block with the other 21 keys through `AdjustmentData { key, data }`, so this
change only adds a decode view, an apply kernel, and an app author path — it does
not touch the codec read/write path.

The grounding probe established:

- The block is `[u16 version = 1][u32 descriptor version = 16][descriptor body]`.
  `pictura_codec::read_descriptor` already expects the `u32 == 16`, so the
  decoder must strip the leading `u16`.
- Descriptor items: `lookupType` enum (`3DLUT` / `abstractProfile` /
  `deviceLinkProfile`), `Nm  ` text, `Dthr` bool, `profile` `tdta`, `LUTFormat`
  enum (`LUTFormatCUBE` / `LUTFormat3DL` / `LUTFormatLOOK`), `dataOrder` and
  `tableOrder` enums (`rgbOrder` / `bgrOrder`), `LUT3DFileData` `tdta` (the raw
  embedded LUT file bytes), `LUT3DFileName` text.
- `LUT3DFileData` is the raw file, not an internal container (a real Photoshop
  sample stores the ASCII header of a `.CUBE`).
- psd-tools 1.19 can read and author the block; the committed ag-psd 31 oracle
  can read it (`layer.adjustment.type === 'color lookup'`).

## Goals / Non-Goals

**Goals:**
- Decode `clrL` into `Adjustment::ColorLookup(ColorLookupParams)`.
- Render a `3DLUT` whose embedded data is a `.CUBE`: parse and sample
  trilinearly, red index fastest.
- Encode a `clrL` block and let the app create a neutral (identity-cube) layer.
- Prove the block with psd-tools/ag-psd and the sampler with known-value tests.

**Non-Goals:**
- Adobe pixel parity for the lookup (no independent oracle exists).
- `.3DL`/`.LOOK` parsing and `abstractProfile`/`deviceLinkProfile` ICC
  transforms (decoded but render as a no-op).
- Dithering (`Dthr` is read but not applied), LUT editing UI, external LUT file
  loading.

## Decisions

**Model**: `ColorLookupParams { kind: ColorLookupKind, lookup: Option<Lut3d> }`
with `Lut3d { size: usize, points: Vec<[f32; 3]> }` in `.CUBE` order. Parse the
embedded `.CUBE` at decode time and keep only the parsed grid, not the raw bytes:
the raw block is already preserved by `AdjustmentData` for re-emission, and the
app's encoder builds a fresh block from source bytes, so params need not carry
them. This avoids re-parsing a large LUT on every composite.
Alternative considered: keep raw `file_data` and parse per `apply` — rejected
for the per-composite re-parse cost (a 64³ cube is 262 144 points).

**Placement**: the kernel and the `.cube` parser go in
`crates/pictura-adjust/src/lut.rs` (the docs contract names
`pictura_adjust::lut`), and the descriptor decode/encode go in
`crates/pictura-render/src/color_lookup.rs`, mirroring `selective_color.rs` so
`composite.rs` stays within its file-size budget. The `tdta` bytes are taken from
the existing `DescValue::Raw` (4-byte ostype + `u32` length + payload) rather
than adding a DOM variant, so the descriptor round-trip goldens do not move.

**`dataOrder`/`tableOrder` are metadata**: a `.CUBE` is self-describing, so the
flags do not change sampling. This is a documented assumption; if a real file is
found where they must reorder, the sampler gains the swap behind those flags.

**Scope the render to `.CUBE`**: `.3DL`/`.LOOK` and the ICC-based kinds decode
to `lookup: None` and are a no-op, the same shape as the existing "not
understood → leave the backdrop unchanged" contract.

## Risks / Trade-offs

- [Wrong `.cube` ordering renders wrong pixels] → the point order is fixed by an
  exact-node test at `red 1, green 0, blue 0`, which only passes with red
  fastest.
- [No Adobe oracle for the lookup] → marked as a ceiling with no parity claim;
  proven by identity, exact-node, and trilinear known-value tests, matching the
  repo's existing no-equivalent policy for filters.
- [`dataOrder`/`tableOrder` assumption is wrong] → they are read into the spec
  as metadata and are a one-line change to honor; documented as an open
  question.
- [A large LUT inflates `ColorLookupParams`] → size is capped at 64 per axis by
  the parser, and the block is decoded lazily per layer.

## Migration Plan

No data migration. `clrL` blocks already round-trip byte-for-byte; this adds a
decode view and an author path. Rollback is a revert of the change (the key
stays preserved).

## Open Questions

- Do `dataOrder`/`tableOrder` ever reorder a `.CUBE` in a way the file does not
  already encode? Needs a real fixture that disagrees to decide.
- Should a future change load an external `.cube` referenced by `LUT3DFileName`
  when `LUT3DFileData` is absent? Out of scope here.
