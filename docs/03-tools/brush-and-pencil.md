# Brush and Pencil Tools

- **Spec ID:** `TOOL-020`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — CS6 adds **erodible** and **airbrush** brush tips, **Brush Pose**, **Brush Projection**, per-stroke (not per-dab) color dynamics by default, a maximum round-brush size of 5000 px, and a HUD that can change opacity as well as size/hardness. The Brush and Pencil tools themselves predate CS6.
- **Depends on:** `01-architecture/rust-core-design.md`, `01-architecture/undo-history.md`, `01-architecture/gpu-rendering-pipeline.md`, `07-color-painting/brush-engine.md`, `07-color-painting/brush-dynamics.md`, `07-color-painting/airbrush-and-flow.md`, `05-layers/blend-modes.md`, `03-tools/eraser-tools.md`.

## CS6 behavior

The Brush tool (`B`) and the Pencil tool (`B`, same slot) paint the current
**foreground color** on the active layer.

- **Brush** produces soft, anti-aliased strokes. Hardness controls the size of
  the brush's hard center; at 100% the Brush is "hardest" but **still
  anti-aliased**.
- **Pencil** produces **hard-edged, aliased** lines. The CS6 Help states the
  Pencil "always paints a hard edge that is not anti-aliased"; hardness is not a
  meaningful Pencil control.
- Both paint the foreground color. Pencil adds **Auto Erase** (see
  `03-tools/eraser-tools.md`): if the drag begins with the cursor center over the
  foreground color, the area is erased to the background color instead of
  painted.

Common options-bar controls (CS6 Help, "Paint tool options"): **Mode**,
**Opacity**, **Flow**, and for the Brush tool an **Airbrush** toggle. The Pencil
tool adds **Auto Erase**. Both have a **brush preset picker**, size, and (for the
Brush) hardness and tablet pressure-override buttons.

Observable semantics:

- **Opacity** caps the transparency of the color applied in a single stroke. No
  amount of back-and-forth movement within one held stroke exceeds the set
  opacity. Releasing and stroking again applies an additional, independent layer
  of color equivalent to the opacity setting.
- **Flow** is the rate at which color builds up as the pointer moves. Example
  from the CS6 Help: opacity 33% + flow 33% means each pass moves the pixel 33%
  of the way toward the brush color, without exceeding 33% opacity for that
  stroke.
- **Airbrush** makes paint build up over **time** while the mouse button is held
  still, as well as over movement. Brush hardness, opacity, and flow control how
  fast paint accumulates. (In the Brush panel this is labeled
  **Airbrush/Build-up**.)
- **Straight lines:** click a start point, then hold `Shift` and click an end
  point.
- **Straight line while dragging:** the shortcut table also lists `Shift`-click
  behavior; the CS6 Help documents the click / `Shift`-click method.
- **Rotation aid:** the Rotate View tool rotates the canvas for easier painting
  (cross-ref `03-tools/rotate-view.md`).

### Brush tip shapes

The Brush panel exposes several tip families; each paints differently (CS6 Help,
"Creating and modifying brushes"):

- **Standard / round / captured tips** — Size, Use Sample Size, Flip X, Flip Y,
  Angle, Roundness, Hardness, Spacing. A tip captured from image pixels can be up
  to **2500 × 2500 px** and has fixed (unchangeable) hardness.
- **Bristle tips** — Shape, Bristles, Length, Thickness, Stiffness, Spacing,
  Angle; realistic multi-bristle strokes. Bristle previews require OpenGL.
  (Bristle tips arrived in CS5; CS6 carries them.)
- **Erodible tips (CS6)** — Size, Softness, Shape, Sharpen Tip, Spacing; the tip
  wears down like a pencil or crayon, shown in a **Live Brush Tip Preview**.
- **Airbrush tips (CS6)** — Size, Hardness, Distortion, Granularity, Spatter
  Size, Spatter Amount, Spacing; a 3D conical spray. Pen pressure can change the
  spread.

Other brush options (Brush panel): **Noise**, **Wet Edges**, **Airbrush/Build-up**,
**Smoothing**, **Protect Texture**, plus the dynamic sets **Shape Dynamics**,
**Scattering**, **Texture**, **Dual Brush**, **Color Dynamics**, and
**Transfer**.

### CS6-specific brush changes

From the "What's new in CS6" chapter and the "Productivity enhancements (JDI's)"
list:

