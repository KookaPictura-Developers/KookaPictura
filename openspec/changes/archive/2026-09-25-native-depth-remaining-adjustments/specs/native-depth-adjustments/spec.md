# native-depth-remaining-adjustments Specification

## MODIFIED Requirements

### Requirement: Covered adjustments and the unsupported ceiling

The native entry SHALL support `Invert`, `Desaturate`, `Levels`, `Curves`,
`BrightnessContrast`, `Exposure`, `Posterize`, `Threshold`, `GradientMap`,
`HueSaturation`, `Vibrance`, `ColorBalance`, `BlackWhite`, `PhotoFilter`,
`ChannelMixer`, `SelectiveColor`, `Auto`, and `ColorLookup`. For any other
`Adjustment` — the generative fill kinds — it SHALL return
`AdjustError::Unsupported` without mutating the store.

#### Scenario: A covered color adjustment applies at native depth

- **WHEN** the native entry is called with `HueSaturation` on a depth-16 store
- **THEN** it succeeds and each sample's low byte is not constrained to the
  8-bit widen

#### Scenario: Auto applies at native depth

- **WHEN** the native entry is called with `Auto(AutoKind::Color)` on a depth-16
  store
- **THEN** it succeeds without an 8-bit round-trip

#### Scenario: An unsupported adjustment is refused without mutation

- **WHEN** the native entry is called with a `SolidFill`/`GradientFill`/
  `PatternFill` adjustment on a depth-16 store
- **THEN** it returns `Unsupported` and the samples are unchanged

## ADDED Requirements

### Requirement: Auto and ColorLookup keep native precision

`Auto` and `ColorLookup` SHALL be applied at the store's own resolution: the
`Auto` histogram and stretch SHALL use the store's level count (`256` for `u8`,
`65536` for `u16`, a `65536`-bin `[0,1]` histogram for `f32`), and `ColorLookup`
SHALL sample its 3-D LUT on unit values without an intermediate 8-bit
quantization. A `ColorLookup` payload that is abstract-profile, device-link, or
otherwise unparseable SHALL remain a no-op as in the 8-bit path.

#### Scenario: A 16-bit ColorLookup keeps precision

- **WHEN** a non-identity parsed 3-D LUT is applied to a depth-16 store through
  the native entry
- **THEN** at least one output sample is not the 8-bit widening of the input

#### Scenario: Native Auto equals the 8-bit Auto on a u8 store

- **WHEN** `Auto(Tone)`, `Auto(Contrast)`, and `Auto(Color)` are applied to a
  fixed `u8` store through both `apply` and `apply_native`
- **THEN** the outputs are byte-identical

#### Scenario: An unparseable ColorLookup is a no-op

- **WHEN** the native entry is called with a `ColorLookup` whose payload cannot
  be parsed
- **THEN** it succeeds and the samples are unchanged
