# Spec Delta

## ADDED Requirements

### Requirement: Shadows/Highlights adjustment

The system SHALL implement `Adjustment::ShadowsHighlights(ShadowsHighlightsParams { shadows_amount, highlights_amount })` as a pointwise operator. Both amounts SHALL default to `0.0` and span `0.0..=100.0`. It SHALL derive a 256-entry luminance delta table: a shadow weight that peaks at black and fades to zero by mid-grey scaled by `(shadows_amount / 100) * 0.35 * 255`, minus a highlight weight that peaks at white and fades to zero by mid-grey scaled by `(highlights_amount / 100) * 0.30 * 255`; each pixel's Rec.601 luminance selects its delta, which is added to all three colour channels and clamped to `0..=255`. Both amounts zero SHALL be an exact identity, alpha SHALL be preserved, and a non-finite or out-of-range amount SHALL be rejected as `AdjustError::InvalidParams` without changing the buffer.

#### Scenario: Lifting shadows and pulling highlights

- **WHEN** Shadows/Highlights with a positive shadow amount is applied to a buffer containing a dark pixel, or with a positive highlight amount to a bright pixel
- **THEN** the dark pixel lightens or the bright pixel darkens respectively

#### Scenario: Neutral amounts are identity

- **WHEN** Shadows/Highlights is applied with both amounts zero
- **THEN** the buffer is bit-exactly unchanged and alpha is preserved

#### Scenario: Invalid amounts are rejected

- **WHEN** either amount is non-finite or outside `0.0..=100.0`
- **THEN** `apply` returns `AdjustError::InvalidParams` and does not panic

### Requirement: Replace Color adjustment

The system SHALL implement `Adjustment::ReplaceColor(ReplaceColorParams { samples, fuzziness, localized, hue, saturation, lightness })`. `samples` is a list of `(x, y, rgb)` picks; `fuzziness` spans `0.0..=200.0`; `hue` spans `-180.0..=180.0` degrees; `saturation` and `lightness` span `-100.0..=100.0`. For each pixel the system SHALL compute a weight as the maximum over samples of a Chebyshev colour-distance ramp (exact match when fuzziness is 0), optionally attenuated by a Gaussian of the pixel-to-sample distance when `localized` is set, then blend the pixel toward its hue-rotated, saturation- and lightness-shifted HSL result by that weight. An empty sample list SHALL be an exact identity, alpha SHALL be preserved, and out-of-range or non-finite parameters SHALL be rejected as `AdjustError::InvalidParams`.

#### Scenario: Zero fuzziness selects exact samples only

- **WHEN** Replace Color with fuzziness 0 and one sample is applied to a buffer
- **THEN** only pixels exactly equal to the sample colour change, and no others

#### Scenario: Fuzziness shifts near colours

- **WHEN** Replace Color with a positive fuzziness and a non-zero hue is applied to pixels near the sample colour
- **THEN** those pixels shift toward the new hue

#### Scenario: Empty samples is identity and alpha is preserved

- **WHEN** Replace Color is applied with an empty sample list, or to an RGBA buffer
- **THEN** the buffer is unchanged, and in the RGBA case the alpha plane is bit-identical

### Requirement: Destructive Color Lookup dialog

`Image ▸ Adjustments ▸ Color Lookup` SHALL open a dialog offering the CS6 preset looks `None`, `Warm Contrast`, `Cool Shadows`, `Faded Film`, `Bleach Bypass`, `Crisp Warm`, and `Moonlight`, defaulting to `None` (the identity). Choosing a preset SHALL rebuild the adjustment's embedded lookup so that `None` leaves the image unchanged and any other preset changes the colour channels while preserving alpha. OK SHALL commit one history state named for the adjustment and Cancel SHALL restore the pre-dialog pixels.

#### Scenario: The dialog opens on the identity look

- **WHEN** the Color Lookup dialog is opened and accepted without changing the preset
- **THEN** the image is unchanged and one adjustment state is recorded

#### Scenario: A non-identity preset changes the image

- **WHEN** `Warm Contrast` is selected and the dialog is accepted
- **THEN** the colour channels change and alpha is preserved

### Requirement: Destructive adjustment dialog enablement

Every destructive `Image ▸ Adjustments` entry backed by an engine kind SHALL be enabled only when the active layer is an editable pixel layer, and choosing it SHALL preview on the canvas without recording history until OK commits exactly one state named for the adjustment; Cancel SHALL restore the pre-dialog pixels bit-identically.

#### Scenario: Cancel restores the pre-dialog pixels

- **WHEN** a destructive adjustment dialog is opened, previewed, and cancelled
- **THEN** the document pixels equal the pre-dialog pixels bit for bit and no history state is recorded

#### Scenario: Refused with no editable layer

- **WHEN** an adjustment dialog is invoked with no document or with a group or adjustment layer active
- **THEN** the command is disabled or refused and the document is unchanged
