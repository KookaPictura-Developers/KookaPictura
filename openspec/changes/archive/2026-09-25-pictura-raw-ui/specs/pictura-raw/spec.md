# pictura-raw Specification

## ADDED Requirements

### Requirement: Pictura Raw apply command

The system SHALL provide a bridge command
`apply_pictura_raw_filter(path, <11 values>)` taking the active layer path and the
11 PV2012 Basic controls (temperature, tint, exposure, contrast, highlights,
shadows, whites, blacks, clarity, vibrance, saturation). When `path` resolves to
a plain convertible raster pixel layer it SHALL first convert the layer to an
embedded smart object and then apply Pictura Raw; when it resolves to an embedded
smart-object layer it SHALL skip the conversion and apply Pictura Raw. On success it SHALL
recomposite and record exactly one `"Pictura Raw"` history state. It MUST
return false and record no history for a missing document or target, or when the
engine refuses the conversion or the apply.

#### Scenario: Raster target converts then filters in one state

- **WHEN** `apply_pictura_raw_filter` runs on a plain raster pixel layer of an 8-bit document
- **THEN** the layer becomes an embedded smart object, its pixels change, and exactly one `"Pictura Raw"` state is recorded

#### Scenario: Embedded smart object re-applies in one state

- **WHEN** `apply_pictura_raw_filter` runs again on that smart-object layer
- **THEN** exactly one additional `"Pictura Raw"` state is recorded

#### Scenario: Ineligible target records nothing

- **WHEN** the target is a group, a legacy placed layer, or there is no document
- **THEN** the command returns false and the history count is unchanged

### Requirement: Pictura Raw settings readback

The system SHALL provide a bridge read command `layer_pictura_raw_settings(path)` that
returns the 11 stored controls of the layer's camera-raw smart filter in the
documented order as a space-separated string, or an empty string when the layer
has no camera-raw filter. It MUST be a pure read that mutates no state.

#### Scenario: Stored settings are returned for prefill

- **WHEN** a camera-raw filter is applied and its settings are read back
- **THEN** the returned string carries the applied values in dialog order

#### Scenario: A layer without a filter returns empty

- **WHEN** `layer_pictura_raw_settings` is called on a layer with no camera-raw filter
- **THEN** it returns an empty string

### Requirement: Pictura Raw dialog

The system SHALL provide a modal `Pictura Raw` dialog exposing the 11
PV2012 Basic controls as labelled numeric fields with OK and Cancel. Temperature
and Tint SHALL use the approximate JPEG `-100…+100` scale, Exposure SHALL use
`-5…+5`, and the remaining controls SHALL use `-100…+100`. The dialog MUST NOT
include ACR tabs, a filmstrip, a histogram, or a live preview. Ranges not stated
in the Camera Raw Filter specification SHALL be treated as inferred.

#### Scenario: OK returns the edited values

- **WHEN** the dialog is accepted after editing a control
- **THEN** it reports the 11 values, prefilled from the supplied initial values

#### Scenario: Cancel reports no values

- **WHEN** the dialog is rejected
- **THEN** the caller applies nothing and no history state is recorded

### Requirement: Pictura Raw menu command

The system SHALL expose `Filter > Pictura Raw…` as an implemented command.
The menu label MUST read exactly `Pictura Raw…`. The command SHALL be
enabled only when a document is open and the current layer is either a
convertible raster pixel layer or a replaceable embedded smart object; it SHALL
prefill the dialog from `layer_pictura_raw_settings` and apply on OK.

#### Scenario: Command is enabled for an eligible target

- **WHEN** a document is open with a convertible raster or embedded smart-object current layer
- **THEN** the command is enabled

#### Scenario: Command is disabled without a document

- **WHEN** no document is open
- **THEN** the command is disabled
