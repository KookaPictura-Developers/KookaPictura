## Context

Four targeted fixes to the Layers panel. The bridge already recomposites on
every Opacity/Fill tick (`set_layers_opacity`/`set_layers_fill`), so the canvas
already previews live; the only defect is that each tick also calls `record`,
pushing a history state. The fix mirrors the move-tool split: a preview call
mutates and repaints without recording, and a commit call records once.

## Goals / Non-Goals

- Goal: one undo state per finished Opacity/Fill edit; live preview throughout.
- Goal: row shows one eye toggle and, when locked, a right-side lock badge.
- Goal: the `%` renders inside the value box.
- Non-Goal: cancel/revert of an in-flight drag (a release always commits).
- Non-Goal: changes to `set_layer_opacity`/`set_layer_fill` (single-layer,
  still one-shot) or to `set_layers_opacity`/`set_layers_fill` (used by tests).

## Decisions

### Bridge: preview / commit split

New `#[qinvokable]` methods on `PictureView` (declaration in
`cxxqt_object.rs`, impl in `impl_layers.rs`):

```rust
fn preview_layers_opacity(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;
fn commit_layers_opacity(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;
fn preview_layers_fill(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;
fn commit_layers_fill(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;
```

State added to `PictureViewRust`: `opacity_preview_changed: bool` and
`fill_preview_changed: bool` (both default `false`).

- `preview_*`: clamp to `0..=255`; apply `set_opacity_paths`/`set_fill_paths`; if
  the change count is non-zero, `recomposite()` and bump `content_revision` (so
  the move-preview cache invalidates) and set the matching flag. No `record`.
- `commit_*`: clamp; apply; if the apply changed something **or** the matching
  flag is set, `recomposite()` and `record("Opacity")` / `record("Fill
  Opacity")`; clear the flag either way. No-op when nothing changed and no
  preview ran.

`content_revision` is bumped in `record` as well; a double bump is harmless (it
only guards caches).

### PercentField: `%` inside, commit signal

- `suffix_` becomes a child of `edit_` (removed from the layout). The edit
  reserves right-hand room with `setTextMargins(0, 0, suffix width + 6, 0)` and
  the suffix is repositioned whenever `edit_` resizes (handle `QEvent::Resize`
  in the existing event filter). The `%` stays a scrub handle; its cursor is
  `SizeHorCursor`.
- New signal `valueCommitted(int pct)`. `valueChanged` remains the live-preview
  signal. A private `pending_` flag is set whenever `applyUserValue` emits
  `valueChanged`, and `commitPending()` emits `valueCommitted` once and clears
  it.
- `commitPending()` runs on: label/field/suffix mouse release after a scrub,
  `QSlider::sliderReleased`, text `editingFinished`, and the popup's
  `QEvent::Hide` (covers keyboard/programmatic slider moves). `setValue` (sync)
  never sets `pending_` and emits nothing.

### Panel wiring

`valueChanged` → `preview_layers_*`; `valueCommitted` → `commit_layers_*`. The
existing `syncing_`/`selectedPaths()` guards are unchanged.

### Row delegate

- `LayersModel::data` stops returning `Qt::CheckStateRole` (and `setData` drops
  that branch); the eye toggle is the only visibility control and is already
  handled in `LayersPanel::eventFilter`.
- `LayerRowDelegate::paint` draws a right-aligned lock badge (`layers.lockAll`)
  when `LockRole != 0`, before the fx/mask badges, so it sits at the row's right
  edge. New public `QRect lockRect(const QRect& itemRect) const` mirrors
  `eyeRect`, for the test hook.

## Risks / Trade-offs

- A drag that returns exactly to its starting value still records one redundant
  state (the flag is sticky). `// ponytail:` ceiling: store per-path originals
  if that ever matters.
- The `%` overlay relies on a fixed text margin; a very narrow field could clip
  the value. The value edit is fixed-width, so this is stable.

## Migration Plan

None — additive bridge methods, no persisted-state or document-format change.

## Open Questions

None.
