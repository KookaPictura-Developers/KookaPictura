# pictura-raw Specification

## Purpose
TBD - created by archiving change pictura-raw-core. Update Purpose after archive.
## Requirements
### Requirement: Pictura Raw settings alias

The system SHALL expose `pictura_core::PicturaRawSettings` as a thin type alias of
`pictura_core::CrsSettings`, carrying the 11 optional `f64` PV2012 Basic controls
(temperature, tint, exposure, contrast, highlights, shadows, whites, blacks,
clarity, vibrance, saturation), and MUST NOT define a second settings struct.

#### Scenario: Alias resolves to the same type

- **WHEN** a `PicturaRawSettings` value is passed where a `CrsSettings` is expected
- **THEN** it compiles without conversion

### Requirement: Pictura Raw Fltr codec

The system SHALL provide `pictura_codec::decode_pictura_raw_settings(&[u8]) ->
PicturaRawSettings` and `pictura_codec::encode_pictura_raw_fltr(&PicturaRawSettings) -> DescValue`. The
encoding SHALL use the on-disk `Fltr` key map grounded on the CC fixture:
temperature `Temp`, tint `Tint`, contrast `Cr12`, highlights `Hi12`, shadows
`Sh12`, whites `Wh12`, blacks `Bk12`, clarity `Cl12`, vibrance `Vibr`, and
saturation `Strt` as `Long`, and exposure `Ex12` as a `Double`. Decoding SHALL
accept `Long` or `Double` and SHALL leave a field `None` for a missing,
non-finite, or unmodeled value.

#### Scenario: Set fields round-trip

- **WHEN** settings with all 11 fields set are encoded and decoded
- **THEN** the decoded settings equal the input

#### Scenario: Decode is tolerant

- **WHEN** the buffer omits a key, carries a non-numeric value, or is not a descriptor
- **THEN** the affected fields are `None` and no panic occurs

### Requirement: Attaching the filter to a layer

The system SHALL provide `pictura_codec::attach_pictura_raw_filter(&mut Layer,
&PicturaRawSettings)`. When the layer already carries a pictura-raw smart filter it SHALL
update that filter's `Fltr` in place. When a preserved `SoLd`/`SoLE` has no
pictura-raw entry it SHALL insert one while preserving every other descriptor key.
When the layer has an authored embedded smart object with no preserved block it
SHALL record the filter so the writer authors it into the `SoLd`. It MUST return
an error without mutating a layer that has no smart object.

#### Scenario: Settings survive save and read

- **WHEN** the filter is attached and the document is written and read back
- **THEN** the layer's pictura-raw smart filter carries the same settings

### Requirement: Pictura Raw render

The system SHALL provide `pictura_adjust::render_pictura_raw(&PixelBuffer,
&PicturaRawSettings) -> Result<PixelBuffer, AdjustError>` over 8-bit 3- or 4-channel
buffers, working in `f32` in the order white balance, exposure, contrast,
highlights/shadows/whites/blacks, clarity, vibrance, saturation, and preserving
the channel count and alpha. Default settings MUST be a byte-identical no-op and
the result MUST be deterministic.

#### Scenario: Default settings are a no-op

- **WHEN** `render_pictura_raw` is called with all-`None` settings
- **THEN** the output is byte-identical to the input

#### Scenario: Exposure is monotonic

- **WHEN** a mid-grey pixel is rendered with positive exposure
- **THEN** it is brighter than the input, and a negative exposure is darker

#### Scenario: Malformed buffers are refused

- **WHEN** the buffer has fewer than 3 channels or a length that does not match its dimensions
- **THEN** `render_pictura_raw` returns an `AdjustError` and does not panic

### Requirement: Pictura Raw document op

The system SHALL provide `pictura_render::apply_pictura_raw(&mut Document, &str,
&PicturaRawSettings) -> bool`. It SHALL resolve a smart-object layer with an embedded,
decodable payload, render its source, run the Pictura Raw pipeline, write the result into
the layer's color channels while preserving alpha and the layer rect, and attach
the settings. It MUST return `false` without mutating for a missing path, a
group, an adjustment layer, a non-smart-object, a non-embedded object, or an
empty or undecodable payload.

#### Scenario: Apply bakes the proxy and records settings

- **WHEN** `apply_pictura_raw` runs on a converted smart-object layer
- **THEN** its color channels change, its alpha is unchanged, and the filter settings round-trip through save and read

#### Scenario: Ineligible targets are refused

- **WHEN** the target is a plain pixel layer or a missing path
- **THEN** `apply_pictura_raw` returns `false` and the document is unchanged

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
returns the 11 stored controls of the layer's pictura-raw smart filter in the
documented order as a space-separated string, or an empty string when the layer
has no pictura-raw filter. It MUST be a pure read that mutates no state.

#### Scenario: Stored settings are returned for prefill

- **WHEN** a pictura-raw filter is applied and its settings are read back
- **THEN** the returned string carries the applied values in dialog order

#### Scenario: A layer without a filter returns empty

- **WHEN** `layer_pictura_raw_settings` is called on a layer with no pictura-raw filter
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

