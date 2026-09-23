## Why

An 8-bit **CMYK** PSD still converts to RGB on save, so the common print mode is
silently lost (roadmap P4, "write-side re-encoding to the source mode"). Lab was
the first mode to save back; CMYK is the valuable one. It is feasible now because
the read-side CMYK→RGB is a plain per-pixel product with an **exact** profile-free
right-inverse, and the retained-source-plane mechanism already added for Lab can
re-emit an unedited CMYK file byte-for-byte.

## What Changes

- `read_psd` retains the pre-normalization CMYK color planes (the composite's four
  planes and every layer's four color channels, recursing into groups) at depth 8,
  in the same store Lab uses.
- `write_psd` writes an 8-bit CMYK document back with header color mode CMYK: the
  composite carries four color planes and each layer four color channels. A plane
  is re-emitted from the retained source bytes when unchanged; an edited plane is
  converted from the working RGB with a profile-free `rgb_to_cmyk` that is an
  exact right-inverse of the read-side `cmyk_to_rgb` (`K = 255`, i.e. no black
  plate), so an edited pixel reads back to exactly the edited RGB. The preserved
  color-mode-data and image-resource sections (including an embedded CMYK profile)
  are re-emitted.
- The CMYK conversion for an edited plane is documented as an approximation (it
  is not Photoshop color management); the unedited path is exact.
- RGB/Grayscale/constructed documents are unchanged; Bitmap and Indexed still save
  the working mode (Indexed's palette was consumed, Bitmap is 1-bit); a 16/32-bit
  CMYK source keeps writing RGB (as today).
- Ceilings (`ponytail:`): an edited CMYK pixel uses a fixed no-black convention,
  not a color-managed transform.

## Capabilities

### Modified Capabilities
- `psd-color-modes`: the "saves in it when the mode maps back" requirement now
  covers 8-bit CMYK as well as Lab; the application notice requirement changes
  accordingly.

## Impact

- `crates/pictura-codec`: `rgb_to_cmyk`, a CMYK retention pass, and the
  four-channel composite/layer write path in `write.rs`. No new dependency.
- Tests/oracles: the CMYK entry in the "saves as RGB" oracle loop is replaced by
  a CMYK round-trip (mode 4, retained planes byte-identical).
