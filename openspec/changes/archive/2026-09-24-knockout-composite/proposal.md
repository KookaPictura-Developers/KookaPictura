# Proposal: knockout-composite

## Why

Roadmap G6/G7: the knockout mode ships (`knko-blend-if-model` — a typed
`Layer.knockout` of `None`/`Shallow`/`Deep` that round-trips) but the compositor
ignores it, so a layer whose Knockout is Shallow/Deep composites exactly as if
it were `None`: the layers beneath it stay visible instead of being punched
through. The docs specify the stopping points and the shape-composited-against-
the-stopping-point mechanism (`docs/05-layers/layers-overview.md:89`,
`layer-groups.md:86`); the exact per-pixel math is an open question
(`layers-overview.md:308`), so this change implements the documented mechanism,
marks it inferred, and claims no Adobe parity.

## What Changes

- **Apply knockout in the CPU compositor.** A top-level, non-bottom layer with
  `Knockout::Shallow` or `Deep` composites its content against the **document
  background** — the bottom layer, or transparency when the knockout layer is
  the bottom — instead of the running backdrop, and its covered pixels **replace**
  the running backdrop, so the layers between it and the background are punched
  through. At the document root both modes resolve to the same background; with
  no group or clipping mask that matches the documented behavior.
- **GPU declines a knockout layer** by reusing `GpuError::UnsupportedAdvancedBlending`,
  so `composite_active` falls back to the CPU oracle. A document with no knockout
  composites byte-identically, so existing GPU parity and goldens are unaffected.
- **No UI.** The Blending Options dialog is not part of this change.
- **Deferred (marked ceilings):** the shallow stopping point **inside a nested
  group** (the first layer below the group) and the clipping-mask base, and the
  `Transparency Shapes Layers` restriction of the knockout to opaque pixels.
  A knockout inside a group is inert in this slice.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `knockout-compositing`: the CPU punch-through behavior and the GPU decline.

### Modified Capabilities

<!-- None. -->

## Impact

- `crates/pictura-render/src/composite.rs`: a coverage flag on `Canvas`, a
  `composite_knockout` path in `composite_layer`, and a `base` canvas threaded
  through `composite_rgba` / `composite_rgba_region`; CPU unit tests.
- `crates/pictura-render/src/gpu/mod.rs`: a knockout check in `check_supported`.
- No new dependency; no app UI; no PSD byte-layout change.
- Ceiling: the per-pixel mechanism is inferred from the documented
  shape-against-the-stopping-point rule (no Photoshop oracle) and marked
  `ponytail:`; nested-group shallow targets and clipping bases are not resolved.
