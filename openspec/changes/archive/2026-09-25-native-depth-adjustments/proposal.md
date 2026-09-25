# Proposal: native-depth-adjustments

## Why

Phase 1 (`bit-depth-sample-model`) made the engine's buffers sample-typed and
the codec retain native samples, but every adjustment still runs through
`pictura_adjust::apply(adjustment, &mut PixelBuffer<u8>)` at 8-bit precision. A
16/32-bit document therefore loses its low bits on any tonal edit. This change
is Phase 2: apply the tonal adjustment family at the document's native depth.

## What Changes

- Add a native-depth entry to `pictura-adjust` that applies an `Adjustment` to a
  `Samples` store (`u8`/`u16`/`f32`) at full precision, without quantizing to
  8-bit first.
- Cover the **tonal map family** first: `Invert`, `Desaturate`, `Levels`,
  `Curves`, `BrightnessContrast`, `Exposure`, `Posterize`, `Threshold`,
  `GradientMap`. The color-preserving and histogram/3-D-LUT adjustments
  (`HueSaturation`, `Vibrance`, `BlackWhite`, `ChannelMixer`, `PhotoFilter`,
  `ColorBalance`, `SelectiveColor`, `Auto`, `ColorLookup`, and the fill kinds)
  stay 8-bit for now and return `Unsupported` on a native store.
- Keep the existing 8-bit `apply` output byte-identical: its kernels and
  rounding are the parity path and SHALL NOT change.
- Prove the precision is real: a depth-16 document edited at native depth,
  saved, and re-read SHALL carry samples whose low byte is not simply the high
  byte (`v != high * 257`), which today's widen-on-edit path always produces.

## Capabilities

### New Capabilities

- `native-depth-adjustments`: applying the tonal adjustment family to a
  `Samples` store at native depth, with the 8-bit path preserved.

### Modified Capabilities

<!-- None. psd-bit-depth's save behavior is unchanged; this adds an editing
     capability on top of the typed store. -->

## Impact

- `crates/pictura-core/src/samples.rs`: a `Sample` abstraction (or equivalent)
  so the kernels can run over `u8`/`u16`/`f32`.
- `crates/pictura-adjust/src/{apply,tonal,common}.rs`: a native application
  path for the tonal family; the 8-bit path untouched.
- `crates/pictura-adjust/src/lib.rs`: export the native entry.
- Tests: a native-depth test in `pictura-adjust` and an end-to-end codec test in
  `pictura-render` (which depends on both `pictura-adjust` and `pictura-codec`)
  that opens a 16-bit fixture, edits at native depth, saves, re-reads, and
  asserts the low bits survive.
- No new dependency; no app UI wiring yet (a later phase).
