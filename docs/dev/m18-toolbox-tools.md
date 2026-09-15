# M18 — Toolbox and core tools

Goal: introduce the tool layer. A tool registry with an active tool, a Tools
panel, and a context-sensitive options bar; and the first eight tools — Move,
Marquee, Lasso, Quick Selection, Crop, Eyedropper, Hand, Zoom — with the small
engine operations they need. OpenSpec change: `m18-toolbox-tools` (new
capabilities `tool-framework`, `shape-selection-tools`, `canvas-tools`).

## Scope

- `pictura-select`: rectangle/ellipse/polygon coverage rasterizers, combine
  modes, Quick Selection (wand union).
- `pictura-render` document ops: `crop_document`, `translate_layer`.
- Bridge: selection shape calls, lasso streaming, quick select, crop, layer
  translate, pixel sample, selection bounds.
- `cpp/image_view.{h,cpp}`: pointer signals, pan flag, overlay polygon.
- `cpp/tools.{h,cpp}`, `toolbox.{h,cpp}`, `options_bar.{h,cpp}`: tool registry,
  Tools dock, options bar, and the tool behaviors.
- Frame wiring: host the tools, route canvas events, expose tool test hooks.
- `main.cpp` self-test: tool switching, selection shapes + combine modes, crop,
  move, eyedropper.

## Out of scope (later milestones)

- Brush/paint, clone/heal, transform/warp, text/vector/shape tools.
- Quick Mask, Refine Edge, transform selection.
- Non-destructive crop and a crop shield; marching ants for arbitrary masks
  (M18 draws a rubber band during the drag and exposes committed bounds).
- The Color panel (the Eyedropper records a frame foreground colour only).

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Sub-agents own all code: E (engine + bridge), V (canvas event
surface), T (tool framework + frame wiring), plus an integration agent for the
self-test. Interfaces are frozen as text in `design.md` before dispatch.

## Verification

- `cmake -S . -B build && cmake --build build`
- fixture self-test and no-argument self-test exit 0 with new tool checks (exit
  codes from 39): tool switching/options, marquee rect+ellipse, lasso, quick
  selection, combine modes, crop remap, layer move, eyedropper sample.
- `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`.
