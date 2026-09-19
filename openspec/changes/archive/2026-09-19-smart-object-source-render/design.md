## Context

The smart-object round-trip work gave `Layer` a typed `SmartObject` with an
embedded `payload`, resolved by the codec on read and authored on write. The
renderer never consumes it: `composite_pixels` draws a layer only from its
channel `0`/`1`/`2` planes and channel `-1` alpha. A smart-object layer created
from a payload has no color channels, so its pixels are read as `None` and the
rect composites as opaque black.

`pictura-codec` already decodes a PSD/PSB into a `Document` (with the stored
merged composite when present) and depends only on `pictura-core`, so the
renderer can decode an embedded source without a dependency cycle. This change
is the render-side consumer the roadmap's render phase calls for; it is
deliberately the smallest one: source pixels placed by the layer rect.

## Goals / Non-Goals

**Goals**

- Render a `Layer` whose `smart_object` is `Embedded` with a non-empty payload
  and no color channel, using the decoded source pixels.
- Reuse the existing raster compositing path so opacity, fill, mask, and blend
  apply identically to a smart-object layer and a raster layer.
- Keep every existing file's rendering byte-identical: a layer with a raster
  proxy still uses the proxy and never decodes its payload.
- Degrade any unusable source to a no-op, never an error or panic.

**Non-Goals**

- The `Trnf`/warp transform and bilinear resampling. Placement is by the layer
  rect with nearest-neighbour scaling.
- Rendering or applying smart filters (`filterFX`), smart-object effects, or a
  `crs:` raw pipeline.
- Decoding an `External`/`Alias` source, or reading a linked path from disk.
- Authoring, resolving, or round-tripping the object (owned by
  `psd-smart-objects`); this change only reads the typed view.
- Caching or pre-rendering decoded sources across frames.

## Decisions

### D1. The raster proxy wins over the embedded source

A layer that has a color channel (`id` 0) renders through the existing channel
path and the payload is never decoded. Only a layer with an `Embedded`
smart-object payload and no color channel takes the new branch. This preserves
existing rendering exactly and avoids a decode cost for files that already carry
a proxy, which is every Photoshop-written smart-object layer.

### D2. Decode through the codec, prefer the stored merged composite

The embedded payload is decoded with `pictura_codec::read_psd`. When the decoded
document's stored merged composite is usable (RGB or grayscale, non-zero size),
those pixels are the source; otherwise the decoded document's layers are
composited to produce the source. The stored composite is what Photoshop shows
for the embedded file, so preferring it matches the expected appearance without
re-implementing the embedded document's own blend semantics.

### D3. Place by the layer rect, nearest-neighbour

The decoded source is sampled nearest-neighbour into the layer's `rect`,
clamped to the canvas, then fed through `blend_into` per pixel. `Trnf`/warp and
bilinear resampling are a deliberate ceiling: the layer rect is the placement
Photoshop records for an untransformed object, and nearest-neighbour keeps the
scaling loop trivial until a transform baseline exists.

### D4. An unusable source is a no-op

`External`, `Alias`, and `Unresolved` kinds, an empty payload, or a decode
failure leave the backdrop unchanged and do not error. This mirrors the
adjustment path, where an undecodable block is preserved on save but not
applied, and keeps a malformed embedded file from breaking the whole composite.

### D5. Reuse the raster compositing loop

The branch produces per-pixel `[r,g,b]` and alpha exactly as `composite_pixels`
does and calls the same `blend_into`, so the layer's opacity, fill, mask, blend
mode, and dissolve behaviour need no second implementation. The only new code is
obtaining the source sample.

## Risks / Trade-offs

- **Embedded PSB decode cost.** A payload may be a large PSB; decoding on every
  composite is expensive. The proxy-wins rule keeps this off the common path,
  and caching is deferred until a profile shows it matters.
- **Source-vs-proxy mismatch.** A file with both a proxy and an embedded source
  renders from the proxy, so a source edited without a proxy refresh will not
  show. This is intentional: the proxy is Photoshop's own rendered result and is
  the higher-fidelity representation.
- **Transform deferred.** A warped or transformed object renders axis-aligned in
  its rect, not warped. Marked as a deliberate ceiling; `Trnf` parsing and
  bilinear sampling are a later change.
- **Nested smart objects.** A decoded source that itself contains a
  smart-object layer recurses through the same path. No depth guard is added;
  an embedded PSD is a finite tree, and a cyclic payload can only be crafted,
  not produced by Photoshop.
