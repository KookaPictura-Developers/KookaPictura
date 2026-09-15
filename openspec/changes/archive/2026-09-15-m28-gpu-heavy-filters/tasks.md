## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m28-gpu-heavy-filters.md` milestone brief
- [x] 1.2 Write `proposal.md`, `tasks.md`, and `design.md`
- [x] 1.3 Freeze in `design.md`: extend the existing `gpu_filter.rs` mechanism (no new pipeline), the unchanged `filter_gpu_available` / `apply_filter_active` API and default-on routing, the profile-driven selection rule (≥ ~150 ms at 1 MP and ±1 LSB parity), the new kernel set (Surface Blur, Median, Maximum, Minimum, Custom 5×5, Oil Paint), the byte-identical CPU fallback, and the explicit exclusion of the stochastic families and render filters
- [x] 1.4 Commit brief + OpenSpec artifacts with a `TASK-ALLOWS-DOCS` message

## 2. Window kernels (Surface Blur, Maximum, Minimum, Median) + parity

- [x] 2.1 Add a `SURFACE` mode (bilateral: spatial Gaussian × luma-range Gaussian, `f32` accumulation, `round_away` quantize) mirroring `pictura_filters::blur::surface`; validate `radius 1..=100`, `threshold 1..=255`
- [x] 2.2 Add max/min op selection to the separable path (or an equivalent window mode) mirroring `pictura_filters::other::morphology`; validate `radius` (clamp 100)
- [x] 2.3 Add an exact order-statistic kernel for Median (workgroup-staged `(2r+1)²` window, 256-bin histogram walk to `floor(k²/2)`) mirroring `pictura_filters::noise::median`; `radius == 0` is a no-op
- [x] 2.4 Extend `plan()` so an unsupported parameter or an unparity-capable kernel yields `None` and falls back to the CPU oracle byte-for-byte
- [x] 2.5 Extend `crates/pictura-render/tests/gpu_parity.rs` with ±1 LSB parity for Surface Blur, Maximum, Minimum, and Median (gradient, flat, and single-colour inputs; alpha unchanged on 4-channel buffers)
- [x] 2.6 `cargo test -p pictura-render`; clippy clean

## 3. Neighbourhood / effect kernels (Custom, Oil Paint) + parity

- [x] 3.1 Add Custom 5×5 to the `KERNEL` mode (`ksize = 5`, `support = 2`, `norm = scale`, plus the `offset` addend) mirroring `pictura_filters::other::custom`; validate finite `scale != 0`, finite `offset`, finite kernel entries
- [x] 3.2 Add an Oil Paint kernel mirroring the M25 `oil_paint` model in `f32` (Sobel tangent basis, edge-stopping directional aggregation, height field, Lambert/Blinn-Phong relief) with `round_away`/`clamp_u8` equivalents and alpha untouched; validate `stylization`/`cleanliness`/`scale`/`bristle_detail`/`shine` in `0..=10`, `angular_direction` in `0..=360`
- [x] 3.3 Admit Oil Paint only if GPU output matches the CPU oracle within ±1 LSB; otherwise leave it on the CPU and report
- [x] 3.4 Extend `crates/pictura-render/tests/gpu_parity.rs` with ±1 LSB parity for Custom and Oil Paint, and confirm a kernel-less filter still falls back byte-identically
- [x] 3.5 `cargo test -p pictura-render`; clippy clean

## 4. Timing evidence + self-test

- [x] 4.1 Add a Surface Blur timing test (1024×1024, CPU vs GPU, print both, assert ≥ ~10× when an adapter is present, skip otherwise)
- [x] 4.2 Confirm the CPU fallback is byte-identical to `pictura_filters::apply` for every new kernel (`gpuCompute` off or no adapter)
- [x] 4.3 Extend the app `--self-test` to report the extended filter backend and pass on default GPU or CPU fallback
- [x] 4.4 `cmake --build build`; filtering still works without a GPU

## 5. Close-out

- [x] 5.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`
- [x] 5.2 `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` (and the fixture variant)
- [x] 5.3 `openspec validate m28-gpu-heavy-filters --strict`; `openspec validate --all --strict`
- [x] 5.4 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 5.5 Update `docs/dev/STATE.md` with the M28 result
- [x] 5.6 Archive the change (`openspec archive m28-gpu-heavy-filters`) and commit
