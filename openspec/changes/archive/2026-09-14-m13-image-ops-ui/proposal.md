# M13 — Image Ops App UI

## Why

M10 (`pictura-ops`) and M12 (`pictura-render::document_ops`) shipped the full
Image Size / Canvas Size / Image Rotation math with oracle verification, but
both milestones explicitly deferred app integration: the running Qt app cannot
resize a document, grow/shrink its canvas, or rotate/flip it. The engine
capabilities exist and are spec'd; the app is the missing consumer.

## What Changes

- Add document-operation commands to the `PictureView` QObject:
  `resize_image` (resample kind + width/height), `resize_canvas` (anchor +
  width/height), `rotate_doc` (quarter turns), `flip_doc` (horizontal).
- Add an **Image** section to the Layers dock in the Qt shell: Image Size
  (width/height spinboxes + resample combo), Canvas Size (width/height
  spinboxes + 9-anchor combo), and Rotate 90 CW / 90 CCW / 180 / Flip
  Horizontal / Flip Vertical buttons.
- Commands mutate the loaded `Document` through the existing
  `pictura-render::document_ops` API, recompute the composite, and refresh the
  view (new dimensions propagate to the displayed image).
- Any applied document operation clears the active selection (dimensions may
  change; matches Photoshop behavior of dropping the selection on Image/Canvas
  Size).
- Invalid parameters (zero sizes, unknown kinds/anchors, quarter turns outside
  1..=3) are rejected with the document untouched, and the command returns
  `false`.
- Extend the headless `--self-test` to prove a document op end-to-end in the
  app (dimensions change, composite recomputed, selection cleared).

## Capabilities

### New Capabilities

- `image-ops-app-ui`: app-level (QObject + Qt shell) commands and controls for
  Image Size, Canvas Size, and Image Rotation / Flip on the loaded document.

### Modified Capabilities

## Impact

- `crates/pictura-app/src/cxxqt_object.rs` — new Q_INVOKABLE methods + kind/anchor parsing.
- `crates/pictura-app/cpp/main.cpp` — dock controls, wiring, self-test extension.
- No new dependencies; no changes to `pictura-ops`, `pictura-render`, or `docs/`.
- Specs consumed: `document-resize`, `document-canvas`, `document-orientation`.
