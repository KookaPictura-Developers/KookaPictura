# Tasks

## 1. The GPU stroke buffer

- [x] 1.1 Add a GPU stroke context in `crates/pictura-render/src/gpu/` that owns
  the target layer's resident source buffer for a stroke (upload at
  `GpuStroke::new`, drop with the object). **Done.** `gpu/stroke.rs::GpuStroke`
  seeds a resident `layer` and an immutable `base` copy (RGBA8, one word per
  pixel) from the target layer and keeps them for the object's lifetime — the
  stroke is the epoch, so a new stroke is a new `GpuStroke`.
- [x] 1.2 Add the tip computation and the dab dispatch that applies the
  accumulated coverage to the resident buffer. **Done.** One dispatch per dab
  over its bounding box; `gpu_stroke_matches_the_cpu_oracle` drives the same dab
  placer as the app over overlapping dabs and proves ≤1 LSB. The shader
  recomposites every changed pixel from the immutable base and keeps coverage
  one word per pixel, so overlapping dabs neither compound nor race. Batching a
  sample's dabs is a later task.

## 2. The app stroke path

- [x] 2.1 Drive the GPU stroke from the app. **Done.** There is no
  `pictura-paint` backend seam: the exact CPU `Stroke` is still created at
  `begin_paint`, the app drives a parallel `GpuStroke`, and `pictura-paint`
  gained `Stroke::tip_params()`, `Stroke::base_layer_rgba_doc` and
  `Stroke::patch_working_layer` so the GPU seed and each dab's write-back happen
  on the exact working document without re-decoding the layer in the app.
- [x] 2.2 Select the GPU only when available and supported, falling back
  otherwise. **Done.** `impl_paint.rs::begin_paint` takes the GPU path when
  `rust.gpu_compute` is on, the mode is Normal or Clear, `auto_erase` is off,
  the target layer is not transparency-locked, the dab is over the 262 144 px
  raster budget, and `GpuStroke::new` succeeds; otherwise it takes the LOD
  preview (over budget) or the exact CPU path, unchanged.
- [x] 2.3 Write back only the changed region and present exact. **Done.**
  `GpuStroke::dab` returns only its bounding box; the app patches that rectangle
  into the working document (`Stroke::patch_working_layer`, layer-local and
  clipped) and presents frame-bounded through `refresh_region`, which composites
  from the working document. No per-dab level-0 patch or deferred pyramid.

## 3. GPU-authoritative commit

- [x] 3.1 Delete the release-time replay. **Done.** `end_paint` commits
  `stroke.finish()` directly; the GPU already patched the working document, so
  there is no `commit_replay_cpu_stroke` on a GPU stroke. The preview path keeps
  its exact replay.
- [x] 3.2 Prove commit parity within ±1 LSB. **Done.** Self-test **546**
  `pp_gpu_commit` renders the same stroke with the GPU on and off and asserts
  every channel differs by ≤1 LSB while the GPU path ran; skips without an
  adapter. Self-test **547** uses the same tolerance.

## 4. Parity and verification

- [x] 4.1 Stroke-parity against the CPU oracle within ±1 LSB. **Done.**
  `gpu::stroke::tests::gpu_stroke_matches_the_cpu_oracle`, `#[ignore]`d per the
  GPU-test policy (`cargo test -p pictura-render --lib gpu_stroke_matches --
  --ignored`); it drives the shared dab placer over overlapping dabs and
  self-skips without an adapter.
- [x] 4.2 Run the full verification gate. **Done.** `cargo fmt --all`; `cargo
  clippy --workspace --all-targets -- -D warnings`; `cargo nextest run
  --workspace`; `cargo test --workspace --doc`; `TASK_ALLOWS_DOCS=1 bash
  scripts/verify-fast.sh`; the CMake build plus both self-tests;
  `openspec validate --all --strict`.

## 5. Documentation

- [x] 5.1 Record the GPU stroke path, the selection gate, the resident buffers
  and the ±1 LSB commit contract in `docs/dev/STATE.md`,
  `docs/dev/canvas-compositing-plan.md` (M37) and `docs/dev/canvas-view-spec.md`.

## 6. Live-drag cost: planar write-back and the seed

- [x] 6.1 Add sub-phase timers to the paint report: the begin split
  (`gpu_seed_interleave`, `gpu_new (alloc+upload)`) and the per-dab split
  (`gpu_dab_dispatch_readback`, `gpu_patch_working_layer`), measured with
  `PICTURA_PAINT_TIMING=1` on the 4000² self-test.
- [x] 6.2 Read each dab's changed region back already planar (region-addressed
  `PLANAR_SHADER`) and patch it row-wise (`Stroke::patch_working_layer` takes
  four planes plus a plane stride), removing the per-pixel host scatter and the
  intermediate interleaved buffer.
- [x] 6.3 Build the seed without a per-pixel document pass: a document-spanning
  layer interleaves straight across its channel planes, and the seed is written
  into the resident layer's mapped memory instead of through `write_buffer`.
- [x] 6.4 Keep the parity contract: `gpu_stroke_matches_the_cpu_oracle` still
  proves ≤1 LSB and now also proves the planar readback is byte-identical to the
  resident layer; the paint unit test covers the plane patch's rectangle mapping.
- [x] 6.5 Re-run the full verification gate and record the before/after phase
  times.
