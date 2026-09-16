## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m29-large-doc-performance.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze in `design.md`: 2-D compute dispatch replacing the 1-D workgroup-count rejection (compositor + filters, index `gid.x + gid.y * stride`); commit through the active backend via `translate_layer_active`; panel fixes (`sample_argb` cached read, downsample thumbnail, ≤512² histogram, in-place preview hide); unchanged CPU oracles and ±1 LSB parity; explicit non-goals (history copy-on-write, off-GUI-thread compute, zero-copy present)
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. 2-D compute dispatch (compositor + filters) + large-size parity

- [x] 2.1 `gpu.rs`: dispatch `dispatch_workgroups(gx, gy, 1)` with `gx = min(65535, count.div_ceil(64))`, `gy = count.div_ceil(gx * 64)`; pass `stride = gx * 64` in a trailing `Params` word
- [x] 2.2 `gpu.rs` blend shader: derive `i = gid.x + gid.y * params.stride`; keep the `i >= count` tail guard
- [x] 2.3 `gpu.rs` `Gpu::new`: replace the 1-D `n.div_ceil(64) > max_compute_workgroups_per_dimension` rejection with the 2-D product check (`n > limit * limit * 64`)
- [x] 2.4 `gpu_filter.rs`: same 2-D dispatch and `stride` field/check (`count > limit * limit * 64`); shader index becomes `wi = gid.x + gid.y * p.stride`
- [x] 2.5 Extend `crates/pictura-render/tests/gpu_parity.rs` with a 4000² compositor parity check (±1 LSB, alpha unchanged) and a large-size filter parity check, so 2-D dispatch is exercised past the old 4.19 MP ceiling
- [x] 2.6 `cargo test -p pictura-render`; clippy clean

## 3. App: active-backend translate

- [x] 3.1 Add `translate_layer_active(doc, dx, dy, gpu_enabled) -> bool` in `document_ops` — shift the topmost pixel layer rect (and mask) as `translate_layer` does, then refresh `doc.composite` with `composite_active`; leave the CPU `translate_layer` and `recompute` as the oracle
- [x] 3.2 `commit_move` calls `translate_layer_active` with `rust.gpu_compute`, then converts the already-current `doc.composite` via `buffer_to_image`
- [x] 3.3 Confirm the CPU path (`gpu_compute` off or no adapter) is byte-identical to the previous `translate_layer` result
- [x] 3.4 `cargo test --workspace`; clippy clean

## 4. Panel / thumbnail / histogram / preview fixes

- [x] 4.1 `sample_argb` reads the current cached image / `doc.composite` at `(x, y)` instead of `current_buffer` → `composite_active`; keep a recompute fallback when the cache is stale
- [x] 4.2 `layer_thumbnail(i, size)` downsamples the planar source directly without materializing a full-size RGBA `QImage`; visual result unchanged
- [x] 4.3 `HistogramPanel::recompute` bins a ≤512² downsample of the current image instead of `convertToFormat` + a full-resolution scan; output visually identical
- [x] 4.4 `begin_move_preview` hides the moved layer in place (set invisible, `document_to_image`, restore) instead of `doc.clone()`
- [x] 4.5 `cmake --build build`; panels and preview still correct

## 5. 4000² timing evidence + self-test

- [x] 5.1 Add a 4000² layer-move timing test: compare the CPU baseline against the active-backend commit; print both; assert the GPU path is materially faster when an adapter is present, skip with a printed note otherwise
- [x] 5.2 Confirm the compositor and filter CPU fallbacks remain byte-identical to `composite_rgba` / `pictura_filters::apply` (`gpuCompute` off or no adapter)
- [x] 5.3 Extend the app `--self-test` to report the active compositing backend and a large-document result; pass on default GPU or CPU fallback
- [x] 5.4 `cmake --build build`; the app still works without a GPU

## 6. Close-out

- [x] 6.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`
- [x] 6.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and the fixture variant)
- [x] 6.3 `openspec validate m29-large-doc-performance --strict`; `openspec validate --all --strict`
- [x] 6.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 6.5 Update `docs/dev/STATE.md` with the M29 result
- [x] 6.6 Archive the change (`openspec archive m29-large-doc-performance`) and commit
