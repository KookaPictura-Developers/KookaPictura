# Design

## Context

See proposal.md — Why. The CPU stroke path and the region compositor are the
reference; the GPU compositor (`crates/pictura-render/src/gpu/`) already caches
its device and pipelines in `Devices` and composites a region within ±1 LSB of
the CPU oracle. What is missing is a stroke that produces its committed pixels
on the GPU instead of one CPU core, without replaying the stroke at release.

## Goals / Non-Goals

**Goals:**

- A large brush's per-dab cost stops scaling with the brush area on the CPU: the
  dab is stamped on the GPU and only its changed region is read back.
- The committed pixels equal the GPU stroke and lie within the documented ±1 LSB
  of the CPU oracle, so the CPU stroke engine stays the CI/golden verifier.
- The GPU path is selected only when an adapter exists and the stack is
  supported; otherwise the current CPU path (and the LOD preview) is unchanged.

**Non-Goals:**

- GPU-resident undo, tiling, or a GPU document model (M38).
- A GPU path for Dissolve, unsupported adjustments, layer effects, smart
  objects, text, Blend If, or transparency-locked layers; those stay CPU.
- Byte-identical commit parity; the contract is the documented ±1 LSB.

## Decisions

**D1 — The target layer stays resident on the GPU for the stroke.** At
`begin_paint` the target layer's pixels are decoded into a document-sized
interleaved straight-RGBA8 buffer at the layer's document rect (zero outside)
and uploaded once; the `GpuStroke` object *is* the epoch — a new stroke is a new
object and its buffers are dropped with it. Seeding from the target layer (not
the composited frame) is what lets each dab edit the layer itself, so the working
document — and therefore the commit — is authored on the GPU. `Stroke` exposes
`base_layer_rgba_doc` so the app never re-decodes the layer.

**D2 — A dab is one GPU dispatch over its bounding box.** Each dab's bounding
box is a compute grid; an invocation computes the tip coverage from the
stroke-constant parameters (the same curve `pictura_paint::TipParams::coverage`
evaluates) and, when the accumulated coverage changed, composites it into the
resident layer in the same pass. Batching a sample's dabs into one instanced
dispatch — or baking the tip into a stamp texture — is a later optimization; the
first cut keeps one dispatch per dab so the per-dab result can be diffed against
the CPU oracle exactly.

**D3 — Each changed pixel recomposites from an immutable base.** The shader
keeps the same 8-bit per-pixel coverage accumulation the CPU oracle uses
(`1 - (1 - cov) * (1 - flow * tip)`, rounded to a byte) but composites
source-over in `f32` from an immutable copy of the seed, not from the layer it
has already written. The CPU stroke likewise recomposites from its untouched
base on every coverage change, so a pixel covered by several dabs matches
instead of compounding; without the base copy an overlapping stroke drifts well
past 1 LSB. Coverage is one `u32` per pixel, not a packed byte plane: four
invocations sharing a word with a read-modify-write race and lose each other's
updates. The parity test drives the same dab placer as the app and covers
overlapping dabs.

**D4 — Write back only the changed region; present exact.** A dab maps back only
its own changed rectangle, which the app writes into the exact `Stroke`'s
working document (`Stroke::patch_working_layer`, clipped to the layer rect and
converted to layer-local coordinates). The present then goes through the
existing `refresh_region`, which composites from that working document and
patches level 0 and the pyramid, so the live view is exact, not a source-over
approximation. The GPU→CPU stall is bounded by the display region, never the
brush.

**D5 — The GPU is authoritative; the CPU is the tolerance.** `end_paint`
commits `stroke.finish()` directly — the working document the GPU already
patched — so there is no replay of the logged dabs. The committed pixels equal
the GPU stroke and lie within ±1 LSB of the same stroke on the CPU, which the
parity test and self-test 546 prove. Any unsupported mode/stack, a
transparency-locked layer (source-over would violate the lock), a **sub-budget
dab**, or a missing adapter falls back to the CPU path, or to the LOD preview
for a dab over the budget, the same frame. The budget gate is deliberate:
seeding the GPU costs a full-frame upload, which a sub-budget dab (a few hundred
microseconds on the CPU) does not repay.

**D6 — De-interleave each dab's changed region on the GPU; patch by row.** The
host's per-dab write-back is the live-drag cost once the dab itself is cheap on
the GPU. `GpuStroke::dab` therefore reads its changed rectangle back already
planar: a region-parameterised planar pass (the same `PLANAR_SHADER` the canvas
readback uses, addressed at `(bx, by)` with row length `bw` over a layer stride
`lw`) writes four byte planes, and `Stroke::patch_working_layer` copies each
plane row straight into the layer's channel plane — no per-pixel gather, no
intermediate interleaved buffer, and no host-side conversion. The planar
readback is byte-identical to the resident layer (proven by the parity test),
so the ±1 LSB contract is unchanged.

**D7 — Build the seed without a per-pixel document pass.** The seed is still one
document-sized interleaved buffer (the shader addresses the layer in document
coordinates), but a layer that spans the document interleaves straight across
its four channel planes with no per-pixel channel lookup or bounds test, and the
seed is written straight into the buffer's mapped memory — no staging belt and
no second device copy before the immutable base snapshot.

## Risks / Trade-offs

- [GPU float rounding drifts from the CPU] → Each changed pixel recomposites
  from the immutable base with the same 8-bit coverage and `f32` source-over as
  the CPU, so the residual is rounding, bounded by ±1 LSB and proven by
  `gpu_stroke_matches_the_cpu_oracle`.
- [The stroke holds three per-pixel planes (layer + base + coverage, 12 bytes a
  pixel)] → Only an over-budget stroke on an available adapter allocates them,
  and they are dropped with the `GpuStroke` at commit or cancel; at 4000² that
  is ~192 MB transient.
- [Readback latency on a discrete GPU] → Only the changed region is read back;
  the present is frame-bounded, so at most one region is mapped per frame.
- [The path is unavailable on a device without a suitable adapter] → Selection
  is gated on the "Use Graphics Processor" flag and `GpuStroke::available()`; the
  CPU path (or the LOD preview) is the fallback and stays the default oracle.

## Open Questions

None outstanding for the first cut. Batching a sample's dabs into one dispatch,
and a stamp-texture tip, remain later optimizations (D2).
