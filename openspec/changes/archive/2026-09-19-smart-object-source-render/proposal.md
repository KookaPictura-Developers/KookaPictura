## Why

The smart-object round-trip models a `SmartObject` with an embedded `payload`,
but the renderer still draws a layer only from its raster channel proxy. A smart
object the engine authors from a payload and no proxy renders as a black
rectangle: the embedded source is never consulted, so the model cannot be seen
or verified end to end.

## What Changes

- `composite_rgba` renders a visible, non-group, non-adjustment layer that has an
  `Embedded` smart object with a non-empty payload and no color channel (`id` 0)
  by decoding the payload as a document and compositing its pixels.
- The decoded source is scaled into the layer's `rect` with nearest-neighbour
  sampling and composited through the layer's opacity, mask, and blend mode,
  exactly like a raster layer.
- Payload decode uses `pictura_codec::read_psd`; the decoded document's stored
  merged composite is preferred when usable, otherwise its layers are composited.
- A layer with a color channel keeps using the raster proxy; the payload is not
  decoded (no behaviour change or cost for existing files).
- An `External`/`Alias`/`Unresolved` object, an empty payload, or a payload that
  fails to decode is a no-op that leaves the backdrop unchanged, never an error
  or panic.
- The `Trnf`/warp transform and bilinear resampling are out of scope: placement
  is by the layer rect, nearest-neighbour.
- **BREAKING**: none. No model or API change; the render path gains one branch.

## Capabilities

### New Capabilities

- `smart-object-rendering`: render a layer's embedded smart-object source when
  the layer has no raster proxy.

### Modified Capabilities

<!-- None. psd-smart-objects owns resolve/round-trip/author and is unchanged;
     this is a new render-side capability that consumes its typed view. -->

## Impact

- `crates/pictura-render/Cargo.toml`: add a runtime dependency
  `pictura-codec = { path = "../pictura-codec" }` (currently a test-only
  dev-dependency). Justification: the renderer must decode an embedded PSD/PSB
  source, and `pictura-codec` is the only decoder; it depends on `pictura-core`
  only, so there is no cycle.
- `crates/pictura-render/src/composite.rs`: the smart-object branch in
  `composite_pixels` plus its tests.
- No `pictura-core` or codec change; the typed `Layer.smart_object` view already
  exists. `Trnf`/warp and bilinear resampling remain deferred.