| Change | Detail |
|---|---|
| Erodible tips | Wear-down drawing tips (pencils/crayons) with Softness, Shape, Sharpen Tip, Live Preview. |
| Airbrush tips | Conical spray tip with Granularity, Spatter Size/Amount, Distortion. |
| Brush Pose | Explicit Tilt X, Tilt Y, Rotation, Pressure, with Override options for a static pose. |
| Brush Projection | Shape Dynamics option that applies stylus tilt/rotation to tip shapes (static captured tips included). |
| Color Dynamics default | Dynamics change color **once per stroke** in CS6; `Apply Per Tip` restores the older per-dab behavior. |
| Max brush size | Raised to **5000 px**. |
| HUD | `Ctrl+Alt`/`Cmd+Option` drag changes opacity in the size/hardness HUD (requires the General preference "Vary Round Brush Hardness Based on HUD Vertical Movement" to be off). |
| Actions | Brush strokes can be recorded in actions via the Actions panel's **Allow Tool Recording**. |
| Texture | Brightness/contrast slider for brush textures. |
| Cursor | Brush tip cursor reflects brush pose and jitter for round and captured tips. |

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Tools panel (Brush slot) | Tool | `B` | `Shift+B` cycles Brush / Pencil / Color Replacement / Mixer Brush group. |
| Options bar | Tool bar | n/a | Mode, Opacity, Flow, Airbrush, preset picker, size, hardness, pressure buttons; Pencil adds Auto Erase. |
| Brush Presets panel | Dock | `Window > Brush Presets` | Preset thumbnails / list / stroke thumbnails. |
| Brush panel | Dock | `Window > Brush` | Brush Tip Shape + dynamics sets + pose + other options. |
| Brush Preset picker (options bar) | Pop-up | n/a | Temporary size/hardness overrides; preset list. |
| Brush panel side menu | Menu | n/a | New Brush Preset, Clear Brush Options, Reset/Load/Save brushes. |
| `Edit > Define Brush Preset` | Menu | n/a | Capture a selection (≤ 2500 × 2500) as a grayscale tip. |
| `Edit > Presets > Preset Manager` | Dialog | n/a | Brush library management. |
| Brush presets panel menu | Menu | n/a | Text Only / Thumbnails / List / Stroke Thumbnail; Load/Replace/Reset/Save Brushes. |
| HUD | On-canvas | `Alt+Right-drag` (size/hardness), `Ctrl+Alt+Right-drag` (opacity) | Drag left/right for size, up/down for hardness. |
| Cursor preferences | Dialog | `Edit > Preferences > Cursors` | Standard / crosshair / brush-tip cursor; Normal vs Full Size tip. |

## Parameters & ranges

Ranges below are quoted where the CS6 Help states them; anything else is marked
`not stated`.

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Size | int px | per preset | 1 – 5000 (CS6 max) | CS6 raised the maximum to 5000 px. |
| Hardness | percent | per preset | 0 – 100 | Brush only; sampled tips cannot change hardness. 100% is still anti-aliased. |
| Roundness | percent | 100 | 0 – 100 | 100 = round, 0 = linear. |
| Angle | degrees | 0 | not stated | For elliptical/sampled tips. |
| Spacing | percent of diameter | per preset | not stated | Deselected = cursor speed determines spacing. |
| Opacity | percent | 100 | 0 – 100 | Per-stroke ceiling. Number keys set in 10% steps / two digits. |
| Flow | percent | 100 | 0 – 100 | Per-pass build-up. `Shift`+number keys. |
| Mode | enum | Normal | Normal, Dissolve, Behind, Clear, Darken, Multiply, Color Burn, Linear Burn, Lighten, Screen, Color Dodge, Linear Dodge (Add), Overlay, Soft Light, Hard Light, Vivid Light, Linear Light, Pin Light, Hard Mix, Difference, Exclusion, Hue, Saturation, Color, Luminosity | `Behind` and `Clear` require Lock Transparency **off**. 32-bit documents allow only a subset (see Edge cases). |
| Airbrush | bool | off | on / off | Same setting as Brush panel "Airbrush/Build-up". |
| Auto Erase | bool | off | on / off | Pencil only. |
| Wet Edges | bool | off | on / off | Paint accumulates at stroke edges. |
| Noise | bool | off | on / off | Per-dab randomness; most visible on soft tips. |
| Smoothing | bool | on (per preset) | on / off | Smoother stylus curves at the cost of lag. |
| Protect Texture | bool | off | on / off | Locks pattern + scale across textured presets. |
| Scatter | percent | 0 | 0 – 100 (not stated) | Both Axes selects radial distribution. |
| Count / Count Jitter | int / percent | per preset | not stated | Increasing count without spacing/scatter can hurt performance. |
| Color Dynamics jitter | percent sliders | 0 | 0 – 100 (not stated) | CS6: applied once per stroke unless "Apply Per Tip". |
| Brush Pose Tilt X/Y, Rotation | degrees | 0 | not stated | CS6; Override toggles hold a static pose. |
| Pressure | percent | 100 | 0 – 100 (not stated) | CS6 Brush Pose. |

