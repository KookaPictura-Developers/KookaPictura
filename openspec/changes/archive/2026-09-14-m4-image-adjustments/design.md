## Context

M4-A implemented the destructive adjustment math and M4-C added the ImageMagick
oracle. Both are shipped in `crates/pictura-adjust` and specified in prose in
`docs/dev/m4-adjustments.md` and the sixteen files under
`docs/04-image-ops/adjustments/`. The code is the de facto contract, but nothing
in OpenSpec pins it. A later wave (adjustment layers, per-range edits, higher bit
depths) would otherwise be free to change the parameter shapes or the error
behavior without a reviewable record.

The crate is intentionally small. It depends on `pictura-core` for `PixelBuffer`
and `thiserror` for the error enum; `pictura-testkit` is a dev-dependency for
byte comparison. No external image or math crate is pulled in.

## Goals / Non-Goals

**Goals:**

- Freeze the `apply` signature, the `Adjustment` enum, and the parameter structs
  for the 15 in-scope adjustments.
- Record the parameter ranges, defaults, and the per-adjustment algorithm the
  code implements, including where Adobe's kernel is closed and the
  implementation approximates.
- Record the shared invariants: in-place planar application, alpha untouched,
  deterministic, errors instead of panics.
- Record which adjustments have a faithful ImageMagick operator and which are
  covered by property or known-value tests instead.

**Non-Goals:**

- Adjustment layers, PSD serialization, masks, opacity, and blending. Those are
  task M4-B and a separate capability.
- Gradient Map, Selective Color, Shadow/Highlight, HDR Toning, Match Color,
  Replace Color, and 3D LUT/Color Lookup.
- Per-range Hue/Saturation and per-channel Levels/Curves. M4 is composite
  (Master) only.
- 16-bit and 32-bit math. M4 is 8-bit only.
- GPU execution. `pictura-adjust` is the CPU reference.

## Decisions

**In-place planar writes.** `apply` takes `&mut PixelBuffer` and rewrites the
color planes in place. The buffer is planar, so the three color channels are
contiguous runs of `width * height` bytes. `planes_mut` splits them with
`split_at_mut`, which avoids a per-pixel bounds check on an alpha plane and makes
the "alpha untouched" rule structural: the fourth slice is bound to `_a` and
never written. A copy-out design would allocate a full-canvas buffer per call,
which the overlay use case cannot afford. Alternative rejected: returning a new
`PixelBuffer`, because the destructive command path wants to mutate the active
tile and record a pixel delta.

**LUT for per-channel maps, direct loops for vector maps.** Levels, Curves,
Brightness/Contrast, Exposure, and Posterize build a 256-entry LUT once and map
every sample through it. HSL-space adjustments (Hue/Saturation, Vibrance, Black &
White, Photo Filter, Channel Mixer, Color Balance) compute per pixel because
their input is the channel triple, not each channel alone. The LUT makes the
per-pixel maps exact at 8-bit and cheap. It also keeps the door open to a GPU
variant that samples the same table.

**Errors, not panics.** `apply` validates the buffer once at the top
(`validate`) and each adjustment validates its own parameters before touching
pixels. `AdjustError` has `Unsupported` for channel counts and `InvalidParams`
for values and buffer shapes. This is the difference between a tool that reports
a bad parameter and one that takes the host process down. The ranges are read
from the matching `docs/04-image-ops/adjustments/*.md` table.

**Approximations are named.** Adobe's kernels for Curves, Brightness/Contrast,
Exposure, Hue/Saturation, Vibrance, Color Balance, Black & White, and Auto are
closed. Each approximation carries an inline comment naming the ceiling. Examples:
the modern Brightness/Contrast S-curve, the vibrance `1 - S` falloff and skin
damping, the Auto percentile estimator, and the color-balance parabola windows.
The project rule is behavioral parity only where an oracle exists, so these are
documented as approximations rather than asserted as parity.

**ImageMagick as a sanity oracle, not a parity oracle.** Only Levels, Invert, and
Desaturate have a faithful operator, so only those run differentially. Levels and
Desaturate allow 1 LSB for rounding; Invert allows 0. The other twelve are
classified no-equivalent with tolerance 0 and covered by property or known-value
tests. The measured divergences are recorded in `tests/README.md`: Threshold uses
Rec.709 in ImageMagick against Rec.601 in Photoshop (delta 255), Hue/Saturation's
HSL law differs (delta 45), Channel Mixer mixes percent against fraction (delta
252), Posterize bins differently (delta 85), Brightness/Contrast differs (delta
14), and Exposure runs in a different space. The oracle script passes percentages
to `-level`, `+level`, `-threshold`, and additive `-evaluate` because a Q16 build
would otherwise read bare numbers as quantum values.

**No committed fixtures.** The differential tests run ImageMagick at test time
and skip with a message when `magick` is missing. This keeps the repository free
of binary references and keeps the tests runnable on a machine without
ImageMagick.

## Risks / Trade-offs

- **Approximation drift.** The closed-kernel models may diverge from CS6 as more
  samples arrive. Mitigation: each is marked inline and listed in this design, so
  a future change can retune one function without touching the enum or the
  parameter structs.
- **ImageMagick version sensitivity.** The operator set and Q16 percentage
  handling can change between builds. Mitigation: the verified version is pinned
  in `tests/README.md` and the differential tests only cover three operators.
- **No differential coverage for twelve adjustments.** A regression in, say,
  Color Balance would only trip a property test. Mitigation: the property tests
  guard the observable contract (identity, known values, monotonicity), which is
  the strongest check available without a closed reference.
- **8-bit only.** The LUT path assumes 256 entries. A 16-bit port needs a wider
  table or direct evaluation. This is deferred, not accidental.
