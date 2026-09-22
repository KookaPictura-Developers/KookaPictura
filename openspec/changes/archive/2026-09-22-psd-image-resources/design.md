## Context

`write_psd` re-emits `Document.image_resources` verbatim, so the section already
round-trips (archived `psd-opaque-preservation`). Nothing reads it, though, so
the embedded ICC profile and metadata are inaccessible. This change adds a
tolerant parser and typed records; it deliberately does not change the write
path or any pixels.

## Goals / Non-Goals

**Goals:**
- Parse the section into `ImageResource { id, name, data }` in stored order.
- Expose the well-known ids and the embedded ICC profile.
- Never panic or over-allocate on a malformed section.

**Non-Goals:**
- ICC conversion (deferred: it also needs the resource rewritten on save so a
  normalized document is not re-tagged with the source profile).
- Rendering or app UI; a File Info dialog.

## Decisions

- New `crates/pictura-codec/src/image_resources.rs` using the existing
  bounds-checked `common::Reader` (the same cursor the rest of the codec uses).
- Block framing: 4-byte signature, `u16` id, Pascal name padded to even, `u32`
  data length, data padded to even. Recognized signatures: `8BIM`, `8B64`,
  `MeSa`, `AgHg`, `PHUT`, `DCSR` (psd-tools accepts the same set).
- On an unrecognized signature, a truncation, or a short trailer, stop and
  return the records parsed so far; the function has no error return, mirroring
  `decode_patterns`' skip-and-continue contract.
- Expose `Document`-independent `decode_image_resources(&Document)` plus id
  constants (`ICC_PROFILE = 1039`, `XMP_METADATA = 1060`, `EXIF_DATA_1 = 1058`,
  `EXIF_DATA_3 = 1059`, `IPTC_NAA = 1028`). No `pictura-core` model change.

## Risks / Trade-offs

- [A mis-sized name/pad desyncs the cursor and drops later resources] → every
  read is checked; a bad block ends parsing rather than misreading, and the
  fixture plus oracle pin the happy path.
- [Non-`8BIM` signatures mid-stream] → the accepted set covers the psd-tools
  set; anything else ends parsing (documented ceiling).
- [Large resource data copies] → `data` is a `Vec<u8>` copy of at most the
  section's own size; the raw section is already in memory.

## Migration Plan

None. The parser is additive and read-only; rollback is a revert.

## Open Questions

- The exact ICC-conversion policy (assign vs convert) and whether to replace or
  strip the resource on save — deferred to a follow-up change.
