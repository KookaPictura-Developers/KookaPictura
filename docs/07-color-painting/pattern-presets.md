# Pattern Presets and Pattern Libraries

- **Spec ID:** `CLR-011`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — CS6 adds **Scripted Patterns** (five included scripts) to the `Edit > Fill` pattern path, and the drawing tools gain vector-based shape layers that can be filled with patterns. Pattern lists, Define Pattern, and the Pattern Stamp are carried over.
- **Depends on:** `03-tools/clone-stamp-and-pattern-stamp.md`, `03-tools/gradient-and-paint-bucket.md`, `05-layers/fill-layers.md`, `05-layers/layer-styles.md`, `03-tools/shape-tools.md`, `01-architecture/document-model.md`, `10-workflow-io/presets-manager.md`, `07-color-painting/gradient-presets.md`.

> Module and widget names below are **design proposals**. Facts not confirmed by
> a fetched CS6/Adobe source are marked *(inferred)*.

## CS6 behavior

A **pattern** is "an image that is repeated, or tiled, when you use it to fill a
layer or selection." Preset patterns appear in pop-up panels for the Paint
Bucket, Pattern Stamp, Healing Brush, and Patch tools, and in the Layer Style
dialog. Source: CS6 reference, "Creating patterns" / "Managing pattern libraries
and presets".

### Define a pattern

- `Edit > Define Pattern` on a **Rectangular Marquee** selection of any open
  image; **Feather must be 0**. Name it in the Pattern Name dialog.
- When a pattern from one image is applied to another, Photoshop **converts the
  color mode**. (Help note.)
- Photoshop ships Illustrator-format files that can be opened, rendered, and
  defined as patterns.
- **Define from a layer**: the Help documents selection-based definition; layer
  bounds definition is *(inferred)* but is the common workflow (hide other
  layers, define from the layer). *Open question.*

### Apply patterns

- **Pattern Stamp tool** paints with a pattern. Choose a brush, set mode/opacity,
  toggle **Aligned** (maintain continuity across strokes vs restart each stroke),
  pick a pattern from the options-bar pop-up, optionally **Impressionist**, then
  drag. (Source: "Paint with a pattern".)
- **Paint Bucket**: choose **Pattern** vs foreground, set tolerance, anti-alias,
  contiguous, All Layers. (Source: "Fill with the Paint Bucket tool".)
- **`Edit > Fill`** (or `Shift+F5`): `Use: Pattern`, pick a pattern, optionally
  **Scripted Patterns** (CS6), set mode/opacity/Preserve Transparency. (Source:
  "Content-aware, pattern, or history fills".)
- **Pattern Fill layer** (`Layer > New Fill Layer > Pattern`): click the pattern
  to choose from the pop-up; **Scale** (percentage of pattern size), **Snap To
  Origin** (align pattern origin to document origin), **Link With Layer** (pattern
  moves with the layer; then drag in the image to position while the dialog is
  open). (Source: "Create a fill layer".)
- **Pattern Overlay layer style** and **Texture** in Bevel & Emboss / Satin /
  Inner Glow: Pattern, **New Preset**, **Snap To Origin**, **Link With Layer**,
  **Scale**, drag to position. Pattern option is unavailable if no patterns are
  loaded. (Source: "Layer style options".)
