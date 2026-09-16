## Why

At 4000×4000 a full GPU composite (`composite_gpu_region`, 2 RGB pixel layers,
RTX 3090) costs ~254 ms and is dominated by host-side CPU work, not the GPU and
not the readback: `build_source` 146 ms (58%), `to_pixel_buffer` 35 ms,
`mapped.to_vec()` 22 ms, `zero_canvas` 18 ms, `build_mask` 16 ms, readback
submit/poll/map 7 ms, GPU dispatch 0.2 ms. M31 and M32 took the Move and paint
paths off the full composite; every other mutation still runs it, so ~80% of the
composite is CPU loops doing work a bulk copy or a memset expresses directly.

M33 removes those loops without changing a single output byte, so the full
composite gets materially faster while the CPU oracle and the ±1 LSB parity
contract stay exactly where they are.

## What Changes

- **Row-wise source assembly.** `Gpu::build_source` assembles each layer's
  planar source planes with whole-row copies when the layer's channel data covers
  the clamped row intersection, instead of a per-pixel loop. Semantics are
  preserved: planar straight-alpha planes, two planes for a grayscale document
  (colour from channel 0), an absent green/blue channel aliases channel 0, an
  absent alpha plane reads 255, an absent colour channel reads 0. A short or
  absent plane falls back to the existing per-pixel path. Output is byte-identical
  to the current implementation.
- **Row-wise mask assembly.** `Gpu::build_mask` fills coverage with row-wise
  operations when the layer has no enabled data-carrying mask (255 inside the
  layer/group/adjustment influence rect, 0 outside), and keeps the per-pixel
  `mask_alpha` path otherwise. Output is byte-identical.
- **Fused planar readback.** The readback de-interleaves the mapped staging bytes
  directly into the planar `PixelBuffer` and no longer materializes a host-side
  packed RGBA `Vec`. Output is byte-identical.
- **GPU-side canvas initialization.** The canvas is cleared with a command-buffer
  clear instead of allocating and uploading a full-canvas host zero buffer. Output
  is byte-identical.
- **Parity and throughput evidence.** The existing `gpu_parity` contract holds
  (0 LSB for the GPU against itself and the region path, ±1 LSB against the CPU
  oracle, alpha unchanged), and an `#[ignore]` profile test prints the phase
  timings so the improvement is measurable.

## Capabilities

### New Capabilities

None. M33 tightens existing `gpu-compositing` requirements.

### Modified Capabilities

- `gpu-compositing`: the planar source and mask assembly, the readback
  de-interleave, and the canvas initialization become bulk or GPU operations with
  byte-identical output, and a full-composite throughput-evidence requirement is
  added.

## Impact

- `crates/pictura-render/src/gpu.rs` — `build_source` and `build_mask` gain the
  row-wise fast paths with their existing per-pixel fallbacks; `zero_canvas`
  becomes a `clear_buffer` command; `read_canvas` de-interleaves from the mapped
  slice and returns the `PixelBuffer` directly, dropping `to_pixel_buffer`'s
  packed-`Vec` input; `composite_gpu_region` returns that buffer. The shader,
  `SrcLayout`, the bind-group layout, the blend/adjustment math, and the packed
  group-inner-canvas path are unchanged.
- `crates/pictura-render/tests/gpu_parity.rs` — the parity contract stays; a new
  `#[ignore]` profile test (in the crate's test module, where the private phase
  methods are reachable) prints the phase timings.
- No new dependency. The CPU compositor, `composite_active`, and
  `composite_region_active` remain the oracles and do not change.
