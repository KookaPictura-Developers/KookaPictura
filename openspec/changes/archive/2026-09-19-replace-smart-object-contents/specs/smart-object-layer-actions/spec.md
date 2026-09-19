## ADDED Requirements

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
