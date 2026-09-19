## Why

Roadmap P3 gap G8: the codec preserves all 18 PSD adjustment keys, but
`pictura-render`'s `decode_adjustment` only understands
`nvrt`/`invr`, `post`, `thrs`, `brit`, `levl`, `hue2`/`hue `, and a 4-byte
in-house `SoCo`. Every other adjustment layer composites as a no-op, even
though `pictura-adjust` already implements the operations. The missing piece
is decoding the preserved payloads into the typed `*Params`.

## What Changes

- `decode_adjustment` in `crates/pictura-render/src/composite.rs` gains three
  decoders, each producing the matching `pictura_adjust::Adjustment` variant:
  - `expA` → `Adjustment::Exposure` (fixed struct: `u16` version = 1, then
    `f32` exposure, offset, gamma).
  - `vibA` → `Adjustment::Vibrance` (descriptor; keys `vibrance`, `Strt`).
  - `blwh` → `Adjustment::BlackWhite` (descriptor; keys `Rd  `, `Yllw`,
    `Grn `, `Cyn `, `Bl  `, `Mgnt`, `useTint`, `tintColor`).
- The descriptor-based keys are parsed with the existing `pictura-codec`
  descriptor DOM. `crates/pictura-codec/src/lib.rs` exposes a public
  `read_descriptor(bytes: &[u8]) -> Result<DescValue, PsdError>` built on the
  current crate-private reader. `camera_raw_options` is unchanged.
- A malformed, truncated, or wrong-version payload returns `None` (no-op); it
  never panics and never returns an error.
- The existing decoded keys keep their exact behaviour. The not-yet-grounded
  keys (`curv`, `selc`, `clrL`, `gdrm`, `phfl`, `mixr`, and a real Photoshop
  `SoCo` descriptor) continue to return `None` and are preserved on disk.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `adjustment-layer-rendering`: decode preserved PSD adjustment-layer payloads
  into `pictura-adjust` operations and render them through the existing
  adjustment composite path.

### Modified Capabilities

<!-- None. `adjustment-layers` owns encode/round-trip/author and is unchanged;
     this is a render-side decode capability that consumes the preserved bytes. -->

## Impact

- `crates/pictura-codec/src/lib.rs`: add a public `read_descriptor` wrapper;
  no change to the DOM or to `camera_raw_options`.
- `crates/pictura-render/src/composite.rs`: three decoder helpers plus the
  `decode_adjustment` match arms and their unit tests.
- No new dependency: `pictura-codec` and `pictura-adjust` are already runtime
  dependencies of `pictura-render`. No `pictura-adjust` change; its ops for
  Exposure, Vibrance, and BlackWhite already exist.
- GPU path: `gpu/mod.rs::adjustment_params` has no shader for the new
  adjustments, so a document containing one continues to be rejected there and
  falls back to the CPU composite, exactly as it does today. Only the CPU path
  gains decoding.
