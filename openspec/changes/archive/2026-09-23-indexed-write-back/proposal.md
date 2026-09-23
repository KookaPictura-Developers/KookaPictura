## Why

An Indexed PSD is read, expanded through its palette to RGB, and the palette is
dropped, so a save writes the working RGB mode and a palette image is lost even
when nothing was edited. The shipped Lab/CMYK write-back keeps a converted
document in its source mode; Indexed is the remaining 8-bit mode with an exact
representation, so it should round-trip too.

## What Changes

- `read_psd` SHALL retain an Indexed document's 768-byte palette and each index
  plane (composite and every layer, recursing into groups), without changing the
  normalized RGB pixels.
- `write_psd` SHALL save an **unchanged** Indexed document back as header mode
  Indexed (mode 2) with one index channel and the retained palette, re-emitting
  the plans byte-identically.
- A **changed** Indexed document SHALL fall back to writing the working mode
  (RGB). An exact in-mode re-encode needs a quantization Photoshop does not
  document, so none is invented. This applies document-wide: any edited or added
  layer (or edited composite) makes it write RGB.
- The app's mode notice SHALL report an Indexed-preserving save.
- No change to Bitmap; it still saves the working mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `psd-color-modes`: the "records its source mode and saves in it when the mode
  maps back" requirement gains the Indexed write-back and drops Indexed from the
  modes that always save the working mode.

## Impact

- `crates/pictura-core/src/lib.rs` (a `Document.source_palette` store),
  `crates/pictura-codec/src/{color_mode.rs,read.rs,write.rs}`,
  `crates/pictura-app/src/cxxqt_object/impl_core.rs` (notice) and the C++
  self-test.
- No new dependency. `Document.color_mode_data` stays cleared on an Indexed read
  (the palette moves to the new store), so the existing "palette is consumed"
  behavior is unchanged.
