## Context

Every engine action the plan lists already exists as a C++-callable
`PictureView` `#[qinvokable]`: selection (`select_all`/`deselect`/`select_rect`/
`select_ellipse`/`begin_lasso`/`lasso_add_point`/`end_lasso`/`quick_select`/
`magic_wand`/`selection_count`/`selection_bounds`, `cxxqt_object.rs:699-841`),
filters (`apply_filter`, `:945`), adjustments (`add_adjustment`, `:938`), layer
operations (`set_layer_*`/`move_layer`/`translate_layer`/`remove_layer`/
`duplicate_layer`/`add_layer`, `:190-243`), and the GPU toggle (`set_gpu_compute`,
`:1179`). `PicturaMainWindow::setActiveTool(ToolId)` exists (`frame.h:135`) but
there is **no string→`ToolId` parser** — `toolIdName(ToolId)` and
`ToolInfo::label` (`tool_catalog.cpp:172`, `:25`) are the only naming.

## Goals / Non-Goals

**Goals:**
- Six methods that drive the document through the app's own non-interactive
  paths: `set_tool`, `selection`, `filter`, `adjustment`, `layer_op`,
  `set_gpu_compute`.
- Prove them in the self-test with observable effects (a selection count, a
  filter's deterministic pixel change, a layer property round-trip).

**Non-Goals:**
- Input synthesis (`pointer`/`key`) — a separate change; it needs an image→widget
  inverse and key-focus handling.
- Typed filter parameters or a caller seed (the app fixes its seeds today).
- The out-of-process `pictura-mcp` frontend.

## Decisions

- **`set_tool` accepts the names `status` already uses.** The parser compares the
  argument against `toolIdName(id)` (the lowercase asset name, e.g. `marquee`)
  and `ToolInfo::label` (e.g. `Marquee`) over `allToolIds()`; the two naming
  schemes the app exposes are both accepted, and an unknown name is
  `invalid_param`. This is the only new helper; ~8 lines.
- **`layer_op` maps to the existing bridge signatures.** `set_blend` takes the
  4-byte key `layer_blend(i)` returns; `set_visible` requires an explicit bool
  (`visible` or `on`) rather than defaulting; `set_lock` takes the flag name the
  bridge accepts (`transparency`/`pixels`/`position`/`nesting`/`all`) with an
  `on` bool; `set_name` requires a non-empty name; `move` is a `delta`;
  `translate` is `dx`/`dy` and targets the active layer, so a supplied `index`
  is made active first (the requested layer must be the one moved);
  `duplicate` returns the new index. The method reports `{ok, layers}` using
  `layer_count()` so a caller can see the result.
- **`selection` returns the observable state.** Every op that can fail returns
  `invalid_param` when it does not apply (a non-positive rect/ellipse, a lasso
  with fewer than three points, a wand outside the document), and an unknown
  `mode` is `invalid_param` rather than silently `new`. On success it returns
  `{has_selection, count, bounds}` from `has_selection()`/`selection_count()`/
  `selection_bounds()`, so an agent can assert the effect without a screenshot.
- **`filter` is kind-only** (`apply_filter`); the app's fixed seeds make it
  deterministic (e.g. `add-noise` seed 1), which the self-test asserts. A
  refusal (no editable pixel target, hidden, or pixel-locked) returns `refused`;
  callers see `invalid_param` only for an unknown kind. This pre-check exists
  because `apply_filter` returns false for both and the app exposes the refusal
  state (`active_layer_visible`, the layer lock bitmask, the active path).
- **Errors are the app's.** A locked/Background layer or a document-less call
  returns the existing codes (`no_document`/`refused`) rather than inventing new
  behavior.
- **Self-test at the next append-only codes (493+).** In `control_server.cpp`'s
  existing block: `set_tool` then `selection {op:rect}` yields a non-zero count
  and bounds; `filter {kind:add-noise}` changes a pixel deterministically (twice
  equal) and an unknown kind is `invalid_param`; a `layer_op set_opacity` +
  `list_layers` round-trip; `set_gpu_compute false` reports CPU.

## Risks / Trade-offs

- **`set_tool` naming drift.** If a tool is renamed the internal name changes but
  the label is stable; accepting both mitigates it.
- **`filter` determinism.** Fixed seeds make it repeatable but not
  caller-controlled; the ceiling is documented and the params are future work.
- **Coordinate mismatch with `ui_tree`.** `ui_tree` rects are frame-local, which
  matters for the future `pointer` change, not this one.
