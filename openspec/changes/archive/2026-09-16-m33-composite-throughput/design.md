## Context

M27 made the compositor data path 8-bit end to end: layer sources upload as raw
planar 8-bit samples over the layer rect, the mask uploads as an 8-bit plane, the
canvas is packed 8-bit RGBA, and the output is read back as packed 8-bit. M31
added `composite_region_active`; M32 moved the Move-preview base and the
visibility toggle onto it and cached the present scale. Every other mutation
still runs the full composite.

The measured full 4000×4000 composite (`composite_gpu_region`, 2 RGB pixel
layers, RTX 3090, ~254 ms) is host-side:

| Phase | Time | Share | What it is |
|---|---:|---:|---|
| `build_source` | 146 ms | 58% | CPU planar source assembly + upload of both layer planes |
| `to_pixel_buffer` | 35 ms | 14% | CPU de-interleave of the readback into the planar `PixelBuffer` |
| `mapped.to_vec()` | 22 ms | 9% | 64 MB copy of the mapped readback |
| `zero_canvas` | 18 ms | 7% | host zero `Vec` + upload |
| `build_mask` | 16 ms | 6% | per-pixel `mask_alpha` coverage |
| readback submit + `poll(wait)` + `map` | 7 ms | 3% | the 64 MB transfer |
| GPU compute dispatch (submit is async) | 0.2 ms | ~0% | |
| allocations | ~0.01 ms | ~0% | |

Constraints: the CPU compositor (`composite_rgba`) is the frozen oracle, the GPU
path is ±1 LSB against it, `composite_active`/`composite_region_active` are the
full/region oracles, wgpu 30.0.1 on Vulkan is the only backend, no new
dependency, and `docs/` is the long-form contract.

Relevant code: `build_source` (`gpu.rs:877`), `build_mask` (`gpu.rs:948`),
`zero_canvas` (`gpu.rs:853`), `make_buffer` (`gpu.rs:861`), `read_canvas`
(`gpu.rs:1122`), `to_pixel_buffer` (`gpu.rs:1157`), the pass-through/group/
adjustment dispatch in `composite_layer` (`gpu.rs:976`), and the helpers
`channel`/`sample`/`mask_alpha` (`lib.rs:466`–`501`).

## Goals / Non-Goals

**Goals:**

- Replace the per-pixel source and mask assembly with whole-row copies where the
  channel data covers the intersection, and with a row fill where the coverage is
  a constant, leaving the existing per-pixel code as the fallback.
- De-interleave the readback directly from the mapped staging slice into the
  planar `PixelBuffer`, with no host-side packed RGBA `Vec`.
- Clear the canvas on the GPU with a command-buffer clear instead of uploading a
  host zero buffer.
- Keep every output byte identical, the CPU oracle unchanged, and the ±1 LSB
  parity contract unchanged.
- Ship an `#[ignore]` profile test that prints the phase timings.

**Non-Goals:**

- Resident per-layer GPU source buffers across a composite session — needs
  content versioning to know when a layer's bytes changed.
- Shader-side planar output that removes the readback de-interleave entirely.
- Any change to the blend/adjustment math, the shader, `SrcLayout`, or the packed
  group-inner-canvas path.
- Zero-copy present (M34); 256² tiles + LoD (M35); off-GUI-thread compute.

## Decisions

### 1. Row-wise assembly with the per-pixel path retained (frozen)

`build_source` and `build_mask` are split into a pure assembly step that returns
the plane bytes and the upload step that wraps them in a `wgpu::Buffer`. The pure
step first tries a row-wise path and otherwise calls the existing per-pixel body.
Both paths can be called directly, so a unit test asserts byte equality between
them.

**`build_source` fast-path condition.** For the clamped intersection
`[x0, x1) × [y0, y1)` and each output row `row = y - y0`:

```
lw        = layer.rect.width()                    // channel row stride (as today)
src_start = (y - layer.rect.top) * lw + (x0 - layer.rect.left)
```

the fast path applies when each required plane satisfies
`plane.get(src_start .. src_start + cw).is_some()` for every row in `y0..y1`,
where the planes are:

