# Brightness/Contrast Adjustment

- **Spec ID:** `ADJ-003`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the algorithm is the same as CS5; CS6 moves the controls into the **Properties panel**, adds the adjustment to the improved **Auto** path, and (already in CS5) exposes the **Use Legacy** checkbox.
- **Depends on:** `ADJ-000` adjustments-overview, `ADJ-001` levels, `01-architecture/color-management.md` (`ARCH-007`), `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/undo-history.md` (`ARCH-009`), `04-image-ops/bit-depth-and-conversion.md` (`IMG-005`), `05-layers/adjustment-layers.md`.

> Module and widget names are **design proposals**. Facts from the fetched CS6
> Help are attributed in `## Sources`; other statements are marked *(inferred)*.

## CS6 behavior

"The Brightness/Contrast adjustment lets you make simple adjustments to the tonal
range of an image. Moving the brightness slider to the right increases tonal
values and expands image highlights, to the left decreases values and expands
shadows. The contrast slider expands or shrinks the overall range of tonal values
in the image."

Two algorithms are selectable via the **Use Legacy** checkbox:

- **Normal (default) mode** — "Brightness/Contrast applies proportionate
  (nonlinear) adjustments to image layer, as with Levels and Curves adjustments."
  This is the CS3-and-later algorithm.
- **Use Legacy** — "Brightness/Contrast simply shifts all pixel values higher or
  lower when adjusting brightness. Since this can cause clipping or loss of image
  detail in highlight or shadow areas, using Brightness/Contrast in Legacy mode is
  not recommended for photographic images (but can be useful for editing masks or
  scientific imagery)."
- "Use Legacy is automatically selected when editing Brightness/Contrast
  adjustment layers created with previous versions of Photoshop." (i.e. the
  legacy flag is stored per adjustment layer, and old layers open in legacy mode.)

Note on the task phrasing: the CS6 Help's normal mode is **nonlinear** and legacy
is the **simple additive shift**; the mapping is the opposite of "simple linear
vs legacy non-linear". This spec follows the Help.

