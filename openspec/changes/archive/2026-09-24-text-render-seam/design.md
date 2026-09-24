# Design: text-render-seam

## Context

The engine is Qt-free and its oracle is the deterministic CPU path
(`docs/dev/testing-conventions.md`). Text never becomes pixels there today.
The ADHD decision (2026-09-24) chose: the engine owns a deterministic
**layout** pass over already-shaped runs; **glyph rasterization** lives behind a
port; the default backend is a pure-Rust rasterizer over a bundled
metric-compatible fallback font; a Qt `QFont` backend is an optional
higher-fidelity interchange; every render carries provenance.

This change builds the layout and the port. It cannot ship a backend without a
font dependency, so it ships none — but the layout and the port are the parts
whose contract the backends and the app must agree on, and they are fully
testable without a font.

## Goals / Non-Goals

**Goals:**

- A deterministic, Qt-free layout over shaped glyph runs.
- A font-agnostic rasterizer port with POD request/response.
- Font policy and provenance as data.
- Golden field tests with synthetic data; no font, no pixels.

**Non-Goals:**

- Shaping (cmap/GSUB/GPOS) or any font parsing — the host shaper supplies glyph
  ids and advances.
- Any rasterizer backend, the bundled font, or Qt.
- The app command and the compositor wiring.

## Decisions

### D1. Layout consumes shaped runs, not text

`ShapedGlyph { id: u16, advance: f32 }` (advance in **font units**).
`layout_lines(lines: &[Vec<ShapedGlyph>], params: &LayoutParams) -> TextLayout`
takes one inner slice per explicit line (the host splits on `\r`/`\n`) and
returns positions. This removes the font from the engine's critical path and
makes layout a pure arithmetic function.

### D2. Layout params and arithmetic

`LayoutParams { font_size: f32, units_per_em: f32, tracking: f32, leading: f32,
align: TextAlign, wrap_width: Option<f32> }`.

- device advance of a glyph = `advance * font_size / units_per_em + tracking *
  font_size / 1000.0` (Photoshop tracking is 1/1000 em).
- glyphs are placed left-to-right at the running x; y is the line baseline.
- a line's advance is the sum of its device advances.
- alignment shifts every glyph of a line by `(wrap_width - line_advance) * f`,
  `f = 0.0` left, `0.5` center, `1.0` right; no `wrap_width` means no shift.
- baselines step by `leading` from the first.
- `TextLayout { lines: Vec<LayoutLine>, width: f32, height: f32 }`,
  `LayoutLine { glyphs: Vec<PlacedGlyph>, advance: f32, baseline: f32 }`,
  `PlacedGlyph { id: u16, x: f32, y: f32 }`.

Determinism: plain `f32` arithmetic, left-to-right reduction, no libm
transcendentals — the same inputs give the same output on any target the engine
supports. Goldens assert exact values on small integer-friendly inputs.

### D3. Alignment maps the EngineData justification

`TextAlign::{Left, Center, Right}`; `TextAlign::from_justification(byte)`: 0 →
Left, 1 → Right, 2 → Center, anything else → Left. (Photoshop's paragraph
justification is 0 left / 1 right / 2 center for the common cases; the remaining
values are justification variants that map to Left here, a marked ceiling.)

### D4. The rasterizer port is POD-only

`RasterRequest { glyph: u16, px_size: f32, subpixel_x: f32, subpixel_y: f32 }`
and `GlyphMask { width: u32, height: u32, left: i32, top: i32, coverage:
Vec<u8> }`. `trait Rasterizer { fn rasterize(&self, request: &RasterRequest) ->
Option<GlyphMask>; }`. No font bytes, no Qt type, no `&mut self`, so a backend
can be a pure function object and the app can inject a Qt backend behind a
cxx-qt callback. The bundled backend is next; this change includes a
`TestRasterizer` so the port has a real implementation and is exercised by
tests.

### D5. Policy and provenance are data

`FontPolicy::{BundledOnly, HostAllowed, ExactOrRefuse}` and
`TextProvenance { requested_family, resolved_family, font_hash: [u8; 32],
backend, backend_version }`. Resolution itself (family → face) needs a font and
lands with the bundled backend; this change defines the types and a
`TextProvenance::new` constructor plus a `Display`/stable record form so the
goldens can assert them.

### D6. No trait with one implementation

The `Rasterizer` port is justified by two concrete planned backends (a
pure-Rust bundled default and a Qt optional). It ships here with the test
backend, so it is never an unimplemented abstraction.

## Risks / Trade-offs

- [f32 layout drift] → simple arithmetic and integer-friendly goldens; a future
  fixed-point layout is a local change behind the same function.
- [Justification variants] → non-0/1/2 map to Left, marked; widen when a
  fixture needs them.
- [Wrapping] → `wrap_width` shifts lines for alignment only; automatic line
  breaking (beyond explicit breaks) is out of scope and the host supplies
  lines.
