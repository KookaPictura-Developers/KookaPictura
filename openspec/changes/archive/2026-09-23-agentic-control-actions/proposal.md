## Why

The control server can read state and vision but cannot *change* the document
except through parameterless menu commands (`dispatch_command`). An agent cannot
make a selection, apply a specific filter or adjustment, set a layer property, or
switch tools with a value — the plan's §5.3/§5.7 action surface. This change adds
those engine actions (the safe half of P3); input synthesis (`pointer`/`key`),
which needs coordinate inversion and key-focus handling, is a separate change.

## What Changes

- `set_tool` (`tool`): switch the active tool, accepting the same name `status`
  reports (`active_tool`) or the display label; an unknown tool is
  `invalid_param`.
- `selection` (`op` = `all` | `deselect` | `rect` | `ellipse` | `lasso_begin` |
  `lasso_point` | `lasso_end` | `quick` | `wand`, with the op's coordinates,
  `mode`, `tolerance`, `contiguous`, `feather`): drives the selection model and
  returns `{has_selection, count, bounds?}`.
- `filter` (`kind`): applies one of the app's fixed filter kinds (seed already
  fixed by the app) and returns `{ok, kind}`; an unknown kind is `invalid_param`.
- `adjustment` (`kind`): adds an adjustment layer of the given kind.
- `layer_op` (`op` = `set_name` | `set_opacity` | `set_visible` | `set_blend` |
  `set_fill` | `set_lock` | `set_color` | `move` | `translate` | `delete` |
  `duplicate` | `add`, with `index` and the op's value): drives the layer
  operations and returns `{ok, layers}`.
- `set_gpu_compute` (`on`): toggles the GPU compute path and returns
  `{gpu_compute, backend}`.
- Ceilings (`ponytail:`): `filter` takes a kind only (typed filter params and a
  caller seed are not plumbed); every action is the app's real non-interactive
  path, so a locked/Background layer or a document-less call returns the app's
  own refusal.

## Capabilities

### Modified Capabilities
- `agentic-control`: an engine-action surface is added to the existing local
  control endpoint.

## Impact

- `crates/pictura-app/cpp/control_server.{h,cpp}`: the six dispatch arms plus a
  string→`ToolId` parser (the only new non-trivial helper; the app has no
  reverse lookup today).
- No Rust change is expected (every backing `#[qinvokable]` exists); no new
  dependency or link.
- Self-test: a `mcp_control_action_*` block using the next append-only codes.
