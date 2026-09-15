## Context

`pictura-filters` implements filter families behind one `apply(&Filter, &mut
PixelBuffer)` entry point with a shared error/alpha/validation contract from
`FILT-001`. M22 added the Artistic family and the shared `artistic` helpers
(posterize, edge magnitude, value noise, procedural surface height + emboss,
`TextureOptions`). The four remaining CS6 families are the last unfiltered set:

- **Brush Strokes** (`docs/06-filters/brush-strokes-filters.md`, `FILT-081`), 8
  filters, all Filter Gallery members.
- **Sketch** (`docs/06-filters/sketch-filters.md`, `FILT-082`), 14 filters, most
  reading foreground/background colours as ink/paper.
- **Texture** (`docs/06-filters/texture-filters.md`, `FILT-083`), 6 filters,
  reusing `TextureOptions` for Texturizer.
- **Oil Paint** (`docs/06-filters/oil-paint.md`, `FILT-091`), a single CS6
  top-level filter that Adobe gates on a supported GPU.

Adobe's kernels (and the Oil Paint OpenCL shader) are closed, so every filter is
**behavioural parity only** — the same posture M6–M22 took.

## Goals / Non-Goals

**Goals:**

- All 29 filters as `Filter` variants, validated, alpha-preserving,
  deterministic.
- Four public enums (`StrokeDirection`, `LightDirection`, `HalftoneType`,
  `GrainType`) shared across the families.
- Reuse the M22 helpers (`crate::artistic::{reduce,noise,texture}`) rather than
  reintroducing posterize/edge/noise/surface kernels.
- Foreground/background colours and `TextureOptions` carried as explicit kernel
  parameters.
- App `filter_from_kind` mappings and a self-test proving effect + determinism.

**Non-Goals:**

- The Filter Gallery dialog and cumulative stack; Smart Filter entries;
  `Edit > Fade`; 16/32-bit and CMYK/Lab capability gating; `Load Texture` file
  I/O; Filter Gallery thumbnail panes.
- A GPU compute path for Oil Paint, and the CS6 `GpuUnsupported` dialog.
- Exact Adobe randomness replication; `seed: u64` is stored for reproducible
  redo, not to match Adobe's output.

## Decisions

### Frozen interface

The following names, field types, and ranges are frozen for this change.
Sub-agents implement against them; changing any of them invalidates the spec.

Four new public enums in `lib.rs`:

```rust
pub enum StrokeDirection { RightDiagonal, Horizontal, LeftDiagonal, Vertical }
pub enum LightDirection {
    Bottom, BottomLeft, Left, TopLeft, Top, TopRight, Right, BottomRight,
}
pub enum HalftoneType { Dot, Line, Circle }
pub enum GrainType {
    Regular, Soft, Sprinkles, Clumped, Contrasty,
    Enlarged, Stippled, Horizontal, Vertical, Speckle,
}
```

29 new `Filter` variants:

```rust
// Brush Strokes (8)
AccentedEdges { edge_width: u8, edge_brightness: u8, smoothness: u8 }
AngledStrokes { direction_balance: u8, stroke_length: u8, sharpness: u8 }
Crosshatch { stroke_length: u8, sharpness: u8, strength: u8 }
DarkStrokes { balance: u8, black_intensity: u8, white_intensity: u8 }
InkOutlines { stroke_length: u8, dark_intensity: u8, light_intensity: u8 }
Spatter { spray_radius: u8, smoothness: u8, seed: u64 }
SprayedStrokes { stroke_length: u8, spray_radius: u8, direction: StrokeDirection, seed: u64 }
SumiE { stroke_width: u8, stroke_pressure: u8, contrast: u8 }

// Sketch (14)
BasRelief { detail: u8, smoothness: u8, light_direction: LightDirection, foreground: [u8; 3], background: [u8; 3] }
ChalkCharcoal { charcoal_area: u8, chalk_area: u8, stroke_pressure: u8, foreground: [u8; 3], background: [u8; 3], seed: u64 }
Charcoal { thickness: u8, detail: u8, light_dark_balance: u8, foreground: [u8; 3], background: [u8; 3], seed: u64 }
Chrome { detail: u8, smoothness: u8 }
ConteCrayon { foreground_level: u8, background_level: u8, texture: TextureOptions, foreground: [u8; 3], background: [u8; 3], seed: u64 }
GraphicPen { stroke_length: u8, light_dark_balance: u8, direction: StrokeDirection, foreground: [u8; 3], background: [u8; 3] }
HalftonePattern { size: u8, contrast: u8, pattern: HalftoneType }
NotePaper { image_balance: u8, graininess: u8, relief: u8, seed: u64 }
Photocopy { detail: u8, darkness: u8 }
Plaster { image_balance: u8, smoothness: u8, light_direction: LightDirection, foreground: [u8; 3], background: [u8; 3] }
Reticulation { density: u8, black_level: u8, white_level: u8, foreground: [u8; 3], background: [u8; 3], seed: u64 }
Stamp { light_dark_balance: u8, smoothness: u8, foreground: [u8; 3], background: [u8; 3] }
TornEdges { image_balance: u8, smoothness: u8, contrast: u8, foreground: [u8; 3], background: [u8; 3] }
WaterPaper { fiber_length: u8, brightness: u8, contrast: u8, seed: u64 }

// Texture (6)
Craquelure { crack_spacing: u8, crack_depth: u8, crack_brightness: u8 }
Grain { intensity: u8, contrast: u8, grain_type: GrainType, background: [u8; 3], seed: u64 }
MosaicTiles { tile_size: u8, grout_width: u8, lighten_grout: u8, seed: u64 }
Patchwork { square_size: u8, relief: u8, seed: u64 }
StainedGlass { cell_size: u8, border_thickness: u8, light_intensity: u8, foreground: [u8; 3], seed: u64 }
Texturizer { texture: TextureOptions }

// Oil Paint (1)
OilPaint { stylization: f64, cleanliness: f64, scale: f64, bristle_detail: f64, angular_direction: f64, shine: f64 }
```