- **Vector shape layers (CS6)** can be filled with preset or user-defined
  patterns via the options bar / shape fill. (Source: JDI/Drawing — "Fill
  objects with preset or user-defined gradients, colors, and patterns".)

### Scripted Patterns (CS6)

Select `Edit > Fill`, `Use: Pattern`, then **Scripted Patterns** (a checkbox in
the Fill dialog) and choose one of the five scripts from the **Script** pop-up.
Sources: CS6 Help "Content-aware, pattern, or history fills" and "Scripted
patterns"; community walkthroughs (Planet Photoshop, Tiny Tutorials).

| Script | Behavior (community description) |
|---|---|
| Brick Fill / Brick Wall | Staggers tiles with a horizontal offset (and a vertical slant) with added color variance. |
| Cross Weave | Places tiles in a cross-weave/rotated arrangement with color variance. |
| Random Fill | Random size, placement, and rotation. |
| Spiral | Places tiles along a spiral. |
| Symmetry Fill | Symmetrical snowflake-like placement. |

The Help names the feature and the "five included" scripts but not all five
names; the names above come from secondary tutorials. Adobe's scripting
documentation ("Programming Deco Pattern Fill Scripts in Photoshop CS6",
secondary) indicates the scripts are driven by a **Deco** framework and stored
as JavaScript files.

### Pattern libraries and management

- **Load** / **Replace Patterns** (pop-up menu or a library file at menu bottom;
  Append or replace), **Save Patterns** as a library, **Reset Patterns** to the
  default set, **Rename Pattern**, **Delete Pattern** (`Alt`/`Option`-click =
  scissors). The Pattern option is dimmed until a pattern library is loaded.
- Display options in pop-up panels (thumbnail/list modes) and the Preset Manager.
- **Pattern Maker** is an *optional plug-in* (`Filter > Pattern Maker`) that
  slices/reassembles an image into tiles and can save a tile as a pattern preset;
  it required 32-bit mode on Mac. This is a separate filter and is noted here for
  completeness.

### Higher bit depths

The Help does not document pattern bit-depth behavior explicitly. Patterns are
preset bitmaps (commonly 8-bit) applied to 8/16/32-bpc documents; whether
16/32-bit pattern fills retain full precision or quantize to 8-bit is
*(inferred)* and listed as an open question.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Toolbox | Pattern Stamp tool | `S` cycles | Grouped with Clone Stamp |
| Options bar (Pattern Stamp) | Pattern pop-up / Aligned / Impressionist | — | Brush, mode, opacity |
| Options bar (Paint Bucket) | Pattern vs Foreground, tolerance, anti-alias, contiguous, All Layers | `G` cycles | |
| `Edit > Define Pattern` | Menu command | — | Requires 0-feather marquee |
| `Edit > Fill` | Dialog | `Shift+F5` | Use: Pattern; Scripted Patterns (CS6) |
| Fill dialog | Script pop-up | — | Brick Fill / Cross Weave / Random Fill / Spiral / Symmetry Fill |
| `Layer > New Fill Layer > Pattern` | Fill layer dialog | — | Scale, Link With Layer, Snap To Origin |
| `Layer > Layer Style > Pattern Overlay` | Layer style dialog | — | Scale, Link With Layer, Snap To Origin, New Preset |
| Layer style Texture (Bevel/Satin/Glow) | Layer style dialog | — | Pattern + Scale + Depth + Invert |
| Shape tool options bar (CS6) | Fill = Pattern | — | Vector shape fill |
| Pattern pop-up panel menu | Load/Replace/Save/Reset/Rename/Delete + display | — | Library management |
| Preset Manager | Dialog | — | Cross-preset management |
| `Filter > Pattern Maker` | Optional plug-in | — | Tile generator |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Pattern | preset | per tool | Any loaded pattern | |
| Scale | percent | 100% | 1–1000% *(inferred)* | Fill layer, Pattern Overlay, texture |
| Link With Layer | bool | On *(inferred)* | On/Off | Fill layer, Pattern Overlay, texture |
| Snap To Origin | action | — | — | Align pattern origin |
| Aligned | bool | On *(inferred)* | On/Off | Pattern Stamp continuity |
| Impressionist | bool | Off | On/Off | Pattern Stamp stylization |
| Texture Invert | bool | Off | On/Off | Inverts texture highs/lows |
| Texture Depth | percent | 100% | −1000…+1000 *(inferred)* | Paint penetration |
| Fill mode / opacity | enum / percent | Normal / 100% | 27 modes, 0–100% | Tool and Fill dialog |
| Tolerance | int | 32 *(inferred)* | 0–255 | Paint Bucket |
| Contiguous | bool | On *(inferred)* | On/Off | Paint Bucket |
| Script | enum | None | Brick Fill / Cross Weave / Random Fill / Spiral / Symmetry Fill | CS6 Scripted Patterns |
| Pattern record color mode | enum | per pattern | Bitmap/Gray/Indexed/RGB/CMYK/Multichannel/Duotone/Lab | `.pat` records *(community)* |
| Pattern bit depth | bits | 8 *(inferred)* | 1/8/16/32 | `.pat` records *(community)* |

## Algorithms & pipeline

- **Tiling.** A pattern is repeated at its native size with an anchor/origin. The
  effective origin is the document origin when Snap To Origin is on; otherwise
  the pattern moves with the layer (Link With Layer) or sits at the layer's
  upper-left. Tiling is a modular index: `src = ((dst − origin) mod size) /
  scale`.
- **Scale.** The pattern is resampled by the Scale percentage (fill layer,
  Pattern Overlay, texture) — resample filter is *(inferred)*; CS6 likely uses
  the current interpolation preference.
- **Pattern Stamp modulation.** Painted per brush mark with the brush tip;
  Aligned offsets the tile by the stroke's start point to preserve continuity;
  Impressionist adds stylized variation (exact effect *(inferred)*, behavior TBD).
- **Color mode conversion** on cross-document use (Help note): the pattern's
  mode is converted to the target document's mode.
- **Scripted Patterns.** The five scripts place/scatter/rotate/offset pattern
  copies within the selection or layer bounds. Exact algorithms are closed; the
  names imply the transforms and community descriptions confirm offset/
  rotation/symmetry. Treat as **behavioral parity only, algorithm TBD**.
- **Pattern Maker** (optional plug-in): slices the source into tiles and
  reassembles a seamless pattern; algorithm is a plugin-level feature and out of
  the core pattern engine's scope.
- **`.pat` container.** *(Community, not an Adobe spec.)* A standalone library is
  `8BPT` version 1 followed by unframed pattern records; records carry name,
  identifier, color mode, dimensions, and PackBits-compressed channel data
  (raw/PackBits at 1/8/16/32 bits), optional indexed palettes, user masks and
  transparency, and a trailing `phry` hierarchy of groups/presets in newer files.
  Pattern records for the Pattern Stamp additionally require an anchor point
  (left/top) in the source plane.

## Rust module mapping

Proposed:

- `pictura_core::pattern` — `Pattern { id, name, size, mode, planes, anchor }`,
  `PatternPlane` (1/8/16/32-bit, raw/PackBits), `PatternSet` (ordered library).
- `pictura_ops::paint::pattern_fill` — `tile(dst_rect, pattern, origin, scale)`
  writing a `PaintSink`; shared by Pattern Stamp, Paint Bucket, Fill dialog, and
  fill layers.
- `pictura_ops::pattern::define` — build a `Pattern` from a selection or layer
  bounds; color-mode conversion via `pictura_color`.
- `pictura_ops::pattern_script` — `Script::{Brick, CrossWeave, Random, Spiral,
  Symmetry}`, deterministic given a seed; emits placements.
- `pictura_io::pat` — `8BPT` v1 reader/writer; preserves unknown records/tagged
  blocks. Also handle the PSD pattern image-resource block.
- Boundary types: `Rect`, `Point`, `ColorMode`, `Pattern`, `PatternPlacement`,
  `PaintSink`.

## Qt6 component mapping

- `PatternPickerButton` (`QToolButton`) — thumbnail/sample button opening a
  pop-up grid; shared by Paint Bucket, Pattern Stamp, Fill dialog, fill layer,
  and layer styles.
- `PatternLibraryModel` (`QAbstractItemModel`) — the ordered pattern list with
  display modes (text / thumbnail / list) and rename/delete.
- `DefinePatternCommand` (`QAction`) — `Edit > Define Pattern`.
- `ScriptedPatternWidget` (`QWidget`) — the Script combo shown when Pattern +
  Scripted Patterns is chosen in `FillDialog`.
- `PatternFillLayerPage` / `PatternOverlayPage` (`QWidget`) — Pattern, Scale
  slider, Link With Layer, Snap To Origin, on-canvas position drag.
- `PatternStampOptions` (`QWidget`) — pattern pop-up, Aligned, Impressionist.

Widgets rather than QML (dense docked forms, consistent with the widget shell).

## Data-model impact

- **Document patterns.** A document can carry embedded patterns (fill layers and
  layer styles reference patterns); a stale reference must be kept so copied
  layers still render. Store pattern definitions in the document or reference a
  shared library by id.
- **PSD serialization.** Patterns are stored in the image-resource block(s) and
  in descriptor-based fill/layer-style blocks; the exact tags are *(inferred)*
  and belong in `01-architecture/file-formats.md`.
- **Undo.** Define Pattern is a preset-library action (not document history
  unless the pattern is embedded). Applying a fill is one history state.
- **Presets.** `.pat` libraries are managed by `10-workflow-io/presets-manager.md`.

## Edge cases

- **Feather ≠ 0**: Define Pattern requires a 0-feather marquee; a feathered
  selection must be rejected or the feather ignored (Help requires 0).
- **Selection vs layer bounds**: defining from a selection crops to it; defining
  from a layer uses layer bounds *(inferred)*.
- **Cross-mode define/apply**: pattern mode is converted to the target document
  mode (Help note) — CMYK/Indexed/Bitmap conversions must be deterministic.
- **8/16/32-bit**: pattern planes at lower depth must upsample without banding;
  at 32-bit the exact precision behavior is unverified.
- **Empty pattern list**: Pattern option/Pattern Overlay is dimmed until a
  library is loaded (Help).
- **Rotated/scaled canvas**: pattern origin is in document coordinates; Rotate
  View does not rotate the pattern tiling.
- **Huge (PSB) fills**: tile patterns on demand; do not materialize a full-canvas
  copy.
- **GPU unavailable**: CPU tiling is the reference.
- **Missing pattern on layer copy**: fall back to the stored definition, not the
  current library, so copied/duplicated layers keep their look.

## Parity acceptance criteria

1. Given a 4×4 two-color pattern, filling a 16×16 selection with Scale 100%
   produces a 4×4 repeating tile exactly; Scale 200% doubles the tile size.
2. Given Snap To Origin on, a pattern's tile boundary aligns to document (0,0);
   with Link With Layer and a layer moved by (dx,dy), the pattern shifts by
   (dx,dy).
3. Given a Pattern Stamp stroke with Aligned on, the pattern is continuous
   across a release-and-drag; with Aligned off it restarts at the new point.
4. Given `Edit > Define Pattern` on a 0-feather selection, the new pattern
   appears in every pattern pop-up and matches the selection pixels within 1 LSB.
5. Given a pattern defined in an RGB document applied to a CMYK document, the
   result is the CMYK conversion of the pattern (no crash, defined result).
6. Given `Edit > Fill` with Pattern + each of the five Scripted Patterns, the
   result matches a CS6 capture within a visual-similarity tolerance, and the
   same seed/pattern reproduces the same output.
7. Given Load / Replace / Save / Reset / Rename / Delete Pattern actions, the
   library list changes accordingly and persists across restart; Reset restores
   the default set.
8. Given `Open` a `.pat` library, records decode to expected pixels, dimensions,
   identifier, color mode, and (where present) palette/transparency.
9. Given a Pattern Fill layer, Scale and Link With Layer behave per
   `05-layers/fill-layers.md` and the layer mask confines the fill.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, downloaded and text-extracted. Establishes:
  "Creating patterns" (pattern definition, preset pop-ups for Paint Bucket /
  Pattern Stamp / Healing Brush / Patch / Layer Style, Define Pattern with
  0-feather marquee, color-mode conversion, Illustrator-format files);
  "Paint with a pattern" (Pattern Stamp options: Aligned, pattern pop-up,
  Impressionist); "Fill with the Paint Bucket tool" (Pattern vs foreground);
  "Content-aware, pattern, or history fills" (Edit > Fill Use: Pattern; CS6
  Scripted Patterns checkbox + Script pop-up, "five included"); "Scripted
  patterns" ("What's New" summary); "Create a fill layer" (Pattern fill layer:
  Scale, Snap To Origin, Link With Layer, image-drag positioning);
  "Layer style options" (Pattern, New Preset, Snap To Origin, Link With Layer,
  Scale; Texture: Invert/Depth/Scale); "Managing pattern libraries and presets"
  (Load/Replace/Save/Reset/Rename/Delete, Presets/Patterns folder);
  JDI/Drawing (vector shape fills with patterns); "Generate a pattern using the
  Pattern Maker" (optional plug-in, tile size, Save as pattern preset).
