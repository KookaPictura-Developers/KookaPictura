## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m33-composite-throughput.md` milestone brief (measured
  phase table, the five items, per-item byte-identity arguments, frozen
  interfaces, explicit non-goals)
- [x] 1.2 Write `proposal.md`, `design.md`, and `tasks.md`
- [x] 1.3 Freeze in `design.md`: the exact `build_source`/`build_mask` fast-path
  conditions and their per-pixel fallbacks, the fused readback, the
  command-buffer clear with its buffer usage, and the byte-identity arguments for
  each
- [x] 1.4 Update `docs/dev/canvas-compositing-plan.md` §3 M33 and `docs/dev/STATE.md`
  "Next" to the frozen scope (row-wise source/mask, fused readback, GPU-side
  clear, evidence; resident source buffers deferred)

## 2. Row-wise `build_source`

- [x] 2.1 Split `build_source` into a pure plane-assembly step and the upload
  step; keep the per-pixel body as the callable fallback
- [x] 2.2 Add the row-wise fast path: whole-row `copy_from_slice` per plane when
  `plane.get(src_start..src_start+cw)` holds for every row, with the RGB
  four-plane and grayscale two-plane layouts and the absent-alpha 255 fill
- [x] 2.3 Fall back to the per-pixel path when a present plane is short or
  channel 0 is absent; keep `pad_to_4`
- [x] 2.4 Unit test: row-wise plane bytes equal the per-pixel assembly for a
  covering RGB layer, a missing-alpha layer, a grayscale layer, and a short-plane
  layer that forces the fallback
- [x] 2.5 `cargo test -p pictura-render`; clippy clean

## 3. Row-wise `build_mask`

- [x] 3.1 Add the row-wise fill for the no-enabled-data-carrying-mask case (255
  inside the layer/group/adjustment influence rect, 0 outside)
- [x] 3.2 Keep the per-pixel `mask_alpha` path for an enabled mask that carries
  data; keep `pad_to_4`
- [x] 3.3 Unit test: the row-wise coverage equals the per-pixel coverage for a
  maskless layer and for a data-carrying mask
- [x] 3.4 `cargo test -p pictura-render`; clippy clean

## 4. Fused planar readback

- [x] 4.1 `read_canvas` de-interleaves the mapped staging slice into the planar
  `PixelBuffer` and returns it; drop the `mapped.to_vec()` packed intermediate
- [x] 4.2 `to_pixel_buffer` takes the mapped slice instead of the packed `Vec`;
  drop the range before `unmap`; `composite_gpu_region` returns the buffer
  directly
- [x] 4.3 `cargo test -p pictura-render`; clippy clean

## 5. GPU-side canvas initialization

- [x] 5.1 `zero_canvas` creates the canvas with `STORAGE | COPY_SRC | COPY_DST`
  usage and clears it with `encoder.clear_buffer(..)` instead of uploading a host
  zero `Vec`
- [x] 5.2 Confirm the group inner-canvas and main-canvas clears are submitted
  before the dispatches that read them
- [x] 5.3 `cargo test -p pictura-render`; clippy clean

## 6. Parity + profile evidence

- [x] 6.1 Confirm the `gpu_parity` contract still holds: region vs full slice
  0 LSB, repeated GPU composites identical, ±1 LSB against the CPU oracle, alpha
  unchanged
- [x] 6.2 Add an `#[ignore]` profile test that times `zero_canvas`,
  `build_source`, `build_mask`, `read_canvas`, and the full composite for a
  4000×4000 two-pixel-layer document and prints the phase table; skip with a
  printed note when no adapter is usable
- [x] 6.3 Run the profile test under `--release -- --ignored --nocapture` on a
  GPU host and record the before/after phase numbers
- [x] 6.4 `cargo test -p pictura-render`; clippy clean

## 7. Close-out

- [x] 7.1 `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D
  warnings`; `cargo test --workspace`
- [x] 7.2 `openspec validate m33-composite-throughput --strict`;
  `openspec validate --all --strict`
- [x] 7.3 `bash scripts/guard.sh` (docs carry the `TASK-ALLOWS-DOCS` marker)
- [x] 7.4 Update `docs/dev/STATE.md` with the M33 result
- [ ] 7.5 Archive the change (`openspec archive m33-composite-throughput`) and
  commit — deferred: runs after this close-out (the archive/commit step is
  intentionally left to the orchestrator, not part of the verification pass)
