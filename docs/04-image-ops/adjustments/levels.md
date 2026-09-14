# Levels Adjustment

- **Spec ID:** `ADJ-001`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — the Levels engine is unchanged from CS5, but in CS6 the controls live in the **Properties panel** and the presets in its **Preset menu**, the `Auto` button uses "improved Auto options", and eyedropper sample-size options appear in a context menu.
- **Depends on:** `ADJ-000` adjustments-overview, `01-architecture/color-management.md` (`ARCH-007`), `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/undo-history.md` (`ARCH-009`), `04-image-ops/image-modes.md` (`IMG-004`), `04-image-ops/bit-depth-and-conversion.md` (`IMG-005`), `05-layers/adjustment-layers.md`, `03-tools/eyedropper-color-sampler-ruler.md`.

> Module and widget names are **design proposals**. Facts from the fetched CS6
> Help are attributed in `## Sources`; other statements are marked *(inferred)*.

## CS6 behavior

`Levels` "correct[s] the tonal range and color balance of an image by adjusting
intensity levels of image shadows, midtones, and highlights." The levels
histogram is the visual guide. It is reachable as an adjustment layer
(Adjustments panel Levels icon, or `Layer > New Adjustment Layer > Levels`) or as
a destructive command (`Image > Adjustments > Levels`, which "makes direct
adjustments to the image layer and discards image information").

The dialog/panel controls (CS6 Help, "Adjust tonal range using Levels"):

- **Input Levels** — three parts:
  - **Shadow / black input slider** (`A`): "maps the pixel value to level 0".
    Dragging right, e.g. to level 5, maps "all the pixels at level 5 and lower to
    level 0".
  - **Gamma / midtone input slider** (`B`): "adjusts the gamma in the image. It
    moves the midtone (level 128) and changes the intensity values of the middle
    range of gray tones without dramatically altering the highlights and
    shadows." Moving the middle slider **left lightens**; right darkens.
  - **Highlight / white input slider** (`C`): maps the chosen value to level 255.
    Dragging left, e.g. to 243, maps "all pixels at level 243 and higher to level
    255".
  - Text boxes accept the shadow, gamma, and highlight values directly.
- **Output Levels** — two sliders (`D`) and two boxes. "By default, the Output
  sliders are at level 0, where the pixels are black, and level 255, where the
  pixels are white." They set the output black/white points; the input range is
  remapped into `[output_black, output_white]`.
- Mapping note: "The mapping affects the darkest and lightest pixels in each
  channel. The corresponding pixels in the other channels are adjusted
  proportionately to avoid altering the color balance."
- **Channel menu** — edit the composite ("RGB"/"CMYK") or an individual channel.
  With a multi-channel selection made in the Channels panel before invoking the
  **command**, the Channel menu shows combined targets (e.g. `CM`); "This method
  does not work in a Levels adjustment layer." Spot and alpha channels are edited
  individually.
- **Clipping preview** — hold Alt/Option while dragging the black/white sliders,
  or `Show Clipping For Black/White Points` from the panel menu.
- **Auto** — "Click Auto to apply the default automatic levels adjustment."
  Alt/Option-click Auto opens the **Auto Color Correction Options** dialog.
- **Eyedroppers** — Set Black Point, Set Gray Point, Set White Point (see below).
- **Presets** — Levels settings can be saved/applied as presets (CS6: Properties
  panel Preset menu).
- **Reset / Toggle visibility / Delete** are the standard panel buttons.

CS6 panel location: the controls are in the **Properties panel's** adjustment-
controls view; the Levels icon is in the Adjustments panel. Presets moved to the
Preset menu in CS6 (CS5 listed them in the Adjustments panel).

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Adjustments > Levels` | Menu (dialog) | `Ctrl+L` | destructive; `Ctrl+L` is the classic Levels shortcut |
| `Layer > New Adjustment Layer > Levels` | Menu | — | non-destructive; opens New Layer dialog |
| Adjustments panel → Levels icon | Button | — | creates a Levels adjustment layer |
| Properties panel | Dock | — | Input/Output Levels, Channel, Auto, eyedroppers, Preset menu, clip button, reset, visibility, delete |
| Panel menu | Menu | — | `Auto Options`, `Save Preset`, `Load Preset`, `Show Clipping For Black/White Points`, `Add Mask by Default`, `Auto-Select …` |
| Channels panel | Dock | — | Shift-select channels before the destructive command to target a combination |
| Eyedropper context menu | Context menu | — | CS6: sample-size options (JDI) |
| Histogram panel | Dock | — | "You can view the adjusted histogram in the Histogram panel." |

## Parameters & ranges

CS6 Help gives no numeric clamps for the Levels dialog; the ranges below are the
documented scripting-DOM ranges for the equivalent operation plus the Help's
stated defaults. Marked accordingly.

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Channel | Enum | RGB / composite | composite, per color channel, spot, alpha | adjustment layer: color channels only |
| Input shadow (black point) | Integer level | 0 | `0 … 253` (DOM) | maps chosen value → 0 |
| Input highlight (white point) | Integer level | 255 | `(shadow+2) … 253` (DOM) | maps chosen value → 255; UI default 255 (DOM max 253 is a scripting clamp) |
| Input gamma (midtone) | Float | 1.00 | `0.10 … 9.99` (DOM) | >1 lightens midtones (slider left), <1 darkens *(convention: see Algorithms)* |
| Output shadow (black) | Integer level | 0 | `0 … 253` (DOM) | output floor |
| Output highlight (white) | Integer level | 255 | `(output shadow+2) … 253` (DOM) | output ceiling |
| Auto → Algorithm | Enum | Enhance Per Channel Contrast (Auto Tone) / Monochromatic (Auto Contrast) / Find Dark & Light (Auto Color) | see `## Algorithms` | shared Auto Color Correction Options |
| Auto → Clip Shadows / Highlights | Percent | 0.1% (Auto option/Auto Tone), 0.5% (Auto Contrast/Auto Color) | `0.0 … 100` recommended 0–1 (Help) | "ignores the first 0.1% of either extreme" |
| Auto → Target shadow color | Color | black (0,0,0) | color picker | darkest target |
| Auto → Target midtone color | Color | 128 gray | color picker | neutral target |
| Auto → Target highlight color | Color | white (255,255,255) | color picker | lightest target |
| Auto → Snap Neutral Midtones | Bool | off (on for Auto Color) | on / off |  |
| Auto → Save as Defaults | Bool | off | on / off | persists clip + target values as the Auto default |
| Set Black Point target | Color | 0,0,0 | R=G=B typed in Color Picker | the sampled pixel maps to this |
| Set Gray Point target | Color | 128,128,128 | equal R,G,B values | midtone neutralizer; unavailable in Grayscale |
| Set White Point target | Color | 255,255,255 | R=G=B | the sampled pixel maps to this |

DOM range caveat: the scripting `adjustLevels` clamps input/output levels to
`0…253` and requires `end ≥ start+2`; the UI's text boxes accept 0–255. Treat
`253` as a scripting artifact, not the UI limit (see Open questions).

Default preset names for Levels are not established by the fetched sources
(Levels presets exist; the list is *(inferred)* — see Open questions).

## Algorithms & pipeline

Behavioral parity; the exact Adobe integer math is closed. The model below is the
standard Levels transfer function.

### Input/output remap with gamma

For a channel sample normalized to `v ∈ [0,1]` (integer depths: `v = code /
max`; 32-bit: linear light), with input black `B`, input white `W`, input gamma
`γ` (`γ = 1.0` is neutral), output black `Ob`, output white `Ow`:

```text
t   = clamp( (v - B) / (W - B), 0, 1 )      # input black/white point remap
t'  = t ^ (1 / γ)                            # Photoshop gamma convention
out = Ob + t' * (Ow - Ob)                    # output black/white point remap
```

- `γ > 1` makes `t' > t` for `t ∈ (0,1)` → **midtones lighten**; this is why
  moving the middle slider **left** (which increases the displayed gamma value)
  lightens. `γ < 1` darkens midtones. *(inferred convention; the DOM documents
  the `0.10…9.99` range but not the exponent direction. Confirm — see Open
  questions.)*
- The formula collapses to the identity for `B=Ob=0`, `W=Ow=1`, `γ=1`.
- Because math is done on normalized values, the same code serves 8/16/32-bit;
  for 32-bit float the clamp is to the nominal `[0,1]` display range but values
  above 1 may be carried through the linear pipeline (see Edge cases).

### Per-channel and composite behavior

- A **composite** edit applies the same `(B, W, γ, Ob, Ow)` to every color
  channel. The Help's "corresponding pixels in the other channels are adjusted
  proportionately" describes the per-channel black/white-point mapping, not an
  extra coupling.
- A **per-channel** edit applies an independent tuple to one channel, which
  shifts color balance.
- The Auto algorithms and the gray-point eyedropper are the two color-cast
  tools.

### Eyedroppers

The Help: "using the eyedroppers undoes any previous adjustment you made in
Levels or Curves. If you plan to use the eyedroppers, it's best to use them first
and then fine-tune your adjustments with the Levels sliders or Curves points."

- **Set Black Point** — double-click to choose the target (default pure black,
  R=G=B=0); then click an image pixel. "The sampled pixel is mapped to the target
  black value" — equivalently, an input-level black point is set to the sampled
  luminance and the low end is stretched.
- **Set White Point** — same with pure white (255,255,255).
- **Set Gray Point** — "works best on images that don't require large adjustments
  and have easily identified neutrals"; it "should reset midtones and remove the
  color cast." It needs a neutral target (equal R=G=B); "unavailable when you work
  with grayscale images." Mechanically it scales each channel's midtone (gamma)
  so the clicked color becomes neutral *(inferred)*.

### Auto Color Correction algorithms

From the CS6 Help, "Auto Color Correction Options dialog box":

- **Enhance Monochromatic Contrast** — "Clips all channels identically. This
  preserves the overall color relationship while making highlights appear lighter
  and shadows appear darker." Used by **Auto Contrast**. It "does not adjust
  channels individually, [so] it does not introduce or remove color casts."
- **Enhance Per Channel Contrast** — "Maximizes the tonal range in each channel
  to produce a more dramatic correction. Because each channel is adjusted
  individually, [it] may remove or introduce color casts." Used by **Auto Tone**
  (and the Levels/Curves `Auto` button default).
- **Find Dark & Light Colors** — "Finds the average lightest and darkest pixels in
  an image and uses them to maximize contrast while minimizing clipping." Used by
  **Auto Color**, together with **Snap Neutral Midtones**.
- **Clip percentages** — "how much to clip black and white pixels"; default
  `0.1%` for the Auto option and Auto Tone, `0.5%` for Auto Contrast and Auto
  Color; "A value between 0.0% and 1% is recommended."
- **Target colors** — shadow/midtone/highlight swatches; Auto Color "neutralizes
  the midtones using a target color of RGB 128 gray."
- **Save as Defaults** — stores clip + target values for future Auto Tone/Contrast/
  Color and the Auto button. "When you save the Auto Color Correction options as
  defaults … it does not matter what algorithm you select … The three
  auto-correction commands use only those values that you set for the target
  colors and clipping," except that Auto Color also uses Snap Neutral Midtones.

Proposed per-channel algorithm for Enhance Per Channel Contrast (Auto Tone):

```text
for each channel c:
    histogram H_c over [0, max]
    lo_c = percentile(H_c, clip_low)          # default 0.1%
    hi_c = percentile(H_c, 100 - clip_high)
    map [lo_c, hi_c] -> [0, max] linearly      # i.e. B=lo_c, W=hi_c, γ=1
```

Enhance Monochromatic Contrast (Auto Contrast) uses a single joint `[lo, hi]`
derived from all channels identically (e.g. from the composite luminance), so no
per-channel cast is introduced. Find Dark & Light Colors uses the *average* of the
darkest/lightest pixel populations and, with Snap Neutral Midtones, additionally
solves a per-channel gamma so an average near-neutral color maps to the midtone
target. The exact percentile estimator and the midtone solver are closed; parity
is behavioral.

## Rust module mapping

Design proposal.

- `pictura-core::adjust::levels` — `LevelsParams { channel: ChannelRef,
  input_black: Scalar, input_white: Scalar, gamma: f32, output_black: Scalar,
  output_white: Scalar }` and `LevelsAdjustment`.
- `pictura-image::adjust::levels` — `build_lut(LevelsParams, depth) -> ToneLut`
  implementing the formula in `## Algorithms`; LUT length 256/4096/65536 for
  8/16-bit, direct evaluation for `f32`.
- `pictura-core::adjust::auto` — `AutoAlgorithm { MonochromaticContrast,
  PerChannelContrast, FindDarkLight }`, `AutoOptions { clip_low, clip_high,
  target_shadow, target_midtone, target_highlight, snap_neutral_midtones }`,
  `histogram_stats()` and `solve_auto()`.
- `pictura-image::adjust::eyedropper` — `LevelsEyedropperKind { Black, Gray,
  White }`, `apply_eyedropper(params, kind, sampled_color, target)`.
- Reuse `pictura-color::sample` for the eyedropper's averaged sample (CS6 sample
  sizes; see `03-tools/eyedropper-color-sampler-ruler.md`).
