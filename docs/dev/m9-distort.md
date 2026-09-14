# M9 — Distort filters

Goal: add the geometric **Distort** warps (`FILT-040`) to `pictura-filters`,
continuing the M6/M7/M8 pattern. Spec: `docs/06-filters/distort-filters.md`.

## Scope

In (5): **Twirl**, **Pinch**, **Spherize**, **Ripple**, **Wave** — single-image
inverse-mapping warps with bilinear resampling. Alpha is untouched; all are
deterministic except Wave, which takes a seed.

Out (later): Diffuse Glow, Displace, Glass, Ocean Ripple, Polar Coordinates,
Shear, ZigZag, Lens Correction; Filter-Gallery stacking; Smart Filters; 16/32-bit;
CMYK/Lab.

## Contract (authoritative — add to the existing `Filter` enum)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpherizeMode { Normal, HorizontalOnly, VerticalOnly }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RippleSize { Small, Medium, Large }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveType { Sine, Triangle, Square }

// new Filter variants
Twirl { angle: f64 },
Pinch { amount: f64 },
Spherize { amount: f64, mode: SpherizeMode },
Ripple { amount: f64, size: RippleSize },
Wave {
    generators: u32,
    wavelength: (f64, f64),
    amplitude: (f64, f64),
    kind: WaveType,
    scale: (f64, f64),
    seed: u64,
    repeat_edge: bool,
},
```

```rust
// src/distort/mod.rs
mod radial;
mod undulate;
pub use radial::{pinch, spherize, twirl};
pub use undulate::{ripple, wave};

// src/distort/radial.rs
pub fn twirl(buf: &mut PixelBuffer, angle: f64) -> Result<(), FilterError>;
pub fn pinch(buf: &mut PixelBuffer, amount: f64) -> Result<(), FilterError>;
pub fn spherize(buf: &mut PixelBuffer, amount: f64, mode: SpherizeMode) -> Result<(), FilterError>;

// src/distort/undulate.rs
pub fn ripple(buf: &mut PixelBuffer, amount: f64, size: RippleSize) -> Result<(), FilterError>;
pub fn wave(buf: &mut PixelBuffer, generators: u32, wavelength: (f64, f64), amplitude: (f64, f64), kind: WaveType, scale: (f64, f64), seed: u64, repeat_edge: bool) -> Result<(), FilterError>;
```

Validation (all non-finite or out-of-range values → `FilterError::InvalidParams`):

| Filter | Parameter | Range |
|---|---|---|
| Twirl | `angle` | −999..=999 |
| Pinch | `amount` | −100..=100 |
| Spherize | `amount` | −100..=100 |
| Ripple | `amount` | −999..=999 |
| Wave | `generators` | 1..=999 |
| Wave | `wavelength.0` (min) | 1..=998 |
| Wave | `wavelength.1` (max) | ≥ `wavelength.0 + 1` |
| Wave | `amplitude.0` (min) | 1..=998 |
| Wave | `amplitude.1` (max) | ≥ `amplitude.0 + 1` |
| Wave | `scale.0`, `scale.1` | each 1..=100 |

Semantics (planar 8-bit, alpha untouched, clamp-to-edge, no panics):
- **Twirl** — angular inverse map; rotation falls off from center to edge, sign of
  `angle` sets direction.
- **Pinch** — radial inverse map; positive squeezes toward the selection center,
  negative pushes away; `0` is a no-op.
- **Spherize** — 3D sphere-wrap inverse map; `mode` restricts displacement to
  Normal / horizontal-only / vertical-only. `0` is a no-op.
- **Ripple** — periodic sinusoidal displacement; `size` sets the spatial
  frequency (Small / Medium / Large). `0` is a no-op.
- **Wave** — sum of `generators` wave generators; each draws a random phase/period
  from `wavelength`, amplitude from `amplitude`, at `kind` (sine/triangle/square);
  `scale` applies axis-wise; `seed` makes Randomize reproducible; `repeat_edge`
  selects repeat-edge vs wrap for undefined areas. Same seed ⇒ bit-identical.

## Oracle

ImageMagick where semantics match; classify the rest no-equivalent with recorded
deltas:
- **Twirl** — `-swirl <angle>` (sign/scale fitted; measure tolerance).
- **Wave** — single-generator sine vs `-wave` (partial: IM's generator/RNG model
  differs; measure and record the divergence).
- No faithful operator (property/known-value + determinism): **Pinch**,
  **Spherize**, **Ripple** — measure the closest operator and record the delta.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M9-A1 | `src/distort/radial.rs` (twirl/pinch/spherize) + tests | agent | `crates/pictura-filters/src/distort/radial.rs` |
| M9-A2 | `src/distort/undulate.rs` (ripple/wave) + tests | agent | `crates/pictura-filters/src/distort/undulate.rs` |
| M9-B | ImageMagick oracle + no-equivalent table | agent | `scripts/filter_oracle.py`, `crates/pictura-filters/tests/**` |
| M9-C | App filter kinds + unit test | agent | `crates/pictura-app/**` |
| M9-D | OpenSpec change + reconcile + verify | orchestrator | `openspec/**`, `docs/dev/**` |

## Exit gate

- `cargo test --workspace` green; per-filter unit tests + oracle differentials
  within tolerance or documented no-equivalent.
- fmt/clippy clean; `scripts/guard.sh` green; `openspec validate --all --strict`
  green with the new `m9-distort` change.
