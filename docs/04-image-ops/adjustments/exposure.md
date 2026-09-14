# Exposure Adjustment

- **Spec ID:** `ADJ-004`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — the Exposure/Offset/Gamma Correction engine and the three eyedroppers are the same as CS5; only the host panel moves to the CS6 Properties panel. The CS6 Help repeats the CS5-era note that "Adjustment layers for 32-bit images are available in Photoshop Extended only."
- **Depends on:** `ADJ-000` adjustments-overview, `ADJ-001` levels, `04-image-ops/32-bit-hdr.md`, `01-architecture/color-management.md` (`ARCH-007`), `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/undo-history.md` (`ARCH-009`), `04-image-ops/bit-depth-and-conversion.md` (`IMG-005`), `04-image-ops/image-modes.md` (`IMG-004`), `05-layers/adjustment-layers.md`.

> Module and widget names are **design proposals**. Facts from the fetched CS6
> Help are attributed in `## Sources`; other statements are marked *(inferred)*.

## CS6 behavior

"The Exposure and HDR Toning adjustments are primarily designed for 32-bit HDR
images, but you can also apply them to 16- and 8-bit images to create HDR-like
effects." Exposure "works by performing calculations in a **linear color space
(gamma 1.0)** rather than the current color space."

The three controls (CS6 Help, "Adjust HDR exposure"):

- **Exposure** — "Adjusts the highlight end of the tonal scale with minimal effect
  in the extreme shadows." With 32-bit images the Exposure slider is also
  available at the bottom of the image window (the 32-bit preview control), but
  that status-bar slider is a *view* setting, not an image edit.
- **Offset** — "Darkens the shadows and midtones with minimal effect on the
  highlights."
- **Gamma Correction** — "Adjusts the image gamma, using a simple power function.
  Negative values are mirrored around zero (that is, they remain negative but
  still get adjusted as if they are positive)." (The mirrored-negative statement
  is a Help quirk; the UI gamma is a positive power-law value. See Open
  questions.)

### Eyedroppers

The Help: "The eyedroppers adjust the luminance values of images (unlike the
Levels eyedroppers that affect all color channels)."

- **Set Black Point** — "sets the Offset, shifting the pixel you click to zero."
- **Set White Point** — "sets the Exposure, shifting the point you click to white
  (1.0 for HDR images)."
- **Midtone eyedropper** — "sets the Exposure, making the value you click middle
  gray."

So Exposure's eyedroppers write the Exposure/Offset parameters rather than a
per-channel black/white point; they are luminance-only. The Help does not document
a preset list for the eyedroppers; any "preset" here is the target value the tool
maps to (black = 0, white = 1.0 HDR, midtone = middle gray) *(inferred)*.

### Availability

- Adjustment layer: `Layer > New Adjustment Layer > Exposure` or the Adjustments
  panel Exposure icon. The Help explicitly notes "Adjustment layers for 32-bit
  images are available in Photoshop Extended only."
