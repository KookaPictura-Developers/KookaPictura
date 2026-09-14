# M14 — Undo / Redo (edit history)

Goal: bounded snapshot-based undo/redo for every mutating app command,
closing the largest remaining functional gap in the shell (all mutations were
previously irreversible). No engine changes. OpenSpec change:
`m14-undo-history` (new capability `edit-history`).

## What landed

- `crates/pictura-app/src/history.rs` — `Snapshot { doc, selection }` and an
  index-free two-stack `History`: `capture` clears redo, pushes, and drops the
  oldest past 20 states (the CS6 default depth); `undo(current)` /
  `redo(current)` swap the current state with the opposite stack and return
  `None` at the boundaries. `ponytail:` comment marks full-clone as the
  ceiling (COW / tile diffs if PSB-size docs hit RAM).
- `PictureView` commands: `undo()`, `redo()` (restore doc + selection,
  `recomposite()`, `false` at boundaries), accessors `can_undo()`,
  `can_redo()`, `history_depth()`. A private `snapshot()` helper dedupes the
  pre-command clone across the eleven capture sites. undo/redo do not capture.
- Capture wired into every mutating command (`apply_filter`,
  `add_adjustment`, `remove_layer`, `set_layer_visible`, `resize_image`,
  `resize_canvas`, `rotate_doc`, `flip_doc`, `select_all`, `deselect`,
  `magic_wand`): clone after validation, push only on success, so failed
  commands leave history untouched. `open()` resets history on success or
  failure — undo never crosses a file boundary.
- Qt shell: Undo/Redo dock buttons + Ctrl+Z / Ctrl+Y `QShortcut`s (Qt 6 lives
  in QtGui), enabled state driven from `refresh()` via `can_undo`/`can_redo`.
- Self-test (exit codes 22/23): rotate → undo → assert the image bit-identical
  to the pre-op capture → redo → bit-identical to the post-op capture, with
  `history_depth` increasing by one; a following op invalidates redo; a
  reopen resets the stack and boundary `undo()` returns `false` unchanged.

## Verification

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings` clean; `cargo test --workspace` 388 passed / 0 failed
  (6 new `History` unit tests: round-trips, boundary no-ops, depth bound at
  20 with oldest-drop, redo truncation).
- `cmake --build build` green (after moving the `QShortcut` include to QtGui
  — Qt 6 relocation the agent could not verify without a compiler);
  `xvfb-run -a ./build/pictura --self-test
  crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with the
  `history rotate=1 depth=15 undo=1 undo_ident=1 redo=1 redo_ident=1` and
  `history redo_invalid=1 open_reset=1 boundary_undo=0` lines logged.
- `openspec validate --all --strict` green; `scripts/guard.sh` OK.

## Deferred

- History palette UI (labeled states, random-access jump), per-command
  labeling, persistence — later app milestones.
- Memory-optimized histories stay behind the ponytail marker.