New modules:

- `crates/pictura-filters/src/brush_strokes.rs`
- `crates/pictura-filters/src/sketch/{mod.rs,relief.rs,paper.rs}`
- `crates/pictura-filters/src/texture.rs`
- `crates/pictura-filters/src/oil_paint.rs`

They reuse the shared helpers `crate::artistic::{reduce,noise,texture}`
(posterize, edge_magnitude, clamp_u8, value_noise, surface_height, emboss,
`TextureOptions`).

Validation ranges (frozen):

```text
AccentedEdges   edge_width 1..=14, edge_brightness 0..=50, smoothness 1..=15
AngledStrokes   direction_balance 0..=100, stroke_length 3..=50, sharpness 0..=10
Crosshatch      stroke_length 3..=50, sharpness 0..=20, strength 1..=3
DarkStrokes     balance/black_intensity/white_intensity 0..=10
InkOutlines     stroke_length 1..=50, dark_intensity/light_intensity 0..=50
Spatter         spray_radius 0..=25, smoothness 1..=15
SprayedStrokes  stroke_length 0..=20, spray_radius 0..=25
SumiE           stroke_width 3..=15, stroke_pressure 0..=15, contrast 0..=40
BasRelief       detail/smoothness 1..=15, foreground/background RGB
ChalkCharcoal   charcoal_area/chalk_area 0..=20, stroke_pressure 0..=5
Charcoal        thickness 1..=7, detail 0..=5, light_dark_balance 0..=100
Chrome          detail/smoothness 0..=10
ConteCrayon     foreground_level/background_level 1..=15
GraphicPen      stroke_length 1..=15, light_dark_balance 0..=100
HalftonePattern size 1..=12, contrast 0..=50
NotePaper       image_balance 0..=50, graininess 0..=20, relief 0..=25
Photocopy       detail 0..=24, darkness 1..=50
Plaster         image_balance 0..=50, smoothness 0..=15
Reticulation    density/black_level/white_level 0..=50
Stamp           light_dark_balance 0..=50, smoothness 1..=50
TornEdges       image_balance 0..=50, smoothness 1..=15, contrast 1..=25
WaterPaper      fiber_length 3..=50, brightness/contrast 0..=100
Craquelure      crack_spacing 2..=100, crack_depth/crack_brightness 0..=10
Grain           intensity/contrast 0..=100
MosaicTiles     tile_size 2..=100, grout_width 1..=15, lighten_grout 0..=10
Patchwork       square_size 0..=10, relief 0..=25
StainedGlass    cell_size 2..=50, border_thickness 1..=20, light_intensity 0..=10
Texturizer      uses TextureOptions (scaling 50..=200, relief 0..=50, light_direction 0..=7)
OilPaint        stylization/cleanliness/scale/bristle_detail/shine 0.0..=10.0,
                angular_direction 0.0..=360.0, non-finite rejected
```

### One module per family, kernels as free functions, wired into `Filter`

Each filter is a function over the planar buffer plus a `Filter` variant
carrying its typed parameters; `apply` dispatches to it. This matches every
other family (`blur.rs`, `distort/`, `pixelate/`, `artistic/`).

