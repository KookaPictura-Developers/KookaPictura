## Context

`normalize` converts a Lab document's color planes to RGB and sets
`doc.source_mode = Some(Lab)` (`read.rs`). An 8-bit read now also retains the
decoded Lab color planes so an unedited save is lossless; the working RGB is the
edited form. `write_container` serializes only `doc.mode` (RGB/Grayscale) and
needs a mode code, a retained-preferring per-plane encode, and the unchanged
3-channel layout. The read inverse is `color_mode.rs::lab_to_rgb` (CIELAB
D50→sRGB D65 via Bradford, matching lcms2 within 1 LSB).

## Goals / Non-Goals

**Goals:**
- A document read from an 8-bit Lab file writes back as Lab.
- An **unedited** Lab document re-emits its retained Lab planes byte-identically,
  so a save does not degrade what the user saw.
- An **edited** plane re-encodes with the profile-free inverse, consistent with
  the read within a small tolerance.
- RGB/Grayscale/constructed documents are byte-unchanged.

**Non-Goals:**
- CMYK, Bitmap, or Indexed write-back.
- Color management or a bundled profile (Lab is device-independent and
  profile-free).

## Decisions

- **A profile-free algebraic inverse.** Add `rgb_to_lab` mirroring `lab_to_rgb`
  (sRGB decode → XYZ D65 → Bradford to D50 → Lab), unit-tested as the inverse
  over the byte grid (the same approximation class as the read side). No new
  dependency; lcms2 stays test-only.
- **Retain the source Lab planes, like the depth path.** An 8-bit Lab read
  captures the decoded Lab color planes into the existing store
  (`Document.source_planes` / `Layer.source_channels`, at `depth = Eight`),
  before `normalize` converts them to the working RGB, recursing into a group's
  descendants. The gate that retained 16/32-bit Grayscale/RGB samples is
  generalized to a `retain_planes` gate that also covers an 8-bit Lab read; the
  depth path's semantics are unchanged.
- **The writer prefers the retained plane, converting only when it changed.**
  `write_container` selects the output mode (Lab when `source_mode == Lab`, the
  source was 8-bit — `source_depth.is_none()` — and the document is 3-channel;
  else the working mode, so a 16/32-bit Lab source keeps writing RGB). For each
  Lab color plane — the composite's and every layer's color channels — it
  re-emits the retained Lab bytes exactly when `lab_to_rgb(retained)` still
  equals the current 8-bit RGB plane, and otherwise emits `rgb_to_lab(current)`.
  The comparison guards against an edit made without clearing the store. The
  channel count stays 3, so the image-data and layer layouts are unchanged. This
  matches the depth change's "retained samples, widened/re-encoded only when
  changed" model.
- **Masks, transparency, extras, and raw channels are untouched** (only color
  planes convert), matching the read side's `convert_layer_color_channels`.
- **The color-mode-data and image-resource sections are re-emitted as today**
  (only Indexed consumed its palette), so a Lab document's sections survive.
- **Bitmap/Indexed/CMYK stay working-mode** (the mode guard still rejects them),
  which the spec states explicitly.

## Risks / Trade-offs

- **Approximation only for edited pixels.** The read is profile-free, so a Lab
  pixel re-encoded from RGB (an edited pixel, whose retained plane no longer
  matches) is not bit-exact to the source; the oracle asserts a tolerance for
  that path and the `ponytail:` ceiling names the limit. An unedited plane is
  retained and re-emitted exactly, so it does not degrade on save.
- **Oracle change.** `color_mode_oracle.rs`'s `normalized_documents_round_trip_as_rgb`
  asserts an RGB Lab output; it must be rewritten to expect Lab for Lab and keep
  RGB for the other converted modes.
- **Channel-count assumptions.** Lab keeps 3 channels, so no layout change; if a
  4-channel mode (CMYK) is added later, that layout must be revisited.

## Migration

An 8-bit RGB, Grayscale, or constructed document is unchanged. A Lab document
now writes mode 9: an unedited document re-emits its retained planes exactly, and
an edited plane is re-encoded with the approximate inverse. A caller that
expected an RGB output must read the header mode. The application's
mode-conversion notice is updated so it no longer implies a save converts Lab to
RGB.