## Algorithms & pipeline

The exact Adobe brush engine is closed. What follows is a **behavioral-parity
proposal**, marked by confidence; it follows the observable contract above and
standard dab-based rendering, not Adobe internals.

1. **Stroke capture.** Tablet/mouse samples are collected as
   `(x, y, pressure, tilt, rotation, timestamp)` at the event rate.
2. **Spline.** Samples are fit to a smooth path (Catmull-Rom / centripetal) so
   fast strokes are not polygonal. *(inferred)*
3. **Dab placement.** Dabs are stamped along the path at an arc-length interval
   of `spacing × diameter`. A stroke that is shorter than one interval still
   stamps the start dab. When spacing is disabled, interval follows cursor speed.
4. **Dab footprint (round tip).** A radial falloff; `hardness` sets the radius
   fraction inside which coverage is 1.0, with a smooth (cosine or smoothstep)
   ramp to 0 at the edge. Anti-aliasing is the same falloff. *(inferred;
   behavioral parity only.)*
5. **Flow vs Opacity.** Each dab composites `flow` × footprint toward the brush
   color; the accumulated stroke buffer is clamped to `opacity` for the stroke
   and flushed to the layer. Airbrush additionally advances accumulation on a
   timer while the button is held, even without motion.
6. **Blending.** Per-pixel blend uses the selected paint mode. `Normal`,
   `Dissolve` (stochastic per-pixel), `Behind` (only where destination alpha <
   1), `Clear` (destination alpha → 0) are tool-specific; the remaining modes
   follow the standard separable/non-separable blend formulas. Media and printing
   behavior use the same formulas as layer blend modes. Reference: W3C
   *Compositing and Blending Level 1* for the standard mode definitions.
7. **Dynamic sets.** Shape Dynamics, Scattering, Color Dynamics, Transfer, and
   Angular Roundness evaluate a per-dab value from a jitter / control source
   (Off, Fade, Pen Pressure, Pen Tilt, Stylus Wheel, Rotation). CS6 evaluates
   Color Dynamics once per stroke unless `Apply Per Tip`.
8. **Special tips.** Bristle tips model a set of bristles deformed by
   stiffness; erodible tips maintain a wear scalar advanced by contact and
   reduced to original sharpness by `Sharpen Tip`; airbrush tips splat random
   droplets with granularity/spatter parameters. Each is "behavioral parity
   only, algorithm TBD."
