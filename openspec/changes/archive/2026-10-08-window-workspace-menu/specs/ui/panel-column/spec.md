# Spec Delta

## MODIFIED Requirements

### Requirement: Default panel groups

The system SHALL open a fresh session in the authentic CS6 Essentials
workspace: a wider main right-hand column holding the groups `Color | Swatches`,
`Adjustments | Styles`, and `Layers | Channels | Paths`, and a narrower
secondary right-hand column collapsed to icons holding `History` and
`Properties`. `Navigator | Histogram | Info` SHALL NOT be in a default visible
group. Every other registered panel — `Actions`, `Gradients`, `Patterns`,
`Notes`, `Brush`, `Clone Source`, and the type panels — SHALL remain registered
and reachable from `Window > Panels` but SHALL NOT be in a default visible
group. (This application does not provide a `Libraries` panel.)

#### Scenario: Default groups match CS6 Essentials [m41_groups]

- **WHEN** the frame starts with a fresh session
- **THEN** the main right-hand column contains the groups `Color | Swatches`,
  `Adjustments | Styles`, and `Layers | Channels | Paths`, and a second
  right-hand column to its left is iconic and holds `History` and `Properties`

#### Scenario: A folded panel stays reachable [m41_groups]

- **WHEN** the `Window > Panels > Navigator` toggle is invoked
- **THEN** the Navigator panel can be shown even though it is not in a default
  visible group
