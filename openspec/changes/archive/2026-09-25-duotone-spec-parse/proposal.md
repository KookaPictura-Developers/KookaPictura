# Proposal: duotone-spec-parse

## Why

A Duotone PSD carries its ink definition in the color-mode-data section, which
`read_psd` currently preserves opaquely (the code and psd-tools both call it
"undocumented"). It is in fact documented in the Adobe "Photoshop File Formats"
spec as **Duotone Options**, and both `psdparse` (`duotone.c`) and
`EmilDohne/PhotoshopAPI` parse the identical 524-byte layout. Parsing it is a
prerequisite for rendering duotone colors and for a future File Info surface;
today the bytes are inert.

## What Changes

- Add `pictura_codec::duotone::{parse_duotone, DuotoneSpec, DuotoneInk, InkColor}`
  decoding the 524-byte Duotone Options block: version, plate count (1–4), each
  plate's ink color, Pascal-string name, 13-point transfer curve plus override
  flag, the dot-gain value, and the overprint colors (0/1/4/11 per plate count).
- The parser SHALL return `None` for a block shorter than 524 bytes or a plate
  count outside 1–4, and MUST NOT panic.
- Rendering the duotone into the composite is **out of scope**: Photoshop's
  Duotone edit model is single-channel grayscale, which is what the open path
  already produces, and the multi-ink compositing algorithm is unpublished.

## Capabilities

### Modified Capabilities

- `psd-color-modes`: the Duotone color-mode data is decoded, not just preserved.

## Impact

- New `crates/pictura-codec/src/duotone.rs` and its `lib.rs` re-export.
- A synthetic-buffer unit test pinning the offsets (no real Duotone fixture
  exists and psd-tools cannot author one); the open path, write-back, and all
  oracles are unchanged.
- No new dependency.
