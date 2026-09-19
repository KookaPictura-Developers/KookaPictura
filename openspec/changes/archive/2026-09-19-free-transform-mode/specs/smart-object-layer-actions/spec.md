## MODIFIED Requirements

### Requirement: The Place Smart Object command records exactly one undo state

The application SHALL expose the command `File > Place…` with the stable id
`file.place`. The command SHALL open a file dialog offering an `Images (…)`
filter beside the existing Photoshop files filter (`*.psd *.psb`) and SHALL be
enabled only when a document is open. On a chosen path the command SHALL route a
`*.psd`/`*.psb` file through the bridge `place_smart_object(path) -> QString`
and every other supported raster image through the bridge `place_image(path) ->
QString` defined by the `image-import` capability; a PSD/PSB file MUST NOT be
routed through Qt. For a Photoshop file the bridge SHALL read the file, derive
the display name from its base name, and invoke the engine operation on the
current document. On success only, the invoked bridge SHALL clear the link sets,
recomposite, record exactly one history state labelled `"Place"`, and return the
new layer path. On refusal the bridge SHALL return an empty string, record no
history state, and leave the document unchanged. After a successful place the
application SHALL select the new layer and start a Free Transform session on it
as defined by the `free-transform` capability; cancelling that session SHALL
leave the placed layer where it landed and MUST NOT remove or roll back the
`"Place"` history state.

#### Scenario: Success records one state and returns the layer path

- **WHEN** the command runs with a valid PSD file and an open document
- **THEN** exactly one history state labelled `"Place"` is added, the layer count increases by one, and the bridge returns the new layer's path

#### Scenario: A raster image records one state and returns the layer path

- **WHEN** the command runs with a supported raster image and an open document
- **THEN** exactly one history state labelled `"Place"` is added, a new smart-object layer is appended, and the image-import bridge returns the new layer's path

#### Scenario: A successful place starts a transform session

- **WHEN** the command places a supported image and returns the new layer path
- **THEN** a Free Transform session begins on the new layer and the `"Place"` history state remains recorded

#### Scenario: Refusal records nothing

- **WHEN** the command runs with a malformed source file
- **THEN** the history count is unchanged, the document is unchanged, and the bridge returns an empty string

#### Scenario: Enabled only when a document is open

- **WHEN** no document is open
- **THEN** the command is disabled