- `channel(0)` (all colour planes derive from it);
- `channel(1).or(channel(0))` and `channel(2).or(channel(0))` for RGB documents;
- `channel(-1)` for alpha when present (an absent alpha plane is filled 255).

For each output row, `copy_from_slice` moves `plane[src_start..src_start+cw]` to
`data[p*n + row*cw ..]`. Grayscale documents (`ColorMode::Grayscale | Bitmap |
Duotone`) copy only channel 0 into plane 0 and the alpha plane into plane 1, so
`planes == 2`. If a present plane is short on any row, or channel 0 is absent (all
colour reads 0, all defaults), the whole layer takes the per-pixel path. `pad_to_4`
still runs after assembly.

- *Why:* the row slice is exactly the sequence `sample(plane, li)` the per-pixel
  loop indexes, so a whole-row copy is a mechanical rewrite, not a semantic one.
- *Alternatives:* process the whole plane with one `copy_from_slice` when the
  region contains the layer rect (rejected — the intersection is usually the
  layer rect already, but the row form also covers partial intersections without
  a second case); assembly on the GPU (rejected — the source plane is a host
  buffer upload, and moving it to a shader does not remove the upload).

**Byte-identity.** Output index `row*cw + col` receives `plane[src_start+col]`,
identical to the per-pixel `sample(plane, src_start+col)`; the RGB plane offsets
`0, n, 2n, 3n` and the grayscale two-plane layout are unchanged; the default fills
match `unwrap_or(255)`/`unwrap_or(0)`; `pad_to_4` sees the same length. 0 LSB.

**`build_mask` fast-path condition.** `mask_alpha` (`lib.rs:479`) returns 255 for
every pixel exactly when the layer has no enabled data-carrying mask: `mask` is
absent, `mask.disabled`, or `mask.data` is absent. In that case the influence
rectangle is a pixel layer's clamped `rect`, or the whole region for a group or an
adjustment layer (`gpu.rs:953`), and the fast path fills 255 over each row of that
rectangle (`data[row + (x0 - self.x0) .. row + (x1 - self.x0)].fill(255)`). The
zero-initialized buffer keeps 0 outside. Any other mask takes the per-pixel
`mask_alpha` path unchanged.

- *Why:* for the maskless layer (the common case) the coverage is a constant and
  the per-pixel `mask_alpha` call is overhead; the identity argument is immediate
  because the loop writes exactly 255 inside the rectangle and nothing outside.
- *Ceiling:* only the constant-255 case is fast; a data-carrying mask still walks
  per pixel (ponytail: constant-coverage fill, tighten to a row-wise masked copy
  if a masked-layer full-composite profile demands it).

### 2. Fused planar readback (frozen)

`read_canvas` currently maps a `MAP_READ` staging buffer and calls
`mapped.to_vec()`, returning `Vec<u8>`; `to_pixel_buffer` then de-interleaves that
packed `Vec`. M33 maps the staging buffer, passes `&mapped` straight to
`to_pixel_buffer`, drops the mapped range, unmaps, and returns the `PixelBuffer`.
`composite_gpu_region` returns it directly. `to_pixel_buffer` keeps its
de-interleave body but its input is the mapped slice, not a copied `Vec`.

- *Why:* the `to_vec()` copy moves 64 MB that is immediately read and discarded;
  the de-interleave already reads packed bytes at known offsets.
- *Byte-identity:* `u32::from_le_bytes([b0, b1, b2, b3])` masked into four channels
  is a pure permutation of those four bytes, so writing `mapped[i*4 + k]` to
  `out.data[k*plane + i]` produces the same `PixelBuffer`. Removing a copy of
  unmodified bytes cannot change a byte.
- *Ordering:* the mapped range must be dropped before `staging.unmap()`; the fused
  body preserves that order.

### 3. GPU-side canvas clear (frozen)

`zero_canvas` creates the canvas with `create_buffer` (usage
`STORAGE | COPY_SRC | COPY_DST`) and submits a command encoder whose only command
is `encoder.clear_buffer(&canvas, 0, None)`, instead of building
`vec![0u8; n*4]` and uploading it through `make_buffer`.

