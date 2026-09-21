## ADDED Requirements

### Requirement: Layers property control row layout

The Layers panel SHALL lay out the blend-mode selection and the Opacity control
in one row that fractions the available slack between them, with the blend-mode
input taking the larger share and neither control dominating the row or pushing
the other out. The row SHALL stay usable at the panel's shared minimum width.

#### Scenario: The blend and opacity controls share the row [wpx_blend_row]

- **WHEN** the Layers panel is laid out at its normal width
- **THEN** the blend-mode input and the Opacity field each keep a usable width,
  and the blend-mode input does not absorb the whole row
