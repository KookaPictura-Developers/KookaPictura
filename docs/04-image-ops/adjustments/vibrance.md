# Vibrance Adjustment

- **Spec ID:** `ADJ-005`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `No` — Vibrance predates CS6 (the exact introduction version is not established here). CS6 only relocates the controls to the **Properties panel** and moves presets to the Preset menu.
- **Depends on:** `ADJ-000` adjustments-overview, `04-image-ops/adjustments/hue-saturation.md`, `01-architecture/color-management.md` (`ARCH-007`), `01-architecture/document-model.md` (`ARCH-008`), `01-architecture/undo-history.md` (`ARCH-009`), `04-image-ops/image-modes.md` (`IMG-004`), `04-image-ops/bit-depth-and-conversion.md` (`IMG-005`), `05-layers/adjustment-layers.md`, `07-color-painting/color-models.md`.

> Module and widget names are **design proposals**. Facts from the fetched CS6
> Help are attributed in `## Sources`; other statements are marked *(inferred)*.

## CS6 behavior

"Vibrance adjusts the saturation so that clipping is minimized as colors approach
full saturation. This adjustment increases the saturation of less-saturated colors
more than the colors that are already saturated. Vibrance also prevents skintones
from becoming over saturated."

The two sliders:

- **Vibrance** — "drag the Vibrance slider to increase or decrease color
  saturation without clipping when colors become more saturated." Moving right
  "applies more adjustment to less saturated colors and prevent[s] colors clipping
  as they reach total saturation."
- **Saturation** — "applies the same amount of saturation adjustment to all
  colors regardless of their current saturation. In some situations, this may
  produce less banding than the Saturation slider in the Hue/Saturation
  Adjustments panel or Hue/Saturation dialog box." Both sliders can be moved left
  to decrease saturation.

