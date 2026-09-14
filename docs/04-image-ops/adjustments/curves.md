# Curves Adjustment

- **Spec ID:** `ADJ-002`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the curve engine is unchanged, but CS6 moves the controls and the **Preset menu** from the CS5 Adjustments panel into the new **Properties panel**, keeps the adjustment icons permanently visible in the Adjustments panel, and folds in the improved `Auto` path.
- **Depends on:** `ADJ-000` adjustments-overview, `ADJ-001` levels, `01-architecture/color-management.md` (`ARCH-007`), `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/undo-history.md` (`ARCH-009`), `04-image-ops/image-modes.md` (`IMG-004`), `04-image-ops/bit-depth-and-conversion.md` (`IMG-005`), `05-layers/adjustment-layers.md`, `03-tools/eyedropper-color-sampler-ruler.md`.

> Module and widget names are **design proposals**. Facts from the fetched CS6
> Help are attributed in `## Sources`; other statements are marked *(inferred)*.

## CS6 behavior

"In the Curves adjustment, you adjust points throughout an image's tonal range.
Initially, the image's tonality is represented as a straight diagonal line on a
graph." For an RGB image the graph shows highlights upper-right and shadows
lower-left; the **horizontal axis is input levels** and the **vertical axis is
output levels**. "The steeper sections of the curve represent areas of higher
contrast while flatter sections represent areas of lower contrast."

The CS6 Help documents the following behavior:

- **Editing a point** — "Click directly on the curve line and then drag the
  control point"; drag up/down to lighten/darken, left/right to increase/decrease
  local contrast. "You can add up to 14 control points to the curve." Points
  "remain anchored until you move them", so an edit can be localized.
- **On-image adjustment tool** — "Select the On-image adjustment tool and then
  drag in the area of the image you want to adjust", or click to "place control
  points along the curve line".
- **Removing a point** — drag it off the graph, select it and press Delete, or
  Ctrl/Cmd-click it.
- **Input/Output boxes** — click a point and "enter values in the Input and Output
  text boxes".
- **Black/white point sliders and eyedroppers** — "Move the Set Black and White
  Point sliders or use the Eyedropper tools to specify the darkest and lightest
  values in the image."
- **Pencil mode** — "Select the pencil icon and draw a new curve over the
  existing one. When you have finished, click the Smooth the Curve Values icon …
  Clicking more than once continues to smooth the curve further."
- **Auto** — "Click Auto … applies an automatic color correction using the current
  default setting." Alt/Option-click Auto (or the panel menu `Auto Options`) opens
  the Auto Color Correction Options dialog (same as `ADJ-001`).
- **Presets** — "You can save Curves adjustment settings as presets"; the
  Properties panel Preset menu lists the built-in and user presets.
- **Color modes** — "The Curves adjustment can also be applied to CMYK, LAB, or
  Grayscale images. For CMYK images, the graph displays percentages of
  ink/pigment. For LAB and Grayscale images, the graph displays light values."
- **Curve Display Options** — panel menu dialog with:
  - `Light (0-255)` — intensity values with black `(0)` at the lower-left.
  - `Pigment/Ink %` — CMYK percentages with highlights `(0%)` at the lower-left.
  - `Simple Grid` — gridlines at 25% increments.
  - `Detailed Grid` — gridlines at 10% increments.
  - `Show Channel Overlays` — "color channel curves superimposed on the composite
    curve."
  - `Histogram` — the original image histogram behind the graph.
  - `Baseline` — the original tonality as a 45° line.
  - `Intersection Line` — horizontal/vertical alignment guides while dragging.
  - Alt/Option-click the grid to change the gridline increment.
- **Keyboard shortcuts (Curves)** — `Shift+Ctrl/Cmd-click` in the image sets a
  point for the selected color in each component channel (not the composite);
  `Shift-click` selects multiple curve points (selected points fill black);
  clicking the grid or `Ctrl/Cmd-D` deselects; `+`/`-` select the next
  higher/lower point; arrow keys move selected points.

