# M4-C — ImageMagick adjustment oracle

These tests diff `pictura_adjust::apply` against **ImageMagick** for the
adjustments that have a usable equivalent. This is a *sanity* oracle, not a
parity oracle: Adobe's exact integer math is closed and the ImageMagick
operators only approximate several Photoshop paths. The mapping table, the IM
flags, the tolerances and the known divergences are below.

- Oracle script: `scripts/adjust_oracle.py`
- Tests: `crates/pictura-adjust/tests/oracle.rs`
- Verified against: ImageMagick **7.1.2-29 Q16-HDRI** (2026-07-27).

Only adjustments with a faithful ImageMagick operator are diffed (`Levels`,
`Invert`, `Desaturate`). Five operators that looked equivalent but diverged
semantically are now classified as having **no** equivalent and are covered by
ImageMagick-independent property/known-value tests; the measured divergences are
recorded below. No test is `#[ignore]`d.

## Adjustment → ImageMagick mapping

| `Adjustment` | ImageMagick | Faithful? | Tolerance | Notes |
|---|---|---|---|---|
| `Levels` | `-level B%,W%,g +level Ob%,Ow%` | yes | 1 | Same input-remap-then-output-remap formula as the spec; rounding only. |
| `Curves` | — | **no** | — | Arbitrary monotone control-point curve; no IM operator. |
| `BrightnessContrast` | — | **no** | — | IM `-brightness-contrast` algorithm differs; observed max delta 14. |
| `Exposure` | — | **no** | — | PS applies the transfer in **linear light**; IM `-evaluate`/`-gamma` run in the encoded space. |
| `HueSaturation` | — | **no** | — | IM `-modulate` operates in its own HSL space, not PS HSL; observed max delta 45. |
| `ChannelMixer` | — | **no** | — | IM `-color-matrix` takes fractions, PS uses percent weights; observed max delta 252. |
| `BlackWhite` | — | **no** | — | Hue-sector decomposition; no IM operator. |
| `PhotoFilter` | — | **no** | — | No faithful IM operator. |
| `Vibrance` | — | **no** | — | No faithful IM operator. |
| `ColorBalance` | — | **no** | — | No faithful IM operator. |
| `Auto` | — | **no** | — | Closed Auto Color Correction solver. |
| `Invert` | `-negate` | yes | 0 | Identical `M - v` per channel. |
| `Posterize` | — | **no** | — | IM `-posterize` bins on an adjacent quantization level; observed max delta 85. |
| `Threshold` | — | **no** | — | IM `-threshold` uses Rec.709 luma, PS uses Rec.601; observed max delta 255. |
| `Desaturate` | `-modulate 100,0,100` | yes | 1 | IM modulate saturation 0 yields HSL `(min+max)/2`, exactly the PS Desaturate formula. |

### Why `-modulate 100,0,100` and not `-colorspace Gray`

`-colorspace Gray` uses Rec.709 luma (`0.2126, 0.7152, 0.0722`): pure red →
54. Photoshop Desaturate is `(min+max)/2` (HSL `L` with `S=0`): pure red → 128.
The task suggested `-colorspace Gray` as a rough stand-in, but it is *not* a
match; `-modulate 100,0,100` is. `-colorspace Gray` remains the operator the
script exposes as `--op gray` for the Hue/Saturation-style luma path.

## Exact ImageMagick flags

```bash
magick -size 8x8 -depth 8 rgb:IN.rgb \
    <OPERATOR ARGS> \
    -depth 8 rgb:OUT.rgb
```

The script handles a raw planar (`--planar`, channel planes like
`PixelBuffer::data`) or interleaved `rgb`/`rgba` 8-bit image and converts to the
interleaved form ImageMagick reads. `--channels 4` uses `rgba:`, but note that
most IM operators also touch alpha (e.g. `-negate` flips it); the tests use 3
channels so alpha is never in play.

Bare numeric arguments to `-level`, `+level`, `-threshold` and additive
`-evaluate` are **quantum** values on a Q16 build, not 8-bit values. The script
always passes percentages (`pct(v) = v/255*100%`) so the 8-bit intent survives.

