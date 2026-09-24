# Proposal: text-shaping-offsets

## Why

`text-shaping-rustybuzz` applied the shaper's advances but dropped its
`x_offset`/`y_offset`, so GPOS **mark positioning** (combining marks, and any
script whose diacritics are positioned rather than precomposed) is lost: the
`ShapedGlyph { id, advance }` seam has no offset fields, and
`render_text_buffer` places every glyph on the line baseline. `rustybuzz`
already returns the offsets; the seam just does not carry them.

## What Changes

- **Carry the shaper's offsets** on the seam: `pictura_core::ShapedGlyph` gains
  `x_offset`/`y_offset` (font units, like `advance`).
- **Apply them in layout**: `layout_lines` places a glyph at
  `pen + device(x_offset)` and `baseline - device(y_offset)` (font y-up → screen
  y-down); the pen still advances by `advance`, so line width is unchanged.
- **Apply the y-offset in the renderer**: `render_text_buffer` takes each glyph's
  baseline from `PlacedGlyph.y` instead of the line's, so a raised/lowered mark
  paints at the right height. (The x-offset is already folded into
  `PlacedGlyph.x`.)
- **No behavior change when offsets are zero** (all Latin text without marks), so
  existing output is unchanged.
- **BREAKING**: none for callers who build `ShapedGlyph` via the new constructor;
  struct literals need the two fields.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `text-shaping`: the `ShapedGlyph` contract now carries the shaper's offsets and
  the layout applies them.

## Impact

- `crates/pictura-core/src/text_render.rs`: `ShapedGlyph` fields + `layout_lines`
  offset handling + tests.
- `crates/pictura-render/src/text_render.rs`: `shape_line` fills the offsets;
  `render_text_buffer` uses `glyph.y`.
- No new dependency, no app UI, no PSD change.
- Ceiling unchanged: no transform/warp, no subpixel rasterization. Offsets move
  the whole glyph (integer pixel placement still rounds).
