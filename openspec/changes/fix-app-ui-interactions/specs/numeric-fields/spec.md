## MODIFIED Requirements

### Requirement: Shared numeric field control

The system SHALL provide one reusable numeric input control (`NumericField`) that
presents a text value, an optional leading text label, and an optional slider
popup, and SHALL be configured by minimum, maximum, step, decimal places, suffix,
integer-or-double value type, and whether the popup is offered. A suffix SHALL be
rendered immediately after the value with no leading or trailing whitespace, so a
`px` suffix renders as `12px` and a percent suffix as `50%`. The control SHALL
emit a preview while a scrub or slider drag is in progress and a single commit
when the edit finishes, and SHALL emit nothing for a programmatic value sync. The
Layers panel Opacity and Fill fields SHALL be a thin configuration of this control
(`0..=100`, integer, `%` suffix) rather than a separate implementation, and every
control migrated to it SHALL keep its existing stored-value conversion.

#### Scenario: A configured field enforces its range and formatting [lpn_config]

- **WHEN** a `NumericField` is configured with a minimum, maximum, and decimal
  places and the user types an out-of-range value
- **THEN** the committed value is clamped to the range and displayed with the
  configured decimal places and suffix

#### Scenario: A suffix renders with no extra space [lpn_suffix_tight]

- **WHEN** a `NumericField` is configured with a `px` suffix and the value is `12`
- **THEN** the field displays `12px` with no space between the number and the
  suffix

#### Scenario: PercentField is one configuration [lpn_percent_thin]

- **WHEN** the Layers panel Opacity field is shown and edited
- **THEN** it exhibits the shared control's scrubbing, popup, and conversion
  behavior, and no second numeric implementation exists for it

#### Scenario: A programmatic sync emits no edit [lpn_sync]

- **WHEN** the owning panel reflects an existing value into the control without
  user input
- **THEN** no preview or commit is emitted

## ADDED Requirements

### Requirement: Options-bar numeric field configuration

The options-bar paint controls SHALL configure their units and popups correctly:
the Hardness, Opacity, and Flow controls SHALL display a `%` suffix, and the
Feather control SHALL be configured without a slider popup so it opens no popup
when clicked. The new-document and layer unit controls SHALL use a `px` suffix
with no leading space.

#### Scenario: Paint controls show a percent sign [lpn_paint_percent]

- **WHEN** the Brush options bar is shown
- **THEN** the Hardness, Opacity, and Flow fields each display their value with a
  `%` suffix

#### Scenario: Feather opens no popup [lpn_feather_no_popup]

- **WHEN** a selection tool's Feather control is clicked or activated
- **THEN** no slider popup opens and the value is edited only through the field
  itself

#### Scenario: The pixel suffix has no leading space [lpn_px_tight]

- **WHEN** a pixel-dimension field is shown with the value `12`
- **THEN** it displays `12px` with no space before the suffix
