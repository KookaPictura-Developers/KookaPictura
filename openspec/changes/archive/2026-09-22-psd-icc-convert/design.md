## Context

`psd-image-resources` already parses resource `1039` and exposes the profile.
The engine works uniformly in sRGB, so a file whose pixels are in another space
must be brought into sRGB on load, or every downstream composite is wrong. The
codec currently normalizes color modes and bit depth on read without an ICC
engine; this change adds the ICC pass.

Constraints:
- The engine is planar 8-bit; `pictura-color::convert` is interleaved.
- `write_psd` re-emits `Document.image_resources` verbatim, so a converted
  document must have the stale profile removed from that blob.
- The project distributes no profile files, so the fixture profile is
  synthesized from `pictura-color::Profile::adobe_rgb()`.

## Goals / Non-Goals

**Goals:**
- Convert a non-sRGB document (composite + layer color channels) to sRGB on read.
- Keep the saved file consistently tagged (no stale profile).
- Keep ordinary sRGB / profile-less files byte-for-byte on the preservation path.

**Non-Goals:**
- Assign vs convert policy UI, black-point compensation, rendering-intent choice.
- True wide-gamut working space (the engine stays sRGB).
- CMYK/Lab profile conversion (those modes are normalized without ICC already).

## Decisions

- **Dependency**: `pictura-codec` depends on `pictura-color`. ICC math is lcms2,
  which only `pictura-color` wraps; duplicating it would be worse. The edge is
  acyclic (`pictura-color` depends only on `pictura-core`).
- **Ordering**: the ICC pass runs after `normalize` (mode/depth), so it sees the
  final planar RGB/Gray buffer. It is independent of mode and depth.
- **Planar → packed**: a helper interleaves the 1 or 3 color planes of a buffer,
  calls `convert(src, srgb, …, channels, 8, RelativeColorimetric, false)`, and
  splits the result back. Layers whose color-channel set is incomplete are left
  unchanged (mirroring `convert_layer_color_channels`).
- **sRGB detection**: `Profile::is_srgb()` matches "srgb" in the ICC
  `Description` tag (lcms2 has no profile compare). A differently-named sRGB
  profile is converted; the deviation is a rounding LSB, documented.
- **Save consistency**: when a document is ICC-normalized, resource `1039` is
  filtered out of `Document.image_resources` at read time, using
  `encode_image_resources` over the `raw` block bytes stored on each
  `ImageResource`. So the writer needs no change and other resources stay
  byte-identical.
- **source_icc**: records the original profile bytes on `Document`, mirroring
  `source_mode`/`source_depth`; not re-emitted.

## Risks / Trade-offs

- [lcms2 per-pixel conversion cost on open] → bounded by document size and run
  once per read; large-document latency is measurable if it matters.
- [The sRGB heuristic misclassifies a differently-named sRGB profile] → converts
  it within a rounding LSB and strips the tag; documented ceiling.
- [Re-serializing resources drops non-`raw` fidelity] → records carry the raw
  block bytes, so re-emission is exact; a fixture test pins it.
- [A layer with partial/mismatched channels] → left unchanged, like the existing
  mode conversion.

## Migration Plan

Behavioral: a previously mis-rendered wide-gamut file now renders correctly.
Rollback is a revert; the parser and raw preservation are unchanged.

## Open Questions

- Whether a future working-space setting should make this an explicit
  Assign/Convert command rather than automatic on load.
