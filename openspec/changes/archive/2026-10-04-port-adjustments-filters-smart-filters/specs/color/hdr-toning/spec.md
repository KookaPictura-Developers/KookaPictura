# Spec Delta

## ADDED Requirements

### Requirement: Local Adaptation HDR Toning operator

The system SHALL provide a deterministic Local Adaptation tone-mapping operator over a planar 8-bit RGB(A) buffer, parameterised by edge-glow radius (`1..=500`), strength (`0.01..=4.0`), gamma (`0.01..=9.99`), exposure (`-5.0..=5.0`), detail (`-100..=300`), and shadow, highlight, vibrance, and saturation (each `-100..=100`). It SHALL blur a copy of the colour planes to a local base, split log-luminance into a magnitude-gated detail term, compress the base about the mean log-luminance pivot using the gamma, apply exposure, recover shadows and highlights, roll off the shoulder and toe, add the luminance delta to all colour channels, and gate the vibrance/saturation adjustment by the original saturation. Alpha SHALL be preserved, and an out-of-range parameter SHALL be rejected with a filter error and no mutation.

#### Scenario: A non-uniform image is tone-mapped and alpha preserved

- **WHEN** the operator runs on a non-uniform RGBA buffer with default parameters
- **THEN** the colour planes change and the alpha plane is bit-identical

#### Scenario: Out-of-range parameters are rejected

- **WHEN** any parameter lies outside its documented range
- **THEN** the operator returns an error and leaves the buffer unchanged

### Requirement: HDR Toning dialog with CS6 presets

`Image ▸ Adjustments ▸ HDR Toning` SHALL open a dialog exposing the nine Local Adaptation controls and a preset list containing CS6's presets (`Default`, `City Twilight`, `Flat`, `Monochromatic Artistic`, `Monochromatic High Contrast`, `Monochromatic Low Contrast`, `Monochromatic`, `More Saturated`, `Photorealistic High Contrast`, `Photorealistic Low Contrast`, `Photorealistic`, `RCS`, `Saturated`, `ScottS`, `Surrealistic High Contrast`, `Surrealistic Low Contrast`, `Surrealistic`) plus `Custom`. Selecting a preset SHALL load all nine controls with the preset's values. The dialog SHALL preview on the canvas without recording history, OK SHALL commit one history state named `HDR Toning`, and Cancel SHALL restore the pre-dialog pixels bit-identically.

#### Scenario: Selecting a preset populates the controls

- **WHEN** a listed preset is chosen in the dialog
- **THEN** the nine controls show that preset's values

#### Scenario: Apply commits one state and cancel restores

- **WHEN** the dialog is accepted after previewing, or cancelled after previewing
- **THEN** OK records exactly one `HDR Toning` history state, and Cancel restores the pre-dialog pixels bit-identically
