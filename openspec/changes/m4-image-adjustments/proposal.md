## Why

`Image > Adjustments` needs working destructive math before adjustment layers can
reuse it. M4-A and M4-C already landed that math in `crates/pictura-adjust` and
diffed part of it against ImageMagick, but the contract only lives in prose
(`docs/dev/m4-adjustments.md`) and in the code. This change records the shipped
behavior as reviewable requirements so later waves (adjustment layers, 16-bit,
per-range edits) can build on a frozen contract.

## What Changes

- New `pictura-adjust` crate exposes `apply(adjustment, buf) -> Result<(), AdjustError>`
  over a planar 8-bit `PixelBuffer` with 3 or 4 channels.
- 15 destructive adjustments ship: Levels, Curves, Brightness/Contrast, Exposure,
  Invert, Posterize, Threshold, Desaturate, Auto (Tone/Contrast/Color),
  Hue/Saturation, Black & White, Photo Filter, Channel Mixer, Vibrance, Color
  Balance.
- Alpha (channel 4) is never modified; malformed buffers and out-of-range
  parameters return `AdjustError`, never panic; every operation is deterministic.
- `scripts/adjust_oracle.py` exposes ImageMagick operators and
  `crates/pictura-adjust/tests/oracle.rs` diffs Levels, Invert, and Desaturate
  against them. The other twelve adjustments have no faithful ImageMagick
  operator (measured divergences documented in `tests/README.md`) and are covered
  by known-value and property tests instead.
- Out of scope for this change: Gradient Map, Selective Color, Shadow/Highlight,
  HDR Toning, Match Color, Replace Color, 3D LUT/Color Lookup, per-range
  Hue/Saturation, per-channel Levels/Curves, and 16/32-bit math.

## Capabilities

### New Capabilities

- `image-adjustments`: the destructive adjustment math in `pictura-adjust`, the
  parameter contract for each of the 15 adjustments, the shared invariants
  (in-place planar application, alpha preservation, deterministic errors), and
  the ImageMagick differential oracle with its no-equivalent classification.

### Modified Capabilities

None. No existing capability has requirements yet.

## Impact

- `crates/pictura-adjust` (library plus `tests/oracle.rs`), new crate in the
  workspace.
- `scripts/adjust_oracle.py`, a test-time helper that drives ImageMagick.
- Dependencies: `pictura-core` and `thiserror` (workspace) for the library;
  `pictura-testkit` as a dev-dependency. No new external crates.
- ImageMagick is optional. Differential tests skip with a message when `magick`
  is not on `PATH`; nothing is marked `#[ignore]`.
- Follows `docs/dev/m4-adjustments.md` and
  `docs/04-image-ops/adjustments/*.md`; those specs are not modified.
