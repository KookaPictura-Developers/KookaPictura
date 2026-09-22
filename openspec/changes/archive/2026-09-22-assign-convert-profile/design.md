## Context

The engine normalises an embedded profile to sRGB on read: `apply_icc`
(`crates/pictura-codec/src/icc.rs:28`) converts the composite and every layer's
color channels, records the source bytes in `Document.source_icc`, and drops
resource 1039 so the save is not mis-tagged. Reading is therefore the only place
a profile is currently honoured, and there is no per-document working profile.

`pictura-color` already provides `Profile::{srgb,adobe_rgb,pro_photo,from_icc,
to_icc,is_srgb}` and `convert(src, dst, data, w, h, channels, bits, intent, bpc)`
(`crates/pictura-color/src/lib.rs`). `pictura-codec` already has the exact
planar transform (`convert_buffer` `icc.rs:66`, `convert_layer` `icc.rs:88`,
`convert_planes` `icc.rs:108`) and resource framing (`frame_image_resource`,
`image_resources.rs`). `write.rs:611` emits `doc.image_resources` verbatim.

The compositor reads layer channels by id in ~70 places and assumes sRGB math;
`pictura-render` does not depend on `pictura-color`. Converting every layer
before each composite would touch all of those and break the working-space
assumption. The composite's appearance is decided by one final buffer, so the
document→sRGB conversion belongs at that boundary, not per layer.

## Goals / Non-Goals

**Goals:**
- A document carries a working ICC profile; assign and convert are genuinely
  different operations (retag vs transform-and-retag).
- Assign leaves every stored pixel byte unchanged; convert rewrites them and is
  reversible exactly by undo.
- The canvas shows the correct appearance in both cases without per-layer
  conversion or compositor changes.
- A saved document carries resource 1039 matching its working profile, or none
  when the working space is sRGB.

**Non-Goals:**
- Installed-profile discovery, loading `.icc` files from disk, or a `.csf`
  Color Settings file.
- Rendering-intent, black-point-compensation, dither, or "Flatten Image"
  choice; intent is fixed to relative colorimetric, no BPC, no flatten.
- CMYK/Lab/16-bit assign or convert (the writer supports RGB/Gray 8-bit).
- Changing read behaviour: a freshly opened document still normalises to sRGB
  and has no working profile (`psd-icc-conversion` unchanged).

## Decisions

- **Store the profile as ICC bytes on `Document`.** `pictura-core` has zero
  dependencies, so it cannot hold a `pictura_color::Profile`. Add
  `document_icc: Option<Vec<u8>>`, where `None` means the sRGB working space.
  It is deliberately separate from `source_icc` (which records what a read
  normalised away); conflating them would make the "Converted from…" notice
  wrong after an assign.
- **Reuse `image_resources` as the single source of truth for the tag.** Assign
  and convert set `image_resources` to contain (or omit) a framed 1039 block via
  `frame_image_resource`, exactly as the IPTC write-back re-frames 1028.
  `write.rs` already emits the section verbatim, so the save path needs no
  change and an sRGB document stays untagged.
- **Convert reuses `convert_buffer`/`convert_layer`.** Expose the existing
  planar transform publicly as `pictura_codec::convert_document(&mut Document,
  &Profile) -> bool`, converting `doc.composite` and every layer, then retagging.
- **Assign is metadata-only.** `pictura_codec::assign_document_profile(&mut
  Document, &Profile)` rewrites only `image_resources`/`document_icc`.
- **Display conversion at the composite boundary.** A small app-side helper
  converts the buffer from `document_icc` to sRGB immediately before
  `buffer_to_image` for the on-canvas composite (full and region refresh), so
  `doc.composite` and layer channels stay in document space (required for a
  correct save). `pictura-render` stays profile-agnostic.
- **Undo uses the existing whole-document history.** Both commands run inside
  one `record(...)`; convert's pre-conversion pixels come back from the history
  snapshot, which is what `ARCH-007`/`IMG-006` require.

## Risks / Trade-offs

- **Blending happens in document space.** After a convert to Adobe RGB, layer
  compositing and adjustments run on Adobe RGB numbers. This matches Photoshop
  (edits are in the document space) and is acceptable because the final display
  transform restores appearance; the engine no longer "works in sRGB" only when
  the user has assigned a non-sRGB profile.
- **`Profile::is_srgb()` is a description-substring heuristic**
  (`pictura-color/src/lib.rs:136`); an assignment of a profile whose description
  contains "srgb" would be treated as no-op. Documented ceiling.
- **Non-canvas display paths** (small thumbnails/eyedropper) that call
  `buffer_to_image` directly are not converted this cycle. Ceiling; the canvas
  and region paths are.
- **Assigning a wider-gamut profile can clip on save** for 8-bit data only if a
  subsequent edit is made; assign itself is lossless (no pixel writes).
- **Save-composite consistency:** because `doc.composite` stays in document
  space, the merged image data and the tagged 1039 agree.
- **Deliberate read-path widening:** read-normalisation now re-emits an unparsed
  resource tail (unknown signature / truncated block) and `convert_buffer`
  accepts a 4-channel RGBA composite with alpha untouched, because the app
  stores RGB composites as 4 planes. Production read output is unchanged (read
  composites are 3-channel and files normally expose no unparsed tail); both
  behaviours are intentional.

## Migration Plan

Purely additive to the model (`document_icc` defaults to `None`; the three
document constructors set it). Read behaviour is untouched, so existing goldens
and oracles are unaffected. The two menu leaves already exist; they gain ids,
handlers, and an enabled provider. No data migration.

## Open Questions

- None blocking. Whether a later Color Settings policy should make read-time
  normalisation conditional (preserve embedded profiles) is deferred; this
  change is user-initiated only.
