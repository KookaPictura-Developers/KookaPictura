# Design: text-shaping-rustybuzz

## Context

`crates/pictura-render/src/text_render.rs`:
- `BundledText` holds a `fontdue::Font` parsed from `FONT_BYTES`
  (`include_bytes!("../assets/LiberationSans-Regular.ttf")`, `&'static [u8]`).
- `shape_line(text, font_size, _tracking) -> Vec<ShapedGlyph>` currently maps each
  `char` to `font.lookup_glyph_index(c)` and `metrics_indexed(id, font_size)`
  `advance_width`.
- `pictura_core::ShapedGlyph { id: u16, advance: f32 }`; `layout_lines` places
  each glyph by `glyph.advance * params.font_size / params.units_per_em`, and the
  caller passes `units_per_em == font_size`, so `advance` is device pixels.
- `render_text_buffer` splits `type_tool.text` by lines and calls `shape_line`
  per line.

`rustybuzz` is a pure-Rust port of HarfBuzz's shaping algorithm (MIT, no C, no
system lib). Its `Face` borrows the font bytes; `UnicodeBuffer` → `shape(&face, &[],
buffer)` yields `glyph_infos()` (glyph ids, clusters) and `glyph_positions()`
(`x_advance`, `y_advance`, `x_offset`, `y_offset`) in font units.

## Goals / Non-Goals

**Goals:**

- Kerning and default substitutions apply, so advances match a real shaper.
- Keep the `ShapedGlyph { id, advance }` seam and the pure-Rust, no-C constraint.

**Non-Goals:**

- Mark positioning (offsets) beyond advances; the seam has no offset fields.
- Vertical text, RTL layout, or script-specific line breaking.
- Transform/warp/subpixel placement.

## Decisions

### D1. Add `rustybuzz`, pure Rust

`rustybuzz = "0.20"` in `crates/pictura-render/Cargo.toml`. It carries no C
dependency, matching the bundled backend's design constraint.

### D2. Build a `rustybuzz::Face` once, alongside the font

`BundledText` gains `face: rustybuzz::Face<'static>` from
`rustybuzz::Face::from_slice(FONT_BYTES, 0)`; `new()` returns `None` if either the
fontdue parse or the face parse fails, so a broken bundled asset still degrades
to no text render.

### D3. `shape_line` shapes the line and returns device-pixel advances

```
let upem = self.face.units_per_em() as f32;
let mut buffer = rustybuzz::UnicodeBuffer::new();
buffer.push_str(&cleaned);            // strip \n / \r as before
buffer.set_direction(rustybuzz::Direction::LeftToRight);
buffer.guess_segment_properties();
let shaped = rustybuzz::shape(&self.face, &[], buffer);
let scale = font_size / upem;
shaped.glyph_infos().iter().zip(shaped.glyph_positions()).filter_map(|(i, p)| {
    let id = u16::try_from(i.glyph_id).ok()?;
    Some(ShapedGlyph { id, advance: p.x_advance as f32 * scale })
}).collect()
```

The default feature set is used, so the font's `kern`/GPOS is applied. The
`_tracking` parameter stays unused (spacing is the layout's job).

### D4. Provenance backend

`BACKEND = "rustybuzz/fontdue"` (shaper/rasterizer) and
`BACKEND_VERSION = "0.20.1/0.9.4"`; update the provenance unit test. The module
doc and the `composite_type_source` "no kerning" note are updated.

### D5. Tests

- Existing: `shape_line` of "AB" still yields two positive-advance glyphs;
  render/composite tests still paint.
- New: a kerned pair. Shape a pair the bundled face kerns (verify at runtime;
  `AV`, `To`, `Va`, `Ya`, `We` are candidates) and assert the shaped total
  advance is strictly less than the sum of the two glyphs shaped alone, and less
  than the sum of their standalone `fontdue` advances; assert the unkerned
  control (e.g. "AA") is not reduced. If no candidate kern reduces, STOP and
  report (the font may have no `kern`/GPOS pairs).

## Risks / Trade-offs

- [No offsets] → mark placement is unmodeled; advances still improve. Marked.
- [Font may lack kerning] → the test discovers it; report rather than weaken.
- [Build time] → one pure-Rust crate; acceptable.
