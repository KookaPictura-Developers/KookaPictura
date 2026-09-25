# Design: text-subpixel-positioning

## Context

`crates/pictura-render/src/text_render.rs`:
- `BundledRasterizer<'a> { font: &'a fontdue::Font }` implements
  `pictura_core::Rasterizer::rasterize(&RasterRequest) -> Option<GlyphMask>` via
  `font.rasterize_indexed(request.glyph, request.px_size)`; it ignores
  `request.subpixel_x/y` and maps fontdue `Metrics { xmin, ymin, .. }` to
  `GlyphMask { left: xmin, top: ymin, .. }`.
- `render_text_buffer`'s `paint_layout` rounds `glyph.x`/`glyph.y` and passes
  `subpixel_x: 0.0, subpixel_y: 0.0`.

`GlyphMask.top` is the ymin (bottom edge relative to the baseline, positive up);
the paint row is `baseline - (height + top)`.

`swash` 0.2 API (pure Rust):
```
use swash::scale::{ScaleContext, Render, Source};
use swash::zeno::{Vector, Placement};
let font = swash::FontRef::from_index(FONT_BYTES, 0)?;
let mut ctx = ScaleContext::new();
let mut scaler = ctx.builder(font).size(px).hint(false).build();
let mut render = Render::new(&[Source::Outline]);
render.offset(Vector::new(subpixel_x, subpixel_y));
let image = render.render(&mut scaler, swash::GlyphId(glyph_id))?;
// image.placement: Placement { left: i32, top: i32, width: u32, height: u32 }
// image.data: width*height alpha bytes (Content::Mask)
```
`zeno::Placement.left` is the horizontal offset from the origin;
`zeno::Placement.top` is the vertical offset from the origin (verify the sign
against `GlyphMask.top`'s ymin convention empirically).

## Goals / Non-Goals

**Goals:**

- `RasterRequest.subpixel_x/y` changes where/how the mask is rasterized.
- A fractional pen position is not rounded away.

**Non-Goals:**

- Removing `fontdue` (kept for metrics; ceiling).
- Changing shaping (`rustybuzz` stays) or layout math.

## Decisions

### D1. swash for rasterization; fontdue kept for metrics

Add `swash = { version = "0.2", default-features = false, features = ["std",
"scale", "render"] }`. `BundledRasterizer` keeps the fontdue handle for
`glyph_count`/`lookup_glyph_index` and metrics, and adds `swash::FontRef` from
`FONT_BYTES` for rendering. Mark the three-rasterizer split with a `ponytail:`
ceiling.

### D2. Honor the offsets

`rasterize` builds a `Scaler` at `request.px_size`, builds a `Render` with
`Source::Outline`, sets `render.offset(Vector::new(subpixel_x, subpixel_y))`,
renders, and maps `image.placement` to `GlyphMask`, converting swash's `top` to
the ymin convention (the subagent verifies: if swash `top` is the top edge,
`ymin = top - height`; if it is already the bottom edge, use it directly).
`None` for an invalid glyph or an empty image, as today.

### D3. Fractional pen in the painter

In `paint_layout`, for each glyph:
```
let fx = glyph.x;              let px = fx.floor();
let fy = glyph.y + baseline_shift; let py = fy.floor();
rasterize(RasterRequest { glyph: glyph.id, px_size: font_size,
    subpixel_x: fx - px, subpixel_y: fy - py })
pen_x = px as i32 + mask.left;
top = py as i32 - (mask.height as i32 + mask.top);
```
Horizontal subpixel is required; vertical is applied if the placement verifies,
else `subpixel_y = 0.0` with a `ponytail:` ceiling.

### D4. Tests

- A glyph rasterized at `subpixel_x = 0.0` and at `0.5` yields different
  coverage **or** a different `left`/width, proving the offset reaches the
  rasterizer.
- A pen at `x = 12.5` paints a mask whose bounding box is not identical to
  `x = 12.0` and not identical to `x = 13.0` (the fractional phase is used).
- Existing tests: `rasterizer_returns_coverage_of_width_times_height`,
  `rasterizer_refuses_an_absent_index_without_panicking`, the render/kerning/
  offset tests still pass. Migrate the kerning test's fontdue baseline if needed.
- **Placement migration check**: rasterize `'A'` at 48 px with both swash and
  fontdue and assert the mask bounding box agrees within 1 px; the subagent
  fixes the `top`/`left` mapping until it does, then may delete the check.

## Risks / Trade-offs

- [Rasterizer swap changes all text pixels] → acceptable: no text golden exists;
  tests assert coverage/placement, not exact AA. Noted in the proposal.
- [swash placement sign] → verified empirically in D4; STOP and report if it
  cannot be made to agree within 1 px.
- [ScaleContext per call] → a fresh context per glyph is simplest under
  `Rasterizer::rasterize(&self)`; mark a cache ceiling if profiling shows cost.