- *Why:* an 18 ms host memset + 64 MB upload becomes a GPU transfer command.
- *Byte-identity:* `clear_buffer` writes zeroes, the same bytes the uploaded
  buffer held. The canvas already carries `COPY_DST` today, so the usage change is
  only that `make_buffer` no longer adds it implicitly.
- *Ordering:* each clear is submitted to the queue before the dispatches that read
  that canvas (the main canvas, and each group's inner canvas created by
  `composite_layer`), and queue commands execute in submission order, so no
  dispatch observes uninitialized memory.

### 4. Oracles, evidence, and checks (frozen)

The CPU compositor, `composite_active`, `composite_region_active`, and the ±1 LSB
GPU parity contract are unchanged. Byte-identity for the assembly items is proven
by calling the row-wise path and the retained per-pixel path on the same layer and
asserting equal plane bytes; the readback and clear identity follow from the
arguments above and are covered by the existing parity suite. Evidence is an
`#[ignore]` profile test that builds the 4000² two-pixel-layer scene, times
`zero_canvas`, `build_source`, `build_mask`, `read_canvas`, and the full
composite, and prints the table; it skips with a printed note when no adapter is
usable. No ratio is asserted (timings vary by machine), consistent with M29's
loose evidence style.

### 5. Process (frozen)

Waves: (1) brief and frozen interfaces; (2) row-wise `build_source` and
`build_mask` with the plane-equality tests; (3) fused readback and GPU-side clear;
(4) parity plus the `#[ignore]` phase profile; (5) close-out.

## Risks / Trade-offs

- **Row-length / stride mismatch.** The fast path assumes the channel row stride
  is `layer.rect.width()`, which is exactly the assumption the per-pixel path
  makes; `plane.get(src_start..src_start+cw)` guards every row, so a channel whose
  data does not cover the intersection falls back instead of panicking or reading
  a wrong row.
- **`pad_to_4`.** Both paths assemble `planes * n` bytes and pad afterward; the
  row writes never run past `planes * n`, so the padded tail is unchanged.
- **Grayscale 2-plane layout.** The shader flag `(flags & 2) != 0` and the
  two-plane offsets are host-independent; the fast path only changes how plane 0
  and the alpha plane are filled.
- **Group inner canvases bind packed RGBA directly** and never call `build_source`,
  so no group path is affected; `build_mask` still runs for a group, where the
  influence rectangle is the whole region.
- **`clear_buffer` usage requirements.** The canvas must carry `COPY_DST` (it
  already does), the offset must be 4-aligned (it is 0), and the size must be a
  multiple of 4 (`n*4` is); `n > 0` is guaranteed by `Gpu::new`'s size checks, so
  no zero-size clear occurs.
- **Clear/dispatch ordering.** The clear runs in its own submitted encoder; a
  future refactor that defers or reorders queue submissions could break it, so the
  profile/parity tests composite a group (which creates an inner canvas) as well as
  a flat stack.
- **Byte-identity to the pre-change GPU is not stored as a baseline.** The change
  is argued structurally and checked by the retained per-pixel reference and the
  frozen CPU oracle; where the GPU was previously ±1 LSB (hue/saturation
  adjustment), the suite's existing tolerance is unchanged rather than tightened.

## Migration Plan

Additive. The fast paths are internal to two assembly functions; the readback and
clear keep their call signatures except `read_canvas`'s return type and
`to_pixel_buffer`'s input, both crate-private. The CPU oracle, the parity
contract, the shader, and all public behaviour are unchanged. Rollback: restore
the per-pixel-only assembly, the packed `Vec` readback, and the uploaded zero
canvas; no public signature changes to revert.

## Open Questions

- Whether `composite_gpu_region` should return `read_canvas`'s `PixelBuffer`
  directly (planned) or keep a named intermediate — pick the smaller diff.
- Whether the phase profile belongs in `gpu.rs`'s test module (private methods
  reachable) or needs a `#[doc(hidden)]` timing hook for an integration test —
  prefer the in-crate test to avoid new API surface.
