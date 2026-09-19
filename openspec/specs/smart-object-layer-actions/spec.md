# smart-object-layer-actions Specification

## Purpose
TBD - created by archiving change convert-to-smart-object. Update Purpose after archive.
## Requirements
### Requirement: Convert to Smart Object eligibility and refusal

The engine operation `pictura-render::convert_to_smart_object(doc, path)` SHALL
convert the layer at `path` only when it is a single raster pixel layer: the path
resolves, the layer is not a group, has no adjustment data, is not the Background
layer, has no existing smart object, and has a `rect` with positive width and
height. On any other target the operation SHALL return `false` and SHALL NOT
mutate the document.

#### Scenario: A raster pixel layer converts

- **WHEN** `convert_to_smart_object` is called on the path of a raster pixel layer that is not the Background and has no smart object
- **THEN** it returns `true` and the layer carries an embedded smart object

#### Scenario: A group is refused

- **WHEN** the target path resolves to a group layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An adjustment layer is refused

- **WHEN** the target layer has adjustment data
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: The Background is refused

- **WHEN** the target layer is the Background layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An existing smart object is refused

- **WHEN** the target layer already has a smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A zero-size layer is refused

- **WHEN** the target layer's `rect` has zero width or zero height
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Embedded smart-object source authoring

On success the operation SHALL author an embedded source document and attach it
to the layer as `SmartObject { kind: Embedded, payload, filename, filetype,
creator }`. The source document SHALL use the document's color mode and bit
depth and the layer's `rect` as its size, SHALL contain a copy of the layer
translated so its top-left is `(0, 0)`, and SHALL carry a merged composite equal
to the layer's raster over that rect. The operation SHALL serialize the source
with `write_psd` and store the bytes as the payload, with `filename` equal to
`"<layer name>.psd"`, `filetype` `8BPB`, and `creator` `8BIM`. The payload SHALL
be non-empty and SHALL be a valid PSD that the codec resolves as an embedded
smart object.

#### Scenario: The payload is a valid embedded source

- **WHEN** a raster layer is converted and the stored payload is read with `read_psd`
- **THEN** the payload parses, its document size equals the layer's `rect`, and its merged composite equals the layer's raster

#### Scenario: SmartObject metadata is stamped

- **WHEN** the conversion succeeds
- **THEN** the layer's smart object reports kind `Embedded`, a non-empty payload, filename `"<layer name>.psd"`, `filetype` `8BPB`, and `creator` `8BIM`

#### Scenario: Same mode and depth

- **WHEN** a Grayscale 8-bit document's layer is converted
- **THEN** the embedded source document is Grayscale 8-bit and `write_psd` accepts it

### Requirement: The raster proxy and rendering are unchanged

The operation SHALL keep the layer's existing pixel channels as a raster proxy
and SHALL NOT change the layer's name, rect, blend, opacity, fill, mask, or
position. Because a layer with a proxy renders from the proxy, the document
composite after conversion SHALL equal the composite before conversion.

#### Scenario: Pixel channels survive

- **WHEN** a raster layer with color channels is converted
- **THEN** the layer's channels are byte-for-byte the same as before

#### Scenario: The composite is unchanged

- **WHEN** the service composite is compared before and after a successful conversion
- **THEN** the two composites are equal

#### Scenario: Rejected destinations are untouched beyond the refusal

- **WHEN** conversion is refused
- **THEN** the layer's channels, rect, name, and smart-object state are unchanged

### Requirement: Save and load preserve the converted object

A document containing a converted layer SHALL round-trip through `write_psd`
and `read_psd` with the smart object preserved as embedded and its payload
unchanged.

#### Scenario: Round-trip preserves the object

- **WHEN** a document with a converted layer is written and read back
- **THEN** the layer resolves as an embedded smart object with a non-empty payload whose bytes equal the written payload

#### Scenario: The proxy still round-trips

- **WHEN** a converted document is written and read back
- **THEN** the layer's pixel channels still resolve and the composite is unchanged

### Requirement: The app command records exactly one undo state

The application SHALL expose the command `Layer > Smart Objects > Convert to
Smart Object` with the stable id `LayerSmartObjectConvertTo`. The command SHALL
be enabled only when the current layer is convertible, SHALL invoke the engine
operation on success, clear the link sets, recomposite, and record exactly one
history state labelled `"Convert to Smart Object"`. A refusal SHALL record no
history state.

#### Scenario: Success records one state

- **WHEN** the command runs on a convertible current layer
- **THEN** exactly one history state labelled `"Convert to Smart Object"` is added and the layer reports a smart object

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a group, an adjustment layer, the Background, or an existing smart object
- **THEN** the history count is unchanged and the document is unchanged

