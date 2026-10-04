# Spec Delta

## ADDED Requirements

### Requirement: Additional adjustment-layer kinds

The New Adjustment Layer menu and the adjustment-layer creation path SHALL additionally support `Levels`, `Curves`, `Exposure`, `Vibrance`, and `Black & White`, each creating a non-destructive adjustment layer whose block carries CS6's dialog defaults and whose display name matches the menu entry. An unrecognised kind SHALL still be refused without adding a layer.

#### Scenario: Every supported kind creates a named layer

- **WHEN** each of the sixteen supported adjustment kinds is requested
- **THEN** a layer carrying that kind's adjustment block is added with the matching display name

#### Scenario: Unknown kind is refused

- **WHEN** an unrecognised adjustment kind is requested
- **THEN** no layer is added and the request fails

#### Scenario: The new kinds open on neutral defaults

- **WHEN** a Levels, Curves, Exposure, Vibrance, or Black & White layer is created
- **THEN** its parameters are that kind's CS6 dialog defaults, so a Curves layer opens as the identity
