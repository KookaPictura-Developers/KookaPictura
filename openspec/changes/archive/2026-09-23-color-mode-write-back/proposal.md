## Why

`read_psd` normalizes a Lab document to RGB and records `Document.source_mode`,
but `write_psd` only writes the working mode, so an open→save silently converts
Lab to RGB (roadmap P4, the remaining "write-side re-encoding to the source
mode"). Lab is the one converted mode with a profile-free inverse and an
independent oracle (lcms2), so it is the honest first mode to save back.

## What Changes

- `write_psd` writes a document read from an 8-bit **Lab** file back as Lab: the
  output header's color mode is Lab, the color-mode-data and image-resource
  sections are preserved as today, and each color plane (the composite and every
  layer color channel) is re-emitted from the retained Lab plane when unchanged,
  else converted from the working RGB back to Lab before it is encoded.
- The read retains the 8-bit Lab color planes (mirroring the depth change's
  retained samples), so an unedited document saves byte-identically; only an
  edited plane is re-encoded.
- The conversion SHALL be the profile-free algebraic inverse of the read-side
  `lab_to_rgb`, so the round trip is consistent with the read approximation (a
  documented approximate transform, like the read side); it is not Photoshop
  color management.
- A document with no recorded source mode, an 8-bit RGB or Grayscale document,
  and a constructed document are unchanged (RGB writes RGB, Grayscale writes
  Grayscale). Bitmap, Indexed, and CMYK continue to save as the working mode —
  Bitmap/Indexed have no natural re-encode and CMYK's inverse is a
  color-management choice that needs its own decision.
- Ceilings (`ponytail:`): only an **edited** Lab pixel is approximate (the read
  is profile-free), so an edit can shift a saturated color by a few LSB; an
  unedited plane is retained and re-emitted exactly.

## Capabilities

### Modified Capabilities
- `psd-color-modes`: the "a normalized document saves as the working mode"
  requirement changes — a Lab document now saves back as Lab.

## Impact

- `crates/pictura-codec`: an `rgb_to_lab` inverse in `color_mode.rs`, the Lab
  mode code and per-plane conversion in `write.rs`, and the header mode/channel
  handling. No new dependency.
- Tests/oracles: `color_mode_oracle.rs` / `color_modes.rs` — a Lab write oracle
  (psd-tools/lcms2). The existing "saves as RGB" Lab expectation changes.
