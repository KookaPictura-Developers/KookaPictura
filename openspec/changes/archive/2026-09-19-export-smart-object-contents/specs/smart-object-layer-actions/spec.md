## ADDED Requirements

### Requirement: Export Smart Object Contents accessor reads the embedded payload

The engine function `pictura-render::smart_object_source_bytes` SHALL accept
`(doc: &Document, path: &str) -> Option<Vec<u8>>`, SHALL resolve `path`, and SHALL
return `Some(payload.clone())` only when the resolved layer is not a group, has no
adjustment data, and its `smart_object` is `Some` with a non-empty payload. On
any other target it SHALL return `None`. The function SHALL be a pure read and
SHALL NOT mutate the document.

#### Scenario: A smart-object layer yields its payload

- **WHEN** `smart_object_source_bytes` is called on the path of a non-group, non-adjustment layer carrying a non-empty embedded payload
- **THEN** it returns `Some(bytes)` where `bytes` equal that layer's payload

#### Scenario: A non-smart layer yields none

- **WHEN** the target layer has no smart object
- **THEN** the function returns `None`

#### Scenario: A group yields none

- **WHEN** the target path resolves to a group layer
- **THEN** the function returns `None`

#### Scenario: An adjustment layer yields none

- **WHEN** the target layer has adjustment data
- **THEN** the function returns `None`

#### Scenario: An empty payload yields none

- **WHEN** the target layer's smart object has no payload or an empty payload
- **THEN** the function returns `None`

#### Scenario: An unresolved path yields none

- **WHEN** `path` does not resolve to a layer
- **THEN** the function returns `None`

#### Scenario: The document is unchanged

- **WHEN** `smart_object_source_bytes` returns a payload and the document is compared with a pre-call clone
- **THEN** the two documents are equal

### Requirement: The Export Contents bridge writes the payload byte-for-byte

The bridge `PictureView::export_smart_object_contents` SHALL accept
`(path: &QString, dest: &QString) -> bool`, SHALL read the source bytes for
`path` through `smart_object_source_bytes`, and SHALL write them to `dest` with
`std::fs::write`, without transcoding, so the destination holds the payload
exactly as stored. It SHALL return `true` only when the write succeeds. A
`None` source or a write failure SHALL return `false` and SHALL produce no
side effects beyond the attempted write.

#### Scenario: A successful export writes the exact bytes

- **WHEN** the current layer is a smart object with a payload and `dest` is a writable path
- **THEN** the bridge returns `true` and the file at `dest` is byte-for-byte the layer's payload

#### Scenario: A missing payload writes nothing and returns false

- **WHEN** `path` resolves to a layer without a non-empty payload
- **THEN** the bridge returns `false` and no file is written

#### Scenario: A failed write returns false

- **WHEN** the payload is available but `dest` cannot be written
- **THEN** the bridge returns `false`

### Requirement: The Export Contents command records no history state

The application SHALL expose the command `Layer > Smart Objects > Export
Contents…` with the stable id `LayerSmartObjectExportContents`
(`"layer.smartObject.exportContents"`). The command SHALL be enabled only when
the current layer is a smart object with a non-empty payload. Its handler SHALL
show a save dialog with a `*.psd` default filter, call
`export_smart_object_contents` for the current layer and the chosen destination,
and report the result. Because export only reads the document, the command SHALL
record no history state, SHALL NOT clear the link sets, and SHALL NOT
recomposite.

#### Scenario: Enabled only for a smart object with a payload

- **WHEN** the current layer is a smart object with a non-empty payload
- **THEN** the command is enabled, and it is disabled when the current layer is a non-smart layer, a group, or an adjustment layer

#### Scenario: The command is registered with its stable id

- **WHEN** the command registry is built
- **THEN** `LayerSmartObjectExportContents` is registered as an implemented `Layer > Smart Objects > Export Contents…` command

#### Scenario: Exporting adds no history state and mutates nothing

- **WHEN** the command runs on a smart-object current layer and a destination is chosen
- **THEN** the destination file holds the payload, the history count is unchanged, and the document is unchanged

#### Scenario: A cancelled or failed export records nothing

- **WHEN** the save dialog is cancelled or the bridge returns `false`
- **THEN** no history state is added and the document is unchanged