Reachability: adjustment layer (Adjustments panel Vibrance icon;
`Layer > New Adjustment Layer > Vibrance`) or destructive command
(`Image > Adjustments > Vibrance`, which "makes direct adjustments to the image
layer and discards image information"). A negative Vibrance value desaturates
less-saturated colors more than already-saturated ones; the Saturation slider can
drive the image to gray at its minimum.

### Difference from Hue/Saturation

The `Hue/Saturation` adjustment (see `hue-saturation.md`) exposes **Hue**,
**Saturation**, and **Lightness** for the master and six color bands
(Reds/Yellows/Greens/Cyans/Blues/Magentas), plus a **Colorize** mode; its
Saturation slider scales saturation uniformly and is the classic source of
over-saturation and skin-tone blowout. Vibrance has only **two** sliders, no hue
or lightness control, no color-band targeting, and no Colorize. The distinction
is the **amount curve**: Hue/Saturation applies one uniform saturation delta;
Vibrance's delta falls off as current saturation rises, so already-saturated
colors and skin tones are protected.

## UI surface

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| `Image > Adjustments > Vibrance` | Menu (dialog) | — | destructive; `Preview` |
| `Layer > New Adjustment Layer > Vibrance` | Menu | — | non-destructive; New Layer dialog |
| Adjustments panel → Vibrance icon | Button | — | creates the adjustment layer |
| Properties panel | Dock | — | Vibrance slider, Saturation slider, Auto, Preset menu, clip button, reset, visibility, delete |
| Panel menu | Property panel | — | `Auto Options`, `Save Preset`, `Load Preset`, `Add Mask by Default`, `Auto-Select …` |
| Sponge tool options bar | Tool option | — | a separate **Vibrance** checkbox (tool, not this adjustment) |

## Parameters & ranges

| Control | Type | Default | Range / options | Notes |
|---|---|---|---|---|
| Vibrance | Number (slider + box) | 0 | **−100 … +100** (community; inferred) | positive boosts less-saturated colors, protects saturated/skin; negative desaturates |
| Saturation | Number (slider + box) | 0 | **−100 … +100** (community; inferred) | uniform saturation scale; −100 = grayscale |
| Auto | Action | — | current Auto options | shared Auto Color Correction Options (`ADJ-001`) |
| Preset | Enum | — | built-in + user | save-preset list in the Help does **not** include Vibrance — see Open questions |
| Channel | — | color | n/a | no per-channel mode |

The CS6 Help does not print numeric clamps for the Vibrance/Saturation sliders.
`±100` with a default of `0` is the universally reported Photoshop range and is
consistent with the Hue/Saturation Saturation range; mark it provisional and
confirm against a CS6 dialog (Open questions).

The Help's `Adjustment and fill layers` and the 32-bpc lists put **Vibrance on
the 32-bpc adjustment-layer list** ("adjustment layers (Levels, Vibrance,
Hue/Saturation, Channel Mixer, Photo Filter, and Exposure)"), although the 32-bpc
"Adjustments" line names only Levels, Exposure, Hue/Saturation, Channel Mixer,
Photo Filter. The two lines conflict; treat 32-bpc Vibrance as **available via an
adjustment layer** pending confirmation (Open questions).

## Algorithms & pipeline

Behavioral parity; Adobe's exact vibrance math and skin-tone model are closed.
The behavior to reproduce is precise, though:

- **Saturation slider** — uniform multiplicative/perceptual saturation scaling
  applied to every color regardless of its current saturation.
- **Vibrance slider** — a saturation delta that **decreases monotonically with
  current saturation** (more boost to muted colors, less to vivid ones) and is
  additionally **attenuated in the skin-tone hue band** so skin does not
  over-saturate.

### Proposed model

Convert the pixel to a hue/chroma space (LCh or HSV/HSL; `pictura-color`). Let
`S ∈ [0,1]` be normalized saturation, `H` the hue in degrees.

Saturation slider (`k_s = saturation/100 ∈ [−1, 1]`):

```text
S' = clamp( S * (1 + k_s), 0, 1 )        # k_s > 0 boosts, k_s = -1 -> grayscale
```

Vibrance slider (`k_v = vibrance/100 ∈ [−1, 1]`):

```text
amount(x) = x * (1 - x)                   # peaks at mid saturation, 0 at x=0 and x=1
                                          # (a concave falloff; "less-saturated colors more")
w(H)      = 1 - skin_weight(H)            # skin-tone protection, in [0,1]
S' = clamp( S + k_v * amount(S) * w(H), 0, 1 )
```

- Unknowns, all *(inferred)*: the exact falloff (`(1-S)`, `(1-S)^p`, or a piecewise
  curve), the base saturation space, and the skin hue band. A common skin
  protection uses a smooth weight over roughly the orange/red band (≈ 20°–50°
  hue) with a falloff to 0 by the neighboring hues; the weight may also depend on
  luminance/chroma. Treat `w(H)` as a **calibration curve** with a small number of
  parameters (center, width, depth), not a hardcoded constant — the spec defers
  the exact shape to CS6 calibration.
- Negative Vibrance must not push near-gray colors below zero (no color cast);
  clamp at `S = 0`.
- Saturation and Vibrance compose; the Help presents them as independent sliders.

### Bit depth and color modes

- Integer depths use a normalized `S`; 32-bit float uses the same normalized
  model after the display decode, so the parameter is depth-agnostic.
- **RGB** is the natural mode. CMYK/Lab behavior is not documented; Lab could use
  chroma `C*` directly, but whether CS6 enables Vibrance there is unknown.
- **Grayscale** has no chroma; the sliders are inert/unavailable *(inferred)*.
- Vibrance is a **vector** operation (it mixes channels via a color-space
  transform), unlike the per-channel maps in Levels/Curves.

### Relationship to the Sponge tool

The Sponge tool has a separate **Vibrance** checkbox that "minimize[s] clipping
for fully saturated or desaturated colors" (`03-tools/dodge-burn-sponge.md`); it
shares the conceptual falloff but is a painted, local operator, not this
adjustment.

## Rust module mapping

Design proposal.

- `pictura-core::adjust::vibrance` — `VibranceParams { vibrance: f32,
  saturation: f32 }`, plus a `SkinProtection` calibration struct `{ center_deg,
  width_deg, depth }`.
- `pictura-image::adjust::vibrance` — `apply_vibrance(dst, params, color_space)`;
  converts to/from the chosen chroma space once per tile, applies the falloff and
  skin weight, converts back.
- `pictura-color::chroma` — shared `sat_from_rgb` / `rgb_from_sat` kernels used by
  Vibrance and Hue/Saturation; also the skin-hue weight.
- `pictura-render::adjust::vibrance` — optional wgpu compute variant; CPU is the
  reference.
- `pictura-core::adjust::auto` — shared Auto statistics.
- Crossing types: `VibranceParams`, `f32` planar buffers, `ColorSpaceId`,
  `Rect`.

## Qt6 component mapping

Design proposal.

- `VibrancePropertiesWidget` (`QWidget`) — two slider+box rows (Vibrance,
  Saturation), Auto button, Preset menu trigger.
- `AutoCorrectionDialog` (`QDialog`) — shared.
- `VibrancePresetModel` — built-in + user presets.

No custom canvas is required. The sliders use the standard scrubber fields.

## Data-model impact

- **PSD key `vibA`** stores the Vibrance adjustment parameters (`ARCH-008`).
- Parameter struct carries the two scalars; presets persist in the preset store.
- Undo: one state per committed change (drag coalesced); destructive Vibrance
  retains pre-edit tiles.
- 32-bit: parameters are normalized floats; the round-trip is depth-agnostic.
- No channel-specific fields beyond the standard adjustment-layer mask.

## Edge cases

- **Grayscale / no chroma** — sliders inert or unavailable; must not create a cast.
- **Fully saturated colors** — Vibrance must add ~0 (already at `S = 1`); the
  Saturation slider still clamps.
- **Near-gray colors** — negative Vibrance must not drive `S` below 0 or shift
  hue; positive Vibrance should still treat them as "less saturated".
- **Skin tones** — the protection band must actually reduce the boost; a hard
  cutoff would create visible hue banding, so use a smooth weight.
- **Hue shifts** — a chroma-space round-trip can shift hue if not handled in a
  hue-preserving space (LCh/HSL with hue held constant); verify ΔH ≈ 0.
- **CMYK / Lab** — define the chroma mapping or disable; do not silently apply an
  RGB-only formula.
- **Bitmap / Indexed** — unavailable.
- **32-bit** — clamp the internal `S` to `[0,1]` but do not clamp the float pixel
  unnecessarily; mark the exact HDR behavior TBD.
- **Identity** — both sliders 0 must be a bit-exact no-op and add no history state.
- **Empty / 1-px / huge documents** — per-tile vector apply; no full-canvas copy.
- **GPU unavailable** — CPU reference; float chroma conversion differences are
  within tolerance.
- **Banding** — the Help claims the Saturation slider here can band less than the
  Hue/Saturation Saturation slider; reproduce the gentler rolloff.

## Parity acceptance criteria

1. Given both sliders at 0, the output is bit-exact unchanged at 8, 16, and
   32 bpc.
2. Given `Saturation = −100`, an RGB image becomes neutral gray (R=G=B) within
   tolerance, with lightness ordering preserved.
3. Given `Saturation = +50`, a low-saturation and a high-saturation patch of the
   same hue receive the **same** absolute saturation increase (uniform delta),
   matching the Help.
4. Given `Vibrance = +50`, a low-saturation patch receives a larger saturation
   increase than a high-saturation patch of the same hue (diminishing boost),
   matching "increases the saturation of less-saturated colors more than the
   colors that are already saturated."
5. Given `Vibrance = +100`, a skin-tone patch receives a smaller saturation
   increase than an equally saturated non-skin patch of a different hue
   (skin-tone protection).
6. Given `Vibrance = −50`, no channel goes below 0 and no hue shift is introduced
   beyond the defined ΔH tolerance.
7. Given a fully saturated patch and `Vibrance = +100`, the patch barely changes
   (clipping is minimized) and does not clip.
8. Given a Grayscale document, the sliders are unavailable or inert.
9. Given a 32-bpc document, the Vibrance adjustment layer is available (pending
   the conflicting Help lines) and leaves values above 1.0 unclamped.
10. Given `Ctrl+Z` after a slider drag, the prior parameters and pixels are
    restored exactly and exactly one history state was added.

## Sources

Fetched for this document:

- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` —
  official CS6 Help reference, extracted with `pdftotext -layout`. Established:
  "Adjust color saturation using Vibrance" — "Vibrance adjusts the saturation so
  that clipping is minimized as colors approach full saturation. This adjustment
  increases the saturation of less-saturated colors more than the colors that are
  already saturated. Vibrance also prevents skintones from becoming over
  saturated"; the Vibrance slider "increase or decrease color saturation without
  clipping when colors become more saturated"; "To apply more adjustment to less
  saturated colors and prevent colors clipping as they reach total saturation,
  move the Vibrance slider to the right"; the Saturation slider "applies the same
  amount of saturation adjustment to all colors regardless of their current
  saturation" and "in some situations … may produce less banding than the
  Saturation slider in the Hue/Saturation Adjustments panel"; creation via the
  Adjustments panel / Layer menu and the destructive `Image > Adjustments >
  Vibrance` path. The 32-bpc list naming Vibrance as an adjustment layer (and the
  conflicting 32-bpc "Adjustments" line omitting it) is from the same PDF.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — the
  Hue/Saturation behavior used for the comparison (master + six bands, Colorize)
  is the same reference; see `hue-saturation.md` for the dedicated spec.

Not fetched this pass: the `±100` slider range, the exact skin-hue protection
band, and the falloff exponent. The Sponge-tool Vibrance checkbox connection is
documented in `03-tools/dodge-burn-sponge.md` from the same Help.

## Open questions

- **Exact slider ranges and defaults** for Vibrance/Saturation in CS6. *Resolves
  with:* a CS6 Properties panel capture.
- **The vibrance falloff function** (base saturation space, exponent, and whether
  it depends on luminance). *Resolves with:* fitting to CS6 output on a
  saturation sweep.
- **Skin-tone protection model** — the hue band, its width, and whether it also
  depends on luminance or chroma. *Resolves with:* a CS6 saturation sweep over
  hues.
- **Negative Vibrance behavior** on near-gray colors (does it desaturate or leave
  them alone?). *Resolves with:* a CS6 test.
- **Saturation slider space/rounding** — whether it is HSL, HSV, or Lab-chroma
  scaling, which affects exact parity and banding. *Resolves with:* a CS6
  comparison (the Help's "less banding" claim hints at a perceptually smoother
  space than Hue/Saturation).
- **Whether the two sliders are independent or compose in a fixed order.**
  *Resolves with:* a CS6 probe with both non-zero.
- **CS6 presets for Vibrance** (the save-preset list omits it). *Resolves with:*
  a CS6 Properties Preset menu capture.
- **32-bpc availability conflict** between the two Help lines, and the behavior
  at 32 bpc. *Resolves with:* a 32-bpc CS6 test.
- **CMYK/Lab/Grayscale availability** and the chroma definition in each. *Resolves
  with:* a CS6 mode test.
- **Introduction version of the Vibrance adjustment layer** (CS3 vs CS4) for the
  "New in CS6" claim. *Resolves with:* the CS5 Help or a CS4 release note.
- **Exact `vibA` PSD serialization.** *Resolves with:* the PSD format
  specification plus a CS6-saved file.
