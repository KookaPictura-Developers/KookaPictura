# M6-E — ImageMagick filter oracle

These tests diff `pictura_filters::apply` against **ImageMagick** for the
Blur / Sharpen / Noise filters that have a usable equivalent. This is a *sanity*
oracle, not a parity oracle: Adobe's exact integer math and convolution kernels
are closed, and the ImageMagick operators only approximate several Photoshop
paths. The mapping table, the IM flags, the tolerances and the known divergences
are below.

- Oracle script: `scripts/filter_oracle.py`
- Tests: `crates/pictura-filters/tests/oracle.rs`
- Verified against: ImageMagick **7.1.2-29 Q16-HDRI** (2026-07-27).

Only filters with a faithful ImageMagick operator are diffed (`GaussianBlur`,
`BoxBlur`, `Median`, `UnsharpMask`). The rest are covered by
ImageMagick-independent property/known-value tests (here and in the module unit
tests); the measured divergences that ruled out a differential test are recorded
below. No test is `#[ignore]`d.

## Filter → ImageMagick mapping

Test image: deterministic 16×16 RGB8, two ramps (R = 16x, G = 16y) plus a 4×4
block checker in B, so every filter sees both gradients and hard edges. All
"measured max delta" figures are absolute 8-bit sample deltas against this image.

| `Filter` | ImageMagick | Faithful? | Tolerance | Notes |
|---|---|---|---|---|
| `GaussianBlur` | `-gaussian-blur 0x{sigma}` (`sigma = radius/3`) | yes | 0 | Same 3σ separable Gaussian, clamp-to-edge; measured max delta 0 (radius 3.0 → σ 1.0). |
| `BoxBlur` | `-statistic mean NxN` (`N = 2·radius+1`) | yes | 0 | Same separable moving average, clamp-to-edge; measured max delta 0 (radius 3). |
| `Median` | `-median {radius}` | yes | 0 | Same `(2r+1)²` per-channel rank filter, clamp-to-edge; measured max delta 0 (radius 1). |
| `UnsharpMask` | `-unsharp 0x{sigma}+{amount/100}+{threshold/255}` | yes | 6 | Same blur-difference gain; IM `amount` is a **fraction** (`1.0` = 100 %). IM's internal blur differs slightly from its standalone `-gaussian-blur`; measured max delta 5 at radius 3.0 / amount 150 / threshold 0. |
| `MotionBlur` | — | **no** | — | IM `-motion-blur` builds a **one-sided Gaussian line** kernel; Pictura averages symmetric uniform taps. Observed max delta 86 (`-motion-blur 0x5+0`). |
| `RadialBlur` | — | **no** | — | ImageMagick 7 has **no** `-radial-blur` (IM 6 only); `-rotational-blur` weights a different angle profile. Observed max delta 58 (Spin 20 vs `-rotational-blur 20`). |
| `Average` | — | **no** | — | Trivial global region mean; no IM operator shares the window/border semantics. Observed max delta 92 (`-statistic mean 16x16`). |
| `Blur` | — | **no** | — | PS fixed `[1 2 1]` separable kernel; IM `-blur` is a Gaussian. Observed max delta 23 (`-blur 0x1`). |
| `BlurMore` | — | **no** | — | Three passes of the PS `[1 2 1]` kernel; observed max delta 14 (`-blur 0x1`). |
| `SurfaceBlur` | — | **no** | — | Bilateral range/space weights are closed; IM has no bilateral operator. Observed max delta 32 (`-gaussian-blur 0x1`). |
| `Sharpen` | — | **no** | — | Fixed 3×3 high-pass kernel; IM `-sharpen` is a Gaussian unsharp. Observed max delta 17 (`-sharpen 0x1`). |
| `SharpenMore` | — | **no** | — | Fixed stronger 3×3 high-pass kernel; observed max delta 17 (`-sharpen 0x1`). |
| `SharpenEdges` | — | **no** | — | Edge-gated high-pass with a fixed gate; observed max delta 17 (`-sharpen 0x1`). |
| `AddNoise` | — | **no** | — | RNG streams differ; the contract is same-seed determinism. Observed max delta 79 (`-attenuate 0.1 +noise Gaussian`). |
| `Despeckle` | — | **no** | — | IM `-despeckle` uses a different rank detector. Observed max delta 13 (`-despeckle`). |

### Why `MotionBlur` is not diffed

The M6 plan suggested `-motion-blur 0x{distance}+{angle}`, but the two operators
are not comparable. An impulse test (a single white pixel) shows ImageMagick's
`-motion-blur` response is a **one-sided** Gaussian-weighted line:

```
magick -motion-blur 2x1.0+0   impulse centre row: [2, 20, 88, 145]  (left of centre)
magick -motion-blur 0x5+0     impulse centre row: [10,14,18,23,27,31,35,37,38, …]
```

Pictura's `MotionBlur` averages `distance` taps placed **symmetrically** about
the pixel (`half = (distance−1)/2`, uniform weights). No `-motion-blur`
invocation reproduces that, and `-convolve` with the same kernel would just
re-implement it, so no faithful operator exists. The observed delta against the
suggested invocation is recorded above; the tap geometry is guarded by
`motion_blur_known_values`.

## Exact ImageMagick flags

```bash
magick -size 16x16 -depth 8 rgb:IN.rgb \
    <OPERATOR ARGS> \
    -depth 8 rgb:OUT.rgb
```

