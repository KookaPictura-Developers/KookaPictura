## Context

`read_psd` reads an Indexed file's 768-byte palette (`palette_from`,
`read.rs:237`), expands every color plane through it (`convert_pixels` →
`indexed_to_rgb`, `color_mode.rs:61`; `convert_layer_color_channels`), sets
`source_mode = Some(Indexed)`, and clears `color_mode_data` (`read.rs:279-281`).
`write_psd` has no Indexed branch (`write.rs:969-979`), so the save writes RGB and
the palette is gone. Lab/CMYK already retain their source planes in
`Document.source_planes` / `Layer.source_channels` and re-emit them.

## Goals / Non-Goals

- **Goal:** an unchanged Indexed document round-trips byte-exactly as Indexed.
- **Non-goal:** an edited Indexed document does not re-quantize. There is no
  documented Photoshop RGB→palette mapping, so an edit falls back to the working
  RGB mode instead of inventing one.
- **Non-goal:** Bitmap. Its 1-bit packed writer is separate work.

## Decisions

### Retain the palette in a new `Document.source_palette`, keep `color_mode_data` cleared

The palette is needed at write time, but `color_mode_data` is deliberately
emptied on an Indexed read (the "palette is consumed" contract and its tests).
Add `Document.source_palette: Option<[u8; 768]>` beside the other `source_*`
fields, so the read contract is unchanged and the writer has the palette. The
composite index plane goes in `Document.source_planes`; each layer's single index
channel goes in `Layer.source_channels` (a new `retain_indexed_layer_planes`
mirroring `retain_cmyk_layer_planes`, filtering channel id `0`, recursing into
groups). The index planes are retained at depth 8, so no depth-model change.

### The whole document must be unchanged to write Indexed

The header color mode is document-wide, so Indexed output requires the composite
**and** every pixel layer to be exactly reconstructible from the retained
indices. The writer's predicate compares each retained plane's forward
conversion (`indexed_to_rgb(retained, palette)`) against the current working
RGB: the composite plane against `doc.composite.data`, each pixel layer's retained
index against its RGB channels. Any mismatch — an edit, an added color layer (no
retained channel), a removed color channel — turns the write into the working RGB
mode. When `merged_composite_present` is false the file has no composite plane, so
the composite check is skipped and only the pixel layers decide; the composite
length is read with `get`, not a slice, because `write_psd` is public and a
malformed document must fall back to RGB rather than panic. This is the same
forward-conversion equality the Lab/CMYK helpers use, except Indexed has no
inverse, so it is all-or-nothing rather than per-plane.

### Write layout

When Indexed: header mode 2, depth 8, one output color channel
(`out_color_channels = 1`), the palette re-emitted as `color_mode_data`, the
composite's single index plane from `composite_retained(doc, depth, 0)`, and each
pixel layer's index channel (id `0`) re-emitted from `layer_retained`, other
channels (mask/alpha) kept. Compression follows `doc.composite_compression`, as
for every other mode. Groups are not pixel layers and keep their bytes.

### App notice

`mode_notice` reports `Converted from Indexed; saved as Indexed`. Ceiling: the
notice is computed from the document's static `source_*` fields, so it keeps
claiming Indexed after an edit even though the save then falls back to RGB
(`// ponytail:` note); recomputing the unchanged predicate live would be the
upgrade.

## Risks / Trade-offs

- Adding a `Document` field must be handled by every `Document { .. }` literal;
  `Document::new` defaults it to `None`.
- A no-op engine edit that rewrites a pixel plane byte-identically still compares
  equal and stays Indexed; an edit that changes one pixel flips the whole save to
  RGB, which is the documented contract.
