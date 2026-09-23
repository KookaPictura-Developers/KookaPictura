## Why

A depth-1 Bitmap PSD is read (bits expanded to RGB) and always saved back as
RGB, so a 1-bit line-art file is lost even when nothing was edited. Every other
mode with an exact representation — Lab, CMYK, Indexed — now round-trips through
the source store; Bitmap is the last one, and it needs a depth-1 writer path the
codec does not have yet.

## What Changes

- `read_psd` SHALL retain a depth-1 Bitmap document's packed composite plane in
  the source store (a depth-1 read of a flat file).
- `write_psd` SHALL save an **unchanged, flat** Bitmap document back as header
  mode Bitmap (mode 0, depth 1, one 1-bit channel) re-emitting the retained
  packed plane byte-identically, at the source compression (Raw or RLE).
- A Bitmap document that has been **edited**, or that has **layers or extra
  channels**, SHALL fall back to writing the working RGB mode. No RGB→1-bit
  threshold is invented, and layered/extra-channel Bitmap write-back is out of
  scope (documented ceiling).
- `depth_of(BitDepth::One)` SHALL map to sample width 1 so the retained packed
  plane is found; `depth_bits(1)` stays unmapped so `source_depth` is unchanged
  for a Bitmap read.
- The app's mode notice SHALL report a Bitmap-preserving save.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `psd-color-modes`: the "records its source mode and saves in it when the mode
  maps back" requirement replaces the "Bitmap saves as the working mode" clause
  with a flat unchanged Bitmap write-back and an RGB fallback otherwise.

## Impact

- `crates/pictura-codec/src/{depth.rs,read.rs,write.rs}` and a new
  `write_bitmap.rs` (the writer is at the file-size cap),
  `crates/pictura-app/src/cxxqt_object/impl_core.rs` (notice) and the C++
  self-test.
- No new dependency. No fixture change.
