# Specs delta: psd-tagged-block-8b64

## MODIFIED Requirements

### Requirement: Malformed input errors, never panics

Malformed or truncated layer data SHALL return a `PsdError` and SHALL NOT panic,
index out of bounds, or loop forever. This includes a bad file signature, a
section whose declared length exceeds the file, an impossible layer count, a
bogus channel length, a blend signature that is not `8BIM`, and a tagged-block
signature that is neither `8BIM` nor `8B64`.

#### Scenario: Truncated file is an error

- **WHEN** a valid PSD is truncated inside or before the layer section
- **THEN** `read_psd` returns an error rather than panicking

#### Scenario: Bogus layer count is an error

- **WHEN** a layer record count claims more records than the section holds
- **THEN** `read_psd` returns an error

#### Scenario: Bogus channel length is an error

- **WHEN** a channel-info length field is set to an impossible value
- **THEN** `read_psd` returns an error

#### Scenario: Bad signatures are errors

- **WHEN** the file signature is not `8BPS`, the layer blend signature is not `8BIM`, or a tagged-block signature is neither `8BIM` nor `8B64`
- **THEN** `read_psd` returns the corresponding error

## ADDED Requirements

### Requirement: A per-layer `8B64` tagged block is accepted

`read_psd` SHALL read a per-layer additional-layer-information block whose
signature is `8B64` exactly as it reads an `8BIM` block, preserving its key and
payload in the layer's opaque blocks, so a file carrying one opens rather than
erroring. The block's signature is not modeled and SHALL be written back as
`8BIM`, matching the document-level block behavior.

#### Scenario: An `8B64` block reads

- **WHEN** a layer record carries a tagged block with the `8B64` signature
- **THEN** `read_psd` succeeds and the block's key and payload appear in that layer's opaque blocks

#### Scenario: An unknown signature still errors

- **WHEN** a tagged-block signature is neither `8BIM` nor `8B64`
- **THEN** `read_psd` returns a typed error
