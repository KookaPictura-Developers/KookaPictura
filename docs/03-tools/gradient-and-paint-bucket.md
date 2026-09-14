# Gradient and Paint Bucket Tools

- **Spec ID:** `TOOL-024`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` for the two tools. In CS6 a **Dither** option was added to the *Gradient Overlay* and *Gradient Stroke* layer styles (not the Gradient tool, which already had Dither), and the Fill dialog gained **Scripted Patterns**; both are layer-style/fill features cross-referenced, not tool changes.
- **Depends on:** `01-architecture/rust-core-design.md`, `01-architecture/undo-history.md`, `01-architecture/color-management.md`, `03-tools/brush-and-pencil.md`, `04-image-ops/image-modes.md`, `05-layers/blend-modes.md`, `07-color-painting/gradient-presets.md`.

## CS6 behavior

### Gradient tool (`G`)

The Gradient tool creates a gradual blend between multiple colors, from preset
gradient fills or user-defined ones. **It cannot be used with Bitmap or
Indexed-color images.**

- The gradient fill applies to the current selection; with no selection it applies
  to the entire active layer.
- The options bar has a wide gradient sample:
  - click the triangle to pick a preset gradient fill;
  - click inside the sample to open the **Gradient Editor** (select a preset or
    create a new gradient; the **Neutral Density** preset is documented as a
    photographic ND filter).
- Five gradient styles, set by how press/drag points define the result:
  - `Linear` — shades from start to end in a straight line.
  - `Radial` — shades from start to end in a circular pattern.
  - `Angle` — shades in a counterclockwise sweep around the start point.
  - `Reflected` — mirrors the same linear gradient on either side of the start.
  - `Diamond` — shades from the middle to the outer corners of a diamond.
- Options bar: blending **Mode**, **Opacity**, **Reverse** (flip color order),
  **Dither** (smoother blend, less banding), **Transparency** (use the gradient's
  transparency mask).
- Drag from start to end; hold `Shift` to constrain the angle to a multiple of
  45°.

### Gradient Editor

- Opens from the options-bar gradient sample.
- **Gradient Type**: `Solid` or `Noise`.
- Solid gradients: color stops under the bar; opacity stops above; midpoint
  diamonds; **Location** in percent (0% left, 100% right); **Smoothness** for
  transition banding; **New** saves a preset; rename/delete; sample colors from
  the image with the eyedropper.
- Noise gradients: **Roughness**, **Color Model** (e.g. HSB) with per-component
  range sliders restricting the colors.
- Transparency: opacity stops at the start/end and intermediate stops; the
  checkerboard preview shows transparency.
- Presets live in the Gradient Picker / Preset Manager / Gradient Editor; can be
  saved to and loaded from libraries, reset to defaults, and displayed as text /
  thumbnail / list.

### Paint Bucket tool (`G` group)

Fills adjacent pixels that are similar in color to the pixel clicked. **It cannot
be used with Bitmap-mode images.** It is grouped with the Gradient tool (click and
hold Gradient to reveal it).

- Choose a foreground color, then fill with the **foreground color or a
  pattern**.
- Options:
  - **Mode** and **Opacity** for the paint.
  - **Tolerance** defines how similar a pixel must be to the clicked pixel to be
    filled. Values can range from **0 to 255**; low tolerance fills a narrow
    range, high tolerance a broad range.
  - **Anti-aliased** smooths the edges of the filled selection.
  - **Contiguous** fills only pixels contiguous to the clicked one; unchecked
    fills all similar pixels in the image.
  - **All Layers** fills based on merged color data from all visible layers.
- Click the image; all specified pixels within tolerance are filled.
- If working on a layer and not wanting to fill transparent areas, the layer's
  transparency should be locked.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel (Gradient slot) | Tool | `G` | Gradient; hidden tool: Paint Bucket. |
| Options bar — Gradient | Tool bar | n/a | Gradient sample, 5 style buttons, Mode, Opacity, Reverse, Dither, Transparency. |
| Gradient sample pop-up | Pop-up | n/a | Preset grid, library menu (Load/Replace/Reset/Save Gradients). |
| Gradient Editor | Dialog | n/a | Solid/Noise type, color/opacity stops, midpoints, Smoothness/Roughness. |
| Options bar — Paint Bucket | Tool bar | n/a | Fill (Foreground/Pattern), pattern picker, Mode, Opacity, Tolerance, Anti-aliased, Contiguous, All Layers. |
| Pattern picker | Pop-up | n/a | Pattern preset selection and library menu. |
| `Edit > Fill` | Dialog | `Shift+F5` | Related fill path; CS6 adds Scripted Patterns. |
| Swatches / Color Picker | Panel / dialog | n/a | Foreground/background color used by the fill. |

## Parameters & ranges

| Control | Tool | Type | Default | Range / options | Notes |
|---|---|---|---|---|---|
| Gradient style | Gradient | enum | Linear | Linear / Radial / Angle / Reflected / Diamond | Defined by press/drag geometry. |
| Gradient sample | Gradient | preset | Black→White (inferred) | any preset / custom | Opens Gradient Editor on click. |
| Mode | Gradient | enum | Normal | CS6 blend-mode list | |
| Opacity | Gradient | percent | 100 | 0 – 100 | |
| Reverse | Gradient | bool | off | on / off | |
| Dither | Gradient | bool | off | on / off | Reduces banding. |
| Transparency | Gradient | bool | on (inferred) | on / off | Use gradient's transparency mask. |
| Gradient Type | Editor | enum | Solid | Solid / Noise | |
| Color stops | Editor | list | 2 | 2+ | Color, Location 0–100%, midpoint. |
| Opacity stops | Editor | list | 2 | 2+ | Opacity 0–100%, Location 0–100%. |
| Smoothness | Editor | percent | not stated | not stated | Solid gradients. |
| Roughness | Editor | percent | not stated | not stated | Noise gradients. |
| Color Model | Editor | enum | not stated | e.g. RGB / HSB | Noise component ranges. |
| Fill | Bucket | enum | Foreground | Foreground / Pattern | Pattern requires a loaded pattern. |
| Mode | Bucket | enum | Normal | CS6 blend-mode list | |
| Opacity | Bucket | percent | 100 | 0 – 100 | |
| Tolerance | Bucket | int | 32 (inferred) | 0 – 255 (documented) | Max explicit numeric range in the Help. |
| Anti-aliased | Bucket | bool | on (inferred) | on / off | |
| Contiguous | Bucket | bool | on (inferred) | on / off | |
| All Layers | Bucket | bool | off | on / off | Fill using merged visible data. |

## Algorithms & pipeline

Behavioral-parity proposals.

### Gradient tool

1. **Geometry.** The drag defines a segment `A → B` in document space. For each
   affected pixel compute a parameter `t` in `[0, 1]`:
   - `Linear`: projection onto `AB`, clamped to `[0, 1]`.
   - `Radial`: `distance(P, A) / |AB|`.
   - `Angle`: `atan2` about `A` mapped to a counterclockwise sweep.
   - `Reflected`: `|projection|` mirrored about `A`.
   - `Diamond`: `(|dx| + |dy|) / |AB|` (L1 / Manhattan radius), giving a diamond.
2. **Color evaluation.** Interpolate between color stops by position using the
   midpoint and Smoothness; evaluate opacity stops the same way when Transparency
   is on. Interpolation space is the document working space (cross-ref
   `01-architecture/color-management.md`); Adobe's exact gradient interpolation
   space is unspecified — *(inferred)*.
3. **Dither.** Add a small ordered or noise dither to `t` (or to the quantized
   output) before quantizing to the document bit depth, to break banding. Only
   when Dither is enabled.
4. **Reverse** swaps the effective stop order.
5. **Composite.** The result is composited onto the layer/selection with Mode and
   Opacity. In a selection, pixels outside are untouched; otherwise the full
   layer is covered.
6. **Commit.** One history state on pointer release.

### Paint Bucket tool

1. **Region selection.** From the clicked pixel, run a region grow:
   - `Contiguous` on: scanline flood fill (`pictura-paint::flood_fill`), 4- or
     8-connected as CS6 does — confirm connectivity.
   - `Contiguous` off: a global pass over the layer (or all layers if All Layers
     is on), selecting every pixel whose color is within `Tolerance` of the
     clicked pixel.
2. **Color test.** Per-channel or perceptual distance ≤ `Tolerance` (0–255).
   *(`inferred` metric; the range 0–255 is documented.)* With **All Layers**, the
   compared pixel is the merged composite of visible layers; the fill is still
   written to the active layer.
3. **Anti-aliasing.** Boundary pixels receive partial fill proportional to their
   match.
4. **Fill.** Paint the foreground color or the pattern (aligned to the document
   origin for patterns) with Mode and Opacity, honoring Lock Transparency (do not
   fill transparent areas when locked).
5. **Commit.** One history state per click.

## Rust module mapping

Design proposal.

- `pictura-paint::gradient` — `GradientStyle { Linear, Radial, Angle, Reflected,
  Diamond }`, `Gradient { kind: Solid | Noise, color_stops, opacity_stops }`,
  `GradientEngine::rasterize(style, a, b, dither, reverse, transparency) ->
  TileDelta`.
- `pictura-paint::gradient::noise` — noise-gradient generation (roughness, color
  model ranges).
- `pictura-paint::flood_fill` — scanline / global tolerance fill; shared with
  Magic Eraser and Magic Wand.
- `pictura-paint::bucket` — `BucketEngine` (`Fill { Foreground, Pattern }`,
  tolerance, contiguous, anti_aliased, all_layers, mode, opacity).
- `pictura-core::command` — `GradientFillCommand { layer, selection, style, a, b,
  dither, reverse, transparency, mode, opacity }` and
  `BucketFillCommand { layer, point, fill, pattern, tolerance, contiguous,
  anti_aliased, all_layers, mode, opacity }`.

Crossing types: `LayerId`, `Rect`, `SelectionRef`, `ColorSpace`, `BlendMode`,
`TileDelta`, `PatternRef`.

## Qt6 component mapping

- `GradientOptionsBar` (`QWidget`) — sample, five style buttons, Mode, Opacity,
  Reverse, Dither, Transparency.
- `GradientSamplePopup` (`QWidget`) — preset grid + library menu.
- `GradientEditorDialog` (`QDialog`) — gradient bar with color/opacity stops,
  midpoint handles, Smoothness/Roughness, Color Model, and a `QGraphicsView`-backed
  stop editor.
- `GradientPresetModel` (`QAbstractListModel`).
- `PaintBucketOptionsBar` (`QWidget`) — fill selector, pattern popup, Mode,
  Opacity, Tolerance, Anti-aliased, Contiguous, All Layers.
- `PatternPickerPopup` (`QWidget`) + `PatternModel`.
- `ToleranceSlider` — dense custom control reused across bucket/wand/erasers.

## Data-model impact

- **No PSD fields for the tools.** The change is to pixel tiles (and selection
  bounds).
- **Gradient presets** are stored outside the document (libraries / Preferences
  file per the Help's Gradient Editor note); cross-ref
  `07-color-painting/gradient-presets.md` / `10-workflow-io/presets-manager.md`.
- **Pattern presets** similarly persist outside the document.
- **Undo.** Gradient = one state on release; Paint Bucket = one state per click.
  Undo restores touched tiles bit-exactly.
- **Selection** clips the gradient; the command references the selection rather
  than baking a mask.
- **Color management.** Stop colors and the composite pass through the document's
  working space and ICC transforms.

## Edge cases

- **Bitmap and Indexed color modes.**
  - Gradient tool: **unavailable** in Bitmap and Indexed.
  - Paint Bucket: **unavailable in Bitmap**; CS6 Help does not state Indexed for
    the bucket — confirm (a single-index palette may still permit it).
- **32-bit documents.** Paint modes are restricted to the documented subset
  (Normal, Dissolve, Darken, Multiply, Lighten, Linear Dodge (Add), Difference,
  Hue, Saturation, Color, Luminosity, Lighter Color, Darker Color).
- **No selection vs selection.** A zero-area or empty selection must be handled;
  no selection means full-layer coverage.
- **Degenerate gradient drag.** Start == end: produce a solid fill of the end
  color (or no-op) deterministically, no divide-by-zero.
- **CMYK / Lab / Grayscale.** Gradient and fill operate in the working space;
  grayscale has no color stops beyond the gray axis.
- **Tolerance extremes.** 0 for the bucket fills only identical pixels; 255 may
  fill the whole layer. Confirm inclusive comparison.
- **Contiguous connectivity** (4- vs 8-connected) is unspecified; pick and
  document a deterministic rule.
- **All Layers.** Fills based on merged color but writes to the active layer;
  transparency-locked and transparent-pixel behavior must match CS6.
- **Huge documents (PSB).** Region grows and gradient rasterization must be
  tiled; the bucket must not materialize a whole-canvas non-contiguous mask at
  PSB sizes if it can stream per tile.
- **Undo/redo.** Deterministic tile deltas for both tools.
- **GPU unavailable.** Both tools must work on the CPU fallback.

## Parity acceptance criteria

1. Given a selected region and a black→white gradient, each of the five styles
   produces the documented geometry (linear straight ramp; radial circular;
   angle counterclockwise sweep; reflected mirrored; diamond L1) within a defined
   pixel tolerance.
2. Given `Reverse`, the gradient color order is flipped; given `Transparency`
   off, opacity stops are ignored.
3. Given `Dither` on, a gradient across a region shows less banding than with it
   off, at the same document bit depth.
4. Given the Gradient tool on a Bitmap or Indexed document, it is unavailable.
5. Given a Paint Bucket click with `Contiguous` on, only the connected
   same-color region is filled; with it off, all similar pixels in the layer are
   filled. Tolerance 0 vs 255 changes the filled region as expected.
6. Given `All Layers`, the fill decision uses merged visible data while writing
   to the active layer.
7. Given Lock Transparency on in a layer, transparent areas are not filled.
8. Given a gradient drag or a bucket click, exactly one history state is created
   and undo restores the touched tiles bit-exactly.
9. Given a Bitmap-mode document, the Paint Bucket is unavailable.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference. "Fill with the Paint Bucket tool"
  (pp. 444–445): foreground/pattern, mode, opacity, Tolerance 0–255,
  anti-aliased, contiguous, All Layers, Bitmap-mode restriction. Gradient
  sections (pp. 449–453): five gradient styles with definitions, options-bar
  Mode/Opacity/Reverse/Dither/Transparency, `Shift` 45° constraint, Bitmap/Indexed
  restriction, Gradient Editor (Solid/Noise, color and opacity stops, Location,
  Smoothness, Roughness, Color Model, preset management, Neutral Density).
  "Productivity enhancements (JDI's) in CS6" (Layers list, p. 8 area): "Added
  dither option to Gradient Overlay and Gradient Stroke layer styles." "Fill a
  selection or layer with color" (pp. 444–445) for the CS6 Scripted Patterns
  addition.
- `https://www.w3.org/TR/compositing-1/` — standard blend/compositing formulas
  referenced for Mode handling.

## Open questions

- **Gradient interpolation space and dithering algorithm** are unspecified by
  Adobe. *Resolves with:* pixel comparison against CS6 gradients and a chosen
  tolerance; the interpolation space materially changes mid-gradient colors.
- **Gradient tool defaults** (default style, sample, Transparency default,
  Dither default) are not stated. *Resolves with:* a CS6 options-bar capture.
- **Paint Bucket defaults** (Tolerance default, Anti-aliased, Contiguous) are
  inferred. *Resolves with:* a CS6 screenshot on tool selection.
- **Region connectivity** for contiguous fill (4- vs 8-connected) and whether the
  tolerance comparison is inclusive are unspecified. *Resolves with:* a CS6
  experiment with a diagonal boundary.
- **Paint Bucket in Indexed mode** is not addressed by the Help (only Bitmap is
  called out). *Resolves with:* a CS6 experiment.
- **Pattern alignment/transform rules** for the bucket (document origin vs
  layer origin) are not stated. *Resolves with:* a CS6 reference.
- **Noise-gradient generation** (random seed handling, deterministic output) is
  unspecified. *Resolves with:* a decision on whether parity requires
  seed-stable noise; document in `07-color-painting/gradient-presets.md`.
