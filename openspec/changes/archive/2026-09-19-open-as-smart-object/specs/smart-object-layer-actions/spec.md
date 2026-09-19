## ADDED Requirements

### Requirement: Open As Smart Object decodes a PSD/PSB source or refuses

The engine operation SHALL decode `bytes` with `pictura_codec::read_psd`; its
signature is `pictura-render::open_as_smart_object(filename: &str, bytes:
&[u8]) -> Option<Document>`. It SHALL return `None` when `bytes` do not parse as
a PSD/PSB document. On success it SHALL return `Some(doc)` where `doc` is a new
document holding the source as a smart object. The operation SHALL NOT mutate
any input.

#### Scenario: A PSD source opens

- **WHEN** `open_as_smart_object` is called with the bytes of a valid PSD/PSB document and a display name
- **THEN** it returns `Some(doc)`

#### Scenario: Malformed bytes are refused

- **WHEN** `open_as_smart_object` is called with bytes that are not a parseable PSD/PSB document
- **THEN** it returns `None`

### Requirement: An opened-as-smart-object document is new, source-sized, and holds one embedded layer

The returned document SHALL be built with `Document::new(src.width,
src.height, src.mode, src.depth)`, preserving the source's dimensions, color
mode, and bit depth. Its merged composite SHALL be set from the decoded source's
composite so the pre-render composite shows the source. It SHALL contain exactly
one layer, appended by the existing
`place_smart_object(&mut doc, filename, bytes)`: a topmost, channel-less,
native-size layer at `(0, 0)` carrying `SmartObject { kind: Embedded, payload:
Some(bytes), filename, filetype: 8BPB, creator: 8BIM, .. }`. Because the layer
has no color channel, the compositor SHALL render it from its embedded payload.

#### Scenario: Geometry, mode, and depth match the source

- **WHEN** a source of size `w × h` in a given color mode and bit depth is opened
- **THEN** the returned document has `width = w`, `height = h`, and the source's mode and depth

#### Scenario: Exactly one embedded smart-object layer

- **WHEN** a valid PSD/PSB source is opened
- **THEN** the returned document has exactly one layer, its `smart_object.kind` is `Embedded`, its payload equals the source `bytes`, and its name is the supplied display name

#### Scenario: The composite shows the source colour

- **WHEN** a solid-colour PSD is opened as a smart object and the document is composited
- **THEN** `composite_rgba` shows the source colour

### Requirement: The Open As Smart Object bridge opens an untitled document and records one undo state

The bridge `PictureView::open_as_smart_object(path: &QString) -> bool` SHALL
read `path`, derive the source display name from the file's base name, and call
the engine operation with those bytes. On success it SHALL store the composited
image and the document, reset the edit state, capture exactly one history state
labelled `"Open As Smart Object"`, and set the view's path to NONE so the new
document is untitled. It SHALL return `false` without mutating the view when the
file is missing, unreadable, or not a PSD/PSB document.

#### Scenario: Success stores one untitled document

- **WHEN** the bridge is called with a valid PSD/PSB file
- **THEN** it returns `true`, exactly one history state labelled `"Open As Smart Object"` is added, and the view holds one document with one smart-object layer

#### Scenario: A refused source mutates nothing

- **WHEN** the bridge is called with a missing, unreadable, or malformed file
- **THEN** it returns `false`, the current document is unchanged, and no history state is added

#### Scenario: The opened document is untitled so Save cannot overwrite the source

- **WHEN** a source file is opened as a smart object
- **THEN** the view's path is empty and the source file is left untouched

### Requirement: The Open As Smart Object command creates a new untitled tab

The application SHALL expose `File > Open As Smart Object…` with the stable id
`FileOpenAsSmartObject` (`"file.openAsSmartObject"`). The command SHALL open a
file dialog filtered to Photoshop files (`*.psd *.psb`) and SHALL be enabled
whenever the application is running, like File Open. On a chosen path it SHALL
mirror `PicturaMainWindow::openPath`: construct a new `PictureView`, call
`open_as_smart_object`, and on success call `addDocument(view, QString())` so a
new untitled tab is created. On refusal it SHALL discard the view and add no
tab.

#### Scenario: The dialog filters to PSD/PSB and needs no open document

- **WHEN** the command is available with no document open
- **THEN** it is enabled and opens a file dialog filtered to `*.psd *.psb`

#### Scenario: Success adds one untitled tab

- **WHEN** the command is dispatched with a valid PSD/PSB path
- **THEN** exactly one new tab is added and its document path is empty

#### Scenario: Refusal adds no tab

- **WHEN** the command is dispatched with a malformed source
- **THEN** no tab is added and the document count is unchanged