### CS6 curve UI change

In CS5 the curve graph, sliders, and preset list lived in the Adjustments panel.
In CS6 the graph and all controls are in the **Properties panel** adjustment-
controls view, and the Presets are a **Preset menu**; the Adjustments panel keeps
only the adjustment icons. The destructive command
(`Image > Adjustments > Curves`) opens a modal dialog with `Curve Display Options`
expanded inline.

### Built-in presets

CS6 ships Curves presets. Community sources list: **Color Negative**, **Cross
Process**, **Darker**, **Increase Contrast**, **Lighter**, **Linear Contrast**,
**Medium Contrast**, **Negative**, **Strong Contrast** *(community; not fetched
from Adobe — see Open questions)*. The Help only confirms that Curves presets
exist and that custom presets can be saved.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Adjustments > Curves` | Menu (dialog) | `Ctrl+M` | destructive; modal; `Curve Display Options` inline |
| `Layer > New Adjustment Layer > Curves` | Menu | — | non-destructive; New Layer dialog |
| Adjustments panel → Curves icon | Button | — | creates a Curves adjustment layer |
| Properties panel | Dock | — | graph, Channel combo, Auto button, Preset menu, black/white/gray point tools, on-image tool, pencil + smooth, clip button, reset, visibility, delete |
| Panel menu (Properties) | Menu | — | `Auto Options`, `Curve Display Options…`, `Save Preset`, `Load Preset`, `Show Clipping For Black/White Points`, `Add Mask by Default` |
| Preset menu | Pop-up | — | built-in + user Curves presets |
| On-image adjustment tool | Canvas interaction | — | click/drag the image to add/move points |
| Curves shortcut set | Keyboard | see `## CS6 behavior` | set/select/move/delete points |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Channel | Enum | RGB (composite) | composite, R/G/B, C/M/Y/K, L/a/b, Gray | per-channel curves |
| Control points | Points (input, output) | 2 endpoints | **2 … 14** (DOM `adjustCurves`) | endpoints movable (black/white point) |
| Point coordinate | Number | line-dependent | per grid mode: 0–255 or 0–100% | shown in Input/Output boxes |
| Input / Output boxes | Integer/Percent | diagonal | grid-range values | exact numeric point entry |
| Pencil mode | Mode | off | on / off | freehand 256-step draw |
| Smooth the Curve Values | Action | — | repeatable | each click smooths further |
| Set Black Point slider | Number | grid min (0) | grid range | sets input black point |
| Set White Point slider | Number | grid max (255) | grid range | sets input white point |
| Set Black/White/Gray Point eyedropper | Tool | — | — | sampled pixel maps to target gray value |
| Auto | Action | — | current Auto options | shares Auto Color Correction Options (`ADJ-001`) |
| Grid mode | Enum | Light (0-255) / Pigment (CMYK) | Light, Pigment/Ink % | depends on color mode |
| Grid spacing | Enum | Simple or Detailed *(default unverified)* | Simple (25%), Detailed (10%) | Alt-click grid changes increment |
| Histogram | Bool | on | on / off | backdrop |
| Baseline | Bool | on | on / off | 45° reference |
| Channel Overlays | Bool | off | on / off | show all channel curves |
| Intersection Line | Bool | off | on / off | drag alignment guides |
| Smoothing | Float | 0 | 0…1 (inferred) | pencil-mode smoothing amount |

The **type of curves supported** is not stated by the Help; the DOM `adjustCurves`
takes an array of 2–14 `Point`s. Photoshop supports smooth spline curves and, via
pencil mode, an arbitrary hand-drawn curve. The Help does not document parametric
curves (the Targeted Adjustment tool set, `Ctrl+Alt+Shift+H/S/L`, is a CS6-era
feature surfaced in the keyboard table; whether it belongs to Curves or the
Properties/TAT surface is not established here).

