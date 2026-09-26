# Proposal: image-resize-bicubic-kernel

## Why

`pictura_ops::resize` implements `Resample::Bicubic` as a Catmull-Rom cubic
(`a = -0.5`), and the ImageMagick oracle validates it against `-filter catrom`,
with a note asserting Catmull-Rom is "the faithful operator". That assertion is
wrong: the published public analysis of Photoshop's resampling (Jason
Summers, entropymine.com/resamplescope) puts Photoshop's **Bicubic** at a
Mitchell–Netravali `cubic(B=0, C=0.75)` kernel (`a = -0.75`), **not**
Catmull-Rom. Every bicubic resize therefore uses the wrong kernel.

## What Changes

- Change `Resample::Bicubic`'s kernel from Keys `a = -0.5` (Catmull-Rom) to
  `a = -0.75` (B=0, C=0.75), keeping the existing per-channel 4×4 convolution,
  edge clamp, and weight normalization unchanged.
- Update the ImageMagick oracle to compare Bicubic against `-filter cubic` with
  `-define filter:b=0 -define filter:c=0.75` (the exact kernel), and record the
  measured delta. `scripts/ops_oracle.py` gains `--im-args` support on the
  `resize` op so the defines can be passed.
- The three-variant enum, entry point, and all other kernels are unchanged.

## Capabilities

### Modified Capabilities

- `image-resize`: the Bicubic kernel coefficients and its ImageMagick mapping.

## Impact

- `crates/pictura-ops/src/resize.rs` (`cubic`), `crates/pictura-ops/tests/oracle.rs`
  (Bicubic row + test), `scripts/ops_oracle.py` (`resize` accepts `--im-args`).
- Updated `openspec/specs/image-resize/spec.md` (MODIFIED requirements).
- Behavioral change: Bicubic output differs from before, so any committed golden
  that used Bicubic must be regenerated. No PSD codec path is touched.
- No new dependency; the oracle still self-skips without `magick`.
