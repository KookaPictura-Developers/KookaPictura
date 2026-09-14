# Color Replacement Tool

- **Spec ID:** `TOOL-021`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the Color Replacement tool predates CS6 and is carried forward unchanged in the CS6 Help.
- **Depends on:** `01-architecture/rust-core-design.md`, `01-architecture/undo-history.md`, `03-tools/brush-and-pencil.md`, `05-layers/blend-modes.md`, `07-color-painting/brush-engine.md`, `04-image-ops/image-modes.md`.

## CS6 behavior

The Color Replacement tool (`B` group) paints a **replacement foreground color
over a targeted color**. It is grouped with the Brush tool: click and hold the
Brush to reveal it.

CS6 Help behavior (verbatim contract):

- "The Color Replacement tool paints over a targeted color with a replacement
  color. While this tool is good for quick edits, it often proves unsatisfactory,
  particularly with dark colors and black."
- **It does not work in Bitmap, Indexed, or Multichannel color mode.**
- The user is advised to keep the **blending mode set to `Color`** ("Generally,
  you should keep the blending mode set to Color"). With `Color`, the tool
  replaces hue and saturation while preserving the underlying luminosity.
- **Sampling** (options bar):
  - `Continuous` — samples colors continuously as the user drags.
  - `Once` — replaces the targeted color only in areas containing the color that
    was first clicked.
  - `Background Swatch` — replaces only areas containing the current background
    color.
- **Limits** (options bar):
  - `Discontiguous` — replaces the sampled color wherever it occurs under the
    pointer.
  - `Contiguous` — replaces colors contiguous with the color immediately under
    the pointer.
  - `Find Edges` — replaces connected areas containing the sampled color while
    better preserving the sharpness of shape edges.
- **Tolerance** — a low value replaces only colors very similar to the clicked
  pixel; a high value replaces a broader range. The Help describes it in
  percentages, not raw 0–255 values (contrast the Paint Bucket, which documents
  0–255).
- **Anti-aliased** — smooths edges of the corrected areas.
- **Foreground color** is the replacement. The user clicks the color to replace
  in the image, then drags.

Interpretation (marked, because the Help is sparse): operating as a brush, the
tool tests each pixel under the brush against the sampled target within
`Tolerance`, and where it matches, replaces the pixel's hue/saturation toward the
foreground color under the `Color` mode. The exact per-pixel test is behavioral
parity only.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel (Brush slot, hidden tool) | Tool | `B` + `Shift+B` cycle | Reveal by holding the Brush tool. |
| Options bar | Tool bar | n/a | Brush tip, Mode, Sampling, Limits, Tolerance, Anti-aliased, plus opacity/flow if present. |
| Brush preset picker | Pop-up | n/a | Same tip options as the Brush tool. |
| Brush panel | Dock | `Window > Brush` | Tip shape and dynamics apply to the tool's brush. |
| Color Picker / Swatches | Dialog / panel | n/a | Sets the replacement foreground color and the Background Swatch mode's color. |
| Foreground/Background swatch | Toolbox | `D`, `X` | Replacement color (foreground); target reference (background, for Background Swatch). |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Mode | enum | `Color` (recommended) | CS6 paint-mode list (Normal, Dissolve, …, Color, …) | Help advises keeping `Color`; other modes produce less useful results. |
| Sampling | enum | `Continuous` (inferred) | Continuous / Once / Background Swatch | Default not stated in the Help. |
| Limits | enum | `Contiguous` (inferred) | Discontiguous / Contiguous / Find Edges | Default not stated in the Help. |
| Tolerance | percent | not stated | Help says "percentage"; numeric max not stated | Distinct from Paint Bucket's documented 0–255. |
| Anti-aliased | bool | on (inferred) | on / off | Smooths corrected edges. |
| Brush tip | preset | per Brush | Size, Hardness, Spacing, etc. | Shared Brush-tip controls. |
| Opacity / Flow | percent | 100 / 100 (inferred) | 0 – 100 | Present on brush-like tools; not itemized for this tool in the Help. |

## Algorithms & pipeline

Behavioral parity proposal; Adobe's implementation is closed.

1. **Target selection.** At stroke start, take the color under the brush hotspot
   (or the background swatch, or a live sample, per Sampling) as the target
   `T` in the document working space.
2. **Per-pixel match test.** For each pixel `P` under the dab, compute a color
   distance `d(P, T)` (a CIELAB ΔE or a per-channel normalized distance are both
   defensible; *(inferred)*). Replace where `d ≤ f(Tolerance)`.
3. **Replacement.** Under `Color` mode, transfer the foreground color's hue and
   saturation while keeping `P`'s luminosity, so shading and texture survive.
   Other modes apply their standard formula (W3C *Compositing and Blending
   Level 1* for the standard modes).
4. **Sampling mode effects.** `Continuous` re-samples `T` along the drag;
   `Once` freezes `T` at the first click; `Background Swatch` uses the current
   background color as `T` and does not sample the canvas.
5. **Limits.** `Discontiguous` applies the test to every pixel under the dab;
   `Contiguous` restricts to regions connected to the clicked pixel;
   `Find Edges` runs the connected-region test but fits the replacement inside
   strong edges to preserve them.
6. **Anti-aliasing.** Edge pixels receive partial replacement proportional to
   their brush coverage; deselecting Anti-aliased produces a hard result.
7. **Commit.** Strokes accumulate in a scratch buffer and commit tiles on pointer
   release, like any brush stroke.

## Rust module mapping

Design proposal.

- `pictura-paint::color_replacement` — `ColorReplacementEngine` with
  `SamplingMode { Continuous, Once, BackgroundSwatch }` and
  `LimitsMode { Discontiguous, Contiguous, FindEdges }`.
- `pictura-paint::color_distance` — `delta_e` / normalized-distance kernels
  (shared with Magic Eraser / Background Eraser, `03-tools/eraser-tools.md`, and
  Magic Wand).
- `pictura-paint::brush` — tip footprint and dab rasterization.
- `pictura-core::command` — `ColorReplacementStrokeCommand { layer, tip, mode,
  sampling, limits, tolerance, anti_aliased, target, replacement, dirty_tiles }`.

Crossing types: `LayerId`, `Rect`, `ColorSpace`, `BlendMode`, `TileDelta`.

## Qt6 component mapping

- `ColorReplacementOptionsBar` (`QWidget`) — Mode, Sampling, Limits, Tolerance,
  Anti-aliased, brush tip picker.
- `BrushPresetModel` / tip picker — reused from the Brush tool.
- `CanvasView` (`QGraphicsView`) — brush cursor overlay; pixels stay in the GPU
  pipeline.

## Data-model impact

- **No PSD fields.** Only pixel tiles and (optionally) the layer's existing
  fields are mutated.
- **Undo.** One history state per completed stroke (`01-architecture/undo-history.md`).
- **Mode restriction is a hard gate.** On Bitmap, Indexed, or Multichannel
  documents the tool must be disabled; it must not silently convert the document.
- **Color management.** The target and replacement colors pass through the
  document's working space and ICC transforms; comparisons should be made in a
  perceptually meaningful space (`pictura-color`).

## Edge cases

- **Color mode restrictions.** Disabled for Bitmap, Indexed, and Multichannel
  (per CS6 Help). Grayscale has no hue/saturation; the tool is effectively inert
  — confirm before enabling.
- **Dark colors / black.** The Help explicitly warns results are often
  unsatisfactory for dark colors and black. Parity means reproducing the
  limitation, not "fixing" it.
- **Tolerance extremes.** Tolerance 0 replaces only (near-)identical pixels;
  maximum tolerance may replace most of the image. Clamp and document.
- **Background Swatch mode** depends on the background color changing mid-stroke;
  define whether a sample is taken once or continuously.
- **Lock Transparency on** restricts writes to opaque pixels, like the Brush.
- **32-bit documents** restrict the available modes.
- **Huge documents.** Dirty-rect tracking must not copy the canvas.
- **Empty / 1-px layers.** No-op safely.
- **Undo mid-stroke.** Stroke is atomic on release.

## Parity acceptance criteria

1. Given an RGB document and a solid target color, painting with
   `Sampling = Once` and a moderate tolerance replaces only pixels matching the
   first-clicked color; pixels of a different color are untouched.
2. Given `Sampling = Continuous`, moving into a new color region adapts the
   target; given `Once`, it does not.
3. Given `Background Swatch`, only pixels matching the background color are
   affected.
4. Given `Limits = Contiguous`, disconnected regions of the same color are not
   changed; given `Discontiguous`, they are.
5. Given `Limits = Find Edges`, shape edges are preserved more than under
   `Contiguous` in a side-by-side diff.
6. Given the `Color` mode, replacing a saturated hue over shaded pixels preserves
   the luminosity ordering of those pixels within a defined tolerance.
7. Given a Bitmap, Indexed, or Multichannel document, the tool is unavailable.
8. Given `Anti-aliased` off, the corrected region has no partial-coverage edge
   pixels; on, it does.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference, "Use the Color Replacement tool"
  (p. 298): purpose and limitations, the three Sampling modes, the three Limits
  modes, Tolerance, Anti-aliased, foreground replacement color, and the statement
  that it does not work in Bitmap, Indexed, or Multichannel mode.
- `https://www.w3.org/TR/compositing-1/` — standard blend-mode formulas
  referenced for non-`Color` modes.

## Open questions

- **Numeric Tolerance range** for the Color Replacement tool is not stated in the
  Help (it says "percentage", while the Paint Bucket documents 0–255).
  *Resolves with:* a CS6 options-bar capture or a scripted probe.
- **Defaults** for Sampling, Limits, Anti-aliased, and Mode are not stated.
  *Resolves with:* a CS6 screenshot of the options bar on tool selection.
- **Whether Opacity/Flow** are exposed on this tool in CS6 is unconfirmed.
  *Resolves with:* the CS6 options bar.
- **Exact color-distance metric and replacement math** are unspecified by Adobe.
  *Resolves with:* pixel-level comparison against CS6 output and a chosen
  tolerance.
- **Grayscale behavior** (no hue/saturation to replace) is not documented.
  *Resolves with:* a CS6 experiment.
