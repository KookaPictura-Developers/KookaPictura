## ADDED Requirements

### Requirement: Numeric field input alignment

The text of every numeric field SHALL be left-aligned, with the value and its
suffix starting at the field's left edge rather than right-aligned.

#### Scenario: Numeric values are left-aligned [lpn_left_align]

- **WHEN** any numeric field (a layers Opacity/Fill field, an options-bar field,
  or a dialog field) shows a value
- **THEN** the value and its suffix are left-aligned within the field
