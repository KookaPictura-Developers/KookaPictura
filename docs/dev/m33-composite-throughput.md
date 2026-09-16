# M33 — Full-composite throughput

Goal: remove the host-side per-pixel loops from the full GPU composite without
changing a byte of its output.

OpenSpec change `m33-composite-throughput` (MODIFIED `gpu-compositing`; no new
capability). The M31–M35 plan is `docs/dev/canvas-compositing-plan.md`; the
measurement that scoped this change is `docs/dev/m32-interactive-canvas.md`.

## Measured evidence

A full 4000×4000 GPU composite (`composite_gpu_region`, 2 RGB pixel layers,
RTX 3090) costs ~254 ms, split into:

| Phase | Time | Share | What it is |
|---|---:|---:|---|
| `build_source` | 146 ms | 58% | CPU planar source assembly + upload of both layer planes |
| `to_pixel_buffer` | 35 ms | 14% | CPU de-interleave of the readback into the planar `PixelBuffer` |
| `mapped.to_vec()` | 22 ms | 9% | 64 MB copy of the mapped readback |
| `zero_canvas` | 18 ms | 7% | host zero `Vec` + upload |
| `build_mask` | 16 ms | 6% | per-pixel `mask_alpha` coverage |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% | the actual 64 MB transfer |
| GPU compute dispatch (submit is async) | 0.2 ms | ~0% | |
| allocations | ~0.01 ms | ~0% | |

The composite is dominated by **host-side CPU per-pixel work**, not the GPU and
not the readback: assembly + de-interleave + copy is ~203 ms (80%), the 64 MB
readback is ~7 ms, and the GPU dispatch is ~0.2 ms. Every mutation that is not a
Move or a paint dab still runs this full composite, so M33 targets the
146 + 35 + 22 + 18 + 16 ms of CPU loops.

## Scope

1. **Row-wise source assembly.** `build_source` copies each source plane with
   whole-row copies (`copy_from_slice`/`clone_from_slice`) when the layer's
   channel data covers the clamped row intersection, instead of a per-pixel loop.
   Semantics are unchanged: planar straight-alpha planes; grayscale documents
   produce two planes with colour from channel 0; an absent green/blue channel
   aliases channel 0; an absent alpha plane reads 255; an absent colour channel
   reads 0. A short or absent plane falls back to the existing per-pixel path.
2. **Row-wise mask assembly.** `build_mask` fills coverage with row-wise
   operations when the layer has no enabled data-carrying mask (255 inside the
   layer/group/adjustment influence rect, 0 outside), and keeps the per-pixel
   `mask_alpha` path otherwise.
3. **Fused planar readback.** The readback de-interleaves the mapped staging
   bytes directly into the planar `PixelBuffer`; it no longer materializes a
   host-side packed RGBA `Vec`.
4. **GPU-side canvas initialization.** The canvas is cleared on the GPU with a
   command-buffer clear instead of allocating and uploading a full-canvas host
   zero buffer.
5. **Parity and throughput evidence.** The `gpu_parity` contract holds
   (0 LSB for the GPU against itself and against the region path, ±1 LSB against
   the CPU oracle, alpha unchanged) and an `#[ignore]` profile test prints the
   phase timings so the improvement is measurable.

## Design by item

### 1. Row-wise `build_source`

Current (`crates/pictura-render/src/gpu.rs:877`): for each layer, allocate
`planes * (cw*ch)` zeroed bytes and fill them pixel by pixel, indexing the
channel data with the layer-rect row stride `lw = layer.rect.width()` and
`li = (y - rect.top)*lw + (x - rect.left)`.

For one output row, the needed source bytes are a contiguous slice:

```
src_start = (y - rect.top) * lw + (x0 - rect.left)
plane[src_start .. src_start + cw]     // covers output row `row = y - y0`
```

so a whole row is one `copy_from_slice` into `data[p*n + row*cw ..]`. The fast
path applies when each required plane covers that slice for every row in
`y0..y1`:

- colour planes: `channel(0)`, and `channel(1).or(channel(0))`,
  `channel(2).or(channel(0))`;
- alpha plane: `channel(-1)` when present (absent → `fill(255)` rows, which is
  exactly the per-pixel `unwrap_or(255)`);
- grayscale documents copy only `channel(0)` into plane 0 and the alpha plane
  into plane 1 (planes == 2), matching the current branch.

If a present plane does not cover a row (a short plane), or channel 0 is absent
(all colour reads 0), the assembly falls back to the existing per-pixel loop
unchanged. The fast and fallback paths are both reachable, so a unit test can
assert their plane bytes are equal.

