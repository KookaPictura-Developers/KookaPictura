## Context

`read_psd` accepts depth 1 only for Bitmap (`read.rs:61-70`), expands the packed
composite and each layer channel to RGB (`bitmap_rows_to_rgb`), and records
`source_mode = Some(Bitmap)`. Nothing is retained, and `write_psd` has no Bitmap
branch, so a save writes RGB. `depth_of` (`depth.rs:32`) maps every depth except
16/32 to 8, so the depth-1 store is invisible to the writer, and `native_plane`
(`write.rs:792`) requires a `width * height` plane, not a packed one.

## Goals / Non-Goals

- **Goal:** a flat, unchanged 1-bit Bitmap document round-trips byte-exactly.
- **Non-goal:** no RGB→1-bit threshold. An edited document writes RGB.
- **Non-goal (ceiling):** layered or extra-channel Bitmap write-back. A 1-bit
  file's extra alpha/mask planes would each need their own retained packed store
  and a depth-1 layer path; scoped out for now. Such a document writes RGB.

## Decisions

### Retain the packed composite plane; keep `source_depth` unchanged

Add `depth == 1 && mode == Bitmap` to the `retain_planes` gate (`read.rs:78-81`),
so `Document.source_planes` holds the raw packed plane(s) (store depth
`BitDepth::One`), exactly as the 16/32-bit path does before narrowing. Leave
`depth_bits(1)` unmapped so `source_depth` stays `None` for a Bitmap read (no
ripple into the Lab/CMYK/Indexed predicates or the depth notice). Instead teach
`depth_of(Some(BitDepth::One))` to return `1`, so `composite_retained(doc, 1, 0)`
finds the packed plane. `retains_source_depth()` becomes true (store depth is not
Eight), but `output_depth` uses `source_depth` (`None`) and returns 8, so an
edited Bitmap still writes RGB 8-bit; the Bitmap branch forces depth 1 itself.

### Flat, unchanged predicate

A new `write_bitmap.rs` holds `writes_bitmap(doc)`: `source_mode == Some(Bitmap)`,
`composite.channels == 3`, `merged_composite_present`, `layers.is_empty()`,
`channels.is_empty()`, the store depth is `One`, and `bitmap_rows_to_rgb(retained,
w, h)` equals the working composite. Anything else writes the working mode.

### Emit the retained packed plane directly

In `write_container`, when `bitmap_mode`: `depth = 1`, `MODE_BITMAP`,
`out_color_channels = 1`, and the composite plane is the borrowed retained packed
plane (bypassing `native_plane`, whose `width * height` check does not suit a
packed row). Compression follows `doc.composite_compression`; a depth-1 read
rejects ZIP (`read.rs:149`), so the source kind is Raw or RLE. The flat document
writes a zero-length layer/mask section as today.

### App notice

`mode_notice` includes `Bitmap` in `preserves_source` (`source_depth` is `None`
for a Bitmap read), giving `Converted from Bitmap; saved as Bitmap`. Ceiling: the
notice does not consult the edit or the layer/extra fallback (`// ponytail:`
note).

## Risks / Trade-offs

- `depth_of(One) = 1` must not change any existing 8/16/32 path; only a Bitmap
  store carries `One`, so it is inert elsewhere.
- The ceiling is real: a layered Bitmap loses 1-bit form on save. Documented in
  the spec and the roadmap; the fixture is flat.
