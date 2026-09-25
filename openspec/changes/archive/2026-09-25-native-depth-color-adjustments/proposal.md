# Proposal: native-depth-color-adjustments

## Why

Phase 2 (`native-depth-adjustments`) gave the tonal family a native-depth path
but the color-preserving adjustments still quantize to 8-bit. This extends the
same `apply_native` entry to the color family so those edits also run at the
document's depth.

## What Changes

- Extend `pictura_adjust::apply_native` to cover `HueSaturation`, `Vibrance`,
  `ColorBalance`, `BlackWhite`, `PhotoFilter`, `ChannelMixer`, and
  `SelectiveColor`, mirroring the 8-bit `color.rs` formulas in the unit domain.
- Keep the 8-bit `apply` byte-identical.
- The remaining unsupported set narrows to `Auto` (histogram-based),
  `ColorLookup` (3-D LUT), and the fill kinds.
- Modify the `native-depth-adjustments` "covered adjustments" requirement to
  list the wider covered set and move the unsupported example to `Auto`.

## Capabilities

### New Capabilities

<!-- None: this extends the existing entry point. -->

### Modified Capabilities

- `native-depth-adjustments`: the covered-adjustment list grows to include the
  color family; the unsupported ceiling shrinks accordingly.

## Impact

- `crates/pictura-adjust/src/native.rs`: add the color-family kernels.
- Tests: extend the u8-equality and precision tests to the new adjustments;
  keep the `Unsupported` test but point it at `Auto`.
- No new dependency, no app wiring.
