# Eyedropper, Color Sampler, and Ruler Tools

- **Spec ID:** `TOOL-015` (Eyedropper); `TOOL-016` (Color Sampler); `TOOL-017` (Ruler)
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the Eyedropper gained the OpenGL **sampling ring**, and CS6 (Creative Cloud) added `Current Layer and Below` and `Ignore Adjustment Layers` to the Sample menu plus sample-size entries in the right-click context menu. The Color Sampler and Ruler are carried over.
- **Depends on:** `07-color-painting/color-picker.md`, `02-ui-ux/panels/info-panel.md`, `01-architecture/document-model.md` (`ARCH-002`), `TOOL-012`.

## CS6 behavior

### Eyedropper tool (`TOOL-015`)

The Eyedropper samples color to set the foreground color (default) or background
color (`Alt`/`Option`-click). It can sample from the active image or anywhere on
screen (press the mouse button over the image, drag to the target, release).
Source: CS6 reference, "Choose colors with the Eyedropper tool".

- **Sample Size**: Point Sample, 3 by 3, 5 by 5, 11 by 11, 31 by 31, 51 by 51,
  101 by 101 (averages of the specified pixel area).
- **Sample**: All Layers / Current Layer (CS6 CC adds Current Layer and Below
  and Ignore Adjustment Layers).
- **Show Sampling Ring**: a ring that previews the sampled color above the
  current foreground color; requires OpenGL.
- Holding `Alt`/`Option` temporarily invokes the Eyedropper from any painting
  tool. A painting tool + `Shift+Alt`/`Shift+Option` + right-drag selects a
  foreground color from the picker.
- The Eyedropper (black/white point in Levels, etc.) exposes sample size in
  context menus.

### Color Sampler tool (`TOOL-016`)

Up to **four** Color Samplers display color information for one or more
locations; they are saved in the image and persist across close/reopen. Source:
CS6 reference, "Color samplers and Info panel" / "Adjusting color samplers".

- Place with the Color Sampler tool click, or the Eyedropper tool + `Shift`
  click. Sample size (Point Sample or average) applies.
- Values appear in the lower half of the **Info** panel while adjusting color
  (Properties panel in CS6) and generally. Move a sampler by dragging; delete one
  by dragging it out of the window, `Alt`/`Option`-click (scissors cursor),
  `Alt+Shift`-click while an adjustment dialog is open, or **Clear** in the
  options bar to remove all.
- Show/hide samplers with `View > Extras`. Toggle the Info-panel readout via the
  panel menu (`Color Samplers`); change the readout color space by holding the
  color-sampler icon in the Info panel and choosing a space.
- Info-panel readouts are affected by the Extras/All/None commands (though color
  samplers are not an entry in the Show submenu).

### Ruler tool (`TOOL-017`)

The Ruler measures distances, locations, and angles. Source: CS6 reference,
"Positioning with the Ruler tool".

- Drag from start to end; `Shift` constrains to 45° increments. The options bar
  and Info panel show start X/Y, horizontal W, vertical H, angle A, total length
  D1, and — when a protractor is made — D1 and D2. All measurements except the
  angle use the Units & Rulers preference unit.
- **Protractor**: `Alt`/`Option`-drag from one end of an existing line, or
  double-click the line and drag; `Shift` constrains to 45° multiples.
- Edit: drag an endpoint to resize; drag the line (away from endpoints) to move;
  drag it out of the image or click **Clear** to remove. Selecting the Ruler
  shows any existing measuring line.
- **Straighten**: the Ruler provides a Straighten option that aligns the image
  with a horizontal/vertical feature; alternatively draw the line and choose
  `Image > Image Rotation > Arbitrary` — the required angle is entered
  automatically into the Rotate Canvas dialog.
- Dragging from the ruler intersection (upper-left) sets the ruler origin, which
  can snap to guides, slices, and grid lines.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Toolbox (`I` group) | Eyedropper | `I` | Grouped with Color Sampler, Ruler, Note, Count |
