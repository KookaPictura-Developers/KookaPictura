# Design

## Context

The engine already embeds (`convert_to_smart_object`), places
(`place_smart_object`), renders (`render_smart_source`), edits/replaces, and
rasterizes smart objects. The layer model (`pictura_core::Layer`) stores the
object's placement only as an axis-aligned `rect` plus an optional raster proxy;
the embedded `SmartObject.payload` holds the source bytes, and the source
document size is recoverable from the payload with `pictura_codec::read_psd`.
There is no transform matrix, no transform history, and no stored "original"
rect.

## Goals / Non-Goals

- **Goal:** add the three remaining `Layer > Smart Objects` actions with one
  undo state each, engine tests, and Qt Test coverage.
- **Non-Goal:** linked smart objects (#113) — a separate change.
- **Non-Goal:** a transform-history system. The native transform is defined as
  the source's own frame, not a remembered matrix.

## Decisions

### D1 — Native transform is the embedded source's frame

Reset Transform decodes the payload for its native size `w × h` and sets the
layer's `rect` to `(0, 0, w, h)` — the source's own frame, which is also how
`place_smart_object` positions a fresh object. The layer's pixel channels are
cleared so the compositor re-renders the embedded source at native scale and
rotation (an implicit re-rasterize); the smart object is **kept**, so the action
is non-destructive and the object stays editable. Rejected: storing an original
rect on the layer, which would be a second source of truth, could go stale after
a save/load, and is not needed to meet the acceptance ("sits at its native
size/orientation"). Rotation/skew live only in the baked proxy and cannot be
represented in `rect`, so reset targets the axis-aligned native frame.

### D2 — Convert to Layers maps the source frame into the object's rect

The operation decodes the source and maps its `(0, 0, w, h)` frame into the
object's `rect` by a per-axis scale plus the object's origin. Each source layer
keeps its kind; a pixel layer's channel planes are resampled to the mapped rect
(a same-size, origin-`(0, 0)` map copies bytes unchanged), groups recurse, and a
source layer's mask rect and data are mapped the same way. A source document
with no layers becomes one raster layer built from its merged composite. The
replacement layers take the object's slot in the stack and the object layer is
removed; the matching linked record is dropped so a re-save authors no orphan.
Rejected: wrapping the layers in a group (the issue asks to splice them into the
parent) and calling `transform_layer` per layer (it refuses groups and
adjustments and would consume nested objects).

### D3 — New Smart Object via Copy drops the copy's link, not its payload

`duplicate_layer` already deep-clones the layer, so the `payload` `Vec` is an
independent copy and an in-memory edit to one object cannot touch the other.
The only sharing is the preserved `SoLd`/`SoLE`/`plLd`/`PlLd` config block and
its document-level `lnk*` record keyed by `uuid`: a verbatim clone would re-emit
the same record and reference the same source on save, and replacing one
object's contents removes that shared record, orphaning the other. The copy
therefore clears its preserved config descriptor, `uuid`, and preserved config
blocks, keeping its deep-copied payload, so the writer re-authors an independent
record from the copy's own bytes. The writer's uuid is a content hash of
filename + payload, so two identical payloads share a record until one is
edited; after an edit their uuids differ and the records diverge, which is the
behavior the acceptance requires. The operation inserts the copy directly above
the original using the shared `paths` container helpers.

### D4 — Identify smart-object rows at the panel, not in the kind string

The layer-kind string stays `pixel` for a smart object (the tooltip and the
row-classification/filter code depend on it), and the panel already exposes
`SmartObjectRole`. `populateRowMenu` gains a `smart` flag; the row mask becomes
`kind | (smart ? SmartObject : 0)`, and the three new rows carry the
`SmartObject` mask. This isolates the change to the row menu instead of
reclassifying smart objects across the panel.

## Risks / Trade-offs

- **Reset moves the object to the origin** rather than preserving its center.
  This is the definition of "native placement" available without stored state
  and matches how a placed object is positioned; called out in the spec.
- **Convert to Layers is bilinear-only** and does not rotate; consistent with the
  existing transform ceiling.
- **Content-hash uuids** mean identical unedited payloads still dedupe to one
  saved record; independence is proven under editing, which is the acceptance.

## Migration Plan

None — new commands; no persisted state changes.

## Open Questions

None.
