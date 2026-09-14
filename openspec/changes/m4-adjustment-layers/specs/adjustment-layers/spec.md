## ADDED Requirements

### Requirement: Adjustment data is carried opaquely in the document model

The document model SHALL carry an adjustment as raw bytes without interpreting
them. `pictura-core` MUST expose `AdjustmentData { key: [u8; 4], data: Vec<u8> }`
and `Layer.adjustment: Option<AdjustmentData>`, and MUST NOT depend on
`pictura-adjust` or any other adjustment-decoding crate.

#### Scenario: AdjustmentData preserves the key and payload verbatim

- **WHEN** an `AdjustmentData` is constructed with a 4-byte key and a payload
- **THEN** the key and every payload byte are readable unchanged, with no
  normalisation or re-encoding by the model

#### Scenario: Core stays free of an adjustment dependency

- **WHEN** the `pictura-core` crate manifest is inspected
- **THEN** it declares no dependency on `pictura-adjust`

#### Scenario: A pixel layer has no adjustment

- **WHEN** a layer without an adjustment is loaded or constructed
- **THEN** its `adjustment` field is `None`

### Requirement: Codec preserves adjustment blocks through PSD read/write

`pictura-codec` SHALL recognise the adjustment additional-layer-info keys
(`levl`, `curv`, `brit`, `expA`, `vibA`, `hue2`, `hue `, `blwh`, `phfl`, `mixr`,
`gdrm`, `invr`, `nvrt`, `post`, `thrs`, `selc`, `clrL`) and SHALL store each
block's key and payload bytes verbatim in `AdjustmentData`. On write it MUST emit
the same key and payload bytes it read, so a document with adjustment layers
round-trips byte-for-byte and unknown payload fields are not lost.

#### Scenario: Reader captures an adjustment key and payload

- **WHEN** a PSD layer record contains an additional-layer-info block whose key
  is one of the recognised adjustment keys
- **THEN** the resulting layer has `adjustment` set to that key and the block's
  exact payload bytes

#### Scenario: Read/write round-trip preserves key and bytes

- **WHEN** a document containing adjustment layers is written with `write_psd`
  and read back with `read_psd`
- **THEN** every adjustment layer's key and payload bytes are identical to the
  originals, including bytes the decoder does not understand

### Requirement: Render decodes the supported adjustment subset

`pictura-render` SHALL decode `AdjustmentData` into a destructive adjustment for
a practical subset: Invert (`nvrt` and the legacy `invr`), Posterize (`post`),
Threshold (`thrs`), Brightness/Contrast (`brit`), and Hue/Saturation (`hue2` and
the legacy `hue `). It MAY additionally decode Levels (`levl`). A key outside the
supported set, or a payload that fails its range checks, MUST decode to `None`
and MUST NOT produce an error.

#### Scenario: Supported keys decode

- **WHEN** a supported key with a valid payload is passed to `decode_adjustment`
- **THEN** it returns the matching adjustment variant with the decoded parameters

#### Scenario: Unknown or invalid payload decodes to no adjustment

- **WHEN** a key outside the supported set, or an out-of-range payload for a known
  key, is passed to `decode_adjustment`
- **THEN** it returns `None` without panicking or returning an error

### Requirement: An adjustment layer applies to the backdrop below it

Applying an adjustment layer SHALL apply the decoded adjustment to the running
backdrop (the accumulated composite of the layers below it) and then gate the
result by the layer's mask, opacity, and blend mode. For a decoded Invert layer
over a pixel layer, the result MUST match applying the same adjustment to the
flattened composite within a per-channel tolerance of ±1. Pixels where the
backdrop is transparent MUST be left unchanged.

#### Scenario: Invert adjustment layer matches the flattened composite

- **WHEN** an Invert adjustment layer is composited over a pixel layer
- **THEN** each output channel is within ±1 of applying Invert to the flattened
  composite of the same pixel layer

#### Scenario: Transparent backdrop gains no content

- **WHEN** the backdrop below the adjustment is transparent at a pixel
- **THEN** the adjustment contributes nothing at that pixel

### Requirement: Mask, opacity, and blend gate the adjustment effect

The adjustment layer's mask, opacity, and blend mode SHALL gate the adjustment
output before it is composited, so that masked-out pixels keep the original
backdrop and reduced opacity scales the effect toward the unadjusted value.

#### Scenario: Mask hides the effect

- **WHEN** a mask value is zero at a pixel
- **THEN** that pixel keeps the unadjusted backdrop

#### Scenario: Opacity scales the effect

- **WHEN** an adjustment layer has opacity 128
- **THEN** the result is approximately halfway between the unadjusted and fully
  adjusted backdrop

### Requirement: Unknown adjustments are preserved and not applied

The compositor SHALL treat an adjustment whose key is unknown to the renderer, or
whose payload cannot be decoded, as a no-op, leaving the composite unchanged
while the raw bytes remain in the document for saving. Neither reading nor
compositing an unsupported adjustment may error or drop its bytes.

#### Scenario: Undecodable key leaves the composite unchanged

- **WHEN** a document with an undecodable adjustment key is composited
- **THEN** the output is identical to compositing the same document without that
  adjustment layer

#### Scenario: Unsupported adjustment survives a save

- **WHEN** a document containing an undecodable adjustment is written to PSD
- **THEN** that adjustment's key and payload bytes are written back unchanged
