## MODIFIED Requirements

### Requirement: Additional adjustment-layer kinds

The New Adjustment Layer menu, the Layers panel New Fill / Adjustment menu, and
the adjustment-layer creation path SHALL support all sixteen CS6 adjustment
kinds — Brightness/Contrast, Levels, Curves, Exposure, Vibrance, Hue/Saturation,
Color Balance, Black & White, Photo Filter, Channel Mixer, Color Lookup, Invert,
Posterize, Threshold, Gradient Map, and Selective Color — each creating a
non-destructive adjustment layer whose block carries CS6's dialog defaults and
whose display name matches the menu entry. The Layers panel creation menu SHALL
additionally offer the Solid Color… and Gradient… fill layers. An unrecognised
kind SHALL still be refused without adding a layer.

#### Scenario: Every supported kind creates a named layer

- **WHEN** each of the sixteen supported adjustment kinds is requested
- **THEN** a layer carrying that kind's adjustment block is added with the matching display name

#### Scenario: Unknown kind is refused

- **WHEN** an unrecognised adjustment kind is requested
- **THEN** no layer is added and the request fails

#### Scenario: The new kinds open on neutral defaults

- **WHEN** a Levels, Curves, Exposure, Vibrance, or Black & White layer is created
- **THEN** its parameters are that kind's CS6 dialog defaults, so a Curves layer opens as the identity

#### Scenario: The panel creation menu offers every kind

- **WHEN** the Layers panel New Fill / Adjustment menu is opened
- **THEN** it lists the Solid Color… and Gradient… fill entries and all sixteen adjustment kinds, and choosing an adjustment kind adds that layer
