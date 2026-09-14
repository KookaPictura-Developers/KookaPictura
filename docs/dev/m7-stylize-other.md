# M7 — Stylize & Other filters

Goal: extend `pictura-filters` with the classic **Other** and **Stylize** filter
families (the subset with the strongest independent oracle), continuing the M6
pattern. Spec: `docs/06-filters/other-filters.md` (`FILT-070`),
`docs/06-filters/stylize-filters.md` (`FILT-050`).

## Scope

In — Other (5): **Maximum**, **Minimum**, **Offset**, **High Pass**, **Custom**.
In — Stylize (3): **Emboss**, **Find Edges**, **Solarize**.

Out (later): Stylize Wind / Tiles / Extrude / Trace Contour, Oil Paint, Diffuse,
Glowing Edges; Distort (geometric warps), Pixelate, Render, Liquify, Blur
Gallery, Camera Raw, Lens Correction; Smart Filters; 16/32-bit; CMYK/Lab.

## Contract (authoritative — add to the existing `Filter` enum)

New variants:
```rust
Maximum { radius: u32 },
Minimum { radius: u32 },
Offset { horizontal: i32, vertical: i32, wrap: bool, background: [u8; 3] },
HighPass { radius: f64 },
Custom { kernel: [[f64; 5]; 5], scale: f64, offset: f64 },
Emboss { angle: f64, height: f64, amount: f64 },
FindEdges,
Solarize,
```

New modules:
```rust
// src/other.rs
pub fn maximum(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError>;
pub fn minimum(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError>;
pub fn offset(buf: &mut PixelBuffer, horizontal: i32, vertical: i32, wrap: bool, background: [u8; 3]) -> Result<(), FilterError>;
pub fn high_pass(buf: &mut PixelBuffer, radius: f64) -> Result<(), FilterError>;
pub fn custom(buf: &mut PixelBuffer, kernel: &[[f64; 5]; 5], scale: f64, offset: f64) -> Result<(), FilterError>;

// src/stylize.rs
pub fn emboss(buf: &mut PixelBuffer, angle: f64, height: f64, amount: f64) -> Result<(), FilterError>;
pub fn find_edges(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn solarize(buf: &mut PixelBuffer) -> Result<(), FilterError>;
```

Semantics (planar 8-bit, alpha untouched, clamp-to-edge, no panics):
- **Maximum / Minimum** — per-color-channel grayscale dilate/erode over a
  `(2r+1)²` square footprint. `radius == 0` no-op; clamp `radius` to 100
  (`FILT-070` scriptable max). Deterministic.
- **Offset** — shift color planes by `(horizontal, vertical)`. `wrap == true`
  wraps around; `wrap == false` fills the exposed area with `background`.
  `0,0` is a no-op.
- **High Pass** — `out = clamp(orig − gaussian(orig, sigma) + 128)` per channel,
  `sigma = sigma_from_radius(radius)`; radius must be finite and `> 0`.
- **Custom** — 5×5 convolution: `out = clamp(Σ kernel·neighbor / scale + offset)`
  per color channel, f64 accumulation, clamp-to-edge. `scale == 0` and a
  non-finite kernel/scale/offset → `InvalidParams`. Identity is the unit impulse
  kernel with `scale 1`, `offset 0`.
- **Emboss** — directional relief: the directional difference along `angle`
  (`height` gain), `+128` gray bias, `amount` percent; output is gray (all color
  channels equal) per `FILT-050`. `angle` finite in `-360..=360`; `height` and
  `amount` finite and `> 0`.
- **Find Edges** — Sobel gradient magnitude; Photoshop renders edges dark on a
  light field, so `out = 255 − clamp(magnitude)`. No parameters.
- **Solarize** — fixed 50% curve: `out = if v >= 128 { 255 − v } else { v }`.
  No parameters.

## Oracle

ImageMagick where semantics match (measure the tolerance; do not guess):
`Maximum`/`Minimum` `-morphology Dilate/Erode Square:{radius}` (IM's `Square:N`
takes the radius), `Offset` wrap `-roll`,
`Custom` `-convolve`, `Emboss` `-emboss`, `Solarize` `-solarize 50%`,
`Find Edges` `-edge` (document the PS dark-on-light inversion — likely
no-equivalent), `High Pass` a `-compose mathematics` recipe (or no-equivalent).
No faithful operator → known-value/property tests with the divergence recorded.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M7-A | Other family + tests | agent | `crates/pictura-filters/src/other.rs` |
| M7-B | Stylize family + tests | agent | `crates/pictura-filters/src/stylize.rs` |
| M7-C | ImageMagick oracle + no-equivalent table | agent | `scripts/filter_oracle.py`, `crates/pictura-filters/tests/**` |
| M7-D | App filter kinds + unit test | agent | `crates/pictura-app/**` |
| M7-E | OpenSpec change + reconcile + verify | orchestrator | `openspec/**`, `docs/dev/**` |

## Exit gate

- `cargo test --workspace` green; per-filter unit tests + oracle differentials
  within tolerance or documented no-equivalent.
- fmt/clippy clean; `scripts/guard.sh` green; `openspec validate --all --strict`
  green with the new `m7-stylize-other` change.
