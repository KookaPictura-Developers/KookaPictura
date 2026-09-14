# M6 — Filters: Blur, Sharpen, Noise

Goal: the destructive Blur / Sharpen / Noise filter math as pure, tested
functions over a planar 8-bit `PixelBuffer`, with an independent ImageMagick
oracle where semantics actually match.

Spec: `docs/06-filters/filters-overview.md`, `blur-filters.md` (`FILT-010`),
`sharpen-filters.md` (`FILT-020`), `noise-filters.md` (`FILT-030`).

## Scope

In (new crate `crates/pictura-filters`):
- **Blur** — Gaussian, Box, Motion, Radial (Spin / Zoom), Average, Blur /
  Blur More, Surface (bilateral).
- **Sharpen** — Sharpen / Sharpen More, Sharpen Edges, Unsharp Mask.
- **Noise** — Add Noise (Uniform / Gaussian, monochromatic, seeded), Median,
  Despeckle.

Out (later): Lens / Shape / Smart Blur, Blur Gallery; Smart Sharpen
`Remove=Lens|Motion` and Advanced Shadow/Highlight; Dust & Scratches; Reduce
Noise; 16/32-bit; GPU; selection/mask + render/app wiring (a later M6-C);
CMYK/Lab.

## Contract (authoritative — agents follow this exactly)

`crates/pictura-filters`, deps `pictura-core` + `thiserror`; dev-dep
`pictura-testkit`. Same shape as `pictura-adjust`: one enum + `apply`, alpha
(channel 4) untouched, planar color channels, `FilterError` instead of panics.

```rust
#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("unsupported: {0}")] Unsupported(String),
    #[error("invalid parameters: {0}")] InvalidParams(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadialMethod { Spin, Zoom }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality { Draft, Good, Best }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseDistribution { Uniform, Gaussian }

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    GaussianBlur { radius: f64 },
    BoxBlur { radius: u32 },
    MotionBlur { angle: f64, distance: u32 },
    RadialBlur { method: RadialMethod, amount: f64, quality: Quality },
    Average,
    Blur, BlurMore,
    SurfaceBlur { radius: u32, threshold: u8 },
    Sharpen, SharpenMore, SharpenEdges,
    UnsharpMask { amount: f64, radius: f64, threshold: u8 },
    AddNoise { amount: f64, distribution: NoiseDistribution, monochromatic: bool, seed: u64 },
    Median { radius: u32 },
    Despeckle,
}

/// Apply `filter` in place (planar 8-bit; channels 3 or 4; alpha untouched).
pub fn apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError>;

// blur.rs
pub fn gaussian(buf: &mut PixelBuffer, radius: f64) -> Result<(), FilterError>;
pub fn box(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError>;
pub fn motion(buf: &mut PixelBuffer, angle_deg: f64, distance: u32) -> Result<(), FilterError>;
pub fn radial(buf: &mut PixelBuffer, method: RadialMethod, amount: f64, quality: Quality) -> Result<(), FilterError>;
pub fn average(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn simple(buf: &mut PixelBuffer, more: bool) -> Result<(), FilterError>;
pub fn surface(buf: &mut PixelBuffer, radius: u32, threshold: u8) -> Result<(), FilterError>;

// sharpen.rs
pub fn sharpen(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn sharpen_more(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn edges(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn unsharp_mask(buf: &mut PixelBuffer, amount: f64, radius: f64, threshold: u8) -> Result<(), FilterError>;

// noise.rs
pub fn add(buf: &mut PixelBuffer, amount: f64, distribution: NoiseDistribution, monochromatic: bool, seed: u64) -> Result<(), FilterError>;
pub fn median(buf: &mut PixelBuffer, radius: u32) -> Result<(), FilterError>;
pub fn despeckle(buf: &mut PixelBuffer) -> Result<(), FilterError>;

// kernel.rs
/// UI radius is the 3σ support per FILT-010; floor σ at 0.1.
pub fn sigma_from_radius(radius: f64) -> f64;
/// Normalized 1-D FIR kernel, support ⌈3σ⌉ each side.
pub fn gaussian_kernel(sigma: f64) -> Vec<f64>;
/// Clamp-to-edge index access (border policy per FILT-010).
pub fn clamp_index(i: isize, n: usize) -> usize;

// luma.rs
pub const LUMA: [f64; 3] = [0.299, 0.587, 0.114];
pub fn luma(r: f64, g: f64, b: f64) -> f64;
```

Rules: validate channel count (3 or 4) and buffer length; empty buffer → error;
deterministic except Add Noise (seeded RNG, same seed ⇒ bit-identical);
clamp-to-edge at borders; tiny (1×1, 1-px) images must not panic; `radius == 0`
/ `distance <= 1` / `amount == 0` are no-ops or near-identity.

## Oracle

ImageMagick (`magick`) where semantics match; otherwise known-value / property
tests with the divergence documented (as in M4):
- Gaussian `-gaussian-blur 0xσ`; Box `-statistic mean NxN`; Motion
  `-motion-blur 0xN+angle`; Median `-median R`; Unsharp `-unsharp 0xRxA+T`.
- **No faithful equivalent** (property/known-value tests, say so): Average
  (trivial region mean), Radial (IM `-radial-blur` semantics differ), Surface,
  Despeckle, Add Noise (RNG streams differ → statistical + same-seed tests),
  Sharpen/Sharpen More/Sharpen Edges (fixed kernels), Blur/Blur More.

## Determinism

Add Noise uses a seeded `rand_chacha`; the seed is part of the filter so a
re-apply is bit-identical. No other filter uses randomness.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M6-A | `pictura-filters` skeleton + contract + shared helpers | agent | `crates/pictura-filters/{Cargo.toml,src/lib.rs,src/kernel.rs,src/luma.rs}`, workspace `Cargo.toml` |
| M6-B | Blur family + unit tests | agent | `crates/pictura-filters/src/blur.rs` |
| M6-C | Sharpen family + unit tests | agent | `crates/pictura-filters/src/sharpen.rs` |
| M6-D | Noise family + unit tests | agent | `crates/pictura-filters/src/noise.rs` |
| M6-E | ImageMagick differential oracle | agent | `scripts/filter_oracle.py`, `crates/pictura-filters/tests/oracle.rs` |
| M6-F | Integrate + verify + OpenSpec proposal | orchestrator | — |

## Exit gate

- `cargo test --workspace` green; per-filter unit tests + oracle differentials
  pass within tolerance or are documented as no-equivalent.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- `scripts/guard.sh` green; `openspec validate --all --strict` green with the
  new `m6-filters` change.