#### Scenario: Enabled only when convertible

- **WHEN** the current layer is not convertible
- **THEN** the command is disabled and its availability predicate mutates nothing

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

### Requirement: Place Smart Object from a file

The engine operation `pictura-render::place_smart_object(doc, filename, bytes)` SHALL decode `bytes`
with `pictura_codec::read_psd` and SHALL return `None`
without mutating the document when `bytes` do not parse as a PSD/PSB document.
Its signature is `place_smart_object(doc: &mut Document, filename: &str, bytes:
&[u8]) -> Option<String>`. On success it SHALL append a new layer at the top of
the document, name it `filename`, and return the new layer's path using the same
path string convention as the other `layer_ops` (`"0"`, `"2/1"`, …). The
operation SHALL NOT change any existing layer or the document size.

#### Scenario: A PSD source is placed

- **WHEN** `place_smart_object` is called with the bytes of a valid PSD and a display name
- **THEN** it returns `Some(path)` for a new topmost layer and mutates no existing layer

#### Scenario: Malformed bytes are refused

- **WHEN** `place_smart_object` is called with bytes that are not a parseable PSD/PSB document
- **THEN** it returns `None` and the document is unchanged

### Requirement: A placed layer is channel-less, native-size, and renders from its embedded source

For a decoded source of size `w × h`, the appended layer MUST have `rect =
(0, 0, w, h)`, `visible` true, `BlendMode::Normal`, `opacity` 255, no pixel
channels, and `smart_object = Some(SmartObject { kind: Embedded, payload:
Some(bytes), filename, filetype: *b"8BPB", creator: *b"8BIM", .. })`. Because the
layer has no color channel, the compositor SHALL render it from its embedded
payload through the existing embedded-source path, and the composite SHALL show
the source content within the layer rect. A `write_psd` followed by `read_psd`
SHALL re-resolve the placed layer as an embedded smart object with an unchanged
payload.

#### Scenario: The layer geometry and metadata are stamped

- **WHEN** a source of size `w × h` is placed as `filename`
- **THEN** the new topmost layer is named `filename`, has `rect = (0, 0, w, h)`, is visible, Normal, opacity 255, has no pixel channels, and carries an `Embedded` smart object with `payload` equal to the source bytes, `filename`, `filetype` `8BPB`, and `creator` `8BIM`

#### Scenario: The composite shows the placed source

- **WHEN** a solid-colour PSD is placed into a document
- **THEN** `composite_rgba` shows the source colour within the placed layer's rect

#### Scenario: Save then load preserves the placed object

- **WHEN** a document with a placed layer is written with `write_psd` and read back
- **THEN** the layer resolves as an embedded smart object whose payload equals the written bytes

#### Scenario: Content beyond the canvas is clipped

- **WHEN** the placed source is larger than the document
- **THEN** the composite is bounded by the document and the overflowing content is not drawn

### Requirement: The Place Smart Object command records exactly one undo state

The application SHALL expose the command `File > Place…` with the stable id
`file.place`. The command SHALL open a file dialog filtered to Photoshop files
(`*.psd *.psb`) and SHALL be enabled only when a document is open. On a chosen
path the bridge `place_smart_object(path) -> QString` SHALL read the file, derive
the display name from its base name, and invoke the engine operation on the
current document. On success only, the bridge SHALL clear the link sets,
recomposite, record exactly one history state labelled `"Place"`, and return the
new layer path. On refusal the bridge SHALL return an empty string, record no
history state, and leave the document unchanged.

#### Scenario: Success records one state and returns the layer path

- **WHEN** the command runs with a valid PSD file and an open document
- **THEN** exactly one history state labelled `"Place"` is added, the layer count increases by one, and the bridge returns the new layer's path

#### Scenario: Refusal records nothing

- **WHEN** the command runs with a malformed source file
- **THEN** the history count is unchanged, the document is unchanged, and the bridge returns an empty string

#### Scenario: Enabled only when a document is open

- **WHEN** no document is open
- **THEN** the command is disabled

### Requirement: Replace Smart Object Contents eligibility and refusal

The engine operation SHALL resolve `path` and SHALL replace the source only
when the target is a single replaceable smart-object layer: the path resolves,
the layer is not a group, has no adjustment data, and its `smart_object` is
`Embedded` with a payload. Its signature is
`pictura-render::replace_smart_object_contents(doc, path, filename, bytes) ->
bool`. The operation SHALL decode `bytes` with `pictura_codec::read_psd` and
SHALL refuse when they do not parse as a PSD/PSB document. On any refusal it
SHALL return `false` and SHALL NOT mutate the document.

#### Scenario: A replaceable smart-object layer replaces its contents

