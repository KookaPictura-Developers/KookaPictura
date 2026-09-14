# M8 — Pixelate filters

Goal: add the **Pixelate** submenu (`FILT-084`) to `pictura-filters`, continuing
the M6/M7 pattern. Spec: `docs/06-filters/pixelate-filters.md`.

## Scope

In (7): **Color Halftone**, **Crystallize**, **Facet**, **Fragment**,
**Mezzotint**, **Mosaic**, **Pointillize**.

Out (later): Distort, Render, Liquify, Blur Gallery, Camera Raw, Lens
Correction; Smart Filters; 16/32-bit; CMYK/Lab; the Filter-Gallery stack (Pixelate
is not a gallery category in CS6).

## Contract (authoritative — add to the existing `Filter` enum)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MezzotintType {
    FineDots, MediumDots, GrainyDots, CoarseDots,
    ShortLines, MediumLines, LongLines,
    ShortStrokes, MediumStrokes, LongStrokes,
}

// new Filter variants
Mosaic { cell_size: u32 },
Crystallize { cell_size: u32, seed: u64 },
Facet,
Fragment,
Mezzotint { kind: MezzotintType, seed: u64 },
Pointillize { cell_size: u32, background: [u8; 3], seed: u64 },
ColorHalftone { max_radius: u32, angles: [f64; 4] },
```

```rust
// src/pixelate.rs
pub fn mosaic(buf: &mut PixelBuffer, cell_size: u32) -> Result<(), FilterError>;
pub fn crystallize(buf: &mut PixelBuffer, cell_size: u32, seed: u64) -> Result<(), FilterError>;
pub fn facet(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn fragment(buf: &mut PixelBuffer) -> Result<(), FilterError>;
pub fn mezzotint(buf: &mut PixelBuffer, kind: MezzotintType, seed: u64) -> Result<(), FilterError>;
pub fn pointillize(buf: &mut PixelBuffer, cell_size: u32, background: [u8; 3], seed: u64) -> Result<(), FilterError>;
pub fn color_halftone(buf: &mut PixelBuffer, max_radius: u32, angles: [f64; 4]) -> Result<(), FilterError>;
```

Semantics (planar 8-bit, alpha untouched, clamp-to-edge, no panics):
- **Mosaic** — average each `cell_size × cell_size` block and write that color to
  every pixel in it. `cell_size` 2..=200 else `InvalidParams`. Deterministic.
- **Crystallize** — Voronoi: scatter one seed point per `cell_size` grid cell
  (jittered from the seeded RNG), assign every pixel to its nearest seed, fill the
  cell with the mean color of its members. `cell_size` 3..=300 else
  `InvalidParams`. Same seed ⇒ bit-identical.
- **Facet** — iterative similar-neighbor local averaging (a fixed pass count)
  that flattens detail into patches. No parameters.
- **Fragment** — four copies offset by a small fixed `(dx,dy)` and averaged
  (deterministic ghost). No parameters.
- **Mezzotint** — per-`kind` seeded procedural dot/line/stroke pattern over the
  image; grayscale uses the luma pattern, color keeps saturated color. Same seed ⇒
  bit-identical.
- **Pointillize** — scattered dots of radius proportional to `cell_size`, each
  filled with the local source color, over `background`. `cell_size` 3..=300 else
  `InvalidParams`. Same seed ⇒ bit-identical.
- **ColorHalftone** — per color channel, a rotated grid at `angles[channel]`
  (grayscale uses `angles[0]`; a 3-channel image uses the first three) with dot
  radius ∝ cell brightness and grid spacing from `max_radius`. `max_radius`
  4..=127 and every angle finite else `InvalidParams`.

## Oracle

ImageMagick where semantics match; classify the rest no-equivalent with recorded
deltas:
- **Mosaic** — `-filter box -resize` down then `-filter point -resize` up (block
  average), tolerance measured.
- **Fragment** — a 4-tap average; known-value/property test.
- No faithful operator (property/known-value + determinism): Crystallize,
  Facet, Mezzotint, Pointillize, Color Halftone. Record the observed delta.

## Task DAG

| ID | Task | Owner | Owns |
|---|---|---|---|
| M8-A | `src/pixelate.rs` (7 filters) + tests | agent | `crates/pictura-filters/src/pixelate.rs` |
| M8-B | ImageMagick oracle + no-equivalent table | agent | `scripts/filter_oracle.py`, `crates/pictura-filters/tests/**` |
| M8-C | App filter kinds + unit test | agent | `crates/pictura-app/**` |
| M8-D | OpenSpec change + reconcile + verify | orchestrator | `openspec/**`, `docs/dev/**` |

## Exit gate

- `cargo test --workspace` green; oracle differentials within tolerance or
  documented no-equivalent.
- fmt/clippy clean; `scripts/guard.sh` green; `openspec validate --all --strict`
  green with the new `m8-pixelate` change.
