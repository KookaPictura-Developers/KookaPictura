## Why

A 16/32-bit CMYK or Lab PSD loses twice on an open→save: the read normalizes it
to the working RGB and retains no samples, so the writer emits **8-bit RGB** —
dropping both the source color mode and the source depth. Every other converted
mode with an exact representation (8-bit Lab/CMYK/Indexed, flat Bitmap) and every
16/32-bit Grayscale/RGB file already round-trips; this is the remaining hole, and
the native layer store needed for it already exists.

## What Changes

- `read_psd` SHALL retain the native-depth source color planes of a 16/32-bit
  Lab or CMYK document (composite and every layer channel), alongside the
  existing Grayscale/RGB retention.
- `write_psd` SHALL write such a document back at its source depth **and** header
  color mode (Lab/CMYK): unchanged planes re-emit the retained native samples
  byte-identically; an edited plane is re-encoded from the working RGB with the
  profile-free 8-bit inverse and widened to the source depth (Lab approximate,
  CMYK exact), exactly the rule the depth-preserve and 8-bit write-back paths
  already use.
- The app save-notice SHALL not claim an RGB save for a converted document whose
  depth is retained.
- No new dependency, no fixture required (tests construct the documents in
  Rust).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `psd-bit-depth`: the sample-retention and depth-preservation requirement
  extends from Grayscale/RGB to Lab/CMYK, and the "a normalized color mode still
  saves 8-bit" clause/scenario is replaced by depth preservation.
- `psd-color-modes`: the "records its source mode and saves in it" requirement
  drops its 8-bit-only scoping and its "a 16-bit Lab or CMYK source keeps writing
  the working mode" clause.

## Impact

- `crates/pictura-codec/src/{read.rs,color_mode.rs,write.rs}`,
  `crates/pictura-app/src/cxxqt_object/impl_core.rs` (notice).
- `write.rs` is at 1186/1200: the new narrowing helpers live in `color_mode.rs`
  so the writer only swaps call sites.
