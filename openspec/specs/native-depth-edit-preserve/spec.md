# native-depth-edit-preserve Specification

## Purpose
TBD - created by archiving change native-depth-edit-preserve. Update Purpose after archive.
## Requirements
### Requirement: A moved or rebased layer keeps its native store valid

A destructive op that only changes a layer's position SHALL update every
translated layer's `Layer.source_channels.rect` by the same offset as
`Layer.rect` (and the mask `-2` plane's bounds follow the mask rect), so the
store's `rect` still equals the layer's `rect`; it SHALL NOT clear the store.
This covers the Move tool's translate paths and the layer-offset half of a
canvas resize/crop.

A canvas resize or crop SHALL offset-blit `Document.source_planes.samples` into
the new canvas and set its recorded `width`/`height` to the new dimensions, so a
save still re-emits native composite samples rather than widening.

#### Scenario: A pure move re-anchors the store and re-emits native samples

- **WHEN** a 16/32-bit layer with a retained native store is translated by an
  integer offset and saved
- **THEN** `source_channels.rect == layer.rect` and the written layer channel
  holds the retained native samples, not the 8-bit widening

#### Scenario: A canvas resize rebases the composite store

- **WHEN** a 16/32-bit document is canvas-resized and saved
- **THEN** `Document.source_planes.width`/`height` equal the new document size
  and the written composite color planes are native samples offset into the new
  canvas

### Requirement: The transform family resamples retained native samples

`transform_layer`, `transform_layer_quad`, and `transform_layer_warp` SHALL,
when the target layer retains a native store, resample every stored plane
through the same plane map as the layer's 8-bit channels (the `-2` mask plane
through the mask's map and destination), set the resulting store's `rect` to the
destination rect, and set each corresponding 8-bit channel (and the mask `data`)
to the resampled plane's `narrow_to_u8()`. The `u8` arm of the resampler SHALL
remain byte-identical to the existing 8-bit kernel, and for an 8-bit document
(no store) the op SHALL be byte-identical to today. The op SHALL continue to
drop an unmodeled `raw_channels` stream on the non-translate paths.

#### Scenario: A scaled high-depth layer keeps a resampled native store

- **WHEN** a 16/32-bit RGB layer with a retained native store is scaled
- **THEN** `source_channels` is `Some`, its `rect` equals the new layer rect,
  every stored plane has the destination plane's length, and a save writes the
  source depth with native plane samples, not the 8-bit widening

#### Scenario: A projective or mesh-warped layer keeps a resampled native store

- **WHEN** such a layer is mapped through a projective quad or a custom mesh
  warp
- **THEN** its retained store is resampled to the destination and the saved
  layer plane is native, not the 8-bit widening

#### Scenario: A dropped raw stream does not block the native store

- **WHEN** a high-depth layer carries both an unmodeled `raw_channels` stream and
  a native store and is scaled
- **THEN** `raw_channels` is empty and `source_channels` is a resampled store

### Requirement: Image size and orientation preserve retained native stores

`resize_document` SHALL resample every retained native plane of the layer stores
and of `Document.source_planes` with the requested resample kernel, updating the
store rects and `source_planes` dimensions. `rotate_document` and `flip_document`
SHALL apply the exact orientation index remap to every retained native plane
(the `-2` plane by the mask dimensions) and to `Document.source_planes`, update
the rects and swap the recorded dimensions for a 90°/270° turn, and SHALL NOT
clear the stores.

#### Scenario: Image resize resamples the layer and composite stores

- **WHEN** a 16/32-bit layered document is scaled with `Image > Image Size` and
  saved
- **THEN** the layer store and `source_planes` have the new dimensions and the
  saved document keeps the source depth with native samples

#### Scenario: A 90° rotation is an exact native remap

- **WHEN** a 16/32-bit document is rotated a quarter turn
- **THEN** each retained plane is the exact index remap of its source plane, the
  store rect follows the layer rect, `source_planes` dimensions swap, and the
  saved document keeps the source depth