| Toolbox (`I` group) | Color Sampler | `I` cycles | |
| Toolbox (`I` group) | Ruler | `I` cycles / hold Eyedropper | |
| Toolbox | Count (Extended) | `I` cycles | See `TOOL-019` |
| Options bar | Sample Size | — | Point / 3×3 / 5×5 / 11×11 / 31×31 / 51×51 / 101×101 |
| Options bar | Sample | — | All Layers / Current Layer (+ CS6 CC additions) |
| Options bar | Show Sampling Ring | — | Requires OpenGL |
| Options bar | Clear (samplers) | — | Delete all |
| Options bar (Ruler) | Straighten | — | Align to drawn line |
| Info panel | Color readouts | — | First/second readout; sampler values in lower half |
| Info panel menu | Color Samplers | — | Show/hide sampler info |
| Menu | `View > Extras` / `View > Show > Notes` | — | Sampler/ruler visibility |
| Menu | `Image > Image Rotation > Arbitrary` | — | Straighten via angle |
| Context | Plus/minus eyedropper | `Shift` / `Alt` | In Color Range and adjustments |
| Context | Select background color | Eyedropper + `Alt`-click | |
| Context | Add color sampler | Eyedropper + `Shift`-click | |
| Context | Delete color sampler | Color Sampler + `Alt`-click | |
| Context | Make protractor | Ruler + `Alt`-drag end | |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Sample Size | Enum | Point Sample | Point, 3×3, 5×5, 11×11, 31×31, 51×51, 101×101 | Average of N×N |
| Sample | Enum | All Layers | All Layers / Current Layer / Current Layer and Below / Ignore Adjustment Layers | Last two CS6 CC |
| Show Sampling Ring | Bool | Off | On/Off | OpenGL required |
| Color samplers | Count | 0 | 0–4 | Saved with image |
| Sampler readout space | Enum | Per Info-panel setting | RGB / HSB / CMYK / Lab / Grayscale / 8-/16-/32-bit | |
| Ruler origin | Point | Upper-left | Ruler intersection drag | Snaps to guides/slices/grid |
| Measurement units | Enum | Units & Rulers pref | px, inches, cm, mm, points, picas, % | Angle always degrees |
| Straighten angle | Angle | From line | Continuous | Entered into Rotate Canvas |
| Constrain | Modifier | — | `Shift` = 45° increments | Ruler and protractor |

## Algorithms & pipeline

- **Sample size average**: arithmetic mean of the N×N pixel neighborhood,
  computed in the document's working space and displayed per the Info-panel
  readout space. Point Sample reads the single clicked pixel.
- **Sample scope**: `All Layers` flattens visible layers at the sample point
  (respecting layer visibility, masks, and blend modes); `Current Layer` reads
  the active layer only. CS6 CC's `Current Layer and Below` stacks the active
  layer and layers beneath; `Ignore Adjustment Layers` bypasses adjustment
  layers. Exact compositing order follows the normal composite pipeline.
- **Color Sampler**: stores up to four document-space points; each is rendered
  and shown in the Info panel. Values are recomputed when the document changes
  (adjustment previews included during modal adjustments).
- **Ruler**: start/end points define W, H, D1 = √(W²+H²), angle
  A = atan2(dy, dx); a protractor adds a second segment (D2) and the angle
  between the two segments. Straighten maps the line's angle to a canvas
  rotation (`-angle`), auto-growing the canvas like Crop straighten (`TOOL-011`).
- Behavioral parity only, algorithm TBD for the exact averaging kernel and
  whether Adobe averages premultiplied values.

## Rust module mapping

Proposed:

- `pictura-core::tools::eyedropper` — `SampleSize` enum, `sample(doc, point,
  size, scope) -> Color`; `SampleScope` enum.
- `pictura-core::document::ColorSampler` — `{ point: Point, id: u8 }`; document
  holds `Vec<ColorSampler>` limited to 4; `sampler_readouts(doc)`.
- `pictura-core::tools::ruler` — `RulerLine { start, end, protractor: Option<Point> }`,
  `measure() -> RulerMeasurement { x, y, w, h, angle, d1, d2 }`.
- `pictura-core::tools::straighten` — angle → rotation command, shared with
  `TOOL-011`.
- Boundary types: `Point`, `Color`, `RulerMeasurement`, `SampleScope`,
  `SampleSize`.

## Qt6 component mapping

- `EyedropperTool` / `ColorSamplerTool` / `RulerTool` — pointer handlers on the
  canvas scene; eyedropper optionally draws the sampling ring via a QML/overlay
  shader (GPU) with a CPU fallback.
- `ToolOptionsBar` variants (`QWidget`) — sample-size and sample-scope combos,
  Show Sampling Ring toggle, Clear; Ruler Straighten button and measurement
  readout.
- `SamplingRingOverlay` — GPU ring preview above the cursor; disabled when
  OpenGL/QRhi is unavailable.
- `InfoPanel` (`QWidget`) — first/second color readouts and the color-sampler
  list; color-space menu per sampler.
- `RulerOverlay` — draws the measuring line, endpoints, and protractor; angle
  label.

