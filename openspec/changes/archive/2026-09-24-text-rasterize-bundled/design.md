# Design: text-rasterize-bundled

## Context

`pictura_core` ships the deterministic `layout_lines` over shaped runs,
`RasterRequest`/`GlyphMask`/`Rasterizer`, and `FontPolicy`/`TextProvenance`.
`pictura-render` depends on `pictura-core` and owns CPU compositing. No font
dependency exists.

`fontdue` is a pure-Rust TTF parser + rasterizer (no C deps), so it keeps the
engine Qt-free and the CPU oracle portable. It gives `lookup_glyph_index`,
`metrics`, and `rasterize_indexed` — enough to shape simple Latin text and
produce coverage masks. It does not do GSUB/GPOS shaping, kerning tables, or
bidi; those are ceilings.

## Goals / Non-Goals

**Goals:**

- A deterministic, dependency-light glyph rasterizer for the Qt-free engine.
- A type layer can be materialized into pixels from its `TypeTool` text+style.
- Provenance for the substitution.
- Rust tests, no Photoshop-oracle claim.

**Non-Goals:**

- Complex shaping, kerning, ligatures, bidi, or vertical text.
- Warp/rotation from `TyTool.transform` (materialize axis-aligned at the layer
  rect).
- The app `Rasterize Type` command and the Qt backend (next changes).
- Persisting provenance into the PSD (the type is returned/recorded in memory).

## Decisions

### D1. Bundle Liberation Sans

`crates/pictura-render/assets/LiberationSans-Regular.ttf` and
`assets/LICENSES/LiberationSans-OFL.txt` (SIL OFL 1.1, copied from the system
package; provenance noted in a README). `include_bytes!` embeds it in the
staticlib. Liberation Sans is metric-compatible with Arial, a defensible
default; the named EngineData family is resolved to it and recorded as a
substitution.

### D2. Shape by the bundled font's cmap

`shape_line(text, font_size, tracking) -> Vec<ShapedGlyph>`: for each `char`,
`font.lookup_glyph_index(c)` and an advance in font units. To keep the layout
contract unchanged, shape advances in **device pixels** and lay out with
`LayoutParams.units_per_em = font_size`, so `device_advance == advance`;
tracking stays the layout's job (`tracking * font_size / 1000`). `fontdue`'s
metrics already include size, so this is exact for the bundled font.

### D3. Rasterize through the port

`BundledRasterizer { font: fontdue::Font }` implements
`pictura_core::Rasterizer`: `rasterize(&RasterRequest)` calls
`font.rasterize_indexed(request.glyph, request.px_size)` and packs the
`(Metrics, Vec<u8>)` into `GlyphMask { width, height, left: metrics.xmin,
top: metrics.ymin, coverage }`. Subpixel positioning is ignored (integer
placement) — a marked ceiling.

### D4. Materialize a type layer

`render_text_layer(doc, path) -> bool`:
1. find the layer (recursing into children) and require `type_tool` with a
   `style`;
2. split the text on `\r`/`\n` into lines, shape each, and `layout_lines` with
   `align = TextAlign::from_justification(style.justification)`,
   `leading = font_size * 1.2`, `wrap_width = Some(layer.rect.width())`;
3. rasterize each glyph and paint `coverage/255 * fill_color` into a fresh
   `0/1/2/-1` channel set of the layer rect, alpha accumulating the coverage;
4. replace the layer's channels, remove the `TySh` block, clear `type_tool`;
5. return true only when it rendered (so the caller records one history state
   and a refusal leaves the document unchanged).

Zero-area rects, a missing type tool, or a rasterizer failure return false
without mutating.

### D5. Provenance, no cryptographic hash dependency

`BundledText::provenance(requested)` returns `TextProvenance` with `backend =
"fontdue"`, `backend_version = env!("CARGO_PKG_VERSION")`-independent literal
from `fontdue`'s version string, `resolved_family = "Liberation Sans"`, and a
`font_hash` derived from a deterministic non-cryptographic 64-bit hash of the
font bytes expanded to 32 bytes (`ponytail:` not a content digest; swap for
SHA-256 if documents pin fonts).

## Risks / Trade-offs

- [Font substitution] → recorded as provenance; the resolved family is not the
  requested one.
- [Non-cryptographic hash] → marked; only a stable identity is needed today.
- [Layout vs shaping] → advances come from the bundled font, so a PSD authored
  with another font reflows; inherent to substitution and out of scope.
- [f32 raster determinism] → `fontdue` is pure Rust; the same font bytes and
  inputs give the same coverage on the CPU.