- Crossing types: `Scalar`, `ChannelRef`, `ToneLut`, `Rgb { r, g, b }`.

## Qt6 component mapping

Design proposal.

- `LevelsPropertiesWidget` (`QWidget`) — composite/per-channel Channel combo,
  the histogram with the three input sliders and two output sliders,
  `ScrubSpinBox`es for the input/output/gamma boxes, Auto button, and the three
  eyedropper buttons.
- `LevelsHistogramWidget` (`QWidget`) — draws the per-channel/composite
  histogram and the slider handles; Alt-drag shows the clipping overlay.
- `AutoCorrectionDialog` (`QDialog`) — shared with Curves/`ADJ-000`: Algorithms
  radio group, Clip shadow/highlight boxes, target color swatches, Snap Neutral
  Midtones, Save as Defaults.
- `ChannelSelectorCombo` (`QComboBox`) — composite + per-channel entries; in an
  adjustment layer the combined multi-channel entries are absent (Help).
- `LevelsPresetModel` — built-in + user presets for the Properties Preset menu.

The histogram/slider widget is a custom `QWidget` paint, not QML, to match the
rest of the Widgets shell; the same widget is reused by Curves for its backdrop.

## Data-model impact

- **PSD key `levl`** stores the Levels adjustment parameters for an adjustment
  layer; serialized per-channel records including the composite record
  (`ARCH-008`).
