## Context

`read_psd` (`crates/pictura-codec/src/read.rs`) decodes the composite image
data by `match compression { 0 => raw, 1 => read_rle, c => Unsupported }` and
each layer channel by `match compression { 0, 1, 2|3 => Unsupported }`. Layer
records call `BlendMode::from_psd_key(key).ok_or_else(Unsupported)`, so one
unknown key aborts the file. The image-data section is read unconditionally, so
a file saved with "Maximize Compatibility" off is truncated at the composite.

Photoshop's ZIP compression is a zlib deflate stream; ZIP-with-prediction
additionally delta-encodes each scanline before deflate. For 8-bit data the
delta is byte-wise: `enc[i] = orig[i] - orig[i-1]` within a row (mod 256), so the
inverse is `orig[i] = orig[i] + orig[i-1]`. `psd-tools` applies the inverse over
the concatenated plane buffer with line width = document width, which is exactly
per-channel-row prediction because planes are concatenated row-major.

## Goals / Non-Goals

**Goals**

- Open the largest practical set of real PSDs: composite and layer channels in
  raw, RLE, ZIP, and ZIP-with-prediction.
- Never fail a whole document because a single tagged value is unrecognized
  (unknown blend key -> Normal).
- Parse a layered document whose merged composite is absent.
- A self-contained deterministic test for each new decode path, plus a
  `psd-tools` oracle when it is installed.

**Non-Goals**

- Writing ZIP or RLE (write stays raw; roadmap P3).
- Preserving the original bytes of anything the codec does not model (roadmap
  P2). An unknown blend key is lost on round-trip in P1.
- 1/16/32-bit data, non-RGB/Gray modes, or image resources.

## Decisions

### D1. One ZIP helper, zlib-first with a raw-deflate fallback

`fn inflate(payload: &[u8], expected: usize) -> Result<Vec<u8>, PsdError>` wraps
`flate2::read::ZlibDecoder`. Photoshop writes zlib-framed deflate; if the zlib
header is absent (some third-party writers emit raw deflate) fall back to
`flate2::read::DeflateDecoder`. A decode shorter than `expected` is
`PsdError::Invalid`; longer is truncated to `expected`. A decode error is
`PsdError::Unsupported("ZIP ...")` so callers can distinguish "we cannot do
this" from "the file is malformed".

### D2. Prediction inverse

`fn undo_prediction(data: &mut [u8], row_len: usize)` walks each `row_len`
chunk and does `data[i] = data[i].wrapping_add(data[i - 1])` for `i` after the
first byte of the row. It is applied only when the compression code is 3, and is
correct for 8-bit (the only depth supported).

### D3. Composite decode

`read_psd` matches compression `0/1/2/3`, and for 2/3 consumes all remaining
reader bytes (the image-data section is last), inflates to
`channels * width * height`, and for 3 runs `undo_prediction(data, width)`.
`read_layer_section` already positions the reader at the image-data section.

### D4. Layer channel decode

`read_channel_data` inflates `declared_len - 2` payload bytes to `width * height`
and runs `undo_prediction(data, width)` for code 3. Identical helper to D3.

### D5. Unknown blend key degrades

`BlendMode::from_psd_key(key).unwrap_or(BlendMode::Normal)`. The `8BIM` signature
check still errors on a corrupt record; only an unrecognized 4-byte key falls
back. Document this as a lossy degrade until P2 preserves the raw record.

### D6. Absent composite

Immediately before reading the image-data compression word, if the reader has no
remaining bytes and the layer section produced layers, return a `Document` whose
`composite` is a zeroed `PixelBuffer` of the header dimensions/mode and whose
`channels` are empty. A header without any trailing data at all (no layers, no
composite) is still malformed and errors as today.

## Risks / Trade-offs

- **Unknown blend key loses its original value** until P2 — accepted and
  documented.
- **Zero-filled composite** for a compat-off file displays blank until the layers
  are composited; rendering the layer stack as a fallback composite is P3.
- **flate2** is a new dependency; it uses the pure-Rust `miniz_oxide` backend, so
  no C toolchain requirement.
