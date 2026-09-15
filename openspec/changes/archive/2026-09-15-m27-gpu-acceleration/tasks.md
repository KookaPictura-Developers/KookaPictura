## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m27-gpu-acceleration.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze the GPU-native data path (planar 8-bit per-rect sources, `u8` mask, packed RGBA8 canvas/readback), the unchanged compositor contract, the filter API (`filter_gpu_available`, `apply_filter_active`, `apply_filter_backend`), the accelerated kernel set, and the CPU-fallback sentinel in `design.md`
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. GPU-native compositor data path + parity/timing

- [x] 2.1 Upload each pixel layer's raw planar 8-bit channels over its clamped rect (RGB or grayscale channel 0 replication + alpha); default a missing alpha plane to 255
- [x] 2.2 Upload mask coverage as a `u8` plane (rect-scoped for pixel layers, canvas-scoped for groups and adjustment layers)
- [x] 2.3 Keep the canvas as packed 8-bit RGBA in a storage buffer and pack the shader result on store; remove the `f32` canvas
- [x] 2.4 Read back packed 8-bit RGBA and de-interleave to the planar `PixelBuffer`; remove the `f32` readback and the host planar→RGBA source assembly
- [x] 2.5 Dispatch a group's already-8-bit inner canvas as its source (source-layout flag), with no `f32` re-materialization
- [x] 2.6 Keep `composite_gpu`, `composite_gpu_or_cpu`, `composite_active`, `gpu_available`, `Backend`, and `GpuError` public and behaviorally unchanged; keep the CPU compositor as the unchanged oracle
- [x] 2.7 Extend `crates/pictura-render/tests/gpu_parity.rs` to the GPU-native path: all modes and adjustment scenes stay within ±1 LSB and the output shape is identical
- [x] 2.8 Update the M26 timing test (or add an M27 timing test) covering 1024²×4 layers on the new path; print CPU vs GPU, assert no ratio
- [x] 2.9 `cargo test -p pictura-render`; clippy clean

## 3. GPU filter path + convolution family + parity

- [x] 3.1 Add `filter_gpu_available()` and `apply_filter_active(filter, buf, gpu_enabled) -> Backend` to `pictura-render::gpu`
- [x] 3.2 Add `GpuError::UnsupportedFilter` for a filter with no GPU kernel; `apply_filter_active` catches it and any `GpuError` by running the CPU kernel and returning `Backend::Cpu` (never an error, never a panic)
- [x] 3.3 Upload the layer's 8-bit samples as planar planes, run the pointwise WGSL kernel, and read back 8-bit; cache the pipeline and layout on the shared device
- [x] 3.4 Add the separable-convolution path (row stage + column stage) for the blur/sharpen/HighPass family with a cached separable pipeline
- [x] 3.5 Implement GPU kernels for the frozen accelerated set: `GaussianBlur`, `BoxBlur`, `MotionBlur`, `Blur`, `BlurMore`, `Sharpen`, `SharpenMore`, `UnsharpMask`, `HighPass`, `Solarize`, `Offset`
- [x] 3.6 Any other `Filter` returns `UnsupportedFilter` and falls back to `pictura_filters::apply` byte-for-byte
- [x] 3.7 Add `apply_filter_backend(layer, filter, mask, gpu_enabled) -> Result<Backend, FilterError>` in `pictura-render::filter`; keep `apply_filter`'s signature and CPU behavior (forwards with `gpu_enabled = false`) as the oracle
- [x] 3.8 Extend `crates/pictura-render/tests/gpu_parity.rs` with filter parity for every accelerated kernel (±1 LSB) and a non-accelerated filter fallback equal to the CPU oracle
- [x] 3.9 `cargo test -p pictura-render`; clippy clean

## 4. App filter routing + self-test

- [x] 4.1 Route `PictureView::apply_filter` through `apply_filter_backend` with `rust.gpu_compute`; keep record + recomposite behavior
- [x] 4.2 Confirm `gpuCompute` off or no adapter gives a filter buffer byte-identical to the CPU oracle
- [x] 4.3 Extend the app `--self-test` to report the filter backend and pass on default GPU or CPU fallback
- [x] 4.4 Build green (`cmake --build build`); filtering a document still works without a GPU

## 5. Close-out

- [x] 5.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`
- [x] 5.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and the fixture variant)
- [x] 5.3 `openspec validate m27-gpu-acceleration --strict`; `openspec validate --all --strict`
- [x] 5.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 5.5 Update `docs/dev/STATE.md` with the M27 result
- [x] 5.6 Archive the change (`openspec archive m27-gpu-acceleration`) and commit
