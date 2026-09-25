# psd-color-modes Specification

## ADDED Requirements

### Requirement: Duotone color-mode data is decoded

The system SHALL provide
`pictura_codec::duotone::parse_duotone(data: &[u8]) -> Option<DuotoneSpec>`
decoding the PSD Duotone Options block (the color-mode-data section of a Duotone
document) into a typed view: a `version`, a plate count, up to four `DuotoneInk`
records (each an ink color, a Pascal-string name, a 13-point transfer curve with
`-1` sentinels preserved, and an override flag), the dot-gain value, and the
overprint colors. The parser SHALL return `None` when the block is shorter than
524 bytes or the plate count is not 1, 2, 3, or 4, and MUST NOT panic. The
document open path, the grayscale normalization, and the opaque write-back SHALL
be unchanged.

#### Scenario: A two-plate spec parses

- **WHEN** a 524-byte Duotone Options block with plate count 2 and known ink colors, names, curves, dot gain, and one overprint color is parsed
- **THEN** `parse_duotone` returns a spec with those values and two inks

#### Scenario: A short or invalid block is rejected

- **WHEN** the block is shorter than 524 bytes, or its plate count is 0 or 5
- **THEN** `parse_duotone` returns `None` and does not panic

#### Scenario: The open path is unchanged

- **WHEN** a Duotone document is read
- **THEN** it still opens as grayscale-normalized RGB with `source_mode` `Duotone` and its color-mode data preserved
