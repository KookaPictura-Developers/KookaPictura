## Why

Photoshop adjustment layers transform everything below them without destroying
pixel data, but the current codebase can only decode adjustment payloads and has
no document representation for them. Without an opaque model, a verbatim codec,
and a compositor path, adjustment layers cannot be opened, re-saved, or rendered.

## What Changes

- Add `AdjustmentData { key: [u8; 4], data: Vec<u8> }` to `pictura-core` and
  `Layer.adjustment: Option<AdjustmentData>`; the model carries the adjustment
  opaquely and `pictura-core` gains **no** `pictura-adjust` dependency.
- `pictura-codec` recognises the adjustment additional-layer-info keys
  (`levl`, `curv`, `brit`, `expA`, `vibA`, `hue2`, `hue `, `blwh`, `phfl`, `mixr`,
  `gdrm`, `invr`, `nvrt`, `post`, `thrs`, `selc`, `clrL`) and reads/writes the
  block verbatim, so unknown fields survive a save unchanged.
- `pictura-render` (which may depend on `pictura-adjust`) decodes a practical
  subset — Invert (`nvrt`/`invr`), Posterize (`post`), Threshold (`thrs`),
  Brightness/Contrast (`brit`), Hue/Saturation (`hue2`/`hue `), and Levels
  (`levl`). Unknown or undecodable keys decode to `None` and apply as a no-op,
  never an error.
- Applying an adjustment layer runs the decoded adjustment over the running
  backdrop (the accumulated composite below it), then gates the result by the
  layer's mask, opacity, and blend mode through the existing compositor path.

## Capabilities

### New Capabilities

- `adjustment-layers`: an adjustment layer carries an opaque adjustment, applies
  it to everything below it gated by mask/opacity/blend, and round-trips through
  PSD read/write with its key and payload bytes intact.

### Modified Capabilities

<!-- None: openspec/specs/ is empty; this change introduces a new capability. -->

## Impact

- `crates/pictura-core`: new `AdjustmentData` struct and `Layer.adjustment`
  field; no new dependency.
- `crates/pictura-codec`: adjustment-key recognition in `read_layer_record`,
  verbatim write in `write_layer_info`, and round-trip tests.
- `crates/pictura-render`: `decode_adjustment`, `composite_adjustment`, and
  encoders for the subset; new `pictura-adjust` dependency.
- Out of scope: full descriptor coverage for every adjustment, adjustment-layer
  clipping subtleties, and Gradient Map / Selective Color / Color Lookup decode.
