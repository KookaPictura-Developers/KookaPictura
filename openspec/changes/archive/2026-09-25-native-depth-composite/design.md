# Design: native-depth-composite

## Context

`Canvas` accumulates straight-alpha RGBA in `f32` (`Px`), so blending and
adjustment math can be high precision; the only 8-bit chokepoints are the layer
content read (kept 8-bit for this phase) and `composite_adjustment`'s
`to_u8`/`/255.0` round-trip through a 3-channel `PixelBuffer`. `Canvas::into_pixel_buffer`
narrows to u8 at the very end.

## Goals / Non-Goals

**Goals**

- A 16/32-bit document applies adjustment layers without the 8-bit round-trip,
  using `pictura_adjust::apply_native` on an `f32` sample store.
- A `composite_native` entry returns the composite at the source depth.
- The 8-bit composite output is byte-identical.

**Non-Goals**

- Reading layer *content* at native depth from `source_channels` (later phase).
- Native masks (`mask_alpha` stays 8-bit), GPU, app wiring, HDR tone map.

## Decisions

**Gate on `doc.source_depth`.** Only a document read at 16/32 has it `Some`, so
an 8-bit document keeps the exact existing code path and byte-identical output.
This is the smallest switch that cannot move goldens.

**Build an `f32` `Samples` from the canvas, not a new buffer type.** Planar
`R||G||B` from `canvas.px`, `Samples::F32`, `apply_native`, then write back. The
existing `Canvas` stays the accumulator; no depth-tagged canvas is introduced.

**Fall back on `Unsupported`.** `apply_native` covers the tonal and
color-preserving families but returns `Unsupported` for `Auto`/`ColorLookup`.
For a high-depth document those fall back to the existing 8-bit path (defined,
if lower precision). `InvalidParams` stays a no-op, matching today.

**`composite_native` mirrors `composite_rgba`.** Same canvas assembly; the
difference is only the emitter (`U16`/`F32` with `round(v.clamp(0,1) * MAX)`
instead of `to_u8`). `None` for a document with no `source_depth`.

## Risks / Trade-offs

- [An 8-bit document must not change] → gated on `source_depth`; the existing
  composite, oracle, and image tests are the gate.
- [Applying at f32 changes a 16-bit doc's adjustment output vs today's 8-bit
  round-trip] → intended and the point; the 8-bit path is unchanged.
- [`apply_native` and `apply` differ at `u8` for some kernels] → only the
  high-depth branch uses `apply_native`; 8-bit keeps `apply`.

## Migration Plan

Additive; rollback is a revert. The app can consume `composite_native` in a
later change (store it on the document after compositing so save retains it).
