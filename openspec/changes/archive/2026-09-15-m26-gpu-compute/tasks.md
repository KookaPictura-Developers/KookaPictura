## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m26-gpu-compute.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze the backend API (`Backend`, `gpu_available`, `composite_active`), the supported mode/adjustment set, the Dissolve exception, and the `gpuCompute` session schema in `design.md`
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. GPU context + backend selection + probe

- [x] 2.1 Add `Backend { Gpu, Cpu }`, `gpu_available()`, and `composite_active(doc, gpu_enabled) -> (PixelBuffer, Backend)` to `pictura-render::gpu`
- [x] 2.2 Keep `composite_gpu`, `composite_gpu_or_cpu`, and `GpuError` public and unchanged for compatibility
- [x] 2.3 Confirm the instance/adapter/device/queue are created once behind the existing `OnceLock`
- [x] 2.4 `gpu_available()` returns the cached capability probe (adapter + device + required limits)
- [x] 2.5 `composite_active` never panics and falls back to CPU for that call on any `GpuError`
- [x] 2.6 Cache the bind-group layout and compute pipeline instead of rebuilding them per `Gpu::new`
- [x] 2.7 `cargo test -p pictura-render`; clippy clean

## 3. Compositor completeness + performance

- [x] 3.1 Implement Hue, Saturation, Color, Luminosity in WGSL (PDF/CSS non-separable triplet math) and extend `mode_id`/`blend`
- [x] 3.2 Implement the five adjustment layers on the GPU — invert, posterize, threshold, brightness/contrast, hue/saturation — at the layer's stack position with mask/opacity/blend
- [x] 3.3 Any other adjustment kind returns `GpuError::UnsupportedAdjustment`; a visible Dissolve layer returns `GpuError::UnsupportedMode(BlendMode::Dissolve)` with the whole composite falling back to CPU
- [x] 3.4 Upload only each layer's rect (uniform origin/size, rect-local source reads, early-out outside the rect); reuse buffers where practical
- [x] 3.5 Keep the CPU compositor (`composite_rgba`) as the unchanged oracle
- [x] 3.6 `cargo test -p pictura-render`; clippy clean

## 4. App wiring (default GPU, toggle, status, session)

- [x] 4.1 Route `current_buffer` / `document_to_image` (`cxxqt_object.rs`) through `composite_active`
- [x] 4.2 Expose backend availability and the active-backend result to the C++ shell
- [x] 4.3 Add `bool gpuCompute = true` to `SessionState`, bump `schemaVersion` to 2, and read a missing field as `true`
- [x] 4.4 Persist `gpuCompute` in `saveSession` / `loadSession` (`cpp/session.{h,cpp}`)
- [x] 4.5 Add the checkable `view.gpuCompute` command ("Use GPU Compute"), disabled/greyed when no adapter is available; toggling re-composites and repaints
- [x] 4.6 Status bar shows `GPU` / `CPU`, and `CPU (no GPU)` when the probe fails
- [x] 4.7 Build green (`cmake --build build`); a document still opens without a GPU

## 5. Parity tests + self-test

- [x] 5.1 Extend `crates/pictura-render/tests/gpu_parity.rs` to all 27 modes including the four non-separable, ±1 LSB
- [x] 5.2 Add parity scenes for the five adjustment layers and mixed stacks, ±1 LSB
- [x] 5.3 Assert Dissolve and unsupported adjustment kinds fall back to a CPU-equivalent buffer
- [x] 5.4 Assert `composite_active` returns `Backend::Cpu` when `gpu_enabled = false`, and never returns `Backend::Gpu` when `gpu_available()` is false
- [x] 5.5 Extend the app `--self-test` to report the active backend and pass on default GPU or CPU fallback
- [x] 5.6 `cargo test --workspace`; fixture and no-argument self-tests exit 0

## 6. Close-out

- [x] 6.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`
- [x] 6.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and the fixture variant)
- [x] 6.3 `openspec validate m26-gpu-compute --strict`; `openspec validate --all --strict`
- [x] 6.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 6.5 Update `docs/dev/STATE.md` with the M26 result
- [x] 6.6 Archive the change (`openspec archive m26-gpu-compute`) and commit