- `https://planetphotoshop.com/scripted-patterns.html` — community (Heath Rowe,
  2012): locates Scripted Patterns in `Edit > Fill`, names the five scripts
  (Brick Fill, Cross Weave, Random Fill, Spiral, Symmetry Fill), and documents
  the Define Pattern + scripted workflow.
- `https://tinytutorials.wordpress.com/2013/01/15/photoshop-cs6-scripted-patterns/`
  — community: describes each script's observable placement behavior.
- `https://pub.dev/packages/patkit` — community codec documentation for the
  Photoshop `.pat` (`8BPT` v1) container: magic/version, unframed records, color
  modes, raw/PackBits channels at 1/8/16/32 bits, indexed palettes, transparency,
  `phry` hierarchy descriptors. Used for the `.pat` algorithm/data-model notes;
  not an Adobe-published specification.
- `https://kipdf.com/programming-deco-pattern-fill-scripts-in-photoshop-cs6-user-guide_5aafd3111723dd329c633fb2.html`
  — secondary: identifies the CS6 scripted-pattern implementation as a JavaScript
  "Deco" framework with five scripts. Used only to characterize the mechanism.

## Open questions

- **All five script names/builds**: the Help names the feature but not every
  script; confirm the exact five labels in a shipping CS6 install (community
  sources differ between "Brick Fill"/"Brick Wall").
- **Scripted-pattern algorithms, seeds, and parameters**: no public spec. Exact
  behavior is TBD; *resolves with* a CS6 capture set and, if reproducible,
  matching stochastic parameters.
- **Pattern resampling filter** used for Scale. *Resolves with:* a scaled-pattern
  pixel diff.
- **Define from layer vs selection** — does CS6 include a layer-bounds path?
  *Resolves with:* CS6 observation.
- **Pattern bit-depth behavior** for 16/32-bpc documents. *Resolves with:*
  controlled 16/32-bit fills.
- **`.pat` details** (header fields, anchor encoding, v16 `phry`) — the cited
  source is a third-party codec; *resolves with* the Adobe file-format spec and
  CS6-saved samples.
- **PSD keys** for embedded patterns and Pattern Overlay fills. *Resolves with:*
  `01-architecture/file-formats.md`.
