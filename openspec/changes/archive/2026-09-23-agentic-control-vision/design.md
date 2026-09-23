## Context

`control_server.{h,cpp}` dispatches newline-delimited JSON on the GUI thread and
already exposes `status`/`get_pixel`/`list_layers`/`list_commands`/
`dispatch_command`/`document`/`edit`/`set_unsaved_policy`. The backing for vision
exists: `PicturaMainWindow` is a `QMainWindow` (`frame.h:55`, so `grab()` works),
`ImageView` is a `QWidget` (`image_view.h:26`), `PictureView::image()` returns the
display `QImage` (`cxxqt_object.rs:149`), and `PictureView::layer_thumbnail(i,
size)` already bridges a layer image (`cxxqt_object.rs:253`). `--headless` uses
the offscreen QPA before `QApplication` (`main.cpp:73`), and the self-test already
grabs a canvas into a `QImage` and asserts pixels (`selftest_canvas_view.cpp:43`),
so widget grabs work without a display.

## Goals / Non-Goals

**Goals:**
- Three read-only methods: `screenshot`, `ui_tree`, `layer_thumbnail`, returning
  the plan's JSON shapes.
- Keep the response bounded (`max_dim` on images, depth/children caps on the
  tree) so a screenshot cannot blow a client's context.
- Prove the methods offscreen in the self-test.

**Non-Goals:**
- Input synthesis (`pointer`/`key`/`set_tool`) and the engine actions — a later
  change.
- The out-of-process `pictura-mcp` frontend — a later change.
- Any new dependency or link.

## Decisions

- **Scope mapping.** `window` = `frame.grab().toImage()`; `canvas` =
  `frame.imageView()->grab().toImage()` (the painted widget: checkerboard, zoom,
  selection overlay; `no_document` when there is no active canvas); `document` =
  `frame.activeView()->image()` (the raw composited display image, no chrome).
  `PictureView::image()` is non-const, so call it on a mutable active view.
- **In-memory PNG.** `QImage::save(&QBuffer, "PNG")` then `QByteArray::toBase64()`.
  No new code path is needed beyond this; PNG saving is already exercised offscreen.
- **Downscale before encode.** If the long edge exceeds `max_dim`, scale with
  `Qt::KeepAspectRatio`/`Qt::SmoothTransformation`; report the pre-scale
  dimensions as `source_width`/`source_height`. Default `max_dim` 1280.
  `ponytail:` a single capped JSON line is the whole size guard.
- **`ui_tree` is a capped DFS** over `QWidget` children from the frame, with
  `max_depth` (default 12) and `max_children` (default 64). The default depth was
  tuned so a default call reaches the layers panel (~9 levels down) — the plan's
  §P2 note to confirm the defaults against the real frame. Rects are window-local
  (`mapTo(frame, {0,0})` + `size()`); `objectName` is populated for panels,
  tools, the options bar, and the panel columns/groups, so the tree is
  navigable. `text` is filled where the widget has one (`QAbstractButton` text,
  `QAction` text, window title); otherwise empty — the plan's shape, not an
  exhaustive accessibility model.
- **Self-test at the next append-only codes (starting 483).** Extend the existing
  `selftest_control.cpp` block: request each method over the socket, base64-decode
  the image, and assert the PNG magic bytes and the reported dimensions; assert
  the tree contains `layersPanel`; assert a pixel layer's thumbnail is a PNG and
  a group's is `invalid_param`.

## Risks / Trade-offs

- **Response size.** Mitigated by `max_dim`/caps; a client that needs a full-size
  image can raise `max_dim`.
- **Offscreen grab fidelity.** The offscreen backend renders widgets into images
  (already relied on by the canvas self-tests); a future platform backend change
  would surface here.
- **`ui_tree` payload.** Caps bound it; if a real frame exceeds them the agent
  can raise the caps per call.