**Byte-identity.** For a covering plane, `copy_from_slice` writes
`plane[src_start + col]` at output index `row*cw + col`, which is exactly the
per-pixel loop's `sample(plane, li)` at the same `li`. The output row is
contiguous (`d = row*cw + col`), the default fills match `unwrap_or(255)` /
`unwrap_or(0)`, grayscale copies the same channel 0, and `pad_to_4` runs on the
same total length. Same bytes, 0 LSB.

### 2. Row-wise `build_mask`

Current (`gpu.rs:948`): allocate `n` zeroed bytes and, over the influence rect,
write `mask_alpha(layer, x, y)` per pixel. `mask_alpha` (`lib.rs:479`) returns
255 whenever the layer has no enabled data-carrying mask — `mask` is absent,
`mask.disabled`, or `mask.data` is absent. In those cases the coverage is a
constant 255 inside the influence rect (a pixel layer's clamped `rect`; a whole
region for a group or an adjustment layer) and 0 outside, which is a row fill
over the zeroed buffer. When the mask is enabled and carries data, the existing
per-pixel `mask_alpha` path is kept byte-for-byte.

**Byte-identity.** The loop writes 255 at exactly the pixels inside the
influence rect and touches nothing else, so the zero-initialized remainder stays
0. The row fill writes the same 255 over the same pixels. Identical.

### 3. Fused planar readback

Current (`gpu.rs:1122`): `read_canvas` copies the canvas to a `MAP_READ` staging
buffer, maps it, `mapped.to_vec()` (22 ms), and returns `Vec<u8>`;
`to_pixel_buffer` (`gpu.rs:1157`) then de-interleaves that packed `Vec` into the
planar `PixelBuffer` (35 ms).

M33 drops the `Vec`: `read_canvas` hands the mapped `&[u8]` straight to
`to_pixel_buffer` and returns the `PixelBuffer`, then drops the mapped range and
unmaps the staging buffer. `composite_gpu_region` returns that buffer directly.

**Byte-identity.** The de-interleave reads the same mapped bytes at the same
offsets; reconstructing a `u32` with `from_le_bytes` and masking is a pure byte
permutation, so writing `mapped[i*4 + k]` to `out.data[k*plane + i]` is the same
output. `to_vec()` copied bytes that were never modified, so removing it cannot
change a byte.

### 4. GPU-side canvas initialization

Current (`gpu.rs:853`): `zero_canvas` allocates `vec![0u8; n*4]` and uploads it
through `make_buffer` (18 ms). M33 creates the canvas with `create_buffer`
(usage `STORAGE | COPY_SRC | COPY_DST`) and issues
`encoder.clear_buffer(&canvas, 0, None)` in a submitted command encoder. The
canvas is used as the main running canvas and as each group's inner canvas, and
each clear is submitted before the dispatches that read that canvas, so the
queue order guarantees the shader never sees uninitialized memory.

**Byte-identity.** `clear_buffer` writes the same zeroes the uploaded `Vec`
contained. The only requirement is `COPY_DST` usage, which today's canvas already
carries via `make_buffer`.

## Frozen interfaces

- `build_source` and `build_mask` output plane bytes are unchanged (the byte
  identity above); the shader, `SrcLayout`, the bind-group layout, `mode_id`, the
  blend/adjustment math, and the packed group-inner-canvas path are untouched.
- The readback returns the same planar `PixelBuffer` bytes; only the host-side
  packed `Vec` disappears. `composite_gpu`, `composite_gpu_region`,
  `composite_region_active`, `composite_active`, `Backend`, and `GpuError` keep
  their signatures.
- `zero_canvas` returns a canvas whose bytes are unchanged (all zero).
- Oracles unchanged: the CPU compositor, `composite_active`, the region path, and
  the ±1 LSB GPU parity contract.
- No new dependency.

## Deferred / non-goals

- **Resident per-layer GPU source buffers** across a composite session — needs
  content versioning to know when a layer's bytes changed, so it is its own
  change.
- **Shader-side planar output** that removes the readback de-interleave entirely.
- Any change to the blend/adjustment math.
- **Zero-copy present** via Qt Quick — M34.
- **256² tiles + LRU + seam gutters + mipmaps + LoD** — M35.

## Verification

- `cargo test -p pictura-render`; the `gpu_parity` suite unchanged and green
  (region vs full slice 0 LSB, ±1 LSB against the CPU, alpha unchanged).
- Unit tests: row-wise `build_source` plane bytes equal the per-pixel assembly
  for a covering layer, a missing-alpha layer, a grayscale layer, and a
  short-plane layer that forces the fallback; row-wise `build_mask` equals the
  per-pixel coverage for a maskless layer and a data-carrying mask.
- `#[ignore]` profile test: `cargo test -p pictura-render --release -- --ignored
  --nocapture composite_phase_profile` prints the per-phase timings and the total
  for the 4000² two-pixel-layer composite; it skips with a printed note when no
  adapter is usable.
- `cargo fmt/clippy`; `openspec validate --all --strict`.
