## Why

Twenty-five filter families ship, but the four remaining CS6 families — **Brush
Strokes (8), Sketch (14), Texture (6), Oil Paint (1)** — are unimplemented. They
close out the Filter Gallery categories and cover the last unfiltered CS6
filter set. Implementing them now extends filter coverage and exercises new
kernel classes: directional stroke rasterization, height-field relief and
halftone screens, cellular tessellation and grain fields, and the first
shader-style painterly lighting model.

## What Changes

- Add `pictura-filters::brush_strokes` with the 8 CS6 Brush Stroke filters:
  Accented Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines,
  Spatter, Sprayed Strokes, Sumi-e.
- Add `pictura-filters::sketch` with the 14 CS6 Sketch filters: Bas Relief,
  Chalk & Charcoal, Charcoal, Chrome, Conté Crayon, Graphic Pen, Halftone
  Pattern, Note Paper, Photocopy, Plaster, Reticulation, Stamp, Torn Edges,
  Water Paper.
- Add `pictura-filters::texture` with the 6 CS6 Texture filters: Craquelure,
  Grain, Mosaic Tiles, Patchwork, Stained Glass, Texturizer.
- Add `pictura-filters::oil_paint` with Oil Paint as a **CPU behavioural model**
  (a deliberate divergence from CS6's hard GPU requirement).
- Add four public enums (`StrokeDirection`, `LightDirection`, `HalftoneType`,
  `GrainType`), the 29 `Filter` variants, parameter validation, alpha
  preservation, and seeded determinism; reuse the M22 shared helpers
  (`crate::artistic::{reduce,noise,texture}`).
- Add app `filter_from_kind` mappings so the filters are reachable from the
  shell.
- Defer the Filter Gallery dialog, Smart Filter entries, `Edit > Fade`, 16/32-bit
  gating, CMYK/Lab gating, `Load Texture` file I/O, and the Oil Paint GPU pass /
  `GpuUnsupported` dialog. The kernels target **behavioural parity only**
  (Adobe's kernels are closed, as for M6–M22).

## Capabilities

### New Capabilities

- `brush-stroke-filters`: the 8 Brush Stroke kernels, their direction enum,
  parameter contract, determinism, and alpha/validation behaviour.
- `sketch-filters`: the 14 Sketch kernels, their light-direction and halftone
  enums, foreground/background ink-and-paper inputs, texture reuse, and
  determinism.
- `texture-filters`: the 6 Texture kernels, the grain-type enum, cellular/tile
  helpers, background-colour use, and determinism.
- `oil-paint-filter`: the single Oil Paint CPU kernel, its six float controls
  (including angular direction and non-finite rejection), and its documented
  non-parity GPU posture.

### Modified Capabilities

None. No existing requirement changes; every family reuses the shared `apply`
contract already established by the other filter families.

## Impact

- `crates/pictura-filters` gains `brush_strokes.rs`, `sketch/`, `texture.rs`,
  and `oil_paint.rs`, plus four enums and 29 `Filter` variants; no new
  dependencies (seeded RNG and surface/emboss helpers already exist in
  `artistic`).
- `crates/pictura-app` `filter_from_kind` gains the 29 kinds; the UI surface
  (gallery/panes) is deferred, consistent with M6–M22 which added kinds without
  dialogs.
- No `pictura-core` document-format change; filters remain destructive pixel ops
  committed as one history state.
