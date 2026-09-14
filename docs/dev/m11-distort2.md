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

Measured against ImageMagick **7.1.2-29 Q16-HDRI** on the 16×16 oracle image.
All four new filters are behaviorally approximated — no ImageMagick operator
reproduces them — so each is classified **no-equivalent, tolerance 0** with the
observed maximum/mean per-sample delta against its closest operator recorded.

| Filter | Closest operator | Status | Measured Δ (max / mean) |
|---|---|---|---|
| Polar Coordinates (Rect→Polar) | `-distort Polar 0` | no-equivalent | 189 / 58.3 |
| Polar Coordinates (Polar→Rect) | `-distort DePolar 0` | no-equivalent | 194 / 58.9 |
| Shear | `-shear 0x26.565` (+ crop) | no-equivalent | 255 / 18.4–23.4 |
| ZigZag | `-swirl 50` | no-equivalent | 170 / 14.3 |
| Ocean Ripple | `-wave 2x8` (+ crop) | no-equivalent | 227 / 53.7 |

The mismatches are structural: Polar Coordinates' IM angle origin is ≈180° from
Pictura's `atan2` and its anchor/resampling differ; Shear's IM operator grows
and background-fills the canvas while Pictura shifts columns in place with a
piecewise-linear curve; ZigZag's cosine radial profile with `ridges` reversals
has no `-swirl` falloff match; Ocean Ripple sums 8 seeded direction sinusoids
that no single `-wave` invocation reproduces. The contracts are guarded by
property/known-value tests (polar directions differ; a zero shear curve is a
bit-exact no-op and the fill modes differ; ZigZag amount 0 is a no-op and the
styles differ; Ocean Ripple magnitude 0 is a no-op and it is seed-deterministic
and seed-sensitive). Authoritative numbers and exact flags:
`crates/pictura-filters/tests/README.md`.

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
