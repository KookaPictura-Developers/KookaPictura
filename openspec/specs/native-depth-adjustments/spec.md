# native-depth-adjustments Specification

## Purpose
TBD - created by archiving change native-depth-adjustments. Update Purpose after archive.
## Requirements
### Requirement: Tonal adjustments apply at native depth

`pictura_adjust` SHALL expose an application entry that applies an `Adjustment`
to a native-depth `Samples` store (`u8`, `u16`, or `f32`) without first
quantizing to 8-bit, given the buffer's width, height, and channel count. For a
`u16` or `f32` store the per-sample math SHALL run in a unit-domain `f64`
representation and the result SHALL be encoded back to the store's depth, so
the output precision is the store's, not 8-bit.

#### Scenario: A 16-bit Levels edit keeps 16-bit precision

- **WHEN** a tonal adjustment is applied to a depth-16 `Samples` store whose
  8-bit narrowing is unchanged by the edit
- **THEN** at least one sample's low byte differs from its high byte, i.e. the
  result is not the 8-bit widen `high * 257`

#### Scenario: A 32-bit store is adjusted in unit space

- **WHEN** an `Exposure` adjustment is applied to a depth-32 `Samples` store
- **THEN** the samples are adjusted in `[0, 1]` and values above `1.0` remain
  representable (not clipped by an 8-bit round-trip)

### Requirement: The 8-bit application is unchanged

The existing 8-bit `apply(adjustment, &mut PixelBuffer<u8>)` SHALL keep its
current kernels, rounding, and results. For each covered tonal adjustment the
native entry applied to a `u8` store SHALL produce the same bytes as `apply` on
the same input.

#### Scenario: Native u8 equals the existing apply

- **WHEN** each covered adjustment is applied to a fixed `u8` image through both
  `apply` and the native entry
- **THEN** the outputs are byte-identical

#### Scenario: The ImageMagick oracle is unmoved

- **WHEN** the `pictura-adjust` oracle tests run after this change
- **THEN** they pass with no golden or tolerance change

### Requirement: Covered adjustments and the unsupported ceiling

The native entry SHALL support `Invert`, `Desaturate`, `Levels`, `Curves`,
`BrightnessContrast`, `Exposure`, `Posterize`, `Threshold`, and `GradientMap`.
For any other `Adjustment` it SHALL return `AdjustError::Unsupported` without
mutating the store.

#### Scenario: An unsupported adjustment is refused without mutation

- **WHEN** the native entry is called with `HueSaturation` on a depth-16 store
- **THEN** it returns `Unsupported` and the samples are unchanged

### Requirement: A native edit survives a PSD round-trip

A depth-16 document read from PSD, adjusted at native depth, and written SHALL
re-emit the adjusted native samples, and reading the output SHALL yield the same
native samples (byte-identically at depth 16).

#### Scenario: Edit, save, re-read keeps the low bits

- **WHEN** a depth-16 RGB document is read, a covered tonal adjustment is applied
  to its retained native composite, the 8-bit composite is set to the narrowing,
  and the document is written and re-read
- **THEN** the re-read retained samples equal the adjusted native samples and are
  not the 8-bit widening of the edited composite

### Requirement: Sample conversion is total and clamped

The unit-domain conversion SHALL be total: `f32` inputs below `0.0` SHALL map to
`0.0`, above `1.0` to `1.0`, and `NaN` to `0.0`, so no adjustment can produce a
non-finite output from a finite store.

#### Scenario: Out-of-range f32 maps into the unit interval

- **WHEN** a depth-32 store holds `-0.5`, `1.5`, and `NaN`
- **THEN** their unit values are `0.0`, `1.0`, and `0.0` respectively