- **WHEN** `replace_smart_object_contents` is called on a non-group, non-adjustment layer whose smart object is embedded with a payload and `bytes` are a valid PSD/PSB document
- **THEN** it returns `true`

#### Scenario: A non-smart layer is refused

- **WHEN** the target layer has no smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A group is refused

- **WHEN** the target path resolves to a group layer
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An adjustment layer is refused

- **WHEN** the target layer has adjustment data
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An embedded object without a payload is refused

- **WHEN** the target layer's smart object is `Embedded` but has no payload
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: Malformed bytes are refused

- **WHEN** the target is a replaceable smart object and `bytes` do not parse as a PSD/PSB document
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Replacing contents swaps only the embedded source and clears the proxy

On success the operation SHALL set the layer's `smart_object.payload` to the new
source `bytes`, set `filename` to the supplied name, set `filetype` to `8BPB`
and `creator` to `8BIM`, and clear the stored `uuid` to empty. It SHALL clear
the layer's pixel channels so the compositor renders the new source through the
existing embedded-source path scaled into the layer's existing `rect`. It SHALL
preserve the layer's transform/geometry and every other property: `rect`, name,
mask, blend, opacity, fill, color, and lock flags SHALL be unchanged. Only the
smart-object source and the pixel channels SHALL change.

#### Scenario: The source metadata is replaced

- **WHEN** a replaceable smart object is replaced with `bytes` and `filename`
- **THEN** its payload equals `bytes`, its filename equals `filename`, and its filetype/creator are `8BPB`/`8BIM`

#### Scenario: The stored uuid is cleared

- **WHEN** the replacement succeeds
- **THEN** the layer's smart object `uuid` is empty

#### Scenario: The proxy is cleared so the new source renders

- **WHEN** a proxy-backed smart object is replaced and the document is composited
- **THEN** the layer has no pixel channels and the compositor draws the new source scaled into the layer's rect

#### Scenario: Geometry and layout are untouched

- **WHEN** a smart object with a mask, a non-default blend, opacity, fill, color, and lock flags is replaced
- **THEN** its `rect`, name, mask, blend, opacity, fill, color, and lock flags are unchanged

### Requirement: Replace drops and re-authors the preserved blocks and linked record

On success the operation SHALL remove the layer's preserved config block
(`SoLd`/`SoLE`/`plLd`/`PlLd`) from `layer.extra_blocks` and, when the previous
`uuid` was non-empty, SHALL remove the matching document-level linked-source
record from `Document.layer_section_extra` via
`pictura_codec::remove_linked_source`. A subsequent `write_psd` followed by
`read_psd` SHALL resolve the layer as an embedded smart object whose payload is
the NEW source bytes, and the old uuid and old payload SHALL be absent from the
saved document.

#### Scenario: The config block is gone

- **WHEN** a layer that carried an `SoLd` config block is replaced
- **THEN** no `SoLd`/`SoLE`/`plLd`/`PlLd` block remains in `layer.extra_blocks`

#### Scenario: The old linked record is gone

- **WHEN** a replaced document is inspected
- **THEN** the old uuid no longer appears in `layer_section_extra`

#### Scenario: Save then load resolves the new source

- **WHEN** a replaced document is written and read back
- **THEN** the layer resolves as an embedded smart object whose payload equals the new source, not the old payload or uuid

### Requirement: The Replace Contents command records exactly one undo state

The application SHALL expose the command `Layer > Smart Objects > Replace
Contents…` with the stable id `LayerSmartObjectReplaceContents` and SHALL show a
file dialog filtered to Photoshop files (`*.psd *.psb`). The command SHALL be
enabled only when the current layer is a replaceable smart object. On a chosen
path the bridge `replace_smart_object_contents(path, file_path) -> bool` SHALL
read the file, derive the display name from its base name, and invoke the engine
operation on the current document. On success only, the bridge SHALL clear the
link sets, recomposite, and record exactly one history state labelled
`"Replace Contents"`. On refusal the bridge SHALL return `false`, record no
history state, and leave the document unchanged.

#### Scenario: Success records one state

- **WHEN** the command runs on a replaceable smart-object current layer with a valid PSD/PSB file
- **THEN** exactly one history state labelled `"Replace Contents"` is added and the layer's payload is the new source

#### Scenario: Refusal records nothing

- **WHEN** the command runs with a missing, unreadable, or malformed file, or on a non-replaceable layer
- **THEN** the history count is unchanged, the document is unchanged, and the bridge returns `false`

#### Scenario: Enabled only for a replaceable smart object

- **WHEN** the current layer is not a replaceable smart object
- **THEN** the command is disabled and its availability predicate mutates nothing

