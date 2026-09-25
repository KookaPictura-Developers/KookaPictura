# Design: hdr-conversion

## Context

A 32-bit Grayscale/RGB read normalizes to the 8-bit editing model but retains
`Document.source_depth = Some(ThirtyTwo)`, the decoded `f32` composite/extra
planes in `Document.source_planes`, and per-layer `source_channels`. A save
re-emits that source depth while `retains_source_depth()` holds
(`write.rs::output_depth`). `composite_native(doc)` composites into `f32` and
`emit_native` clamps to `[0, 1]`, so it loses the values above white; only the
retained `source_planes` of a *clean* open still carry them.

## Goals / Non-Goals

**Goals**

- Turn a 32-bit document into a real 16- or 8-bit document through the
  documented Exposure & Gamma operator, with the store, depth fields, and
  composite all consistent so the next save writes the new depth.

**Non-Goals**

- Local Adaptation, Equalize Histogram, Highlight Compression, the Method combo,
  and the f-stop/gamma ranges CS6 documents (closed or unverified kernels).
- 32-bit Lab/CMYK (already excluded by their `source_mode`), and up-conversion
  8/16 -> 32 (a separate roadmap item).

## Decisions

**Where it lives.** `pictura_render::document_ops::depth`, beside the other
document-scope ops, returning `pictura_ops::OpsError`. It needs the retained
samples, the operator, and the composite, all reachable from `pictura-render`.

**Gate.** `source_depth == Some(ThirtyTwo)`, `source_mode.is_none()`, mode
`Rgb | Grayscale`, `out in {Sixteen, Eight}`; anything else is refused with the
document untouched.

**Source samples.** Prefer `doc.source_planes.samples` when it is `F32` (a clean
open keeps values above `1.0`); otherwise fall back to `composite_native(doc)`,
which is pre-clamped at `1.0` by `emit_native` — the marked ceiling. Tone-map
the first `color_channels * width * height` samples only; alpha/extra planes are
copied verbatim (still quantized to the output store).

**Quantization.** 16-bit is `clamp(v, 0, 1) * 65535` rounded
(`Samples::to_u16`); 8-bit is `Samples::narrow_to_u8`. The retained store is
rebuilt at the output depth so `write_psd` sees matching depth and samples, and
every layer `source_channels` is dropped so a 32-bit layer store cannot leak
into a converted save (layers widen from their 8-bit channels instead).

**8-bit.** `depth = Eight`, `source_depth = None`, `source_planes = None`; the
document is now an ordinary 8-bit document.

## Risks / Trade-offs

- The ranges/defaults beyond Exposure `0` / Gamma `1.0` are inferred. Marked
  `ponytail:`.
- An already-edited 32-bit document was clamped at `1.0` by `emit_native`, so
  only a clean open tone-maps faithfully. Marked `ponytail:`.
- Layered documents tone-map the merged composite, not per layer. Marked
  `ponytail:`.

## Migration Plan

Additive; revert is a revert.
