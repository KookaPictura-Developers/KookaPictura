# Design: blend-if-render

## Context

`Layer.blend_if: Option<BlendIf>` is a derived view of the layer-record
`blending_ranges` body: `composite_source: (u16, u16)`,
`composite_dest: (u16, u16)`, then per-channel `((u16, u16), (u16, u16))`
groups. The raw `Layer.blending_ranges` bytes stay the serialization source.

The CPU compositor funnels every per-pixel blend through
`composite.rs::blend_into`, which computes an effective alpha
`src_a * opacity * fill * mask` and calls `blend_parts`. Layer effects, fills,
adjustment layers, smart sources, and groups all reach the canvas through
`composite_layer` and (for content) `blend_into`; adjustment layers call
`blend_into` too. The GPU compositor builds its own stack and rejects a layer it
cannot do (`gpu/mod.rs` `walk`).

Grounding: psd-tools 1.19 `LayerBlendingRanges` — each range is
`(black, white)` over `0..=65535`, default `(0, 65535)`. The docstring's "2
black values followed by 2 white values" contradicts the default; this model
follows the field layout (`source`, then `dest`), as the codec's parser already
does.

## Goals / Non-Goals

**Goals:**

- A non-default `BlendIf` gates the layer's per-pixel weight in the CPU
  compositor.
- GPU declines such a layer so the CPU oracle is authoritative.
- Absent/default `BlendIf` leaves compositing byte-identical.

**Non-Goals:**

- Knockout punch-through (`Shallow`/`Deep`).
- The Blending Options UI.
- Split-slider feathering (the stored model has no feather values).
- Photoshop pixel parity (no oracle).

## Decisions

### D1. Gate inside `blend_into`, factor 1 when inactive

`blend_into(canvas, layer, x, y, cs, src_a)` already has the source colour `cs`
and, via `canvas.idx(x, y)`, the backdrop. Compute
`let gate = blend_if_factor(layer.blend_if.as_ref(), cs, &backdrop);` and use
`src_a * gate` in place of `src_a`. When `layer.blend_if` is `None` or
`is_default()`, return `1.0` before touching the backdrop, so a document without
Blend If is unchanged and the hot loop pays one `Option` check.

### D2. Range semantics

Normalize stored `u16` to `0..=1` (`/ 65535.0`). For a range `(black, white)`:

- `black == 0 && white == 65535` → the range is inactive (factor 1).
- `black > white` (inverted) → factor 1; Photoshop only produces `black <= white`
  for a single unsplit range, and inverting would hide the whole layer.
- otherwise factor 0 when `value <= black` or `value >= white`, else 1.

The source range gates on the source pixel gray, the destination range on the
running backdrop gray. Per-channel group `i` gates group `i`'s source range on
`cs[i]` and its dest range on the backdrop channel `i`, for `i < 3` (RGB).
Channel groups beyond the colour channels (Photoshop writes 4 for RGB) are
ignored.

### D3. Composite gray

`gray(c) = 0.299*c.r + 0.587*c.g + 0.114*c.b` (Rec.601). Photoshop's exact
composite weighting is unpublished; mark the choice `ponytail:` in the code.

### D4. One gate, product over active ranges

The factor is the product of the composite gate and every applicable
per-channel gate, each either 0 or 1 in this slice. Numeric product is exact for
0/1.

### D5. GPU declines

Add `GpuError::UnsupportedAdvancedBlending` and, in `walk`, return it when a
layer (recursively) has a `blend_if` view that is not the default. The existing
`composite_active` path already turns a `GpuError` into the CPU composite, so no
caller change is needed. `Display` gets an arm.

### D6. Pure function, unit-tested

`blend_if_factor` takes `Option<&BlendIf>`, `cs: [f32; 3]`, and the backdrop
`Px` (or its three channels + alpha) and returns `f32`. Unit tests exercise:
`None` → 1; default → 1; a source gate that hides a below-black layer and keeps
an above-black one; a dest gate; a per-channel gate; an out-of-range group
ignored.

## Risks / Trade-offs

- [Adobe feather unmodeled] → hard 0/1 gate; the stored model has no feather
  bytes. Widening needs a reader for the split values, if they exist.
- [Composite gray weighting] → Rec.601 assumption, marked; a Photoshop fixture
  with a Blend If layer is the resolver.
- [Per-channel order] → assumed R,G,B; marked. The model does not label groups.
- [GPU divergence] → declining non-default layers keeps one oracle.
