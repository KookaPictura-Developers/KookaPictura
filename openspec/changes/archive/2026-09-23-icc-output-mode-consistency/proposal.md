## Why

`write_psd` re-emits the preserved image-resource section verbatim, so a file
read as CMYK or Lab and normalized to RGB can be saved as RGB while still
carrying its CMYK or Lab ICC profile (resource `1039`). The saved file then lies
about its own color space, which corrupts any downstream color management. The
codec already drops `1039` for the profile-normalized path; the gap is the
converted-without-profile-transform path (a 16/32-bit CMYK or Lab source, whose
CMYK/Lab profile cannot drive an RGB transform).

## What Changes

- `write_psd` SHALL emit resource `1039` only when its ICC data-space signature
  matches the output header color mode (`RGB `→RGB, `GRAY`→Grayscale,
  `CMYK`→CMYK, `Lab `→Lab); a preserved profile whose signature differs SHALL be
  dropped.
- A matching profile, and a profile too short to carry a data-space signature,
  SHALL be re-emitted byte-for-byte, as SHALL every other resource.
- No model, read-path, or fixture change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `psd-icc-conversion`: add a requirement that a saved ICC profile's data space
  matches the output header color mode, so a converted document is not saved
  mis-tagged.
- `psd-image-resources`: qualify the byte-preservation guarantee so a framable
  resource `1039` whose data space does not match the output header mode is
  dropped instead of re-emitted verbatim.

## Impact

- `crates/pictura-codec/src/icc.rs` (new output filter helper),
  `crates/pictura-codec/src/write.rs` (one call at the resource-emit choke
  point).
- No new dependency. No file-format change for a document whose profile already
  matches its mode (byte-identical output).