- Adjustment-layer node holds `LevelsParams` typed; destructive command holds a
  pixel-delta undo record.
- Presets (`LevelsParams` blobs) are persisted in the preset store
  (`11-cross-cutting/preference-storage.md`), not in the PSD.
- Undo: an interactive drag coalesces to one state; Auto/eyedropper application
  is one state. Destructive Levels stores the pre-edit tiles.
- 32-bit: parameters are stored as normalized floats; the same record bytes are
  only valid at the bit depth that produced them — DOM/PSD round-trip must
  preserve the depth-specific integer encoding (mark: exact PSD `levl` layout
  for 16/32-bit is not sourced here).

## Edge cases

- **Grayscale** — the Channel menu collapses to the single gray channel; the Set
  Gray Point eyedropper is explicitly unavailable.
- **CMYK** — Levels text values are percentages of ink in the UI *(inferred)*;
  the gamma convention must be checked to avoid inverting the slider direction.
- **Lab** — L/a/b channels; a/b are signed, so "black/white point" is not
  meaningful for the chroma channels — restrict per-channel Levels on a/b or map
  to the signed range carefully.
- **Spot / alpha channels** — editable individually in the destructive command;
  not available as an adjustment layer.
- **32-bit** — Levels is on the documented 32-bpc adjustment list; integer LUTs
  do not apply to float values.