Reachability: adjustment layer (Brightness/Contrast icon; `Layer > New Adjustment
Layer > Brightness/Contrast`) or destructive command
(`Image > Adjustments > Brightness/Contrast`, which "makes direct adjustments to
the image layer and discards image information"). There is no channel selector —
the adjustment always acts on all color channels identically (there is no
composite/per-channel split), so it cannot introduce a color cast by construction
*(inferred from the absence of a Channel control)*.

Brightness/Contrast is **not** on the CS6 32-bpc feature list, so it is
unavailable at 32 bpc *(documented by absence)*.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Adjustments > Brightness/Contrast` | Menu (dialog) | — | destructive; `Legacy` checkbox + `Preview` |
| `Layer > New Adjustment Layer > Brightness/Contrast` | Menu | — | non-destructive; New Layer dialog |
| Adjustments panel → Brightness/Contrast icon | Button | — | creates the adjustment layer |
| Properties panel | Dock | — | Brightness slider, Contrast slider, Use Legacy checkbox, Auto button, Preset menu, clip button, reset, visibility, delete |
| Panel menu | Menu | — | `Auto Options`, `Save Preset`, `Load Preset`, `Add Mask by Default`, `Auto-Select …` |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Brightness | Number (slider + box) | 0 | **−150 … +150** (CS6 UI) | positive = lighter; normal mode is nonlinear |
| Contrast | Number (slider + box) | 0 | **−50 … +100** (CS6 UI) | positive = more contrast |
| Use Legacy | Bool | off (auto-on for pre-CS3 layers) | on / off | selects the linear-shift algorithm |
| Auto | Action | — | current Auto options | CS6 improved Auto (see `ADJ-001`) |
| Preset | Enum | — | built-in + user presets | Brightness/Contrast presets are **not** in the help's save-preset list; treat as *(inferred)* — see Open questions |
| Channel | — | all channels | n/a | no channel control |

Scripting DOM discrepancy: `ArtLayer.adjustBrightnessContrast(brightness,
contrast)` documents both parameters as `−100 … 100`. That DOM predates the CS6
non-legacy ranges and likely maps onto the **legacy** path; the CS6 properties
panel exposes the wider `−150…+150` / `−50…+100` UI ranges. Reconcile before
implementing scripting parity.

## Algorithms & pipeline

Behavioral parity; Adobe's exact non-legacy curve is closed. Both modes are
per-channel point operations (same map applied to R, G, B — or C, M, Y, K, or L
in Grayscale), with the output clamped to the channel range.

### Legacy mode (linear shift + linear contrast)

The Help: legacy "simply shifts all pixel values higher or lower when adjusting
brightness". Proposed standard model (normalized `v ∈ [0,1]`):

```text
b = brightness / 150          # normalized [-1, 1]  (exact divisor unverified)
c = contrast   / 100          # normalized [-0.5, 1] (exact divisor unverified)
mid = 0.5
v1 = v + b                    # brightness: pure additive shift
v2 = (v1 - mid) * (1 + c) + mid   # contrast: expand/shrink about mid-grey
out = clamp(v2, 0, 1)         # hard clipping is the documented legacy failure mode
```

- The additive `v + b` shift is why legacy "can cause clipping or loss of image
  detail in highlight or shadow areas"; the Help recommends it only for masks and
  scientific imagery.
- Order (brightness then contrast, or the reverse) and the exact normalization
  divisors are *(inferred)*; both produce the same qualitative behavior. Confirm
  against CS6 (Open questions).

### Normal (non-legacy) mode

The Help says it is "proportionate (nonlinear) … as with Levels and Curves", i.e.
it behaves like a gentle curve rather than a shift: highlights roll off instead of
clipping, and contrast pivots around the midtones without hard-clamping. A
standard implementation that matches this description and is commonly used as a
behavioral model is a monotone curve that fixes `0` and `1` and adjusts the
midtones:

```text
# Define a slope-mapped brightness curve and an S-curve for contrast.
# brightness b in [-1,1] shifts the midpoint of the curve; contrast c in
# [-1,1] scales the local slope. Exact closed form: TBD (Adobe closed).
```

Proposed model (to be calibrated, not asserted):

- **Brightness** — a monotone transfer that passes through `(0,0)` and `(1,1)` and
  moves the midtone like a gamma-like curve, so highlights "expand" rather than
  clip. One defensible form is the Levels-style gamma `y = x^(1/γ)` with
  `γ = 2^(b)`; another is a rational soft-clip. Both avoid the legacy hard clip.
- **Contrast** — an S-curve about `mid`, e.g. a scaled error-function or a
  three-point monotone cubic through `(0,0)`, `(mid, mid)`, `(1,1)` whose central
  slope is `1 + c`. Negative contrast flattens the slope (reduces range).
- The two controls compose; the Help does not specify a single composited formula.

Adobe's exact non-legacy math is closed, so the spec targets **behavioral parity
only**: exactness is defined by the acceptance criteria (no hard clipping, midtone
pivot, monotone), plus a calibration curve fitted to CS6 output where bit-exact
match is desired.

### Auto (CS6)

The CS6 "improved auto corrections" statement names Brightness/Contrast alongside
Levels and Curves. The Auto button applies the current Auto Color Correction
Options (Enhance Monochromatic Contrast / Per Channel / Find Dark & Light, clip
percentages, target colors, Snap Neutral Midtones — see `ADJ-001`) and expresses
the result as a brightness and contrast value. The Help's Brightness/Contrast
section does not itemize the Auto controls; the shared Auto Options dialog governs
them. Exact fitting from auto statistics to `(brightness, contrast)` is closed.

## Rust module mapping

Design proposal.

- `pictura-core::adjust::brightness_contrast` — `BrightnessContrastParams {
  brightness: i16, contrast: i16, use_legacy: bool }`.
- `pictura-image::adjust::brightness_contrast` — `build_lut(params, depth) ->
  ToneLut` with `legacy_map()` and `modern_map()`; the modern map is a monotone
  cubic / gamma composite constructed from `(brightness, contrast)`.
- `pictura-image::adjust::calibration` — a fitted approximation for `modern_map`
  (coefficients versioned with the parity target); the CPU reference calls the
  exact numeric model, the GPU path may sample the same LUT.
- `pictura-core::adjust::auto` — shared; `solve_bc(stats) -> (i16, i16)`.
- Crossing types: `ToneLut`, `Scalar`, `Rect`.

## Qt6 component mapping

Design proposal.

- `BrightnessContrastPropertiesWidget` (`QWidget`) — two slider+box rows
  (`ScrubSpinBox`), the `Use Legacy` checkbox, an `Auto` button, and the Preset
  menu trigger.
- `AutoCorrectionDialog` (`QDialog`) — shared with `ADJ-001`/`ADJ-002`.
- `BrightnessContrastPresetModel` — presets (if CS6 provides them; see Open
  questions).

No custom canvas is required (no curve is shown); numeric fields follow the
options-bar scrubber style used across the app.

## Data-model impact

- **PSD key `brit`** stores the Brightness/Contrast adjustment parameters for an
  adjustment layer (`ARCH-008`), including the legacy flag (the Help guarantees
  the flag is persisted and re-applied on open).
- Adjustment node carries typed `BrightnessContrastParams`; destructive command
  carries a pixel-delta undo record.
- Undo: one state per committed change (drag coalesced); destructive edits retain
  pre-edit tiles.
- No mask/channel-specific fields beyond the standard adjustment-layer mask.
- 32-bit: unavailable; the parameter record need not round-trip at 32 bpc.

## Edge cases

- **Legacy clipping** — legacy mode must clip exactly as CS6 does (hard clamp);
  do not "fix" it, since reproducing the clipping is the point of legacy mode.
- **Pre-CS3 layers** — `use_legacy` defaults to true when loading an adjustment
  layer whose stored form predates the modern algorithm; do not silently convert.
- **Channels** — the same map applies to all color channels; there is no
  per-channel path. This is why B/C cannot neutralize a cast.
- **CMYK / Lab / Grayscale** — the adjustment applies per component; for CMYK a
  raise in brightness increases ink values (darker), so the UI direction must be
  verified per mode *(inferred)*.
- **32-bit** — unavailable per the CS6 list; confirm.
- **Bitmap / Indexed** — unavailable.
- **Identity** — brightness 0, contrast 0 must be an exact no-op and add no
  history state.
- **Extremes** — brightness ±150 with contrast +100 in normal mode must remain
  monotone and in range (soft roll-off); in legacy mode it clips.
- **Empty / 1-px / huge documents** — LUT-once, tile apply; no full-canvas copy.
- **GPU unavailable** — CPU LUT; identical for 8/16-bit.

## Parity acceptance criteria

1. Given brightness 0 / contrast 0 in either mode, the output is bit-exact
   unchanged at 8 and 16 bpc.
2. Given legacy mode with brightness +20 on a grey ramp, every pixel is shifted
   by the same offset and values near white clip to the maximum (reproducing the
   documented clipping).
3. Given normal mode with brightness +150 and contrast +100, the transfer is
   monotone, endpoints stay at the range limits, and no plateau/clip occurs except
   at the extremes.
4. Given normal mode with contrast +50, the slope across the midtones is greater
   than 1 and the endpoints are unchanged.
5. Given a Brightness/Contrast adjustment layer created in an older Photoshop
   format, `Use Legacy` is selected by default on open.
6. Given a gray ramp image, B/C affects all three RGB channels identically, so a
   neutral gray input stays neutral (no cast) in RGB.
7. Given a 32-bpc document, Brightness/Contrast is unavailable.
8. Given the Auto button, the result corresponds to the current Auto Color
   Correction settings within the testing tolerance and adds exactly one history
   state.
9. Given `Ctrl+Z` after a slider drag, the prior `(brightness, contrast)` values
   are restored and the image is bit-exact.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, extracted with `pdftotext -layout`. Established:
  "Apply a Brightness/Contrast adjustment" — the slider behavior ("brightness …
  increases tonal values and expands image highlights … contrast … expands or
  shrinks the overall range"); "In normal mode, Brightness/Contrast applies
  proportionate (nonlinear) adjustments … as with Levels and Curves"; "When Use
  Legacy is selected, Brightness/Contrast simply shifts all pixel values higher or
  lower when adjusting brightness"; the clipping/data-loss warning and the
  masks/scientific-imagery note; "Use Legacy is automatically selected when
  editing … created with previous versions"; the `−150 … +150` Brightness and
  `−50 … +100` Contrast ranges; creation via the Adjustments panel / Layer menu
  and the destructive `Image > Adjustments` path. Also the "What's new" line
  naming Brightness/Contrast among the improved Auto corrections (same PDF).
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/ArtLayer/adjustBrightnessContrast.html`
  — scripting reference: `adjustBrightnessContrast(brightness, contrast)` with
  both ranges `−100 … 100`.

Secondary / community (surfaced by search; **not fetched this pass**): the CS3
introduction of the modern algorithm and the `Use Legacy` toggle (e.g.
photoshopessentials/geraldbakker.nl). These corroborate but are not the basis of
any asserted fact above.

## Open questions

- **Exact normal-mode transfer function** (the curve form and how brightness and
  contrast compose). *Resolves with:* fitting to CS6 output on a grey ramp and a
  colour grid, or Adobe documentation if ever released.
- **Exact legacy normalization** (divisors for `b` and `c`, the applied order,
  and the legacy slider ranges, which may be `−100…100`). *Resolves with:* a CS6
  probe and the scripting DOM.
- **The `−100…100` DOM vs the `−150…+150` / `−50…+100` UI discrepancy** — which
  the CS6 adjustment layer uses. *Resolves with:* a CS6 UI/script comparison.
- **Whether Brightness/Contrast has a CS6 Preset menu entry** and what built-in
  presets it ships. The Help's save-preset list omits it. *Resolves with:* a CS6
  Properties panel capture.
- **Auto-to-`(brightness, contrast)` mapping** and whether the improved Auto is
  the same solver as Levels/Curves. *Resolves with:* a CS6 comparison on a common
  image.
- **CMYK/Lab direction and range semantics** (does +brightness raise or lower ink
  values?). *Resolves with:* a CS6 mode test.
- **Whether B/C is available at 32 bpc in any panel** despite its absence from the
  32-bpc feature list. *Resolves with:* a 32-bpc CS6 menu test.
- **Exact `brit` PSD serialization** and the legacy-flag bit. *Resolves with:* the
  PSD format specification plus a CS6-saved file.
