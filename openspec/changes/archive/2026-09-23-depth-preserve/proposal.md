## Why

`read_psd` narrows a 16/32-bit document to 8-bit for the engine and
`write_psd` always writes an 8-bit header, so an open→save silently downgrades
a high-depth file to 8-bit. That is the last lossy-in-depth gap in the codec
(roadmap G4/P4). This change keeps the source depth on save: an unedited plane
is re-emitted with its exact source-depth samples, and an edited plane keeps the
source depth with the 8-bit working precision widened into it.

## What Changes

- `read_psd` retains the decoded **source-depth samples** of the composite color
  planes, the document extra channels, and every layer channel (color,
  transparency, mask, and unmodeled), alongside the existing 8-bit narrowing.
  This happens only for a Grayscale or RGB document, where the read path's
  color-mode normalization is a no-op; a mode the read path converts (CMYK,
  Lab) keeps no source samples and still saves 8-bit.
- `write_psd` writes the output header at the document's source depth
  (16 or 32) when one is recorded, else 8. For each plane it compares the
  current 8-bit bytes with the narrowing of the retained samples:
  - **unchanged** → re-encode the retained source-depth samples at the recorded
    compression (raw, PackBits RLE, ZIP, or ZIP-with-prediction with the
    depth-specific predictor);
  - **changed** (edited, or a layer that moved) → widen the 8-bit plane to the
    source depth (`v * 257` at 16-bit, the 8-bit value scaled to `[0,1]` at
    32-bit) and encode it; this is documented as an approximation that cannot
    recover the source low bits or HDR range.
- The application's depth notice changes: opening a 16/32-bit file still reports
  a conversion for editing, but no longer claims the save will be 8-bit.
- Ceilings (`ponytail:`): retained source samples cost 2×/4× the plane size;
  the widened approximation is not true 16-bit editing; the depth-1 Bitmap and
  the mode-normalized paths are unchanged.

## Capabilities

### Modified Capabilities
- `psd-bit-depth`: the "records its source depth and saves as 8-bit" requirement
  is replaced (a document now preserves its source depth on save); the layer
  channel-narrowing requirement and the application notice requirement change
  to match.

## Impact

- `crates/pictura-core`: a retained-source store on `Document` and `Layer`.
- `crates/pictura-codec`: capture in `read.rs`, depth-aware encoders in
  `depth.rs`/`write.rs`, and the depth-driven header and plane encoding. No new
  dependency.
- `crates/pictura-app`: the depth notice text (and its self-test string).
- Tests: a `psd-tools`-backed write oracle proving the output depth and samples,
  plus the existing depth oracle updated from "saves as 8-bit".
