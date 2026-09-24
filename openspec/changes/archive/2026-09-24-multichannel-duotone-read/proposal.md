# Proposal: multichannel-duotone-read

## Why

Roadmap G2/G3: Multichannel (mode 7) and Duotone (mode 8) still return `PsdError::Unsupported`, so those files cannot open at all. Duotone has a documented path (treat as grayscale, preserve the duotone spec); Multichannel needs a channel-count-aware map to the RGB working space.

## What Changes

- **Duotone (8):** open as a 1-channel document, normalize to RGB with the existing grayscale path, retain source planes, set `source_mode = Duotone`, preserve `color_mode_data` (the undocumented duotone spec), and write an unchanged document back as mode 8.
- **Multichannel (7):** open when the header channel count is 1 or 3.
  - 1 channel → grayscale→RGB (same as Duotone).
  - 3 channels → treat as subtractive CMY plates → RGB (`r = 255 - c`, etc.; marked approximation, not lcms).
  - Any other channel count → still `Unsupported`.
  - Retain source planes for an unchanged write-back as mode 7; edited/layered documents fall back to working RGB (no invented plate layout).
- Update `psd-color-modes` requirements that currently require modes 7/8 to be rejected.
- Update the `unsupported_depths_and_color_modes_are_rejected` test (modes 7/8 no longer always fail).
- **BREAKING**: callers that relied on `Unsupported` for modes 7/8 will now get a `Document` for the supported shapes above.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `psd-color-modes`: Multichannel/Duotone open by normalizing; Duotone and 1/3-channel Multichannel save back when unchanged; other Multichannel channel counts remain unsupported.

## Impact

- `crates/pictura-codec/src/read.rs`: accept mode codes 7/8; channel-count gate; normalize path for both modes; retain planes.
- `crates/pictura-codec/src/color_mode.rs`: CMY→RGB helper (and wire Multichannel/Duotone into `convert_pixels` / layer conversion).
- `crates/pictura-codec/src/write.rs`: write mode 7/8 when `source_mode` matches and the document is unchanged (same shape as Indexed/Bitmap write-back).
- Tests: open/write-back fixtures (hand-built or psd-tools); update rejection test.
- No new dependency; no app UI beyond the existing conversion notice path.