## Algorithms & pipeline

Behavioral parity; the exact Adobe curve-interpolation math is closed.

### Curve evaluation

A curve is an ordered set of control points `(x_i, y_i)`, `i = 0…n-1`, `n ≤ 14`,
`x` = input, `y` = output, both in the grid range. Endpoints are initially
`(min, min)` and `(max, max)` (the 45° baseline). The curve defines a monotone map
`y = f(x)` used as a per-channel LUT.

Proposed interpolation (matches Photoshop's monotone, non-overshooting behavior):

- **Endpoints / monotone cubic Hermite (Fritsch–Carlson)** through the sorted
  control points, with the tangent limiter that prevents overshoot. This yields a
  smooth curve that passes through every control point, is monotone between
  points when the point sequence is monotone, and reduces to the straight line for
  two points. *(inferred)* Adobe's exact spline is not published.
- **Pencil mode** produces a dense hand-drawn polyline (one `y` per input step,
  typically 256 steps). `Smooth the Curve Values` applies a repeated moving-
  average/relaxation to the drawn polyline; each click smooths further. The drawn
  curve is then sampled into the same LUT.

### LUT construction and application

```text
for depth in {8,16}:
    for i in 0..=max:
        x = i / max
        y = clamp( f(x * range + x0), floor, ceil )     # curve evaluated in grid units
        lut[i] = round(y)
apply: out = lut[in]                                      # per channel
for 32-bit (if supported): out = f(in) directly           # no integer LUT
```

- Curves is **unavailable at 32 bpc** per the CS6 32-bpc feature list (it is absent
  while Levels/Exposure are named). Treat as 8/16-bit only unless disproved.
- Each channel has its own curve; the composite curve is duplicated onto every
  color channel.

### Black / white / gray point tools

- The **Set Black Point / Set White Point sliders** set the input `x` of the two
  endpoint points directly.
- The **eyedroppers** are the same three tools as Levels: black point (map the
  sampled pixel to the target black), white point (to target white), gray point
  (neutralize a cast by moving each channel's midtone). The Help states eyedropper
  use "undoes any previous adjustment" and is best done first. Gray point
  unavailable in Grayscale.
- **Show Clipping For Black/White Points** previews clipped regions.

### Auto

The Curves `Auto` button uses the **Auto Color Correction Options** currently
saved as default (Algorithms: Enhance Monochromatic Contrast / Enhance Per Channel
Contrast / Find Dark & Light Colors; clip percentages; target colors; Snap Neutral
Midtones). Applying Auto against the current curve rebuilds the curve from the
computed black/white/(gray) points — see `ADJ-001` for the full model.

## Rust module mapping

Design proposal.

- `pictura-core::adjust::curves` — `CurvePoint { input: f32, output: f32 }`,
  `Curve { points: Vec<CurvePoint> }`, `CurvesParams { channel_curves:
  ChannelMap<Curve>, mode: Smooth | Pencil, smoothing: f32 }`.
- `pictura-image::adjust::curves` — `interpolate(curve) -> SmallVec<f32>` (monotone
  cubic Hermite), `smooth_pencil(points, passes)`, `build_lut(CurvesParams,
  depth) -> ToneLut`.
- `pictura-core::adjust::auto` — shared with `ADJ-001`; `CurvesParams` rebuilt
  from `AutoResult` endpoints.
- `pictura-image::adjust::eyedropper` — shared with `ADJ-001`; sets curve endpoints.
- Crossing types: `Curve`, `ToneLut`, `ChannelMap<T>`, `Scalar`, `Rect`.

## Qt6 component mapping

Design proposal.

- `CurveEditorWidget` (`QWidget`, custom paint) — the graph, histogram backdrop,
  baseline/intersection lines, channel overlays, draggable control points, and
  the smooth/pencil mode. Candidate for a `QQuickWidget` + QML canvas if GPU
  drawing is preferred, but a Widgets paint path keeps hit-testing and keyboard
  navigation simple.
- `CurvesPropertiesWidget` (`QWidget`) — Channel combo, Auto button, Preset menu
  trigger, on-image tool toggle, pencil + `Smooth the Curve Values`, Input/Output
  `ScrubSpinBox`es for the selected point, black/white/gray eyedropper buttons.
- `CurveDisplayOptionsDialog` (`QDialog`) — Light/Pigment, Simple/Detailed grid,
  Histogram, Baseline, Channel Overlays, Intersection Line.
- `CurvesPresetModel` — built-in + user presets for the Preset menu.
- Shared `AutoCorrectionDialog` and `ChannelSelectorCombo` from `ADJ-001`.

The on-image adjustment tool is a mode on the canvas cursor: dragging dispatches
image coordinates to the curve editor, which inserts/moves a point from the
sampled color.

## Data-model impact

- **PSD key `curv`** stores the Curves adjustment parameters (per-channel point
  lists) for an adjustment layer (`ARCH-008`).
- `CurvesParams` is typed on the adjustment node; presets persist in the preset
  store, not the PSD.
- Undo: adding/moving/removing a point during a drag coalesces into one history
  state at commit; pencil drawing is one state; Auto/eyedropper is one state.
  Destructive Curves stores the pre-edit tiles.
- The endpoint black/white-point state is part of the curve (first/last points),
  so it round-trips; a separate "clipped" flag is not needed.
- 32-bit is out of scope for Curves (see Edge cases).

## Edge cases

- **Minimum/maximum points** — 2 points is the identity (or a straight line if the
  endpoints were moved); 14 is the documented maximum. The UI must prevent adding
  a 15th and must not drop endpoints.
- **Duplicate / out-of-order inputs** — two points at the same input, or a point
  dragged past its neighbour, must be handled (merge or clamp) to keep a function.
- **Pencil mode then smooth** — smoothing must remain monotone-capable and must
  not overshoot beyond the grid; repeated smoothing tends toward a straight line.
- **CMYK** — reverse graph orientation (0% ink lower-left); the interpolation is
  the same but the LUT is in ink units, so "lighten" means *less* ink.
- **Lab** — L is `0…100`; a/b are signed and not appropriate for black/white-point
  tools; restrict appropriately or map to the signed range.
- **Grayscale** — single curve; gray-point eyedropper unavailable.
- **32-bit** — not on the documented list; if CS6 does allow it, integer LUTs are
  invalid and the curve must be evaluated in float.
- **Bitmap / Indexed** — unavailable.
- **No-op curve** — identity must be detected and skipped to avoid a needless
  history state.
- **Empty / 1-px documents** — no panic; LUT builds over the full range.
- **Huge documents** — LUT once, apply per tile.
- **GPU unavailable** — CPU LUT; identical for 8/16-bit.
- **Undo mid-drag** — coalesce; released edits atomic.
- **Channel count** — an adjustment layer can carry a curve per color channel but
  not per alpha/spot channel.

## Parity acceptance criteria

1. Given a two-point identity curve, applying it leaves the image bit-exact
   unchanged at 8 and 16 bpc.
2. Given a control point at input `x` with output `y`, the pixel whose input
   equals `x` maps to `y` within ±1 code (8-bit).
3. Given an S-curve with points in the shadows/highlights, midtone contrast
   increases (the slope between the midtone points is > 1) and no region of the
   curve overshoots its neighbouring control points.
4. Given more than 14 points, the UI refuses to add the 15th.
5. Given pencil mode, drawing a hand curve then clicking `Smooth the Curve Values`
   n times produces a progressively flatter/monotone curve.
6. Given `Show Channel Overlays`, all color-channel curves are drawn over the
   composite; toggling it hides them.
7. Given `Light (0-255)` vs `Pigment/Ink %`, an RGB vs CMYK document shows the
   corresponding axis orientation and units.
8. Given the Set White Point slider moved to 243, inputs ≥243 map to the grid max.
9. Given the Set Gray Point eyedropper on a cast patch, the patch becomes neutral
   within tolerance; the tool is unavailable on Grayscale.
10. Given Auto with default options, the resulting curve's endpoints match the
    Auto-computed black/white points within tolerance.
11. Given a saved Curves preset, re-applying it to another image reproduces the
    same curve parameters, and `Ctrl+Z` after applying produces exactly one
    history state.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, extracted with `pdftotext -layout`. Established:
  "Curves overview" (diagonal baseline, axis meaning, steep = high contrast);
  "Adjust image color and tone with Curves" (on-image tool click/drag, up to 14
  control points, anchored points, removal gestures, Input/Output boxes, black/
  white sliders, set Black/White Point sliders, eyedroppers, pencil icon +
  `Smooth the Curve Values` repeatable); removal methods; "Set Curve display
  options" (Light 0-255, Pigment/Ink %, Simple Grid 25%, Detailed Grid 10%, Show
  Channel Overlays, Histogram, Baseline, Intersection Line, Alt-click grid);
  "Apply an Auto correction in Curves" and the Auto Options reference; the CMYK/
  LAB/Grayscale applicability and graph-unit statement; the "Set black and white
  points … black point and white point sliders" section with clipping preview;
  and "Keyboard shortcuts: Curves".
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/ArtLayer/adjustCurves.html`
  — scripting reference: `adjustCurves(curveShape)` with "The number of points
  must be between 2 and 14."
- `http://www.photoshopforphotographers.com/3101-1901/Help_guide/tp/propertiespanel.html`
  — CS6 Properties panel: Preset popup for pre-supplied adjustment settings and
  `Save Preset`, confirming the CS6 preset relocation.
- `https://www.apogeephoto.com/photoshop-cs6-cc-the-adjustments-panel-and-properties-panel`
  — CS6 Adjustments panel creates the layer; controls appear in the Properties
  panel.

Secondary / community (surfaced by search; **not fetched this pass**): the
Curves preset names (Color Negative, Cross Process, Darker, Increase Contrast,
Lighter, Linear Contrast, Medium Contrast, Negative, Strong Contrast) from
community tutorials; and the `Ctrl+Alt+Shift+H/S/L` Targeted Adjustment shortcuts
from the CS6 Help keyboard table (read, but their owning feature is unverified).

## Open questions

- **Exact curve interpolation** (monotone cubic vs Catmull-Rom vs Adobe's own)
  and the tangent/limiter. *Resolves with:* sampling CS6 Curves output on a ramp
  and fitting.
- **Exact pencil smoothing kernel** and how many passes map to one click.
  *Resolves with:* CS6 pencil-then-smooth capture.
- **Built-in CS6 Curves presets** (names and parameter values). *Resolves with:*
  a CS6 Properties Preset menu capture (community list needs confirmation).
- **Whether Curves is truly unavailable at 32 bpc**, and if it is, whether the
  destructive command differs from the adjustment layer. *Resolves with:* a
  32-bpc CS6 menu test.
- **The `Smooth` parameter range** and whether `Smooth the Curve Values` is a
  stored property or a one-shot action. *Resolves with:* CS6 observation.
- **The Targeted Adjustment tool** (`Ctrl+Alt+Shift+H/S/L`) — which adjustment(s)
  own it and whether it edits Curves. *Resolves with:* CS6 UI exploration.
- **Whether black/white point sliders are stored as endpoint control points** in
  the `curv` PSD block or as separate fields. *Resolves with:* the PSD spec plus a
  CS6-saved file.
- **CMYK/Lab grid orientation and signed-channel handling**. *Resolves with:* a
  CS6 mode test.
- **Gray-point math for Curves** (per-channel gamma solve). *Resolves with:* a
  calibration against Levels.
