## Context

`read_psd` decodes each plane at the header depth and immediately narrows it
(`read.rs:161-163` for the composite, `read.rs:955-972` for layer channels),
recording only `Document.source_depth` (`read.rs:74-78`) and forcing
`Document.depth = Eight` (`read.rs:173`). `write_psd` rejects a non-8-bit
document (`write.rs:643-645`) and hardcodes the header depth `8u16`
(`write.rs:688`). This change keeps the narrowed 8-bit model untouched and adds
a retained source-depth copy plus depth-aware output.

## Goals / Non-Goals

**Goals:**
- An open→save of a 16/32-bit Grayscale or RGB document keeps the source depth.
- An unchanged plane re-emits its exact source-depth samples.
- An edited plane keeps the source depth with the 8-bit bytes widened.
- All recorded compression kinds work at the source depth.
- The output is valid to an independent decoder (`psd-tools`).

**Non-Goals:**
- True 16/32-bit editing in the engine (the model stays 8-bit).
- A `PixelBuffer` sample model, GPU depth, or a 32-bit HDR tone map.
- Preserving depth for a mode the read path converts (CMYK/Lab) or depth-1.
- Recovering the source low bits or HDR range of an edited plane.

## Decisions

- **Retain, do not re-encode byte streams.** Store the decoded source-depth
  samples (not the original compressed stream) so the write path has one uniform
  input: a native-depth byte plane per channel. The writer then re-encodes with
  the recorded compression, so a changed plane and an unchanged plane share one
  encoder. Ceiling (`ponytail:`): this costs 2×/4× the plane size while a
  high-depth document is open, and does not preserve the source *bytes* when the
  recorded compression differs — only the sample values.
- **Storage on `Document` and `Layer`, not `Channel`.** `Channel` has no
  `Default` and many literal sites; a new field there is a wide edit. Add
  `Document.source_planes: Option<SourcePlanes>` (composite color planes and
  document extra channels) and `Layer.source_channels: Option<SourceChannels>`
  (the layer's channels keyed by id, including `-1`, `-2`, and unmodeled). A
  layer's retained samples travel with the layer, so add/delete/reorder cannot
  desync; a moved layer is detected by comparing its retained `rect` with the
  current `rect` and falls back to widening. Only `read.rs:103`, `read.rs:169`,
  and `Document::new` build a `Document` literal directly; a `..Default::default()`
  site in `pictura-render` is unaffected.
- **Depth-aware encoders are small.** PackBits is byte-wise, so `encode_scanlines`
  only needs a row length of `row_bytes(width, depth)` instead of `width`; ZIP is
  a plain deflate of the concatenated native bytes. Only ZIP-with-prediction
  needs a depth branch: at 16-bit a per-`u16` forward difference, at 32-bit the
  byte difference plus the forward four-byte-plane shuffle — the inverses of
  `depth.rs::undo_prediction` (`depth.rs:70-111`) and `depth.rs::unshuffle_32`
  (`depth.rs:116-134`). The forward helpers already exist as test-only
  `predict16`/`predict32` (`crates/pictura-codec/src/tests/depth.rs:17-55`);
  promote them into `depth.rs` and share with the tests.
- **Exactness check.** A plane is "unchanged" when `narrow(retained)` equals its
  current 8-bit bytes (the narrowing functions are `color_mode.rs:43-45`/`53-55`).
  This needs no edit tracking and is computed once per plane at write time.
- **Scope by mode.** Retain only when `source_depth` is 16/32 *and* the mode is
  Grayscale or RGB (the `normalize` no-op, `read.rs:240-242`). A converted mode
  retains nothing and saves 8-bit, matching the existing behaviour.
- **Output depth from `source_depth`, not `depth`.** The model `depth` stays
  `Eight`, so the `write.rs:643` guard and the "reject a non-8-bit document"
  scenario are unchanged; the header and encoders read the output depth from
  `source_depth` (defaulting to 8).

## Risks / Trade-offs

- **No write-side depth oracle today.** `undo_prediction` has no committed
  forward counterpart, so a bad encoder silently produces a file `psd-tools`
  mis-decodes. Mitigation: add the `psd-tools` write oracle (depth + native
  samples) *first*, then implement against it.
- **Memory.** Retained samples double/quadruple the high-depth plane storage
  while open; marked as a ceiling. A future sample model removes the copy.
- **Widening is lossy by construction.** Documented; the spec's requirement says
  so explicitly.
- **Layer matching.** Keying retained channels per `Layer` and comparing `rect`
  keeps add/delete/reorder safe; a mismatch only widens (safe, never a
  corruption).

## Migration

A constructed or 8-bit document is byte-identical (source store empty, header
8). A high-depth Grayscale/RGB document saves at its source depth; the app
notice is updated so the user is not told the save is 8-bit.
