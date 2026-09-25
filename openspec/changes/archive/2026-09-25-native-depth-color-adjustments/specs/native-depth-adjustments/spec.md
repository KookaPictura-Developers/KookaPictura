# native-depth-color-adjustments Specification

## MODIFIED Requirements

### Requirement: Covered adjustments and the unsupported ceiling

The native entry SHALL support `Invert`, `Desaturate`, `Levels`, `Curves`,
`BrightnessContrast`, `Exposure`, `Posterize`, `Threshold`, `GradientMap`,
`HueSaturation`, `Vibrance`, `ColorBalance`, `BlackWhite`, `PhotoFilter`,
`ChannelMixer`, and `SelectiveColor`. For any other `Adjustment` it SHALL return
`AdjustError::Unsupported` without mutating the store.

#### Scenario: A covered color adjustment applies at native depth

- **WHEN** the native entry is called with `HueSaturation` on a depth-16 store
- **THEN** it succeeds and each sample's low byte is not constrained to the
  8-bit widen

#### Scenario: An unsupported adjustment is refused without mutation

- **WHEN** the native entry is called with `Auto` on a depth-16 store
- **THEN** it returns `Unsupported` and the samples are unchanged
