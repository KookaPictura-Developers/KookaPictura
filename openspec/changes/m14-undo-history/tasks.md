# Tasks: m14-undo-history

## 1. M14-A — history module and commands (Rust)

- [ ] 1.1 Add `crates/pictura-app/src/history.rs`: `Snapshot { doc: Document, selection: Option<Selection> }`, `History` with an index-based bounded stack (depth 20), `capture(snapshot)`, `undo(current) -> Option<Snapshot>`, `redo(current) -> Option<Snapshot>`, `can_undo()`, `can_redo()`, `depth()`; mark the full-clone ceiling with a `ponytail:` comment naming COW/tile-diff as the upgrade path
- [ ] 1.2 Add `undo() -> bool`, `redo() -> bool`, `can_undo() -> bool`, `can_redo() -> bool`, `history_depth() -> i32` Q_INVOKABLE methods to `qobject::PictureView`; `undo`/`redo` restore doc + selection into `PictureViewRust`, call `recomposite()`, and return `false` when unavailable
- [ ] 1.3 Wire capture into every mutating command (`apply_filter`, `add_adjustment`, `remove_layer`, `set_layer_visible`, `resize_image`, `resize_canvas`, `rotate_doc`, `flip_doc`, `select_all`, `deselect`, `magic_wand`): clone state after validation, push only on success
- [ ] 1.4 Reset history in `open()` before loading, success or failure
- [ ] 1.5 Unit-test `History` in `crates/pictura-app/src/history.rs` (no Qt): capture/undo/redo round-trip on small documents, failed-capture no-op behavior simulated by direct calls, depth bound at 20 (21st capture drops the oldest), truncate-on-new-op invalidates redo

## 2. M14-B — Qt shell controls

- [ ] 2.1 Add Undo/Redo buttons to the dock in `crates/pictura-app/cpp/main.cpp`, wired to `view.undo()`/`view.redo()` with `refresh()` on success
- [ ] 2.2 Add `QShortcut` Ctrl+Z / Ctrl+Y on the main window wired to the same commands
- [ ] 2.3 Drive `setEnabled(view.can_undo() / view.can_redo())` from `refresh()` so the controls track availability after every state change

## 3. M14-C — Self-test coverage

- [ ] 3.1 In the `--self-test` codecLoaded block, after the existing checks: capture the image, run a mutating command, assert `history_depth()` increased and `can_undo()`; `undo()` and assert the image bit-identical to the pre-command image; `redo()` and assert bit-identical to the post-command image; run another command and assert `can_redo()` is `false`
- [ ] 3.2 Assert `undo()` at the empty-stack boundary returns `false` and leaves the image unchanged (after `open()` reset or exhaustion)
- [ ] 3.3 Run the undo/redo checks last inside the codecLoaded block; new distinct exit codes (22, 23) that do not collide with 2–21

## 4. M14-D — Verify and reconcile

- [ ] 4.1 `openspec validate --all --strict` green
- [ ] 4.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` green
- [ ] 4.3 `cmake --build build` green; `xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with the new undo/redo assertions logged
- [ ] 4.4 `bash scripts/guard.sh` green (no `docs/` edits)
- [ ] 4.5 Record the milestone in `docs/dev/m14-undo-history.md`, update `docs/dev/STATE.md`, archive the change (commit carries `TASK-ALLOWS-DOCS`)