The script handles a raw planar (`--planar`, channel planes like
`PixelBuffer::data`) or interleaved `rgb`/`rgba` 8-bit image and converts to the
interleaved form ImageMagick reads. `--channels 4` uses `rgba:`, but most IM
operators also touch alpha, so the tests use 3 channels and alpha is never in
play.

| Filter | Flags |
|---|---|
| Gaussian Blur | `-gaussian-blur 0x{sigma}` |
| Box Blur | `-statistic mean {N}x{N}`, `N = 2·radius + 1` |
| Motion Blur | `-motion-blur 0x{distance}+{angle}` (documented, **not** used by a differential test) |
| Median | `-median {radius}` |
| Unsharp Mask | `-unsharp 0x{sigma}+{amount/100}+{threshold/255}` |

### Unsharp Mask scaling

Empirically (16×16 test image, `pictura_filters` with σ = radius/3 against
`-gaussian-blur 0xσ`):

- IM `amount` is a **fraction**: `amount/100` (so Pictura 150 → IM `1.5`).
  Passing the percentage literally (`+150`) diverges by 43.
- IM `threshold` is a **fraction of the quantum range**: `threshold/255` (so
  Pictura level 8 → IM `0.0314`), matching `-threshold`'s percentage convention
  used by the M4 oracle.
- The residual (≤5) is IM's `-unsharp` applying a slightly different internal
  blur than its standalone `-gaussian-blur`; it shrinks to 1 at larger radii
  (Pictura radius 6.0 → σ 2.0). The differential uses radius 3.0 / amount 150 /
  threshold 0 with tolerance 6.

## Known divergences (IM vs Photoshop / Pictura)

- **Motion blur** is directional in IM and symmetric-uniform in Pictura; see
  above. Observed max delta 86.
- **Radial blur.** IM 7 dropped `-radial-blur`; `-rotational-blur` is a
  convolution with an angle falloff, not Pictura's polar resample + midpoint
  smear. Observed max delta 58.
- **Average** is the exact global mean; `-statistic mean WxH` is a per-pixel
  windowed mean with clamp-to-edge borders, so it is not the same operator
  (observed max delta 92). Guarded by the constant-mean property test.
- **Sharpen family** uses fixed 3×3 high-pass kernels; IM `-sharpen` is a
  Gaussian unsharp (observed max delta 17). Guarded by flat-field identity and
  edge-contrast property tests.
- **Blur / Blur More** use the fixed `[1 2 1]` separable kernel (Blur More = 3
  passes); IM `-blur` is a Gaussian (observed max delta 23 / 14). Guarded by the
  solid-field near-identity property test.
- **Surface Blur** is bilateral; no IM operator (observed max delta 32 vs a
  plain Gaussian). Guarded by the zero-radius identity test.
- **Despeckle** uses fixed spike/band thresholds; IM `-despeckle` is a different
  rank detector (observed max delta 13).
- **Add Noise** is randomized; IM's RNG stream is different and not reproducible
  (observed max delta 79). Guarded by the same-seed determinism test at the
  `Filter::apply` level.
- **16/32-bit, CMYK/Lab, selection/mask/gpu wiring.** Out of M6 scope.

Tolerances apply only to the four differential rows. They are absolute
per-8-bit-sample allowances for `pictura_testkit::compare`: 0 where the formula
is identical and integer arithmetic agrees exactly, 6 for Unsharp Mask where
IM's internal blur can differ. Rows marked **no** keep tolerance 0 — they are
not compared to IM.

## Running

All tests run with a plain `cargo test`; the differential ones skip (with a
message) when `magick` is not on `PATH`:

```bash
cargo test -p pictura-filters
```

Nothing is `#[ignore]`d.

Manual inspection:

```bash
python3 scripts/filter_oracle.py version
python3 scripts/filter_oracle.py apply --size 16x16 --planar \
    --op gaussian --sigma 1.0 IN.rgb OUT.rgb
python3 scripts/filter_oracle.py apply --size 16x16 --op box --radius 3 \
    IN.rgb OUT.rgb
python3 scripts/filter_oracle.py apply --size 16x16 --planar \
    --im-args="-motion-blur 0x5+45" IN.rgb OUT.rgb
```

## Regeneration

There are no committed fixtures: the oracle runs ImageMagick at test time and
skips (with a message) when `magick` is not on `PATH`. To reproduce the
reference bytes for a filter by hand:

```bash
# planar 16x16 RGB8 input -> IM result, then compare against apply's output
python3 scripts/filter_oracle.py apply --size 16x16 --planar --op unsharp \
    --sigma 1.0 --amount 150 --threshold 0 in.rgb reference.rgb
```

`MAGICK=/path/to/magick` overrides the binary; `--im-args=...` must be attached
with `=` so argparse does not read the leading `-` as another option.

## Not expressed by this oracle

- Motion Blur, Radial Blur, Average, Blur / Blur More, Surface Blur, Despeckle,
  Add Noise, and the Sharpen family: no faithful ImageMagick equivalent (see the
  table); covered by property/known-value tests instead.
- Alpha-channel behaviour: the tests use 3-channel buffers, and `apply` is
  specified never to modify channel 4 (guarded by the module unit tests).
- 16/32-bit filter math (M6 is 8-bit only).
