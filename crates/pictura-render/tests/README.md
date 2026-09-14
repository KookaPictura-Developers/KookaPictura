# M2-B — ImageMagick compositing oracle

These tests diff `pictura_render::composite_rgba` against **ImageMagick**, a
second, independent implementation of the blend modes. ImageMagick implements
the SVG/W3C Compositing blend operators, so for the modes it supports the two
should agree up to integer rounding; where it uses a deliberately different
algorithm the mode is not used as an oracle at all.

- Scenes and reference generation: `scripts/im_compose.py`
- Fixtures: `crates/pictura-render/tests/fixtures/*.rgba`, raw **interleaved
  RGBA8**, 8×8, row-major. `*_base.rgba` / `*_src.rgba` are the two input
  layers (base = destination, src = source); `<scene>_<Mode>.rgba` is the
  `magick -compose` result for that PSD mode.
- Tests: `crates/pictura-render/tests/oracle.rs`. The tests that call
  `composite_rgba` are `#[ignore]`d until task M2-A lands.
- Verified against: ImageMagick **7.1.2-29 Q16-HDRI**.

## PSD mode → ImageMagick compose operator

19 of the 27 modes have an ImageMagick operator that computes the same function.
The remaining 8 do not and are covered by hand-computed W3C unit tests in
`pictura-render` instead (per the M2 exit gate).

| PSD mode | PSD key | IM `-compose` operator | Supported | Tolerance | Notes |
|---|---|---|---|---|---|
| Normal | `norm` | `Over` | yes | 0 | Porter–Duff source-over |
| Dissolve | `diss` | — | **no** | — | stochastic; IM needs `-define compose:args` and uses its own dither pattern |
| Darken | `dark` | `Darken` | yes | 0 | |
| Multiply | `mul ` | `Multiply` | yes | 0 | |
| Color Burn | `idiv` | `ColorBurn` | yes | 0 | |
| Linear Burn | `lbrn` | `LinearBurn` | yes | 0 | |
| Darker Color | `dkCl` | — | **no** | — | IM `DarkenIntensity` compares luminance, Photoshop compares the channel **sum** |
| Lighten | `lite` | `Lighten` | yes | 0 | |
| Screen | `scrn` | `Screen` | yes | 0 | |
| Color Dodge | `div ` | `ColorDodge` | yes | 0 | |
| Linear Dodge (Add) | `lddg` | `LinearDodge` | yes | 0 | `min(1, Cb+Cs)`, keeps Over alpha (unlike `Plus`, which adds alpha) |
| Lighter Color | `lgCl` | — | **no** | — | IM `LightenIntensity` compares luminance, not the channel sum |
| Overlay | `over` | `Overlay` | yes | 0 | |
| Soft Light | `sLit` | — | **no** | — | this IM build's `SoftLight` is not the W3C/PS formula (grey-on-grey returns white) |
| Hard Light | `hLit` | `HardLight` | yes | 0 | |
| Vivid Light | `vLit` | `VividLight` | yes | 1 | |
| Linear Light | `lLit` | `LinearLight` | yes | 0 | |
| Pin Light | `pLit` | `PinLight` | yes | 0 | |
| Hard Mix | `hMix` | `HardMix` | yes | 0 | threshold at 0.5; the exact `Cb+Cs == 1` boundary differs, the scene avoids it |
| Difference | `diff` | `Difference` | yes | 0 | |
| Exclusion | `smud` | `Exclusion` | yes | 0 | |
| Subtract | `fsub` | `MinusSrc` | yes | 0 | PS Subtract = base − source. IM's operand order is reversed vs. its docs, so `MinusSrc` (not `MinusDst`) yields base − source |
| Divide | `fdiv` | `DivideSrc` | yes | 0 | PS Divide = base / source; same operand-order reversal, so `DivideSrc` |
| Hue | `hue ` | — | **no** | — | IM `Hue` blends in HSL space, not SVG `SetSat`/`SetLum`; measured divergence up to 70/255 |
| Saturation | `sat ` | — | **no** | — | IM `Saturate` blends in HSL space; divergence up to 94/255 |
| Color | `colr` | — | **no** | — | IM `Colorize` blends in HSL space; divergence up to 92/255 |
| Luminosity | `lum ` | — | **no** | — | IM `Luminize` blends in HSL space; divergence up to 35/255 |

`Plus` is **not** used for Linear Dodge: `Plus` also adds the alpha channels,
whereas Photoshop's Linear Dodge uses normal source-over alpha. IM's
`Subtract`/`Add` are the modulus (wrap-around) operators and are also not used.

The 8 unsupported modes stay covered by M2-A's hand-computed W3C unit tests:
Dissolve (stochastic), Darker/Lighter Color (channel-sum comparison), Soft
Light (formula variant), and Hue/Saturation/Color/Luminosity (IM uses HSL-space
blending rather than the SVG non-separable model).

## Scenes

| Scene | Base | Source | Modes generated |
|---|---|---|---|
| `solid` | uniform `(64,128,192,255)` | uniform `(200,100,50,255)` | all 19 |
| `ramp` | opaque color gradient | opaque color gradient | all 19 |
| `alpha` | opaque color gradient | solid color, varying alpha | `Normal` only |

`solid` isolates the blend function per channel; `ramp` sweeps many color
pairs; `alpha` validates source-over with a partial-alpha source. Blend modes
other than Normal are not tested with a partial-alpha source because
ImageMagick's blend operators do not reproduce the W3C source-over mix there.

## Regenerating the fixtures

Requires `magick` (ImageMagick 7) on `PATH` (override with `MAGICK=...`) and
`python3`. No Python packages are needed.

```bash
python3 scripts/im_compose.py gen     # rewrite tests/fixtures/
python3 scripts/im_compose.py check   # regenerate in a temp dir and diff
python3 scripts/im_compose.py list    # `magick -list compose` (add --grep)
```

`im_compose.py compose <operator> <base> <src> <out>` composites two layers
directly; inputs/outputs may be `.png` or raw `.rgba`. Internally the script
routes through PNG: reading and writing raw RGBA directly makes ImageMagick
mangle the alpha channel of some operators (notably `Difference`), while PNG
keeps the alpha semantics correct.

The scenes are pure functions of pixel coordinates, so generation is
deterministic; `check` is exercised by `imagemagick_fixtures_reproduce` in
`oracle.rs`.