| Operation | Flags |
|---|---|
| Levels | `-level 0%,100%,1.0 +level 0%,100%` (percents for B/W/Ob/Ow, raw float for gamma) |
| Gamma | `-gamma g` — IM computes `v^(1/g)`, matching the PS Levels convention |
| Brightness/Contrast | `-brightness-contrast BxC` — both are percentages of the range |
| Invert | `-negate` |
| Posterize | `-posterize n` |
| Threshold | `-threshold T%` |
| Gray | `-colorspace Gray` (Rec.709 luma) |
| Desaturate | `-modulate 100,0,100` |
| Hue/Saturation | `-modulate 100,S,H` |
| Channel Mixer | `-color-matrix "m0,…,m8"` |
| Exposure (approx) | `-evaluate multiply 2^E` / `-evaluate add O%` / `-gamma G` — encoded space only |

## Known divergences (IM vs Photoshop)

- **Linear light.** Photoshop Exposure works in gamma-1.0 linear light and
  re-encodes; IM's operators are gamma-encoded. There is no faithful 8-bit IM
  equivalent, so no differential test is attempted.
- **Brightness/Contrast.** The IM operator is not equivalent to the PS legacy
  path: PS applies an additive brightness shift (normalized by 150) then a
  linear contrast about mid-grey; IM scales in its own curve. The differential
  test was retired after observing a max delta of 14. The property test guards
  neutral identity, monotonicity in brightness, and that `+N` raises the mean.
- **Hue/Saturation law.** PS's saturation multiply and the color-range
  trapezoid are undocumented; IM `-modulate` uses its own HSL-based law. They
  are not the same operator (observed max delta 45), so the differential test
  was retired. The property test guards identity at 0/0/0 and that `+S`
  increases mean channel spread.
- **Threshold luma.** PS uses Rec.601 luma weights; IM uses Rec.709. They
  disagree on the primaries (observed max delta 255), so the differential test
  was retired. The property test guards binarization to `{0,255}` and that the
  split is monotone in the level (pixels only turn white → black as `T` rises).
- **Posterize binning.** IM `-posterize` assigns samples to an adjacent
  quantization level rather than PS's uniform `round(v·(n−1)/M)` rule
  (observed max delta 85), so the differential test was retired. The property
  test guards that every output sample is an allowed quantization level and
  that `Posterize(255)` is the 8-bit identity.
- **Channel Mixer scale.** PS weights are percentages (`100` = identity); IM
  `-color-matrix` takes fractions (`1` = identity), so the two operators are not
  comparable as written (observed max delta 252). IM 7.1.2 also ignores the
  Constant/offset column. The property test guards the exact identity matrix and
  a hand-computed green-blend result.
- **Per-channel / CMYK / Lab / 16-bit.** Out of M4 scope and not exercised.

Tolerances apply only to the three differential rows. They are absolute
per-8-bit-sample allowances for `pictura_testkit::compare`: 0 where the formula
is identical and integer arithmetic agrees exactly, 1 where only rounding can
differ. Rows marked **no** keep tolerance 0 — they are not compared to IM.

## Running

All tests run with a plain `cargo test`; the differential ones skip (with a
message) when `magick` is not on `PATH`:

```bash
cargo test -p pictura-adjust
```

Nothing is `#[ignore]`d.

Manual inspection:

```bash
python3 scripts/adjust_oracle.py version
python3 scripts/adjust_oracle.py apply --size 8x8 --planar \
    --im-args="-level 0%,100%,2.0" IN.rgb OUT.rgb
python3 scripts/adjust_oracle.py apply --size 8x8 --op desaturate IN.rgb OUT.rgb
```

## Regeneration

There are no committed fixtures: the oracle runs ImageMagick at test time and
skips (with a message) when `magick` is not on `PATH`. To reproduce the
reference bytes for an adjustment by hand:

```bash
# planar 8x8 RGB8 input -> IM result, then compare against apply's output
python3 scripts/adjust_oracle.py apply --size 8x8 --planar --op posterize \
    --levels 4 in.rgb reference.rgb
```

`MAGICK=/path/to/magick` overrides the binary; `--im-args=...` must be attached
with `=` so argparse does not read the leading `-` as another option.

## Not expressed by this oracle

- 16/32-bit adjustment math (M4 is 8-bit only).
- Per-channel Levels/Curves and Hue/Saturation color ranges (composite/Master
  only in M4).
- Alpha-channel behaviour: the tests use 3-channel buffers, and `apply` is
  specified never to modify channel 4.
- `Curves`, `Exposure`, `BlackWhite`, `PhotoFilter`, `Vibrance`, `ColorBalance`
  and `Auto`, plus `BrightnessContrast`, `HueSaturation`, `ChannelMixer`,
  `Posterize` and `Threshold`: no faithful ImageMagick equivalent (see the
  table). The latter five are covered by property/known-value tests.
