## ADDED Requirements

### Requirement: Rasterize Smart Object eligibility and refusal

The engine operation `pictura-render::rasterize_smart_object(doc, path)` SHALL
resolve `path` and SHALL rasterize only when the target is a single
smart-object layer: the path resolves, the layer is not a group, has no
adjustment data, and `smart_object.is_some()`. On any other target the operation
SHALL return `false` and SHALL NOT mutate the document. When materialization
requires decoding the embedded payload (the layer has no color channel) and the
payload is empty or cannot be decoded, the operation SHALL also return `false`
with no mutation.

#### Scenario: A smart-object layer rasterizes

- **WHEN** `rasterize_smart_object` is called on the path of a non-group, non-adjustment layer carrying a smart object
- **THEN** it returns `true` and the layer no longer carries a smart object

#### Scenario: A group is refused

- **WHEN** the target path resolves to a group layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An adjustment layer is refused

- **WHEN** the target layer has adjustment data
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A non-smart layer is refused

- **WHEN** the target layer has no smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An undecodable payload is refused

- **WHEN** the target is a channel-less smart-object layer whose embedded payload is empty or cannot be decoded
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Rasterizing materializes the object content into layer channels

On success the operation SHALL materialize the object's rendered content into
the layer's pixel channels and SHALL set `layer.smart_object` to `None`. When
the layer already has a color channel (`id == 0`), that existing raster proxy IS
the content, so the operation SHALL leave every pixel channel byte-for-byte
unchanged and SHALL NOT decode the payload. When the layer has no color channel,
the operation SHALL decode the embedded payload through the existing
embedded-source render path, scale the decoded source into the layer's `rect`,
and write channels `0..mode.color_channels()` plus a `-1` alpha channel. The
operation SHALL NOT change the layer's name, `rect`, blend, opacity, fill, mask,
or position.

#### Scenario: A proxy is left unchanged

- **WHEN** a smart-object layer with a color channel is rasterized
- **THEN** the layer's pixel channels are byte-for-byte the same as before

#### Scenario: A source-only layer materializes from the payload

- **WHEN** a smart-object layer with no color channel but a decodable embedded payload is rasterized
- **THEN** the layer's color channels and `-1` alpha hold the decoded source scaled into the layer `rect`

#### Scenario: The smart object is cleared

- **WHEN** rasterizing succeeds
- **THEN** `layer.smart_object` is `None`

#### Scenario: Layout fields are untouched

- **WHEN** a layer is rasterized
- **THEN** its name, `rect`, blend, opacity, fill, mask, and position are unchanged

### Requirement: Rasterize drops the preserved smart-object blocks and linked record

On success the operation SHALL remove the layer's preserved config block
(`SoLd`/`SoLE`/`plLd`/`PlLd`) from `layer.extra_blocks` and SHALL remove the
matching document-level linked-source record from `Document.layer_section_extra`,
so that a subsequent save does not re-emit a smart object and does not carry an
orphan `lnk*` record. A re-save followed by a load SHALL resolve no smart object
on the rasterized layer.

#### Scenario: The config block is gone

- **WHEN** a layer that carried an `SoLd` config block is rasterized
- **THEN** no `SoLd`/`SoLE`/`plLd`/`PlLd` block remains in `layer.extra_blocks`

#### Scenario: The linked record is gone

- **WHEN** a rasterized document is saved
- **THEN** its preserved section carries no linked-source record for the rasterized layer

#### Scenario: Save then load resolves no smart object

- **WHEN** a rasterized document is written and read back
- **THEN** the layer has no smart object and the document carries no orphan `lnk*` record

### Requirement: Codec removes a linked-source record by uuid

The codec function `remove_linked_source(layer_section_extra, uuid) -> Option<Vec<u8>>` SHALL
walk the preserved top-level tagged blocks. For a `lnkD`/`lnk2`/`lnk3`/
`lnkE` block it SHALL parse the `u64`-length-prefixed record list, remove the
record(s) whose Pascal-string uuid equals `uuid`, rebuild that block with the
same key, a recomputed length, and even padding, and copy every other block
byte-for-byte. It SHALL return `Some(new_bytes)` when at least one record was
removed and `None` when nothing matched, leaving the caller's bytes untouched.
It SHALL never panic; a malformed block SHALL be copied verbatim.

#### Scenario: A matching record is removed

- **WHEN** a section holds a `lnk2` with two records and an unrelated block, and one record's uuid is removed
- **THEN** the call returns `Some`, the surviving record and the unrelated block are byte-preserved, and the removed uuid no longer appears

#### Scenario: A non-matching uuid returns none

- **WHEN** no record's uuid equals the requested uuid
- **THEN** the call returns `None`

#### Scenario: A malformed block does not panic

- **WHEN** a tagged block's record list is malformed
- **THEN** the call returns without panicking and leaves that block verbatim

### Requirement: The Rasterize Smart Object command records exactly one undo state

The application SHALL expose the command `Layer > Rasterize > Smart Object` with
the stable id `LayerRasterizeSmartObject`. The command SHALL be enabled only when
the current layer is a rasterizable smart-object layer, SHALL invoke the engine
operation on success, then clear the link sets, recomposite, and record exactly
one history state labelled `"Rasterize Smart Object"`. A refusal SHALL record no
history state.

#### Scenario: Success records one state

- **WHEN** the command runs on a rasterizable smart-object current layer
- **THEN** exactly one history state labelled `"Rasterize Smart Object"` is added and the layer no longer reports a smart object

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a non-smart layer, a group, or an adjustment layer
- **THEN** the history count is unchanged and the document is unchanged

#### Scenario: Enabled only for a rasterizable smart-object layer

- **WHEN** the current layer is not a rasterizable smart-object layer
- **THEN** the command is disabled and its availability predicate mutates nothing
