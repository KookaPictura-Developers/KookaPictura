# Design: knockout-composite

## Context

`Layer.knockout: Knockout` (`pictura_core::advanced_blending`) is `None`,
`Shallow`, or `Deep`; it round-trips through `knko` and is currently read by no
renderer. `composite_rgba` / `composite_rgba_region` walk the bottom-first
`doc.layers` (PSD on-disk z-order, `read.rs:26`) and call `composite_layer`,
which dispatches groups, fill/adjustment content, smart sources, live type, and
plain pixels, feeding every per-pixel blend through `blend_into` -> `blend_parts`
against the running `Canvas`.

Docs: Shallow knocks out to "the first layer after the layer group or the base
layer of the clipping mask"; Deep "to the background; with no background, to
transparency"; "with no group or clipping mask, either option reveals the
background layer (or transparency if the bottom layer is not a background)"; the
layer's "shape is composited against the layer found by skipping down ... instead
of against the immediately preceding backdrop" — the mechanism is *inferred*
(`layers-overview.md:181`), the per-pixel math is an open question
(`layers-overview.md:308`). This change implements exactly the documented
mechanism and marks it inferred.

## Goals / Non-Goals

**Goals:**

- A top-level Shallow/Deep layer punches through the intermediate layers to the
  document background within its covered pixels.
- `None` composites byte-identically to before.
- The GPU declines a knockout layer so the CPU oracle is authoritative.

**Non-Goals:**

- Nested-group shallow stopping points and clipping bases.
- `Transparency Shapes Layers` gating of the knockout to opaque pixels.
- Blending Options UI.
- Photoshop pixel parity (no oracle).

## Decisions

### D1. Background canvas, built only when a knockout exists

At the start of `composite_rgba` / `composite_rgba_region`, if any layer below
the top has a non-`None` knockout, build `base` = a `Canvas` (same region as the
output) with only `doc.layers[0]` (the bottom layer) composited into it — the
"background". If the bottom layer is itself a knockout, it composites as a plain
layer (no base). When no knockout is present, `base` is `None` and the loop is
unchanged, so a document without knockout is byte-identical.

### D2. Only a non-bottom layer knocks out

The main loop passes `Some(&base)` for every layer except index 0 and `None`
inside groups (group recursion passes `None`). So a knockout layer at the bottom
of the document is inert (nothing below it to punch to), and a knockout inside a
group is inert — the deferred nested-group ceiling.

### D3. Coverage flag on the canvas

`Canvas` gains `cover: Option<Vec<bool>>`. `blend_parts` sets `cover[i] = true`
when it actually contributes (`as_ > 0`, past the dissolve gate). The normal
output canvas leaves `cover` `None`, so the hot path is unchanged.

### D4. `composite_knockout`

For a knockout layer, build `tmp` with `tmp.px = base.px.clone()` and
`tmp.cover = Some(vec![false; n])`, then composite the layer's content into `tmp`
(its inner dispatch, so its blend mode and Blend If are evaluated against the
background, and layer effects ride along). Then for each output pixel,
`canvas.px[i] = tmp.px[i]` **only where `tmp.cover[i]`**; uncovered pixels keep
the running backdrop. Because the covered result was blended against the
background, the intermediate layers vanish there; because uncovered pixels are
left alone, a transparent part of the layer does not erase the intermediate.

The coverage is the layer's own contribution (content + effects), so a
`Transparency Shapes Layers` restriction to the content's opaque pixels is not
applied — marked as a ceiling.

### D5. Refactor, not duplicate

`composite_layer`'s current body becomes `composite_layer_inner`; `composite_layer`
adds only the knockout branch. This keeps one content dispatch and one place that
decides knockout.

### D6. GPU declines

`check_supported`'s `walk` returns `GpuError::UnsupportedAdvancedBlending` for a
layer whose `knockout != None`, next to the existing non-default-`BlendIf`
check. Reusing the variant avoids a new error kind; `composite_active` already
turns any `GpuError` into the CPU composite.

### D7. Tests

Unit tests in `crates/pictura-render/src/tests/composite.rs`, using the
`doc`/`solid`/`full`/`px` helpers:

- A 3-layer 1x1 stack `[red, green, blue(fill 128)]`: with `None` the result is
  green-tinted (intermediate visible); with `Deep` the green channel is gone
  (background red shows), proving punch-through.
- `Shallow` at the top level equals `Deep`.
- `None` is byte-identical to the same stack built without the field.
- A GPU test asserts `composite_gpu` returns `UnsupportedAdvancedBlending` for a
  knockout stack (reuses the existing text-decline test's pattern).

## Risks / Trade-offs

- [Per-pixel math unverified] → implements the documented
  shape-against-the-stopping-point rule only; marked `ponytail:`; a controlled
  CS6 fixture is the resolver (`layers-overview.md:308`).
- [Coverage includes effects] → a drop shadow/glow could punch slightly outside
  the layer's content; `Transparency Shapes Layers` is the deferral. Marked.
- [Bottom layer assumed the background] → the model has no Background flag, so
  the bottom layer is treated as the background; a non-background bottom resolves
  to its content rather than transparency. Marked.
- [GPU divergence] → declining knockout keeps a single oracle.