## Data-model impact

- Color samplers are document state (up to four points), serialized with the
  PSD and restored on open.
- Ruler measuring lines are transient view state (shown when the Ruler is
  selected); CS6 does not save them with the document.
- Eyedropper settings (sample size/scope/ring) are tool preferences persisted in
  the app preferences store, not the document.
- Undo: sampler add/move/delete can be recorded as document edits or kept out of
  history; CS6 behavior on undo of a sampler is unverified (Open questions).
  Ruler edits are not undoable document edits.

## Edge cases

- **Empty/1-px document**: sample size clamps to available pixels; Point Sample
  is exact.
- **32-bit HDR**: Info panel must support 32-Bit readout (`Eyedropper` icon in
  the Info panel → 32-Bit); sampling returns float values.
- **CMYK/Lab/Indexed/Bitmap**: readout space may not equal the document space;
  convert for display, keep native for painting where required.
- **GPU-unavailable / no OpenGL**: Show Sampling Ring must degrade gracefully
  (disabled), matching CS6's OpenGL requirement.
- **Hidden layers / masks / blending**: `All Layers` respects visibility and
  masks; a fully masked/hidden pixel samples as transparent/background.
- **Sampler limit**: the fifth placement is rejected/clamped at four.
- **Adjustment dialogs**: samplers update live and `Alt+Shift`-click deletes
  while a modal adjustment is open.
- **Units**: changing the Info-panel units also changes Ruler units; angle is
  always degrees.
- **Protractor**: only available from an existing line; D2 only reported with a
  protractor.

## Parity acceptance criteria

1. Given a flat-color 8-bit RGB document, Point Sample returns exactly that
   pixel's RGB; a 5×5 Average at the center of a 1-px-wide feature returns the
   documented arithmetic mean within 1 LSB per channel.
2. Given two stacked layers with different colors at a point, `All Layers`
   returns the composite color and `Current Layer` returns the active layer's
   color.
3. Given four placed Color Samplers, placing a fifth is rejected; each sampler's
   value appears in the Info panel and updates after an image edit.
4. Given a sampler saved in a PSD, reopening restores it at the same
   coordinates with the same value.
5. Given a Ruler drag from (0,0) to (3,4), the options bar shows W=3, H=4,
   D1=5 and A=53.13° within 0.1°.
6. Given `Shift` held during a Ruler drag, the angle snaps to a multiple of 45°.
7. Given an `Alt`-drag protractor, D1, D2, and the included angle are reported.
8. Given a Ruler line along a feature rotated by θ, Straighten rotates the
   canvas by −θ within 0.1° and adds exactly one history state.
9. Given no OpenGL/QRhi, Show Sampling Ring is disabled and the Eyedropper still
   samples correctly.

## Sources

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  "Choose colors with the Eyedropper tool" (sample sizes, Sample menu, Show
  Sampling Ring requires OpenGL, background color, temporary eyedropper);
  "Color samplers and Info panel" (up to four, persistence, sample size);
  "Adjusting color samplers" (move/delete/hide, readout space, Extras);
  "Positioning with the Ruler tool" (measurements, protractor, edit line,
  Straighten/Arbitrary rotation, ruler origin); "Straighten an image" (Ruler
  Straighten option); "What's new — Eyedropper" (ignore adjustment layers,
  current layer and below, context-menu sample sizes); key shortcuts for
  color/measurement.
- `https://www.bapugraphics.com/blog/adobe-photoshop-note-tool` — cross-toolbox
  grouping context (Eyedropper group) and note-tool reference; used mainly for
  `TOOL-018`.

## Open questions

- **Does CS6 Standard expose `Current Layer and Below` / `Ignore Adjustment
  Layers`**, or are these Creative Cloud-only? The reference labels them under
  the CS6 "What's new" item without an edition tag. *Resolves with:* CS6 vs
  CS6 CC options-bar capture.
- **Exact averaging semantics** (premultiplied alpha? linear vs. working space?)
  for Sample Size > 1. *Resolves with:* controlled test images on CS6.
- **Are color samplers undoable** in CS6? *Resolves with:* CS6 observation.
- **PSD serialization keys** for color samplers. *Resolves with:* PSD
  file-format spec plus a CS6-saved sample.
- **Ruler Straighten button availability** in the CS6 Ruler options bar (the
  reference documents both a Straighten option and the Arbitrary menu route).
  *Resolves with:* CS6 UI capture.
- **32-bit Info-panel readout defaults** and whether ring preview works in
  32-bit. *Resolves with:* CS6 observation.
