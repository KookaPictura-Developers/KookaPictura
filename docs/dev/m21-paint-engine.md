# M21 — Painting engine (foundation)

Goal: turn a pointer path into marks. A new `pictura-paint` crate renders a
procedural round/elliptical tip into per-pixel coverage with fixed or
velocity-driven spacing and a flow/opacity model, and the Brush and Pencil tools
paint the foreground colour into the topmost raster layer, one history state per
stroke. OpenSpec change `m21-paint-engine` (new capabilities `paint-engine` and
`brush-tools`; MODIFIED `tool-framework`).

## Scope

- Rust `pictura-paint`: tip coverage (size/hardness/roundness/angle, anti-aliased
  Brush, aliased Pencil), spacing (fixed percent and velocity-driven with residue
  carry-over), flow/opacity accumulation, modes `Normal`/`Dissolve`/`Behind`/
  `Clear`, `StrokeConfig` validation.
- Bridge: `begin_paint` / `paint_dab` / `end_paint` / `cancel_paint`; a
  pre-stroke base plus a coverage scratch composited live; history capture at
  `end`; topmost-raster-layer targeting.
- App: `ToolId::Brush`/`Pencil`, `B`/`Shift+B`, toolbox entries, options bar
  (size, hardness, opacity, flow, mode, Auto Erase), `[`/`]` and `Shift+[`/`Shift+]`.
- Self-test: scripted stroke mutation + dirty, opacity cap, flow ordering, Brush
  anti-aliased vs Pencil aliased, one state per stroke, undo restoration.

## Out of scope (later milestones)

- Sampled, bristle, erodible, airbrush tips; dynamic sets (Shape Dynamics,
  Scattering, Texture, Dual Brush, Color Dynamics, Transfer, Brush Pose);
  airbrush time build-up; tablet pressure mapping; `.abr` preset I/O; Brush and
  Brush Presets panels; HUD; the remaining paint modes; 16/32-bit and non-RGB
  painting; lock transparency; sparse tile scratch storage.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Sub-agents own all code, in waves: (1) `pictura-paint` crate,
(2) bridge + tool/options/frame integration, (3) self-test.

## Verification

- `cmake -S . -B build && cmake --build build`
- `cargo test -p pictura-paint` and `cargo test --workspace`
- fixture and no-argument self-tests exit 0 with new paint checks (exit codes
  from 53)
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
