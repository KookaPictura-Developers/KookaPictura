# text-shaping-offsets

Carry the shaper's x/y offsets through `ShapedGlyph` and apply them in
`layout_lines`/`render_text_buffer` so GPOS mark positioning paints correctly.
