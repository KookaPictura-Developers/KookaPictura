## Why

The engine can composite, adjust, filter, and select, but it cannot paint. Every
painting tool in `docs/03-tools/` and `docs/07-color-painting/` — Brush, Pencil,
Eraser, Clone, Smudge, Dodge/Burn, History Brush — shares one dab engine that
does not exist yet. This milestone builds that engine's foundation and the
reference Brush/Pencil tools on top of it.

## What Changes

- Add a `pictura-paint` crate implementing a dab-splatting stroke engine:
  standard round/elliptical procedural tip with anti-aliased hardness falloff,
  fixed and velocity-driven spacing, and flow/opacity accumulation with a
  per-stroke opacity cap.
- Add the Pencil tool's aliased hard-edge painting and its Auto Erase behavior.
- Add tool paint modes `Normal`, `Dissolve`, `Behind`, and `Clear`.
- Commit exactly one history state per completed stroke; ignore undo mid-stroke.
- Add the Brush and Pencil tools to the tool set, with an options bar
  (size, hardness, opacity, flow, mode, Auto Erase) and on-canvas
  press/drag/release painting against the topmost raster layer.
- Defer the CS6 tip families (sampled, bristle, erodible, airbrush), the dynamic
  sets (Shape Dynamics, Scattering, Texture, Dual Brush, Color Dynamics,
  Transfer, Brush Pose), tablet pressure dynamics, `.abr` presets, the Brush
  panel, the Brushes/HUD UI, and the remaining paint modes. These stay specified
  in `docs/` and get their own milestones.

## Capabilities

### New Capabilities

- `paint-engine`: the Rust dab engine — tip coverage, stroke spacing, dab
  compositing (flow/opacity cap), pencil aliasing + Auto Erase, paint modes, and
  per-stroke document commit.
- `brush-tools`: the Brush/Pencil tools, options-bar controls, canvas pointer
  routing into the engine, and one-history-state-per-stroke integration.

### Modified Capabilities

- `tool-framework`: the fixed tool set gains Brush and Pencil; the options bar
  gains the paint controls.

## Impact

- New crate `pictura-paint` (workspace member); `pictura-app` bridge gains
  stroke begin/dab/end and brush-setting methods; `crates/pictura-app/cpp/tools.*`
  gains `ToolId::Brush`/`Pencil` and the paint options; `frame.*` wires undo.
- No `pictura-core` field changes: the stroke writes existing layer pixels.
- No new external dependencies; the tip and blend math use `std` only.
- Deferred areas are explicitly listed above so no reader assumes the full
  `BRU-001`/`TOOL-020` surface ships here.
