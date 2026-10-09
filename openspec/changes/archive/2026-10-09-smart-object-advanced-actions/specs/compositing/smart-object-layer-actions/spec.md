# smart-object-layer-actions Specification (delta)

## ADDED Requirements

### Requirement: Reset Smart Object Transform eligibility and refusal

The engine operation `pictura-render::reset_smart_object_transform(doc, path)`
SHALL resolve `path` and SHALL act only on a single smart-object layer: the path
resolves, the layer is not a group, has no adjustment data, and its
`smart_object` is `Embedded` with a non-empty payload that parses as a PSD/PSB
document (so its native size is known). On any other target, or a payload that
does not parse, the operation SHALL return `false` and SHALL NOT mutate the
document.

#### Scenario: A transformed embedded object can be reset

- **WHEN** `can_reset_smart_object_transform` is called on the path of a non-group, non-adjustment layer whose `Embedded` smart object has a payload that parses as PSD/PSB
- **THEN** it returns `true`

#### Scenario: A group, an adjustment, or a non-smart layer is refused

- **WHEN** the target path resolves to a group, a layer with adjustment data, or a layer with no smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A non-embedded or unparseable source is refused

- **WHEN** the target layer's smart object is not `Embedded`, or its non-empty payload does not parse as a PSD/PSB document
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Resetting restores the source's native transform

On success the operation SHALL read the embedded source document's native size
`w × h` and SHALL set the layer's `rect` to `(0, 0, w, h)`, i.e. the source's
native transform at the origin. It SHALL clear the layer's pixel channels so the
compositor renders the embedded source again at native scale and rotation, and
SHALL keep `layer.smart_object` so the object stays editable. It SHALL NOT
change the layer's name, blend, opacity, fill, mask, or lock flags. The
operation SHALL be non-destructive: no transform history is stored and the
source payload is unchanged.

#### Scenario: The rect returns to the native size at the origin

- **WHEN** a smart object whose `rect` was scaled or moved is reset
- **THEN** its `rect` is `(0, 0, source_width, source_height)`

#### Scenario: The composite shows the unrotated native source

- **WHEN** a reset object is composited
- **THEN** the layer paints the embedded source with no scale or rotation, clipped to the document

#### Scenario: The object stays a smart object

- **WHEN** resetting succeeds
- **THEN** `layer.smart_object` is still `Some` and its payload is byte-for-byte unchanged

#### Scenario: Layout fields are untouched

- **WHEN** a smart object with a mask, non-default blend, opacity, fill, and lock flags is reset
- **THEN** its name, blend, opacity, fill, mask, and lock flags are unchanged

### Requirement: The Reset Transform command records exactly one undo state

The application SHALL expose `Layer > Smart Objects > Reset Transform` with the
stable id `LayerSmartObjectResetTransform` (`"layer.smartObject.resetTransform"`).
The command SHALL be enabled only when the current layer can be reset, SHALL
invoke the engine operation on success, then clear the link sets, recomposite,
and record exactly one history state labelled `"Reset Transform"`. A refusal
SHALL record no history state.

#### Scenario: Success records one state

- **WHEN** the command runs on a resettable smart-object current layer
- **THEN** exactly one history state labelled `"Reset Transform"` is added and the layer's `rect` is its native size at the origin

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a non-smart layer, a group, an adjustment layer, or an object whose payload does not parse
- **THEN** the history count is unchanged and the document is unchanged

#### Scenario: Enabled only for a resettable object

- **WHEN** the current layer is not a resettable smart object
- **THEN** the command is disabled and its availability predicate mutates nothing

### Requirement: Convert Smart Object to Layers eligibility and refusal

The engine operation `pictura-render::convert_smart_object_to_layers(doc, path)`
SHALL resolve `path` and SHALL act only on a single smart-object layer: the path
resolves, the layer is not a group, has no adjustment data, and its
`smart_object` is `Embedded` with a non-empty payload that parses as a PSD/PSB
document. On any other target, or a payload that does not parse, or a source
document with a zero dimension, the operation SHALL return `false` and SHALL NOT
mutate the document.

#### Scenario: An embedded object converts

- **WHEN** `convert_smart_object_to_layers` is called on a non-group, non-adjustment layer whose `Embedded` smart object has a payload that parses as a PSD/PSB document
- **THEN** it returns `true`

#### Scenario: A group, an adjustment, or a non-smart layer is refused

- **WHEN** the target path resolves to a group, a layer with adjustment data, or a layer with no smart object
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: A non-embedded or unparseable source is refused

- **WHEN** the target layer's smart object is not `Embedded`, or its non-empty payload does not parse as a PSD/PSB document
- **THEN** the operation returns `false` and the document is unchanged

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the operation returns `false` and the document is unchanged

### Requirement: Converting to layers splices the source stack into place

