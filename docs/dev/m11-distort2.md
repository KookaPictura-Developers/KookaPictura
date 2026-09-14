# M11 — Distort filters, part 2

Goal: add the remaining geometric **Distort** warps (`FILT-040`) to
`pictura-filters` — **Polar Coordinates**, **Shear**, **ZigZag**, **Ocean
Ripple** — continuing the M9 pattern. Spec: `docs/06-filters/distort-filters.md`.

This change is the scaffold: the `Filter` contract, dispatch, module split, and
stubs are landed here. The warp math and the oracle measurement follow in
M11-A/M11-B.

## Scope

In (4): **Polar Coordinates** (rectangular ⇄ polar), **Shear** (piecewise-linear
vertical column shift from a control-point curve), **ZigZag** (radial
displacement driven by amount + ridges + style), **Ocean Ripple** (seeded random
ripples). All are single-image inverse-mapping warps with bilinear resampling;
alpha is untouched; ZigZag is deterministic and only Ocean Ripple takes a seed.

Out (later): Displace, Glass, Diffuse Glow, Lens Correction, Filter-Gallery
stacking, Smart Filters, 16/32-bit, CMYK/Lab.

## Contract (authoritative — add to the existing `Filter` enum)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarKind { RectangularToPolar, PolarToRectangular }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShearFill { WrapAround, RepeatEdgePixels }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZigZagStyle { AroundCenter, OutFromCenter, PondRipples }

// new Filter variants
PolarCoordinates { kind: PolarKind },
Shear { curve: Vec<(f64, f64)>, fill: ShearFill },
ZigZag { amount: f64, ridges: u32, style: ZigZagStyle },
OceanRipple { size: u32, magnitude: u32, seed: u64 },
```

```rust
// src/distort/mod.rs
mod coord;
mod ripples;
pub use coord::{polar_coordinates, shear};
pub use ripples::{ocean_ripple, zigzag};

// src/distort/coord.rs
pub fn polar_coordinates(buf: &mut PixelBuffer, kind: PolarKind) -> Result<(), FilterError>;
pub fn shear(buf: &mut PixelBuffer, curve: &[(f64, f64)], fill: ShearFill) -> Result<(), FilterError>;

// src/distort/ripples.rs
pub fn zigzag(buf: &mut PixelBuffer, amount: f64, ridges: u32, style: ZigZagStyle) -> Result<(), FilterError>;
pub fn ocean_ripple(buf: &mut PixelBuffer, size: u32, magnitude: u32, seed: u64) -> Result<(), FilterError>;
```

Validation (all non-finite or out-of-range values → `FilterError::InvalidParams`):

| Filter | Parameter | Range |
|---|---|---|
| Shear | `curve` | ≥ 2 finite points, strictly increasing x; each x ∈ −1..=1, y ∈ −1..=1 |
| ZigZag | `amount` | −100..=100 |
| ZigZag | `ridges` | 0..=20 |
| Ocean Ripple | `size` | 1..=15 |
| Ocean Ripple | `magnitude` | 0..=20 |
| Polar Coordinates | — | no parameter validation |

Semantics (planar 8-bit, alpha untouched, no panics):
- **Polar Coordinates** — coordinate transform; `rect→polar` and `polar→rect`
  with bilinear resampling. Round-trip approxes the input within resampling
  tolerance.
- **Shear** — curve control points interpolated piecewise-linearly; columns shift
  vertically by the curve. A straight default curve is a no-op. Undefined rows
  follow `fill` (wrap vs repeat-edge).
- **ZigZag** — radial displacement; `amount` scales magnitude, `ridges` sets the
  number of direction reversals from center to edge; `AroundCenter` rotates,
  `OutFromCenter` pushes radially, `PondRipples` biases diagonally. `ridges = 0`
  is a single direction. Deterministic.
- **Ocean Ripple** — small randomly placed ripples; `size` sets frequency,
  `magnitude` the amplitude; `seed` makes it reproducible. Same seed ⇒
  bit-identical.

## Oracle

Planned measurement against ImageMagick (M11-B); M9 established the pattern that
the geometric Distort family is behaviorally approximated, so most rows will be
classified **no-equivalent, tolerance 0**.

| Filter | Closest operator | Status |
|---|---|---|
| Polar Coordinates | `-distort DePolar` / `-distort Polar` | measure |
| Shear | `-shear x<angle>` (+ crop/debackground) | measure |
| ZigZag | `-swirl` / none faithful | no faithful operator |
| Ocean Ripple | `-wave` (closest) | no faithful operator |

`-distort DePolar`/`Polar` and `-shear` may line up structurally; record the
observed max delta. ZigZag and Ocean Ripple have no faithful operator, so the
closest candidate is measured and the delta recorded, and the contracts are
guarded by property/known-value tests instead. Authoritative numbers and exact
flags: `crates/pictura-filters/tests/README.md`.

The scaffold itself is guarded by the `src/distort/coord.rs` and
`src/distort/ripples.rs` stub tests: every function reports
`FilterError::Unsupported` until M11-A lands the math.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M11-A1 | `src/distort/coord.rs` (polar_coordinates/shear) + tests | agent | `crates/pictura-filters/src/distort/coord.rs` |
| M11-A2 | `src/distort/ripples.rs` (zigzag/ocean_ripple) + tests | agent | `crates/pictura-filters/src/distort/ripples.rs` |
| M11-B | ImageMagick oracle + no-equivalent table | agent | `scripts/filter_oracle.py`, `crates/pictura-filters/tests/**` |
| M11-C | App filter kinds + unit test | agent | `crates/pictura-app/**` |
| M11-D | OpenSpec change + reconcile + verify | orchestrator | `openspec/**`, `docs/dev/**` |

## Exit gate

- `cargo test --workspace` green; per-filter unit tests + oracle differentials
  within tolerance or documented no-equivalent.
- fmt/clippy clean; `scripts/guard.sh` green; `openspec validate --all --strict`
  green with the new `m11-distort2` change.