- *Why:* uniform error handling and alpha preservation come from the existing
  `apply` contract; no per-filter trait object or registry is needed for a fixed
  family.

### Behavioural-parity models, not Adobe internals

The documented one-line behaviour and control names are the contract; the pixel
math is a plausible model. Brush Strokes = edge/orientation detection +
directional stroke rasterization + tonal gating; Sketch = relief/emboss, halftone
screens, posterized stroke renderers, and threshold/quantization; Texture =
crack network, grain distributions, tiling/cellular segmentation; Oil Paint =
edge-aware directional smoothing + relief/Blinn-Phong shading. Every kernel
carries a `// ponytail:` note naming its ceiling where it is a deliberate
approximation.

- *Alternatives considered:* black-box reconstruction of Adobe output — rejected; no
  oracle exists and the project already classifies closed kernels as
  "no-equivalent".

### Oil Paint is implemented as a CPU behavioural model

`Filter::OilPaint` is a CPU kernel. This is a deliberate divergence from CS6,
which hard-requires an OpenCL-capable GPU and has **no CPU fallback**: this
project's GPU path is non-authoritative, and there is no OpenCL capability gate
yet. The CPU path is the parity build here.

- *Explicit non-goal:* no GPU compute pass and no `GpuUnsupported` dialog. The
  CS6 GPU requirement and its "supported graphics card" notice are recorded as
  non-parity.
- *Why:* the doc's open question ("disable vs CPU fallback") is resolved in
  favour of a CPU path so Oil Paint is usable on Linux; the divergence is named
  rather than silently approximated.

### Randomness is seeded in the variant

Spatter, Sprayed Strokes, Chalk & Charcoal, Charcoal, Conté Crayon, Note Paper,
Reticulation, Water Paper, Grain, Mosaic Tiles, Patchwork, and Stained Glass carry
a `seed: u64` field, so a redo reproduces bit-for-bit. Deterministic kernels
(Accented Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Sumi-e,
Bas Relief, Chrome, Graphic Pen, Halftone Pattern, Photocopy, Plaster, Stamp,
Torn Edges, Craquelure, Texturizer, Oil Paint) carry no seed.

- *Why:* the family determinism requirement; PSB/undo reproducibility. It is not
  an attempt to match Adobe's exact random sequence.

### Reuse the M22 shared helpers

Posterize, edge magnitude, `clamp_u8`, value noise, surface height, emboss, and
`TextureOptions` all live in `artistic/`. The new modules import them rather than
re-implementing.

- *Why:* avoids a second noise/emboss implementation drifting from the first;
  Texturizer and Conté Crayon use the same `TextureOptions` contract as Rough
  Pastels/Underpainting.

### Deferred surface

The Filter Gallery dialog/stack, Smart Filter entries, `Edit > Fade`, 16/32-bit
gating, CMYK/Lab gating, and `Load Texture` file I/O stay out of scope, matching
M22.

- *Why:* the kernels are the dependency for all of those; adding the surface
  later does not rework them.

### Process: orchestrator + waves

Orchestrator freezes the interface, then runs waves: (1) interface freeze in
`lib.rs`; (2) family implementation in parallel — Brush Strokes, Sketch, Texture,
Oil Paint — each with its own tests; (3) app mapping + self-test; (4) archive.

- *Why:* 29 filters is a large surface; disjoint modules let each family land
  with its own tests and no shared-file contention.

## Risks / Trade-offs

- **No oracle** → parity is behavioural; tests assert effect, parameter
  monotonicity, determinism, alpha preservation, and no-panic edges, and the
  families are documented as approximation.
- **29 filters is a large surface** → split into waves, each family in its own
  module with its own tests.
- **Oil Paint parity gap** → the CPU model cannot be compared against CS6 output
  and deliberately omits the GPU requirement; documented as non-parity.
- **Texture memory** → surfaces and grain are generated procedurally per tile;
  cellular/tile kernels are bounded, never materialized at PSB scale.
- **Performance** → `ponytail:` naive per-pixel neighbourhood scans; optimize
  only if a documented filter is measurably slow.

## Migration Plan

Additive; new modules, enums, and variants only. Rollback deletes the new
modules, enums, and `Filter` variants. No document-format or API change.

## Open Questions

- Exact CS6 defaults/ranges (see the docs' Open questions) — the implementation
  uses the documented `(inferred)` defaults and clamps to the sourced ranges.
- Whether CS6's scattered randomness is seed-reproducible — our stored `seed` is
  a behavioural choice and may not match Adobe's exact output.
- Halftone cell geometry (size units, screen angle) and the exact crack/cell
  generators are undocumented; behavioural parity only.