On success the operation SHALL decode the embedded source and SHALL replace the
smart-object layer with the source document's layers. Each source layer SHALL be
mapped from the source's `(0, 0, w, h)` frame into the object's `rect`: a source
`rect` `r` becomes the document rect `obj.left + r.left * (obj.width / w)`,
`obj.top + r.top * (obj.height / h)`, and so on, and a pixel layer's channel
planes SHALL be resampled to the mapped rect (a same-size, origin-`(0, 0)` map
SHALL copy bytes unchanged). A source document with no layers SHALL become a
single raster layer covering the object's `rect`, built from the source's merged
composite. The replacement layers SHALL occupy the object's slot in the stack;
the object layer SHALL be removed. The matching document-level linked-source
record SHALL be removed so a re-save does not carry an orphan record.

#### Scenario: An untransformed object yields its source stack at the object position

- **WHEN** a smart object whose `rect` is `(left, top, left + w, top + h)` for a source of size `w × h` is converted
- **THEN** the object layer is replaced by the source's layers, each at its source `rect` translated by `(left, top)`

#### Scenario: A scaled object scales its source stack

- **WHEN** a smart object whose `rect` is half or double the source size is converted
- **THEN** the replacement layers' rects are scaled by the same factor and their channel planes are resampled into the mapped rects

#### Scenario: A payload with no layers becomes one raster layer

- **WHEN** the embedded source document carries no layers but has a merged composite
- **THEN** the object layer is replaced by one raster layer covering the object's `rect` whose channels hold the source composite

#### Scenario: The linked record is dropped

- **WHEN** a converted-from object that carried a linked record is written
- **THEN** the saved document carries no linked-source record for the consumed object

### Requirement: The Convert to Layers command records exactly one undo state

The application SHALL expose `Layer > Smart Objects > Convert to Layers` with the
stable id `LayerSmartObjectConvertToLayers`
(`"layer.smartObject.convertToLayers"`). The command SHALL be enabled only when
the current layer can be converted to layers, SHALL invoke the engine operation
on success, then clear the link sets, recomposite, and record exactly one history
state labelled `"Convert to Layers"`. A refusal SHALL record no history state.

#### Scenario: Success records one state and replaces the object

- **WHEN** the command runs on a convertible smart-object current layer
- **THEN** exactly one history state labelled `"Convert to Layers"` is added and the layer count changes by the source layer count minus one

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a non-smart layer, a group, an adjustment layer, or an object whose payload does not parse
- **THEN** the history count is unchanged and the document is unchanged

#### Scenario: Enabled only for a convertible object

- **WHEN** the current layer is not a convertible smart object
- **THEN** the command is disabled and its availability predicate mutates nothing

### Requirement: New Smart Object via Copy gives an independent embedded source

The engine operation `pictura-render::new_smart_object_via_copy(doc, path)` SHALL
resolve `path` and SHALL act only on a single non-group, non-adjustment layer
whose `smart_object` is `Embedded` with a non-empty payload. On any other target
it SHALL return `None` and SHALL NOT mutate the document. On success it SHALL
deep-clone the layer, name the copy `"<name> copy"`, clear the copy's preserved
config descriptor and `uuid` so the copy does not share a preserved `SoLd`/`SoLE`
link record with the original, keep the (deep-copied) payload, insert the copy
directly above the original in its container, and return the copy's path using
the same path-string convention as the other `layer_ops`. The original SHALL be
unchanged, and editing either object's source SHALL NOT change the other.

#### Scenario: The copy is inserted above and owns its payload

- **WHEN** `new_smart_object_via_copy` roots at a copyable smart object
- **THEN** a new layer directly above it carries an `Embedded` smart object whose payload equals the original's, whose `uuid` is empty, and whose name is `"<name> copy"`

#### Scenario: The original is unchanged

- **WHEN** the copy is created
- **THEN** the original layer's payload, smart object, and preserved blocks are byte-for-byte unchanged

#### Scenario: Editing one source leaves the other alone

- **WHEN** the copy's embedded source is replaced and both layers' smart objects are compared
- **THEN** the original's payload is unchanged and the two payloads differ

#### Scenario: A non-smart or non-embedded target is refused

- **WHEN** the target has no smart object, is a group, has adjustment data, or is not an `Embedded` object with a payload
- **THEN** the operation returns `None` and the document is unchanged

### Requirement: The New Smart Object via Copy command records exactly one undo state

The application SHALL expose `Layer > Smart Objects > New Smart Object via Copy`
with the stable id `LayerSmartObjectNewViaCopy`
(`"layer.smartObject.newViaCopy"`). The command SHALL be enabled only when the
current layer is a copyable smart object, SHALL invoke the engine operation on
success, then clear the link sets, recomposite, and record exactly one history
state labelled `"New Smart Object via Copy"`, and SHALL return the copy's path so
the caller can select it. A refusal SHALL return an empty path and record no
history state.

#### Scenario: Success records one state and returns the copy path

- **WHEN** the command runs on a copyable smart-object current layer
- **THEN** exactly one history state labelled `"New Smart Object via Copy"` is added, the layer count increases by one, and the bridge returns the new layer's path

#### Scenario: Refusal records nothing

- **WHEN** the command runs on a non-smart layer, a group, an adjustment layer, or a non-embedded object
- **THEN** the history count is unchanged, the document is unchanged, and the bridge returns an empty string

#### Scenario: Enabled only for a copyable object

- **WHEN** the current layer is not a copyable smart object
- **THEN** the command is disabled and its availability predicate mutates nothing
