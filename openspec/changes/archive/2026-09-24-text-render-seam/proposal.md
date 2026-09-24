# Proposal: text-render-seam

## Why

Roadmap P3: the engine now decodes a type layer's EngineData into a typed style
(`type-engine-data`) but has no path from text to pixels. The ADHD architecture
decision for live text: keep the Qt-free engine deterministic and let glyph
rasterization sit behind an explicit seam, with a pure-Rust bundled fallback as
the default backend and Qt as an optional higher-fidelity backend. This change
builds that seam and the deterministic layout half; the backends follow.

## What Changes

- **Deterministic text layout** in `pictura-core`: a Qt-free
  `layout_shaped(lines, params) -> TextLayout` that positions already-shaped
  glyphs (advance accumulation, tracking, leading, alignment/justification, and
  optional wrapping) in device pixels. The engine never shapes or parses a font;
  the host supplies shaped runs, so layout is testable with no font file.
- **The rasterizer seam**: POD `RasterRequest`/`GlyphMask` plus a `Rasterizer`
  port. Nothing Qt-shaped crosses it; a backend takes a glyph id, a pixel size,
  and a subpixel position and returns a coverage mask. Two backends are planned
  (pure-Rust bundled default, Qt optional); a test backend ships here.
- **Font policy and provenance**: `FontPolicy` (`BundledOnly` / `HostAllowed` /
  `ExactOrRefuse`) and `TextProvenance` (requested and resolved family, font
  content hash, backend and version), so a missing or substituted font is a
  recorded, auditable fact rather than a silent approximation.
- **Alignment** maps the EngineData justification byte to `TextAlign`.
- **No font dependency and no pixels** in this change; the bundled `fontdue`
  backend and the Qt backend are the next changes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `text-render-seam`: deterministic shaped-run layout, the glyph rasterizer
  port, and font policy/provenance resolution.

## Impact

- `crates/pictura-core`: new `text_render` module and re-exports.
- Tests: layout goldens over synthetic shaped runs; a test `Rasterizer`
  implementation; alignment and provenance unit tests.
- No app change, no new dependency.