- **Bitmap / Indexed** — Levels unavailable.
- **Input black ≥ input white** — the DOM enforces `end ≥ start+2`; the UI clamps
  to avoid division by zero. Guard `/ (W - B)`.
- **Gamma 0.10 / 9.99 extremes** — verify no NaN in `0^(1/γ)`; treat `0` and `1`
  as fixed points.
- **Empty / 1-px documents** — LUT builds, apply is trivial; no panic.
- **Huge documents** — compute the histogram for Auto over tiles, then a single
  LUT pass; do not allocate a float copy.
- **GPU unavailable** — CPU reference LUT; identical for 8/16-bit.
- **Undo** — eyedropper "undoes any previous adjustment" is a UX reset, not a
  history state; represent as clearing the current edit.

## Parity acceptance criteria

1. Given a linear grey ramp and input black 0 / white 255 / gamma 1 / output
   0/255, the output equals the input exactly.
2. Given input black 5, all pixels at code ≤5 map to output black; given input
   white 243, all pixels at code ≥243 map to output white.
3. Given input gamma `g`, the pixel whose input is the midpoint of the input range
   maps to the midpoint of the output range shifted per the convention: a gamma
   value that lightens on the CS6 slider produces a lighter midpoint than the
   neutral result.
4. Given output black 20 / white 235, no output sample falls below 20 or above
   235 after the map.
