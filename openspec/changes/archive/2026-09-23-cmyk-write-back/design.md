## Context

`normalize` converts a CMYK document to RGB and sets `source_mode = Some(Cmyk)`,
retaining nothing today. The read conversion `color_mode.rs::cmyk_to_rgb` is a
per-pixel product: `RGB_c = color_c * K_c / 255` over planar planes, with stored
`0` = full ink and `255` = no ink. That has an **exact** right-inverse
`rgb_to_cmyk(rgb) = [R, G, B, 255]` (planar), because `R * 255 / 255 == R`.
Lab write-back already established the retained-plane pattern and the per-plane
"unchanged ⇒ re-emit, else convert" write.

## Goals / Non-Goals

**Goals:**
- An 8-bit CMYK document writes back as CMYK (mode 4, four color planes), exact
  for unchanged planes, a documented profile-free inverse for edited ones.
- RGB/Grayscale/constructed unchanged; 16/32-bit CMYK keeps writing RGB.

**Non-Goals:**
- Color management: the edited-plane inverse is a fixed no-black convention, not a
  CMYK profile transform; Bitmap/Indexed write-back.
- Retained planes for 16/32-bit CMYK depth (depth stays out of scope).

## Decisions

- **Exact right-inverse, not an approximation, for the channel math.** `K = 255`
  makes `rgb_to_cmyk` a true right-inverse, so an edited pixel round-trips exactly
  through our own read; the retained planes make the unedited case byte-exact.
  This is stronger than the Lab inverse, which quantizes.
- **Retain the pre-normalization CMYK planes.** Add `depth == 8 && mode == Cmyk`
  to the `retain_planes` gate (the composite is captured by the existing
  pre-`split_planes` clone, four color planes plus extras) and add a
  `retain_cmyk_layer_planes` pass mirroring `retain_lab_layer_planes` for channel
  ids `0..4`, before `normalize` converts the layer to RGB.
- **A separate output color count.** The working composite has 3 channels
  (RGB); the CMYK output has 4. `write_container` computes
  `out_color_channels = 4` for CMYK and uses it for the header count, the
  composite plane loop, and the **extras offset** (`4 + i`, not `3 + i` — a wrong
  offset silently reads a color plane as an extra), while keeping the working 3
  for `doc.composite.data` length validation.
- **Layer channels are synthesized.** A normalized CMYK layer has ids `0,1,2,-1`;
  the CMYK writer emits `(0,C),(1,M),(2,Y),(3,K)` from
  `cmyk_layer_color_planes` (retained 4 when `cmyk_to_rgb(retained)` equals the
  working RGB, else `rgb_to_cmyk`), then the `-1`/raw/mask channels unchanged, so
  `write_record`'s channel count is automatic.
- **Keep resource 1039.** For an 8-bit CMYK document the embedded CMYK profile
  now matches the CMYK output; the writer already re-emits image resources
  verbatim, so the previously incoherent "RGB bytes tagged CMYK" save is removed.
  A 16/32-bit CMYK source still writes RGB with the CMYK profile intact (the
  pre-existing mis-tag); fixing that is bundled with the depth decision, not here.
- **Self-test oracle.** `color_mode_oracle.rs`'s loop drops CMYK; a CMYK analogue
  of `lab_document_saves_as_lab` asserts header mode 4, four channels, and the
  retained planes byte-identical via our own `read_psd` (psd-tools reports mode 4
  but returns `255 - raw` channel bytes, so it is used for the mode and the
  edited-path RGB only).

## Risks / Trade-offs

- **Channel-count/layout mistakes** are the main risk (header count, extras
  offset, synthesized per-layer id 3); the oracle's byte-identical retained-plane
  assertion and a layer round-trip catch them.
- **Edited CMYK is a convention**, documented and marked `ponytail:`.
- **psd-tools polarity trap** (`255 - raw`) — the exact assertion uses our own
  re-read, not PIL channel bytes.

## Migration

An RGB, Grayscale, constructed, or 16/32-bit CMYK document is unchanged. An 8-bit
CMYK document now writes header mode 4; the application notice reports CMYK. The
CMYK "saves as RGB" oracle entry is replaced by the round-trip.
