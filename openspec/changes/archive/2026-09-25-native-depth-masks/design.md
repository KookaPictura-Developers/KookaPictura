# Design: native-depth-masks

## Context

`blend_into` computes `mask_alpha(layer, x, y) as f32 / 255.0`, where
`mask_alpha` = `(raster_mask_alpha * vector_coverage + 127) / 255` in u8. The
raster part reads `layer.mask.data` (8-bit). `Layer.source_channels` may hold a
`-2` plane (native mask, `mask.rect` dimensions) for a 16/32-bit read.

## Goals / Non-Goals

**Goals**

- Gate a high-depth layer by its native raster mask when the store matches.
- Keep the 8-bit path byte-identical.

**Non-Goals**

- Vector-mask geometry changes (its coverage stays the existing u8 raster).
- Native mask *editing*, app wiring, HDR, GPU.

## Decisions

**Thread `doc` into `blend_into`.** The mask lookup needs `doc.source_depth` and
the layer store. Adding one `&Document` parameter is mechanical; the call sites
already have `doc` except `composite_solid_fill`/`composite_gradient_fill`,
which gain one (their callers have it). `composite.rs` line count is not affected
materially.

**`mask_alpha_unit(doc, layer, x, y) -> f32`** in `composite_native.rs`:
- Mirror `raster_mask_alpha`'s branches exactly: no mask / disabled / no data ->
  `1.0`; out of the mask rect or empty -> `default_color / 255`; otherwise the
  sample.
- Native sample when `doc.source_depth.is_some()` AND `layer.source_channels` has
  a `-2` plane whose length is `mask_w * mask_h` (masks are valid in any
  high-depth mode, so no `source_mode` gate), else the 8-bit
  `raster_mask_alpha(layer,x,y) / 255.0`.
- Combine with the vector coverage as `raster * vector` (unit), replacing the u8
  `(raster*vector+127)/255`. For an 8-bit document `blend_into` keeps calling
  `mask_alpha` unchanged, so that rounding is preserved byte-for-byte.

**Gate only `blend_into`.** Layer effects and other paths that read
`mask_alpha` directly keep the 8-bit value; effects gate by content coverage, not
this path.

## Risks / Trade-offs

- [Threading `doc` across nine call sites] → the compiler names every site; no
  behavior change where `doc` is unused for 8-bit.
- [Native vs 8-bit mask at the seam] → 8-bit documents never take the native
  branch; the existing suite is the gate.
- [Mask plane length mismatch] → the helper length-checks and falls back.

## Migration Plan

Additive; revert is a revert. Native mask editing/app wiring are later.
