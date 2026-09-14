# M14 — Undo / Redo (Edit History)

## Why

Every mutating app command (filters, adjustments, layer changes, document ops)
is destructive and irreversible: one mis-click on "Apply Filter" permanently
alters the loaded document, and the only recovery is re-opening the PSD.
Photoshop CS6 ships a 20-state history stack; an editor without undo is the
largest remaining functional gap in the app shell.

## What Changes

- Add a bounded snapshot history (`crates/pictura-app/src/history.rs`): full
  clones of `(Document, Option<Selection>)` captured before each successful
  mutating command, with index-based undo/redo, depth-bounded at 20 states
  (matching the CS6 default).
- Add `PictureView` commands: `undo() -> bool`, `redo() -> bool`,
  `can_undo() -> bool`, `can_redo() -> bool`, plus a `history_depth() -> i32`
  reporting accessor.
- Wire history capture into every existing mutating command
  (`apply_filter`, `add_adjustment`, `remove_layer`, `set_layer_visible`,
  `resize_image`, `resize_canvas`, `rotate_doc`, `flip_doc`).
- Add Undo/Redo dock buttons and Ctrl+Z / Ctrl+Y shortcuts in the Qt shell,
  enabled/disabled from `can_undo`/`can_redo`.
- Extend the headless `--self-test`: apply an op, undo, assert bit-identical
  to the pre-op image; redo, assert bit-identical to the post-op image; assert
  redo is invalidated by a new op.

## Capabilities

### New Capabilities

- `edit-history`: bounded snapshot-based undo/redo of document mutations in
  the app shell (capture rules, restore semantics, depth bound, app controls).

### Modified Capabilities

## Impact

- `crates/pictura-app/src/history.rs` — new module (history stack, unit-tested
  without Qt, matching the crate's helper-test pattern).
- `crates/pictura-app/src/cxxqt_object.rs` — new QObject methods + one-line
  capture in each mutating command.
- `crates/pictura-app/cpp/main.cpp` — dock buttons, shortcuts, self-test.
- No engine changes; no new dependencies; `Document` and `Selection` already
  derive `Clone`.
