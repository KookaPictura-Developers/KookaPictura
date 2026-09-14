## 1. M6C-A — Render filter application

- [x] 1.1 Add `pictura-filters = { path = "../pictura-filters" }` to `crates/pictura-render/Cargo.toml`
- [x] 1.2 Implement `pub fn apply_filter(layer: &mut Layer, filter: &Filter, mask: Option<&LayerMask>) -> Result<(), FilterError>` in `pictura-render`, returning `Ok(())` for a degenerate `rect`
- [x] 1.3 Extract channels `0,1,2` into a planar `lw × lh × 3` `PixelBuffer`, returning `FilterError::InvalidParams` for a missing or wrong-length channel
- [x] 1.4 Apply the filter to the extracted buffer and propagate any `FilterError` without writing the layer
- [x] 1.5 Gate the write back per local pixel using the mask coverage (or `mask.default_color` outside the mask rect, or 255 when `None`) with `round(orig + (filtered - orig) * coverage / 255)`
- [x] 1.6 Leave channel `-1`, `rect`, `mask`, `opacity`, and `blend` untouched
- [x] 1.7 Unit-test full-frame apply, zero/partial/255 coverage, default-color outside the mask rect, missing/wrong-length channels, filter-error propagation, alpha preservation, metadata preservation, and a degenerate-rect no-op

## 2. M6C-B — App command, dock, and self-test

- [x] 2.1 Add the `PictureView::apply_filter(kind: &QString) -> bool` qinvokable and declare it in the cxx-qt bridge
- [x] 2.2 Map each kind to its `Filter` defaults (gaussian-blur, box-blur, motion-blur, median, despeckle, sharpen, sharpen-more, unsharp-mask, add-noise), returning `None` for an unknown kind
- [x] 2.3 Find the topmost pixel layer (last bottom-first `doc.layers` with `adjustment.is_none() && !is_group`) and return false for no document or no pixel layer
- [x] 2.4 Build the mask from the active selection via `selection_to_mask` (`None` when no selection), call `apply_filter`, then recomposite and emit `changed`
- [x] 2.5 Add a filter combo box and an "Apply Filter" button to the dock, wired to the command
- [x] 2.6 Add the `--self-test` filter-confinement check: select a subset, apply a filter, assert pixels outside are unchanged and exit non-zero on failure, then clean up

## 3. M6C-C — Verification and proposal

- [x] 3.1 Run `cargo test --workspace` green with the render `apply_filter` unit tests and the app self-test
- [x] 3.2 Run the headless self-test against a layered PSD and confirm the confined change
- [x] 3.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 3.4 Run `scripts/guard.sh` green
- [x] 3.5 Run `openspec validate m6-filter-integration --strict` and `openspec validate --all --strict` green
