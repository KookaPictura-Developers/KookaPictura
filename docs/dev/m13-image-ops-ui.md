# M13 — Image ops app UI

Goal: expose the M10/M12 image and document operations in the running Qt app,
closing the "app integration deferred" item both milestones left open. No
engine changes; the app consumes `pictura-render::document_ops` as shipped.
OpenSpec change: `m13-image-ops-ui` (new capability `image-ops-app-ui`).

## What landed

- `PictureView` commands (`crates/pictura-app/src/cxxqt_object.rs`), following
  the `apply_filter` pattern (validate → mutate doc → clear selection →
  `recomposite()`):
  - `resize_image(kind, w, h)` — kind `nearest`/`bilinear`/`bicubic` mapped to
    `pictura_ops::Resample` via `parse_resample`.
  - `resize_canvas(anchor, w, h)` — the nine anchor strings mapped to
    `pictura_ops::Anchor` via `parse_anchor` (`center-left`/`center-right`
    map to the `MiddleLeft`/`MiddleRight` variants).
  - `rotate_doc(quarter_turns)` — 1/2/3 → 90° CW / 180° / 90° CCW; anything
    else rejected.
  - `flip_doc(horizontal)` — always succeeds on a loaded document.
  - Every successful op clears the active selection (dimensions may change;
    matches Photoshop dropping the selection on Image/Canvas Size).
- `pictura-render` re-exports `pub use pictura_ops::{Anchor, Resample};` so
  callers of the document ops can name the parameter types without a direct
  `pictura-ops` dependency.
- Dock "Image" section (`crates/pictura-app/cpp/main.cpp`): Image Size
  (width/height spin boxes 1..=32767 seeded from the document, resample combo,
  Apply), Canvas Size (spin boxes + nine-entry anchor combo, Apply), and
  Rotate 90° CW / 90° CCW / 180° / Flip Horizontal / Flip Vertical buttons,
  all wired through the existing `refresh()` path.
- Headless `--self-test` doc-op block (exit codes 19/20/21): select all →
  rotate 90° CW and assert the exact pixel remap `(x,y) → (7-y,x)` on captured
  samples plus the cleared selection; assert `rotate_doc(0)`,
  `resize_image("bicubic", 0, 8)`, and `resize_canvas("nope", 10, 10)` return
  `false` bit-identically; assert CCW undoes CW bit-exactly; grow the canvas
  to 10×12 bottom-right and assert the `(x+2, y+4)` remap with the new
  top-left area transparent.
  The assertions are relative to pre-op captures (the earlier self-test
  stages leave a hidden layer, an active invert, and noise on the blue
  quadrant, so absolute color checks would be meaningless).

## Verification

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings` clean; `cargo test --workspace` 382 passed / 0 failed
  (3 new app tests: parser mapping ×2, engine wiring).
- `cmake --build build` green; `xvfb-run -a ./build/pictura --self-test
  crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with the new
  `rotate_cw` / `reject` / `canvas_grow` lines logged.
- `openspec validate --all --strict` green; `scripts/guard.sh` OK.

## Deferred

- Full Photoshop Image Size dialog (percent scaling, previews), relative
  canvas units, `rotate_arbitrary` at document scope — later app milestones.
- Undo/history remains the leading M14 candidate.