9. **Rasterization target.** Dabs are rendered into a per-stroke scratch tile
   (only the stroke's dirty rectangle) and then committed to the layer's tiles.
   This keeps undo cost proportional to the touched area (cross-ref
   `01-architecture/undo-history.md`).

## Rust module mapping

Design proposal. Painting is interactive and time-based, so this proposes a
dedicated `pictura-paint` crate (added to the workspace in
`01-architecture/rust-core-design.md`); it produces `TileDelta`s that the
existing command/history layer commits.

- `pictura-paint::brush` — `BrushTip` (round/captured/bristle/erodible/airbrush
  variants), `BrushPreset`, `BrushEngine::begin_stroke / dab / end_stroke`.
- `pictura-paint::pencil` — `PencilEngine`; aliased coverage, hard edge, Auto
  Erase decision at stroke start.
- `pictura-paint::dynamics` — `DynamicsSet`, `ControlSource`
  (`Off | Fade | PenPressure | PenTilt | StylusWheel | Rotation`), `Jitter`.
- `pictura-paint::scatter` / `::texture` / `::dual_brush` — dynamic-set
  evaluation.
- `pictura-paint::stroke` — `Stroke`, `StrokeSample`, `StrokeBuffer` (sparse
  tile scratch), arc-length dab iterator.
- `pictura-paint::blend` — `PaintMode` blend kernels (shared with `pictura-render`
  for CPU/GPU parity).
- `pictura-core::command` — `PaintStrokeCommand { layer, tip, preset, mode,
  opacity, flow, samples, dirty_tiles }`; one record per completed stroke.

Crossing types: `LayerId`, `Rect`, `ColorSpace`, `BlendMode`, `TileDelta`,
`PixelBuffer` (all defined in `pictura-core`).

## Qt6 component mapping

Widgets, per `01-architecture/qt6-ui-design.md` (Widgets shell, not QML).

- `BrushOptionsBar` (`QWidget`) — hosts mode/opacity/flow/airbrush controls,
  preset picker, size/hardness spins, and the pressure-override toggles.
- `BrushPresetModel` (`QAbstractListModel`) / `BrushPresetView`
  (`QListView`, icon-mode + stroke thumbnails).
- `BrushPanel` (dock, `QWidget`) — stacked Brush Tip Shape and dynamics editors.
- `DynamicsCurveWidget` (`QWidget`) — jitter/control graph editor.
- `BrushTipPreview` (`QWidget`) — live tip rendering for the Brush panel and HUD.
- `BrushPoseEditor` (`QWidget`) — Tilt X/Y, Rotation, Pressure with Override.
- `CanvasView` (`QGraphicsView`) — paints the brush-tip cursor overlay and the
  HUD; document pixels stay in the GPU pipeline (`ARCH-003`/`ARCH-006`).

Per `ARCH-003`, models are GUI-thread-only; stroke rendering requests are queued
to a worker and results are committed on the GUI thread.

## Data-model impact

- **No new PSD/PSB fields.** Brush settings are not stored in the document; only
  the resulting pixel tiles and the layer's existing opacity/blend/mask/lock
  fields change.
- **Presets.** Brush presets save to brush libraries (`.abr`) and to tool presets;
  the CS6 Help says new presets saved from a panel live in a Preferences file
  until explicitly saved to a library. Cross-ref
  `07-color-painting/brush-presets.md` / `10-workflow-io/presets-manager.md`.
- **Undo.** One history state per completed stroke, created on pointer release
  (`01-architecture/undo-history.md`). Reverting a stroke restores the changed
  tiles bit-exactly at the document bit depth.
- **Lock transparency.** `Behind`/`Clear` paint modes require the layer's
  transparency lock to be off; the engine must refuse or ignore them otherwise
  rather than silently producing normal-mode output.
- **Pencil Auto Erase** reads foreground/background colors and writes the
  background color or transparency, so it mutates color channels directly.

## Edge cases

- **Anti-aliasing contract.** Brush @ 100% hardness is anti-aliased; Pencil is
  not. A shared "hardness" slider must not turn the Pencil into a soft brush.
- **32-bit documents.** The CS6 Help notes only Normal, Dissolve, Darken,
  Multiply, Lighten, Linear Dodge (Add), Difference, Hue, Saturation, Color,
  Luminosity, Lighter Color, and Darker Color are available in 32-bit; the paint
  mode list must be filtered.
- **Bitmap / Indexed.** Bitmap mode has no anti-aliasing/hardness and paints
  1-bit; Indexed mode has a single transparency index. Both restrict painting.
  Cross-ref `04-image-ops/image-modes.md`.
- **CMYK / Lab.** Painting is allowed; blend math runs in the document's working
  space. The same mode list applies where Photoshop allows it.
- **Lock Transparency on.** `Behind`/`Clear` are unavailable; a stroke otherwise
  writes only into already-opaque pixels.
- **Group and adjustment layers.** Painting targets a raster layer; on a
  non-pixel layer the command must be refused or rasterize explicitly, matching
  CS6 behavior.
- **Empty / 1-px documents.** Dab placement and dirty-rect math must handle
  zero-area and 1-pixel layers without panicking.
- **Huge documents (PSB).** A 5000 px brush touches many tiles; the stroke
  buffer must be sparse and undo must not copy the whole canvas.
- **Undo mid-stroke.** `Ctrl+Z` while the button is down is undefined; the stroke
  is atomic on release and mid-stroke undo should be ignored or cancel the
  stroke.
- **GPU unavailable.** The stroke must still render through the CPU fallback
  (`01-architecture/gpu-rendering-pipeline.md`); bristle previews degrade.
- **Tablet absent.** Pen-pressure/tilt controls are inert with a mouse; the
  engine falls back to the preset's static values.

## Parity acceptance criteria

1. Given the same brush preset, bounds, spacing, and a scripted sample path,
   Kooka Pictura produces a stroke whose coverage matches CS6 within the tolerance
   set in `11-cross-cutting/testing-strategy.md`, for 8/16/32-bit documents.
2. Given Opacity `O` and no pointer release, repeated back-and-forth passes leave
   the painted area at ≤ `O` coverage; releasing and re-stroking increases it.
3. Given Opacity 100% and Hardness 100%, the Brush is anti-aliased and the
   Pencil is aliased (Pencil edge pixels are exactly foreground or untouched).
4. Given Airbrush on and a stationary held button, coverage increases with time;
   with Airbrush off, a stationary button does not increase coverage.
5. Given a `Behind` or `Clear` mode with Lock Transparency on, the mode is
   unavailable/refused; with it off, `Clear` drives alpha to 0.
6. Given a 32-bit document, the paint-mode menu offers only the modes listed for
   32-bit above, and selecting an unsupported mode is impossible.
7. Given a stroke touching K tiles, the commit references only those K tiles in
   the undo record (store instrumentation), not the full canvas.
8. Given brush size 5000 px, the stroke is accepted and does not allocate a
   full-canvas buffer at PSB dimensions.
9. Given Color Dynamics and CS6 defaults, color variation is constant within a
   stroke and varies between strokes; enabling `Apply Per Tip` varies within a
   stroke.
10. Given `[`, `]`, `Shift+[`, `Shift+]`, number keys, and `Shift+number`, size,
    hardness, opacity, and flow change by the documented amounts.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official Photoshop CS6 Help reference. Sections used: "Painting tools" /
  "Paint tool options" (opacity/flow semantics, airbrush, Pencil Auto Erase,
  cursors, pp. 400–402); "Brush presets" (p. 403); "Standard brush tip shape
  options" (p. 406); "Bristle tip shape options" (p. 407); "Erodible tip options
  | CS6" (p. 408); "Airbrush tip options | CS6" (p. 408); "Brush pose options |
  CS6" (p. 408); "Other brush options" (p. 409); "Brush scattering" (p. 409);
  "What's new in CS6" painting/patterns and JDI entries (pp. 6–8); blend-mode
  list and 32-bit restriction (p. 47 area, "Blending mode descriptions").
- `https://html.duckduckgo.com/html/?q=Photoshop+CS6+erodible+brush+tips+airbrush+brush+pose+new+features` —
  search results corroborating that erodible and airbrush tips are new in CS6
  (Adobe/Envato Tuts+, AGI Training, Simon Sez IT snippets). Search-results page,
  not a primary reference.
- `https://www.w3.org/TR/compositing-1/` — W3C *Compositing and Blending Level 1*:
  standard separable and non-separable blend-mode formulas used as the blend
  definition for modes whose Adobe implementation is not documented.

## Open questions

- **Per-preset defaults** (spacing, roundness, dynamics) for the shipped CS6
  brushes are not itemized in the Help PDF; they live in Adobe's `.abr` assets.
  *Resolves with:* a CS6 preset export or a documented default table.
- **Exact numeric ranges** for Spacing, Scatter, Count, Jitter, Angle, and Brush
  Pose are not stated in the CS6 Help. *Resolves with:* a CS6 UI capture or an
  authoritative reference.
- **Brush falloff shape** (cosine vs smoothstep vs a proprietary curve) is
  inferred; only anti-aliasing behavior is documented. *Resolves with:*
  pixel-diff calibration against CS6 strokes.
- **Erodible wear model and bristle deformation** are behavioral-parity only.
  *Resolves with:* publicly documenting constraints or an accepted tolerance.
- **Dab-level color dynamics** exact jitter distribution is unspecified.
  *Resolves with:* statistical comparison of CS6 stroke pixels.
- **Whether the Brush tool has a "Sample All Layers"** — the CS6 Help lists that
  option for Mixer Brush, Eraser variants, Clone Stamp, and others, but not for
  Brush/Pencil. *Resolves with:* a CS6 options-bar screenshot.
