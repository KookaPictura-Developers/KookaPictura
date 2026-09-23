## Why

The control server (archived `app-control-server`) can read state and dispatch
actions but cannot *see* — an agent has no screenshot, no widget tree, and no
layer thumbnail, so it cannot confirm that the UI it drove actually looks right.
This is plan §5.2 (vision) and §P2 of `docs/dev/mcp-agentic-control-plan.md`.
Every backing call already exists (`QWidget::grab`, `PictureView::image()`,
`PictureView::layer_thumbnail`), so this change adds only the JSON/PNG wrappers
and one self-test block.

## What Changes

- `screenshot` (`scope` = `window` | `canvas` | `document`, `max_dim` default
  1280) returns `{mime:"image/png", base64, width, height, source_width,
  source_height}`: the whole frame, the active canvas (checkerboard, zoom,
  selection overlay — the painted widget), or the raw composited document image.
  The long edge is downscaled to `max_dim` (keep aspect) before encoding, and the
  original dimensions are still reported.
- `ui_tree` (`max_depth` default 12, `max_children` default 64) returns a depth-
  and breadth-capped array of `{class, objectName, rect{x,y,w,h}, visible,
  enabled, text, tooltip}` with window-local rects, so an agent can find a
  control and translate it into a `pointer`/`dispatch_command` call.
- `layer_thumbnail` (`index`, `size` default 64) returns a PNG of the layer's
  thumbnail via the existing bridge method, or `invalid_param` when the layer has
  none (a group, an adjustment layer, or an out-of-range index).
- Ceilings (`ponytail:`): `max_dim` protects the response size; there is still no
  TCP transport and no input synthesis (that is a later change), so `ui_tree`
  reports rects an agent cannot yet click.

## Capabilities

### Modified Capabilities
- `agentic-control`: three read-only vision methods are added to the existing
  local control endpoint.

## Impact

- `crates/pictura-app/cpp/control_server.{h,cpp}`: three dispatch arms, an
  in-memory PNG/base64 encoder, a keep-aspect scaler, and the `ui_tree` walk. No
  new dependency and no new link (`QImage` is `Qt6::Gui`, `QBuffer` is
  `Qt6::Core`; `Qt6::Gui` is already linked; PNG saving is already proven
  offscreen by the self-test).
- Self-test: a `mcp_control_*` vision block using the next append-only codes.
