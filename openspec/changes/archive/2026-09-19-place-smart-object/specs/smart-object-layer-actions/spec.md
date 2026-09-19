## ADDED Requirements

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
