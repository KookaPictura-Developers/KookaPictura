## Context

`pictura-filters` implements ten families behind one `apply(&Filter, &mut
PixelBuffer)` entry point with a shared error/alpha/validation contract from
`FILT-001`. The Artistic family (`docs/06-filters/artistic-filters.md`,
`FILT-080`) is the largest remaining set: 15 painterly filters, several reading
the foreground/background colours and two taking a textured surface. Adobe's
kernels are closed, so every filter is **behavioural parity only** — the same
posture M6–M15 took.

## Goals / Non-Goals

**Goals:**

- All 15 Artistic filters as `Filter` variants, validated, alpha-preserving,
  deterministic.
- Shared helpers: posterize/quantize, gradient/edge magnitude, a seeded RNG, an
  oriented stroke/daub helper, and a procedural paper height map + light emboss.
- Foreground/background/glow colours carried as explicit kernel parameters.
- App `filter_from_kind` mappings and a self-test proving effect + determinism.

**Non-Goals:**

- The Filter Gallery dialog and cumulative stack; Smart Filter entries;
  `Edit > Fade`; 16/32-bit and CMYK/Lab capability gating; `Load Texture` file
  I/O; the Filter Gallery thumbnail panes. Exotic texture presets beyond the four
  documented surfaces.

## Decisions

### One `artistic` module, kernels as free functions, wired into `Filter`

Each filter is a function over the planar buffer plus a `Filter` variant carrying
its typed parameters. `apply` dispatches to it. This matches every other family
(`blur.rs`, `distort/`, `pixelate/`, `stylize.rs`).

- *Why:* uniform error handling and alpha preservation come from the existing
  `apply` contract; no per-filter trait object or registry is needed for a fixed
  family.

### Behavioural-parity models, not Adobe internals

The documented one-line behaviour and control names are the contract; the pixel
math is a plausible model. Cutout = posterize + contour simplification; Poster
Edges = posterize + edge magnitude + darkening; Dry Brush/Fresco/Watercolor/etc.
= colour reduction + oriented daub reconstruction; Film Grain = tonal-zone noise;
Neon Glow = blurred luminance glow in the glow colour. Every kernel carries a
`// ponytail:` note naming its ceiling where it is a deliberate approximation.

- *Alternatives considered:* black-box reconstruction of Adobe output — rejected; no
  oracle exists and the project already classifies closed kernels as
  "no-equivalent".

### Randomness is seeded and carried in the variant

Film Grain, Paint Daubs, Sponge, Palette Knife, Smudge Stick, Watercolor, Fresco,
Dry Brush, Colored Pencil, and Rough Pastels use a `seed: u64` field and an
inline/reused seeded RNG, so a redo reproduces bit-for-bit.

- *Why:* the family's determinism requirement; PSB/undo reproducibility.

### Foreground/background colours and texture options are explicit parameters

The two colour-dependent filters and the glow colour take `[u8; 3]`/`[u8; 4]`
fields on their variant; Rough Pastels and Underpainting take a shared
`TextureOptions { surface, scaling, relief, light_direction, invert }`. The
surface is a procedural grayscale height map per preset (Brick/Burlap/Canvas/
Sandstone), embossed by the light direction.

- *Why:* keeps `pictura-filters` free of document/colour-state coupling; the app
  supplies the current colours when it maps a kind.

## Risks / Trade-offs

- **No oracle** → parity is behavioural; tests assert effect, parameter
  monotonicity, determinism, alpha preservation, and no-panic edges, and the
  family is documented as approximation.
- **15 filters is a large surface** → split into waves (shared helpers, then two
  filter groups) so each lands with its own tests.
- **Texture memory** → surfaces are generated procedurally at the needed tile
  size, never materialized at PSB scale.
- **Performance** → `ponytail:` naive per-pixel neighbourhood scans; optimize
  only if a documented filter is measurably slow.

## Migration Plan

Additive; new module and variants only. Rollback deletes `artistic/` and its
`Filter` variants. No document-format or API change.

## Open Questions

- Exact CS6 defaults/ranges (see the doc's Open questions) — the implementation
  uses the documented `(inferred)` defaults and clamps to the sourced ranges.
- Whether `Poster Edges` posterization max is 6 or 10 — the implementation
  accepts the wider documented range.
