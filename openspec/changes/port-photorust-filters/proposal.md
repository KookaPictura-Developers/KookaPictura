# Proposal: port-photorust-filters

## Why

Issue #221 (part of #183). photorust's filters are more capable than Kooka's
own: richer models, tuned against CS6 output, and about 450 tests. 89 filters
exist in both codebases. This change replaces the 68 whose parameter models
already match Kooka's (identical or renamed fields, no GPU plan, no dialog
change). The rest follow in #222–#227. The ported code is GPL-3.0, as its owner
confirmed on #221.

## What Changes

- **New engine:** `pictura-filters` gains a `photorust` module, with photorust's
  Artistic, Brush Strokes, Sketch, Texture, Pixelate, Distort, Stylize, and
  convolution code over an 8-bit interleaved `Pixmap` shim.
  `photorust::dispatch` validates each variant against its spec ranges, runs it
  under the variant's seed, and writes back the colour planes only.
- **Removed Kooka code:** the superseded implementations go (`artistic/`,
  `brush_strokes`, `pixelate/`, `sketch/`, `texture`, most of `stylize` and
  `distort`, and Add Noise and Dust & Scratches in `noise`). Kooka keeps
  Solarize (ImageMagick-exact), Shear, Ocean Ripple, Median, Despeckle,
  Maximum, Minimum, Offset, High Pass, Custom, the blur and sharpen family,
  Oil Paint, HDR Toning, and the Render filters.
- **Port adaptations, each marked in code:**
  - Seeds fold into photorust's noise hashes, so Kooka's Seed controls re-roll
    the patterns.
  - Rayon float reductions stay deterministic.
  - Colored Pencil's paper takes the background colour, per CS6 Help.
  - Mosaic rounds to the exact block mean.
- **Test coverage:**
  - The replaced filters' unit tests now run through `apply` against the new
    engine (`tests/scenarios`).
  - A new contract suite, `tests/ported`, covers all 68 filters.
  - photorust's own tests come along.

## Capabilities

### Modified Capabilities

- `imaging/filter-application`: the ported engine and its shared contract.
- `imaging/stylize-filters`: Emboss traces edges in their colour (CS6 Help);
  opposite lights swap highlight and shadow.
- `imaging/sketch-filters`: Bas Relief is a lit height field between the two
  colours.
- `imaging/brush-stroke-filters`: Accented Edges has no neutral brightness.
- `imaging/artistic-filters`: Film Grain's no-op needs no highlight area; Paint
  Daubs' seed re-rolls Sparkle, whose contours are drawn as lines of light;
  Palette Knife is photocraft's Kuwahara-and-palette knife and drops its seed.
- `imaging/filter-app-ui`: colour parameters show as a swatch that opens the
  colour picker; the Artistic dialogs open on new defaults, drop their
  foreground/background swatches, and keep up to eight controls in one column;
  filter dialogs preview as soon as they open.
- `imaging/noise-filters`: Add Noise's scale and seeded hash.

## Impact

- Mostly `pictura-filters`. The `Filter` API loses `PaletteKnife::seed`;
  `filter_map.rs` and the dialogs change only for Palette Knife's slot count,
  the Artistic defaults and slot layouts, and the colour swatch. `rayon` joins `pictura-filters`; it is already in the workspace
  through `pictura-render`, so no new crate enters the tree.
- **Output changes:** every ported filter's output changes. No golden baseline
  covers these filters. ImageMagick-exact Mosaic stays exact.
- **Defaults standing in for colours:** Neon Glow, Halftone Pattern, Photocopy,
  and Note Paper take document colours in photorust, but their variants carry
  none. CS6's default black and white stand in until the variants gain colour
  fields (ponytail).
