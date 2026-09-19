# adjustment-layer-rendering Specification

## Purpose
TBD - created by archiving change adjustment-payload-decode. Update Purpose after archive.
## Requirements
### Requirement: Committed adjustment payloads decode to typed parameters

`decode_adjustment` SHALL decode the committed PSD adjustment payloads into the
matching `pictura_adjust::Adjustment` variant: `expA` to
`Adjustment::Exposure(ExposureParams)`, `vibA` to
`Adjustment::Vibrance(VibranceParams)`, and `blwh` to
`Adjustment::BlackWhite(BlackWhiteParams)`. The fixed struct `expA` SHALL be
read as a `u16` version equal to 1 followed by `f32` exposure, offset, and
gamma. The descriptor payloads `vibA` and `blwh` SHALL be read as a 4-byte
descriptor version equal to 16 followed by a descriptor object whose keys carry
the slider values.

#### Scenario: Exposure payload decodes to ExposureParams

- **WHEN** an `expA` payload has version 1 and finite exposure, offset, and gamma values
- **THEN** `decode_adjustment` returns `Adjustment::Exposure` carrying those three values

#### Scenario: Vibrance payload decodes to VibranceParams

- **WHEN** a `vibA` descriptor contains the `vibrance` and `Strt` integer keys in range
- **THEN** `decode_adjustment` returns `Adjustment::Vibrance` with those vibrance and saturation values

#### Scenario: Black and white payload decodes to BlackWhiteParams

- **WHEN** a `blwh` descriptor contains the `Rd  `, `Yllw`, `Grn `, `Cyn `, `Bl  `, `Mgnt`, `useTint`, and `tintColor` keys
- **THEN** `decode_adjustment` returns `Adjustment::BlackWhite` with the six channel percentages, the tint flag, and the tint colour

#### Scenario: Malformed exposure payload is a no-op

- **WHEN** an `expA` payload is truncated, has a version other than 1, carries a non-finite value, or has gamma at or below zero
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed vibrance payload is a no-op

- **WHEN** a `vibA` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Malformed black and white payload is a no-op

- **WHEN** a `blwh` payload is truncated, does not parse as a descriptor object, or carries an out-of-range or wrong-typed key
- **THEN** `decode_adjustment` returns `None`

### Requirement: Descriptor payloads are parsed with the shared codec DOM

The descriptor-based payloads SHALL be parsed with the `pictura-codec`
descriptor DOM. `crates/pictura-codec/src/lib.rs` SHALL expose a public
`read_descriptor(bytes: &[u8]) -> Result<DescValue, PsdError>` built on the
existing descriptor reader, and `pictura-render` SHALL use it rather than a
second descriptor parser. The existing `camera_raw_options` entry point SHALL
continue to decode Camera Raw `Fltr` options unchanged.

#### Scenario: Descriptor keys are read through the shared DOM

- **WHEN** a `vibA` or `blwh` payload is decoded
- **THEN** its version-16 descriptor is parsed by `pictura-codec::read_descriptor`

#### Scenario: Camera Raw options still decode

- **WHEN** `pictura_codec::camera_raw_options` is called on a Camera Raw `Fltr` buffer
- **THEN** it returns the same `DescValue` it returned before this change

#### Scenario: Truncated descriptor is an error, not a panic

- **WHEN** a descriptor payload is truncated
- **THEN** `read_descriptor` returns a `PsdError` and the renderer turns it into `None`

### Requirement: Existing decoded keys are unchanged

The renderer SHALL keep decoding the keys it already understood
(`nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, and the 4-byte
`SoCo`) with their current payload layouts and current `Adjustment` values.

#### Scenario: Existing keys still decode

- **WHEN** an `nvrt`, `post`, `thrs`, `brit`, `levl`, `hue2`, or 4-byte `SoCo` payload is decoded
- **THEN** it yields the same `Adjustment` as before this change

#### Scenario: Unknown key is still a no-op

- **WHEN** a payload key is not a committed or existing key
- **THEN** `decode_adjustment` returns `None` and the composite leaves the backdrop unchanged

### Requirement: A decoded adjustment renders as a non-no-op composite

The renderer SHALL composite a document containing an adjustment layer whose
payload is one of the committed keys without decoding the payload to `None`, and
the resulting backdrop SHALL differ from the same document with the adjustment
layer absent.

#### Scenario: Committed adjustment changes the composite

- **WHEN** a document has an adjustment layer whose payload is a well-formed committed key over a non-uniform backdrop
- **THEN** the composited result differs from the backdrop-only composite

#### Scenario: Masked-out layer stays a no-op

- **WHEN** such a layer's mask hides the whole canvas
- **THEN** the composite equals the backdrop-only composite

### Requirement: Deferred adjustment payloads remain no-ops

The renderer SHALL return `None` from `decode_adjustment` for the deferred
payloads `curv`, `selc`, `clrL`, `gdrm`, `phfl`, `mixr`, and a real Photoshop
`SoCo` descriptor. These payloads SHALL remain preserved on disk and their
layers SHALL leave the backdrop unchanged.

#### Scenario: Deferred keys return None

- **WHEN** a `curv`, `selc`, `clrL`, `gdrm`, `phfl`, or `mixr` payload is decoded
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Real solid-color descriptor still returns None

- **WHEN** a `SoCo` payload is a Photoshop descriptor rather than the 4-byte in-house form
- **THEN** `decode_adjustment` returns `None`

#### Scenario: Deferred layer does not change the composite

- **WHEN** a document contains an adjustment layer with a deferred payload over a backdrop
- **THEN** the composite equals the backdrop-only composite

