# Tasks: m13-image-ops-ui

## 1. M13-A — `PictureView` document-op commands

- [ ] 1.1 Add `resize_image(kind: &QString, width: i32, height: i32) -> bool`, `resize_canvas(anchor: &QString, width: i32, height: i32) -> bool`, `rotate_doc(quarter_turns: i32) -> bool`, and `flip_doc(horizontal: bool) -> bool` Q_INVOKABLE methods to `qobject::PictureView` in `crates/pictura-app/src/cxxqt_object.rs`, following the `apply_filter` pattern
- [ ] 1.2 Parse `kind` to `pictura_ops::Resample` (`"nearest"`, `"bilinear"`, `"bicubic"`) and `anchor` to the nine `pictura_ops::Anchor` values; unknown strings return `false` with no mutation
- [ ] 1.3 Reject non-positive `width`/`height` and out-of-range `quarter_turns` (outside 1..=3) before touching the document; return `false` when no document is loaded
- [ ] 1.4 On success call the matching `pictura_render` document op, clear `selection`, then `recomposite()`
- [ ] 1.5 Unit-test the Rust command layer in `crates/pictura-app` (or via the cxx-qt headless path used by existing tests): valid kinds/anchors succeed and resize/rotate/flip the doc; invalid kinds, anchors, dimensions, and quarter turns return `false` with the document bit-identical; selection is cleared after each success

## 2. M13-B — Qt shell dock controls

- [ ] 2.1 Add an "Image" section to the Layers dock in `crates/pictura-app/cpp/main.cpp`: width/height `QSpinBox` (1..=32767, initialized from the loaded document) + resample combo (Nearest/Bilinear/Bicubic) + "Apply Image Size" button
- [ ] 2.2 Add canvas-size controls: width/height spin boxes + nine-entry anchor combo + "Apply Canvas Size" button
- [ ] 2.3 Add orientation buttons: Rotate 90° CW, Rotate 90° CCW, Rotate 180°, Flip Horizontal, Flip Vertical, wired to `rotate_doc`/`flip_doc`
- [ ] 2.4 Wire every control through the existing `refresh()` path so the layer list, image, and selection label update after any successful command

## 3. M13-C — Self-test coverage

- [ ] 3.1 Extend the `--self-test` block in `main.cpp`: after the existing checks, rotate the fixture document 90° CW and assert the command returns `true`, the displayed image keeps the expected dimensions, the red quadrant moved from top-left to top-right, and `has_selection`/`selection_count` report a cleared selection
- [ ] 3.2 Assert an invalid command (`rotate_doc(0)` and `resize_image("bicubic", 0, 8)`) returns `false` and leaves the displayed image unchanged
- [ ] 3.3 Restore/undo state or run the doc-op checks last so existing exit codes 2–18 assertions stay green

## 4. M13-D — Verify and reconcile

- [ ] 4.1 `openspec validate --all --strict` green
- [ ] 4.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` green
- [ ] 4.3 `cmake -S . -B build && cmake --build build` green; `xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with the new doc-op assertions logged
- [ ] 4.4 `bash scripts/guard.sh` green (no `docs/` edits)
- [ ] 4.5 Record the milestone in `docs/dev/m13-image-ops-ui.md` and update `docs/dev/STATE.md` (commit message carries `TASK-ALLOWS-DOCS`)