5. Given an RGB image and a composite Levels map, all three channels receive the
   same map; given a per-channel map, only that channel changes.
6. Given Auto Contrast, color relationships are preserved (no new cast) within
   the tolerance; given Auto Tone, each channel is independently stretched.
7. Given Auto Color with Snap Neutral Midtones, an image with an average cast
   has its near-neutral midtone moved toward the RGB-128 target within tolerance.
8. Given `Clip` of 0.5%, the darkest/brightest 0.5% of the population is ignored
   when finding the endpoints, verified by histogram population counts.
9. Given the Set Black/White Point eyedropper with a non-default target gray, the
   clicked pixel maps to that target value within tolerance.
10. Given a Grayscale document, the Set Gray Point eyedropper is unavailable.
11. Given a Levels adjustment layer, `Ctrl+Z` after a drag restores the prior
    parameter values and exactly one history state was added.
12. Given a 32-bpc document, Levels is available and does not clamp float values
    to `[0,1]` beyond the display mapping.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, extracted with `pdftotext -layout`. Established:
  "Levels overview"; the shadow/gamma/highlight input sliders with the level-5 /
  level-243 examples and "maps the pixel value to level 0/255"; the gamma
  "moves the midtone (level 128)" description and left/right direction; the
  Output Levels default 0/255; "The mapping affects the darkest and lightest
  pixels in each channel … adjusted proportionately to avoid altering the color
  balance"; the Channel menu and the Shift-select-Channels command-only rule;
  Alt-drag clipping preview; spot/alpha individual editing; the eyedropper
  sections ("Set black and white points using the Eyedropper tools", "Color
  correct using the eyedroppers") including the "eyedroppers undo any previous
  adjustment" note and the Grayscale unavailability of the gray point; and the
  full "Auto Color Correction Options dialog box" (three algorithms, clip
  percentages and defaults, target colors, Snap Neutral Midtones, Save as
  Defaults, and the Auto Color/Auto Tone/Auto Contrast mapping).
- `https://theiviaxx.github.io/photoshop-docs/Photoshop/ArtLayer/adjustLevels.html`
  — Photoshop scripting reference: `adjustLevels(inputRangeStart, inputRangeEnd,
  inputRangeGamma, outputRangeStart, outputRangeEnd)` with ranges `0…253`,
  `(start+2)…253`, gamma `0.10…9.99`, output `0…253`, `(start+2)…253`.

Secondary / not fetched this pass: the improved-Auto and 32-bpc facts also
appear in the CS6 Help "What's new" and 32-bpc feature list sections (both read
from the same PDF). Default Levels preset names and exact UI clamps for the text
boxes are **not** established here.

## Open questions

- **Gamma exponent direction and the displayed gamma value.** The DOM documents
  the `0.10…9.99` range but not whether the exponent is `1/γ` (as assumed) or
  `γ`. *Resolves with:* a CS6 probe (set gamma to 2.00 and 0.50 on a grey ramp).
- **Exact Levels dialog ranges** for the input/output text boxes (0–255 vs the
  DOM 0–253 clamp) and the slider step. *Resolves with:* a CS6 dialog capture.
- **Built-in Levels presets** shipped in CS6 (names and parameter values).
  *Resolves with:* a CS6 install / Properties Preset menu capture.
- **Auto percentile estimator** (exact histogram binning, tie-breaking, and
  whether the clip is on population or cumulative count). *Resolves with:*
  fitting to CS6 Auto Tone output on synthetic ramps.
- **Snap Neutral Midtones math** (how the "average nearly-neutral color" is
  chosen and the per-channel gamma solved). *Resolves with:* a CS6 calibration.
- **Set Gray/Black/White Point mapping** — whether the eyedropper re-solves the
  whole tuple or only the corresponding level. *Resolves with:* CS6 experiments
  with known patches.
- **CMYK/Lab Levels semantics** (ink percentages, signed a/b channels) and the
  available channels. *Resolves with:* a CS6 mode test.
- **Whether the destructive command's combined-channel mode** (`CM` etc.) has an
  adjustment-layer equivalent in any CS6 path. *Resolves with:* CS6 observation.
- **Exact `levl` PSD serialization** for 8/16/32-bit and per-channel records.
  *Resolves with:* the Adobe File Formats Specification plus a CS6-saved sample.
