# M13 — Image Ops App UI: Design

## Context

`PictureView` (cxx-qt QObject, `crates/pictura-app/src/cxxqt_object.rs`) already
exposes the established command pattern: a Q_INVOKABLE method validates input,
mutates `PictureViewRust::doc` via a `pictura-*` API, then calls
`recomposite()` (which rebuilds `doc.composite`/the QImage and emits
`changed()`). The Qt shell (`cpp/main.cpp`) wires dock buttons to those methods
and proves behavior headlessly in `--self-test`. The M12 document ops
(`pictura_render::resize_document`, `resize_canvas_document`,
`rotate_document`, `flip_document`) have no app entry point yet.

## Goals / Non-Goals

**Goals:**
- Q_INVOKABLE commands for the four M12 document ops, following the
  `apply_filter`/`add_adjustment` pattern exactly.
- Dock controls in an "Image" section: Image Size, Canvas Size, and the five
  orientation commands.
- Selection cleared by any applied document op; composite + view refreshed
  with the new dimensions.
- Self-test coverage proving an end-to-end document op in the app.

**Non-Goals:**
- Dialogs with pixel-dimension previews, percent scaling, or relative canvas
  units (Photoshop's full Image Size dialog is a later milestone).
- `rotate_arbitrary` at document scope (not implemented by M12 either).
- Undo/history (candidate M14).
- Any change to `pictura-ops`/`pictura-render` math or specs.

## Decisions

- **Wrap only what M12 shipped.** The app calls
  `pictura_render::document_ops` functions unchanged; no new engine API, no
  new traits. Alternative (a new unified `apply_doc_op` dispatcher in
  pictura-render) is rejected: one caller, zero reuse.
- **Kinds/anchors as strings, parsed app-side.** `resize_image("nearest"|"bilinear"|"bicubic", w, h)`,
  `resize_canvas(anchor, w, h)` with the 9 `Anchor` names from
  `pictura-ops::canvas` ("center", "top-left", …). Unknown strings → `false`,
  document untouched. This matches how `apply_filter` maps kind strings to
  `Filter` values today.
- **Selection cleared, not remapped.** Doc ops change dimensions; the app
  selection is stored independently of the doc. Photoshop drops the selection
  on Image/Canvas Size; clearing is both simplest and correct. The `changed()`
  signal already refreshes the selection label.
- **No dialogs; dock widgets only.** The dock already hosts parameterized
  controls (combos + buttons). Image/Canvas Size get width/height `QSpinBox`
  pairs (1..=32767, defaulting to the document's current size) plus a combo;
  rotation gets five buttons. Consistent with the existing UI idiom and keeps
  the self-test drivable headlessly.
- **Rotate/flip need no recomposite special-casing.** The document ops already
  recompute `doc.composite`; `recomposite()` re-derives the QImage from the
  doc, so dimension changes flow through the existing refresh path.

## Risks / Trade-offs

- [Dock grows crowded] → Group the new controls under a `QGroupBox`-style
  labeled section (a titled `QWidget`); acceptable for a working shell.
- [Selection cleared on ops that keep dimensions (180°/flip)] → Deliberate:
  uniform behavior across all document ops is simpler to spec and test;
  Photoshop also cancels selections on these commands.
- [cxx-qt signal `changed()` may not fire for pure dimension changes] →
  `recomposite()` is called explicitly after every successful op, same as
  `apply_filter`; the self-test asserts the new dimensions appear in
  `view.image()`.

## Open Questions

None — scope is fully determined by the M12 API surface.
