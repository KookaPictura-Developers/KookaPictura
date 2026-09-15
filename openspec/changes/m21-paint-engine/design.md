## Context

`pictura-render` composites a full layer stack and `apply_filter` mutates a
layer destructively, but there is no code that turns a pointer path into marks.
`docs/07-color-painting/brush-engine.md` (`BRU-001`) and
`docs/03-tools/brush-and-pencil.md` (`TOOL-020`) specify a large engine: five
tip families, seven dynamic sets, tablet pose, dual brush, texture, and per-tool
modes. M21 builds the shared core those all sit on and the two reference tools.

The existing shell already has a tool controller that routes image-space pointer
events (`crates/pictura-app/cpp/tools.cpp`), an `ImageView` with press/move/
release signals, a labeled per-stroke history in
`crates/pictura-app/src/history.rs`, and a dock/options-bar frame. M21 plugs a
new paint crate into those.

## Goals / Non-Goals

**Goals:**

- A `pictura-paint` crate: procedural round/elliptical tip, spacing (fixed and
  velocity-driven), flow/opacity accumulation with a per-stroke cap, pencil
  aliasing and Auto Erase, and the four tool modes `Normal`, `Dissolve`,
  `Behind`, `Clear`.
- Brush and Pencil tools that paint the foreground colour into the topmost
  raster layer, one undo state per completed stroke.
- Options-bar controls for size, hardness, opacity, flow, mode, and Auto Erase.

**Non-Goals:**

- Sampled, bristle, erodible, and airbrush tips; the dynamic sets (Shape
  Dynamics, Scattering, Texture, Dual Brush, Color Dynamics, Transfer, Brush
  Pose); airbrush time build-up; tablet pressure mapping; `.abr` preset I/O; the
  Brush panel, Brush Presets panel, and HUD.
- The remaining 23 paint modes, 16/32-bit and non-RGB painting, lock
  transparency, and sparse tile scratch storage.

## Decisions

### New `pictura-paint` crate, operating on `pictura-core::Document`

The engine needs to read the document bounds and write a layer's planar
channels, so it depends on `pictura-core` and nothing else. It is a separate
crate because painting is interactive and time-based, and `docs/` already
proposes `pictura-paint`/`pictura-brush`; folding it into `pictura-render` would
mix a stroke engine into the compositor.

- *Alternatives considered:* a module inside `pictura-render` (rejected: the
  compositor has no stroke/tip concepts); a crate over raw buffers (rejected:
  the crossing types would be redefined).

### Coverage scratch buffer, composited from a pre-stroke base

A stroke accumulates an 8-bit coverage value per pixel in a layer-sized scratch
buffer, then composites the scratch onto a copy of the pre-stroke layer through
the opacity and mode. The bridge keeps the pre-stroke document and re-composites
after every sample so the canvas updates live; `end` captures history.

- *Why:* compositing from the base + full scratch every sample makes the final
  result independent of how the samples were chunked, so a scripted path and an
  interactive path give identical pixels. Keeping the base clone costs one
  document copy per stroke — the same order as the history snapshot already
  taken.
- *Alternatives considered:* incremental compositing per dab (rejected:
  sequential alpha blends do not commute, so coverage would depend on sample
  rate); scratch tiles (deferred — `ponytail:` a layer-sized `Vec<u8>` coverage
  buffer is allocated per stroke; move to sparse tiles when PSB sizes matter).

### Flow/opacity model

Per pixel the scratch accumulates `acc = 1 - (1 - acc)·(1 - flow·tip)`. The
composited paint alpha is `opacity · acc`. One dab with flow 33% moves 33% of
the way; repeated passes within one stroke approach the opacity cap and never
exceed it; a new stroke starts from `acc = 0` and adds another independent step.

- *Why:* reproduces the CS6 Help example (opacity 33% + flow 33% per pass) and
  the acceptance rule "no amount of back-and-forth exceeds the opacity".

### Procedural tip with a bounded anti-aliasing ramp

The tip alpha is a radial profile: `1` inside `hardness%` of the radius, then a
smoothstep falloff to `0`. The Brush keeps a minimum ramp width of about one
pixel so hardness 100% is still anti-aliased; the Pencil thresholds the profile
to `0`/`1` and never anti-aliases.

- *Why:* the two tools differ only in the profile, so one tip generator serves
  both and the aliasing contract is one flag.

### One history state per stroke, captured at `end`

`begin_paint` stores the pre-stroke document, `paint_dab` mutates a working
copy and emits `changed`, and `end_paint` captures the post-state labelled
"Brush" or "Pencil" and marks the document dirty. A stroke that painted nothing
captures nothing. `undo` during a stroke is ignored while the button is down.

- *Why:* matches `ARCH-009` undo granularity and the M14 labeled-history model.

### Bridge surface mirrors the interactive gestures

`PictureView` gains `begin_paint(settings)`, `paint_dab(x, y, pressure)`,
`end_paint() -> bool`, and `cancel_paint()`. Settings are passed per stroke
(size, hardness, opacity, flow, mode, auto-erase, aliased) so the tool state
lives in the Qt options bar, not in the bridge.

- *Why:* keeps the Rust side a pure engine and the tool UI in Qt, matching the
  M18 `ToolController` split.

## Risks / Trade-offs

- **Layer-sized scratch per stroke** → Fine for M21 document sizes; the
  `ponytail:` comment names sparse tiles as the upgrade. A 5000 px brush on a
  very large PSB is out of M21's tested envelope.
- **Base clone per stroke doubles peak document memory** → Bounded by one
  stroke; freed at `end`/`cancel`.
- **Coverage accumulates at full layer resolution** → Simple and correct; costs
  one `u8` per document pixel per in-flight stroke.
- **Dissolve needs randomness with deterministic redo** → Seed the stroke RNG
  at `begin` from a fixed value so a scripted redo reproduces; interactive
  strokes differ by advancing the seed per stroke.
- **Parity claims** → The falloff shape and dissolve distribution are not
  Adobe-verified; they are behavioral and tested by property, not against CS6.

## Migration Plan

Additive. New crate added to the workspace; the bridge and tool UI gain methods
and a tool. Rollback is deleting `crates/pictura-paint`, the bridge methods, and
the Brush/Pencil tool entries. No document/PSD format changes.

## Open Questions

- Exact `pictura-paint` type names may shift during implementation; the frozen
  contract is the `StrokeConfig` fields and the begin/dab/end bridge surface.
- Whether M21 should expose the paint mode list in the options bar or fix
  `Normal`; the design allows a four-entry combo and leaves the rest to the
  mode-list milestone.
