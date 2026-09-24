# Design: text-shaping-offsets

## Context

`pictura_core::ShapedGlyph { id: u16, advance: f32 }`; `layout_lines` computes
`device_advance(glyph) = glyph.advance * font_size / units_per_em + tracking`,
places each glyph at the running `x`, and records `PlacedGlyph { id, x, y }`
with `y = baseline`. `pictura-render::render_text_buffer` uses
`line.baseline` for the paint row and `glyph.x` for the pen.

`rustybuzz::GlyphPosition` has `x_advance`, `y_advance`, `x_offset`, `y_offset`
in font units. HarfBuzz semantics: draw glyph at `(pen + x_offset,
baseline - y_offset)`, then `pen += x_advance`.

## Goals / Non-Goals

**Goals:**

- Offsets ride the seam and are applied deterministically in layout.
- Zero offsets (all plain Latin) produce byte-identical output to today.

**Non-Goals:**

- Subpixel rasterization (the mask is still placed at integer pixels).
- Vertical text; `y_advance` is ignored.

## Decisions

### D1. `ShapedGlyph` gains two font-unit offsets

```
pub struct ShapedGlyph { pub id: u16, pub advance: f32, pub x_offset: f32, pub y_offset: f32 }
```

Derive `Default` and add `ShapedGlyph::new(id, advance)` (offsets `0.0`) so the
many existing construction sites and tests stay terse; struct literals add the
two fields or use `..ShapedGlyph::new(id, advance)`.

### D2. `layout_lines` applies offsets, pen uses the advance

```
let pen_x = x + device(glyph.x_offset);
let y = baseline - device(glyph.y_offset);   // font y-up -> screen y-down
glyphs.push(PlacedGlyph { id, x: pen_x, y });
x += device_advance(glyph, params);           // offsets do not change the pen
```

`advance`/`width` stay the sum of device advances, so alignment and wrapping are
unaffected. `device(v) = v * font_size / units_per_em`.

### D3. The renderer uses the per-glyph baseline

`render_text_buffer`: `let baseline = (glyph.y + baseline_shift).round() as i32;`
instead of the line baseline. With zero offsets `glyph.y == line.baseline`, so
behavior is unchanged; `glyph.x` already includes the x-offset.

### D4. `shape_line` fills the offsets

`x_offset = pos.x_offset as f32 * scale`, `y_offset = pos.y_offset as f32 * scale`,
the same `scale = font_size / units_per_em` as the advance.

### D5. Tests

- `pictura-core`: `layout_lines` shifts `x` by the x-offset and `y` below the
  baseline by the y-offset, with the line `advance`/`width` unchanged; a second
  glyph's pen is unaffected by the first's offset.
- `pictura-render`: `render_text_buffer` paints a glyph with a synthetic
  `ShapedGlyph` y-offset at a shifted row (build the layout directly, no font
  shaping needed); and `shape_line` of plain Latin returns zero offsets while the
  kerning test still holds.
- If the bundled face gives no nonzero GPOS offset for a combining-mark probe,
  report it rather than faking an integration assertion.

## Risks / Trade-offs

- [Struct-literal churn] → mitigated by `new()` + `Default`.
- [Latin-only engine] → offsets only matter for marks; still correct to carry.
- [Rounding] → offsets move the origin, the mask still lands on integer pixels
  (subpixel is a separate ceiling).
