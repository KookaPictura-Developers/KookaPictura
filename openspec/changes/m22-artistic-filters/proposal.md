## Why

Ten filter families ship, but the 15 **Artistic** filters — the largest remaining
CS6 family — are unimplemented. They are the headline contents of
`Filter > Filter Gallery` and the reference case for the still-missing gallery
stack. Implementing them now extends filter coverage and exercises a new class of
kernel: painterly stylization driven by foreground/background colours and
procedural paper texture.

## What Changes

- Add `pictura-filters::artistic` with all 15 CS6 Artistic filters: Colored
  Pencil, Cutout, Dry Brush, Film Grain, Fresco, Neon Glow, Paint Daubs, Palette
  Knife, Plastic Wrap, Poster Edges, Rough Pastels, Smudge Stick, Sponge,
  Underpainting, Watercolor.
- Add shared kernel helpers: posterize/quantize, gradient/edge magnitude, a
  seeded RNG, an oriented stroke/daub helper, and a procedural grayscale paper
  height map with light-direction emboss.
- Add the 15 `Filter` variants, parameter validation, alpha preservation, and
  seeded determinism, plus app `filter_from_kind` mappings so the filters are
  reachable from the shell.
- Defer the Filter Gallery dialog, Smart Filter entries, `Edit > Fade`, 16/32-bit
  gating, CMYK/Lab gating, and `Load Texture` file I/O. The kernels target
  **behavioural parity only** (Adobe's kernels are closed, as for M6–M15).

## Capabilities

### New Capabilities

- `artistic-filters`: the 15 Artistic filter kernels, their shared helpers,
  parameter contract, determinism, and foreground/background/texture inputs.

### Modified Capabilities

None. No existing requirement changes; the family reuses the shared `apply`
contract already established by the other filter families.

## Impact

- `crates/pictura-filters` gains an `artistic` module and `Filter` variants; no
  new dependencies (seeded RNG is inline or reuses the existing noise RNG).
- `crates/pictura-app` `filter_from_kind` gains the 15 kinds; the UI surface
  (gallery/panes) is deferred, consistent with M6–M15 which added kinds without
  dialogs.
- No `pictura-core` document-format change; filters remain destructive pixel ops
  committed as one history state.