- Destructive: `Image > Adjustments > Exposure` ("direct adjustments to the image
  layer [and] discards image information").
- Exposure is on the CS6 **32-bpc** adjustment list (the Help names Levels,
  Exposure, Hue/Saturation, Channel Mixer, Photo Filter); "Although the Exposure
  command can be used with 8- and 16-bpc images, it is designed for making
  exposure adjustments to 32-bpc HDR images."
- No channel selector: Exposure is a single luminance operation; it has no
  per-channel mode *(inferred)*.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Adjustments > Exposure` | Menu (dialog) | — | destructive; `Preview` |
| `Layer > New Adjustment Layer > Exposure` | Menu | — | non-destructive; 32-bit layer = Extended only |
| Adjustments panel → Exposure icon | Button | — | creates the adjustment layer |
| Properties panel | Dock | — | Exposure / Offset / Gamma Correction sliders + three eyedroppers + Auto + Preset menu + clip/reset/visibility/delete |
| Panel menu | Menu | — | `Auto Options`, `Save Preset`, `Load Preset`, `Add Mask by Default`, `Auto-Select …` |
| Document window status bar (32-bit) | Slider | — | 32-bit **preview** exposure; a per-view setting, not an image edit |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Exposure | Number (slider + box) | 0.00 | **−20.00 … +20.00 EV** (community; see below) | linear-light gain `× 2^EV`; in stops |
| Offset | Number | 0.0000 | **−0.5000 … +0.5000** (community) | additive linear shift |
| Gamma Correction | Number | 1.00 | **0.01 … 9.99** (community; DOM gamma analog) | power function on the linear result |
| Set Black Point eyedropper | Tool | target 0 | — | sets Offset |
| Set White Point eyedropper | Tool | target 1.0 (HDR) | — | sets Exposure |
| Midtone eyedropper | Tool | target middle gray | — | sets Exposure |
| Auto | Action | — | current Auto options | shares Auto Color Correction Options (`ADJ-001`) |
| Preset | Enum | — | built-in + user | Exposure is in the Help's save-preset list |
| Channel | — | luminance | n/a | no per-channel mode |

Range sourcing: the CS6 Help does not print numeric clamps for the Exposure
sliders. The values above are the widely reported Photoshop ranges (Exposure
±20 EV; Offset ±0.5; Gamma 0.01–9.99) and a 2026 training source fetched this
pass; mark them provisional and confirm against a CS6 dialog (Open questions).
The gamma range matches the scripting `adjustLevels` gamma convention
`0.10…9.99` in spirit but Exposure's is stated as `0.01…9.99`.

## Algorithms & pipeline

Behavioral parity; Adobe's exact numbers are closed, but the Help pins the
essential model: **linear light**.

### Linear-space transfer

For a channel sample decoded to linear light `c ≥ 0` (32-bit float documents are
already linear; 8/16-bit are decoded through the document gamma first — see
`ARCH-007`):

```text
c1 = c * 2^E                 # Exposure: one EV doubles the linear value
c2 = c1 + O                 # Offset: additive linear shift (shadows/midtones)
c3 = clamp(c2, 0, ∞)        # keep non-negative before the power
out = c3 ^ G                 # Gamma Correction: power function, G > 0
```

- `E` is in **stops/EV** (`E = 1` doubles luminance, matching camera exposure
  compensation). This is why Exposure clips highlights fast: bright linear values
  exceed 1 and clip at the display/encode stage.
- `O` shifts the whole linear signal; because the linear domain compresses
  shadows, a given offset changes shadows/midtones far more than highlights —
  matching the Help's description.
- `G`: with `G = 1` neutral, `G < 1` **brightens** midtones and `G > 1` darkens —
  the same left-brightens behavior as the Levels middle slider but with the
  opposite numeric convention *(community source; flagged in Open questions)*.
  The Help's "Negative values are mirrored around zero" describes the internal
  parameter passing, not the UI value.
- **Order** — Exposure → Offset → Gamma, as listed in the Help. The exact order
  of operations is not proven; confirm (Open questions).

### 8/16-bit and 32-bit paths

- **32-bit**: pixels are linear `f32`; the map is applied directly and values may
  exceed `[0,1]`; the *preview* (status-bar Exposure / `View > 32-Bit Preview
  Options`) maps to display via Exposure and Gamma or Highlight Compression, and
  "Preview adjustments don't edit the HDR image file". Only the Exposure
  adjustment edits pixels.
- **8/16-bit**: decode sRGB/working-space gamma → linear, apply, re-encode. The
  Help stresses "calculations in a linear color space (gamma 1.0) rather than the
  current color space", so an 8-bit round-trip will not be bit-exact for non-
  identity parameters, and the re-encode may clip.

### Eyedropper math

- **Set Black Point** — solve `O` so the sampled linear value maps to 0:
  `O = -(s * 2^E)` (with current `E`).
- **Set White Point** — solve `E` so the sampled linear value maps to 1.0:
  `E = log2(1 / s)` (with current `O`; or jointly if `O ≠ 0`).
- **Midtone** — solve `E` (and/or jointly with `O`/`G`) so the sampled value maps
  to middle gray (linear ≈ 0.18 or code 128 depending on convention).
- The Help states the eyedroppers "adjust the luminance values", i.e. they act on
  the luminance and do not alter color balance. Exact solve ordering (which
  parameter each tool touches when others are non-zero) is *(inferred)*.

### Auto

Exposure is not named in the CS6 "improved Auto" line (that names Levels, Curves,
Brightness/Contrast), yet the Properties panel exposes an Auto button and Exposure
is in the save-preset list. Treat Exposure's Auto as reusing the shared Auto Color
Correction statistics to choose `(E, O, G)`; exact mapping is closed.

## Rust module mapping

Design proposal. Exposure is the one adjustment that must run in a linear working
buffer by design.

- `pictura-core::adjust::exposure` — `ExposureParams { exposure_ev: f32, offset:
  f32, gamma: f32 }`.
- `pictura-image::adjust::exposure` — `apply_linear(dst: &mut [f32], params)` for
  the float path; `apply_encoded_u8/u16(..., color_space)` that decodes to linear,
  applies, and re-encodes via `pictura-color`.
- `pictura-color::linear` — `to_linear` / `to_encoded` for 8/16-bit using the
  document's working-space TRC; caches a decode/encode LUT.
- `pictura-image::adjust::exposure::eyedropper` — `solve_offset`, `solve_exposure`,
  `solve_midtone` for the three tools.
- `pictura-core::adjust::auto` — shared; `solve_exposure(stats) -> ExposureParams`.
- Crossing types: `f32` planar buffers, `ColorSpaceId`, `ExposureParams`.

## Qt6 component mapping

Design proposal.

- `ExposurePropertiesWidget` (`QWidget`) — Exposure / Offset / Gamma Correction
  slider+box rows (`QDoubleSpinBox`/scrubber), the three eyedropper buttons, Auto,
  Preset menu trigger.
- `AutoCorrectionDialog` (`QDialog`) — shared with `ADJ-001`/`ADJ-002`.
- `ExposurePresetModel` — built-in + user presets.
- 32-bit status-bar preview control belongs to the document window, not this
  panel (it is a view setting).

Numbers use enough precision for Offset (4 decimals) and Gamma (2 decimals);
sliders are logarithmic-feel for EV.

## Data-model impact

- **PSD key `expA`** stores Exposure adjustment parameters (`ARCH-008`).
- Parameter struct is depth-agnostic: `exposure_ev` (f32), `offset` (f32),
  `gamma` (f32), so the same layer round-trips at 8/16/32.
- The 32-bit status-bar preview value is **document view state**, stored in the
  PSD/PSB/TIFF preview fields, not in the adjustment (`04-image-ops/32-bit-hdr.md`).
- Undo: one state per committed change; destructive Exposure retains pre-edit
  tiles.
- Because 8/16-bit edits decode→apply→re-encode, undo must store the pre-edit
  encoded pixels (round-trip is not lossless for general parameters).

## Edge cases

- **32-bit precision** — never clamp intermediate linear values to `[0,1]`; the
  power function must accept `c ≥ 0` and `0^G = 0`.
- **Negative gamma** — the Help's "negative values are mirrored around zero"
  suggests the internal parameter can go negative; the UI gamma is positive.
  Decide whether to expose negative gamma and mirror `c` accordingly.
- **8/16-bit clipping** — converting back to encoded integers clips highlights;
  that is expected and matches CS6, but must not be silently hidden.
- **Gamma 0.01 / 9.99 extremes** — guard against NaN/Inf.
- **Offset pushes all values below 0** — clamp to 0 (black) as CS6 does.
- **Eyedropper sequence** — the eyedroppers overwrite settings, so black→white→
  midtone order changes the result; document the order.
- **Grayscale** — luminance-only, so Exposure is well-defined; confirm availability.
- **CMYK** — linear-light exposure is defined for additive RGB, not ink; CMYK is
  likely unavailable/unsupported *(inferred)*.
- **Bitmap / Indexed** — unavailable.
- **Curves at 32-bit** — note that the companion Curves adjustment is unavailable
  at 32-bit, so HDR workflows rely on Exposure/Levels.
- **Identity** — `E=0, O=0, G=1` must be a bit-exact no-op and add no history
  state; the 8/16-bit decode→encode path must be bypassed for identity to remain
  bit-exact.
- **Empty / 1-px / huge documents** — stream by tile; no full-canvas float copy.
- **GPU unavailable** — CPU path; float math makes CPU/GPU differences small but
  non-zero (parity tolerance).

## Parity acceptance criteria

1. Given `Exposure = +1.0 EV` on a linear float ramp, every pixel doubles in
   linear value (within float tolerance); `−1.0 EV` halves it.
2. Given `Exposure = 0, Offset = 0, Gamma = 1`, the output is bit-exact
   unchanged at 8, 16, and 32 bpc.
3. Given a positive Offset, the linear value increases and pure black is lifted
   above 0 (matte effect); given a negative Offset, shadows are driven to 0.
4. Given Gamma `< 1`, midtones brighten; given Gamma `> 1`, midtones darken,
   with the endpoints preserved (`0→0`, `1→1`).
5. Given a 32-bpc document, values above 1.0 survive the adjustment (no clamp)
   and the display preview maps them separately.
6. Given the Set Black Point eyedropper, the clicked pixel's luminance maps to 0
   within tolerance; given Set White Point, it maps to 1.0; given Midtone, to
   middle gray.
7. Given a 32-bpc document with a 32-bit Exposure **adjustment layer**, the layer
   is available in the Extended edition and (per the Help) unavailable in Standard.
8. Given an 8-bit sRGB image with a non-identity Exposure, the decode→apply→
   encode path matches a reference lcms2-based linear round-trip within the
   testing tolerance.
9. Given `Ctrl+Z` after a slider change, the prior parameters and pixels are
   restored exactly and exactly one history state was added.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, extracted with `pdftotext -layout`. Established:
  "Adjust HDR exposure and toning" (primarily for 32-bit, also 16/8-bit);
  "Exposure works by performing calculations in a linear color space (gamma 1.0)
  rather than the current color space"; the Exposure / Offset / Gamma Correction
  descriptions (highlight end; shadows+midtones; simple power function, negative
  values mirrored); "The eyedroppers adjust the luminance values … unlike the
  Levels eyedroppers that affect all color channels"; Set Black Point sets
  Offset to zero, Set White Point sets Exposure to white (1.0 for HDR), Midtone
  sets Exposure to middle gray; "Adjustment layers for 32-bit images are
  available in Photoshop Extended only"; the 32-bpc adjustment list naming
  Exposure; "Although the Exposure command can be used with 8- and 16-bpc
  images, it is designed for making exposure adjustments to 32-bpc HDR images";
  the 32-bit preview options (Exposure and Gamma / Highlight Compression) and the
  status-bar 32-Bit Exposure preview that does not edit the file.
- `https://lecontephotographicsociety.org/training/editing/photoshop/student/ps-lesson-23-adjustment-exposure-student.html`
  — secondary training source (2026, non-Adobe) used only for the numeric
  slider ranges (`Exposure −20…+20 EV`, `Offset −0.5…+0.5`, `Gamma 0.01…9.99`,
  neutral 1.0) and the EV/stop semantics; **not authoritative for CS6** — see
  Open questions.

Secondary / not fetched this pass: the working "gamma < 1 brightens" direction
and the decode/apply/encode order surfaced from community discussions
(dpreview, Adobe community) are corroborating only.

## Open questions

- **Exact CS6 slider ranges and steps** for Exposure, Offset, and Gamma
  Correction, and their displayed precision. *Resolves with:* a CS6 Properties
  panel/dialog capture.
- **Gamma numeric convention** — whether `out = c^G` (as assumed) or `c^(1/G)`,
  and whether negative values are mirrored in the UI. *Resolves with:* a CS6
  probe on a grey ramp.
- **Operation order** — Exposure → Offset → Gamma (as assumed) or another order.
  *Resolves with:* a CS6 probe with two non-default sliders.
- **Eyedropper parameter interaction** — when other sliders are non-zero, which
  parameter(s) each eyedropper rewrites (e.g. does Midtone also touch Offset?).
  *Resolves with:* CS6 experiments.
- **Middle-gray definition** for the Midtone eyedropper (linear 0.18 vs encoded
  128). *Resolves with:* a CS6 calibration.
- **Exposure Auto behavior** and whether it shares the exact Levels/Curves solver.
  *Resolves with:* a CS6 comparison.
- **Availability in CMYK/Grayscale/Indexed** and the exact behavior in CMYK.
  *Resolves with:* a CS6 mode test.
- **Exact `expA` PSD serialization** and how the three floats are encoded per bit
  depth. *Resolves with:* the PSD format specification plus a CS6-saved file.
- **Standard vs Extended** — confirm which 32-bit Exposure behaviours (adjustment
  layer, status-bar slider) are missing in Photoshop CS6 Standard. *Resolves
  with:* the edition comparison doc and a Standard install.
