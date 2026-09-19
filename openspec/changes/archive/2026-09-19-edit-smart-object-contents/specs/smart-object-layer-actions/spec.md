## ADDED Requirements

### Requirement: Edit Contents eligibility and refusal

The picture view SHALL expose a read-only predicate
`layer_can_edit_smart_object_contents(path) -> bool`. It SHALL return `true`
exactly when the resolved target is a non-group, non-adjustment layer whose
`smart_object` is of kind `Embedded` and carries a non-empty payload AND
`pictura_codec::read_psd(payload)` parses that payload as a PSD/PSB document. It
SHALL return `false` for every other target, including a non-`Embedded` object
and a layer whose non-empty payload is not a parseable PSD/PSB (for example a
placed JPEG). The predicate SHALL be a pure read and SHALL NOT mutate the
document. When it returns `false` the Edit Contents command SHALL be disabled
and SHALL record no history state.

#### Scenario: A PSD/PSB source is editable

- **WHEN** `layer_can_edit_smart_object_contents` is called on a non-group, non-adjustment layer whose `Embedded` smart object has a payload that parses as a PSD/PSB document
- **THEN** it returns `true`

#### Scenario: A non-parsing source is refused

- **WHEN** the target layer carries a non-empty payload that does not parse with `read_psd` (a placed JPEG)
- **THEN** the predicate returns `false` and the command is disabled

#### Scenario: A non-embedded object is refused

- **WHEN** the target layer's smart object is not of kind `Embedded` (for example an external, alias, or unresolved record)
- **THEN** the predicate returns `false` and the command is disabled

#### Scenario: A non-smart layer is refused

- **WHEN** the target layer has no smart object
- **THEN** the predicate returns `false`

#### Scenario: A group or adjustment layer is refused

- **WHEN** the target path resolves to a group or a layer with adjustment data
- **THEN** the predicate returns `false`

#### Scenario: An empty payload is refused

- **WHEN** the target layer's smart object has no payload or an empty payload
- **THEN** the predicate returns `false`

#### Scenario: An unresolved path is refused

- **WHEN** `path` does not resolve to a layer
- **THEN** the predicate returns `false`

#### Scenario: The predicate mutates nothing

- **WHEN** the predicate is called on any target and the document is compared with a pre-call clone
- **THEN** the two documents are equal

### Requirement: The Edit Contents command opens the source as a new untitled editor document

The application SHALL expose `Layer > Smart Objects > Edit Contents` with the
stable id `LayerSmartObjectEditContents` (`"layer.smartObject.editContents"`).
The command SHALL be enabled only when
`layer_can_edit_smart_object_contents` returns `true` for the current layer. On
dispatch it SHALL export the target's embedded payload byte-for-byte to a
per-session temporary file, load that file into a new document as an ordinary
document (its own layers and composite), and add it through
`addDocument(editor, QString())` so a new untitled tab opens. The editor document
SHALL be the decoded source, NOT a document wrapping the source as a smart-object
layer, and the editor tab SHALL be untitled regardless of the temporary file's
path.

#### Scenario: The command is registered with its stable id

- **WHEN** the command registry is built
- **THEN** `LayerSmartObjectEditContents` is registered as an implemented `Layer > Smart Objects > Edit Contents` command

#### Scenario: Enabled only for an editable source

- **WHEN** the current layer is an editable smart object
- **THEN** the command is enabled, and it is disabled when the current layer is a non-smart layer, a group, an adjustment layer, a non-`Embedded` object, an object with an empty payload, or a payload that does not parse as PSD/PSB

#### Scenario: One new untitled editor tab opens

- **WHEN** the command runs on an editable smart-object layer
- **THEN** exactly one tab is added, its document path is empty, and it holds the decoded source content

#### Scenario: Opening records no state on the origin

- **WHEN** the command opens an editor
- **THEN** the origin document is unchanged and its history count is unchanged

#### Scenario: An ineligible target adds no tab

- **WHEN** the command is dispatched while the current layer is not editable
- **THEN** no tab is added, no history state is recorded, and no origin state changes

### Requirement: Saving an editor commits the source and records exactly one Edit Contents undo state

The bridge SHALL expose `commit_smart_object_edit(layer_path, file_path) ->
bool`. It SHALL apply the same engine operation as
`replace_smart_object_contents` (read `file_path`, decode it as a PSD/PSB
source, swap the origin layer's embedded source, clear its pixel proxy, and drop
the preserved config block and matching linked-source record) and, on success
only, SHALL clear the link sets, recomposite, and record exactly one history
state labelled `"Edit Contents"`. The existing
`replace_smart_object_contents` method SHALL keep its signature, behavior, and
`"Replace Contents"` label. Saving an editor tab SHALL be intercepted in the
application's Save path: it SHALL call `editor->save(temp_file)` then
`origin->commit_smart_object_edit(layer_path, temp_file)`, refresh, and return
success WITHOUT opening a Save dialog and WITHOUT writing any user-chosen file.
The editor tab SHALL remain open and SHALL be marked clean. On any refusal the
bridge SHALL return `false`, record no history state, and leave the origin
unchanged.

#### Scenario: Save commits and records one state

- **WHEN** an editor tab is saved after the document was edited
- **THEN** the origin layer's embedded payload is the edited source and exactly one history state labelled `"Edit Contents"` is added

#### Scenario: The origin recomposites to the edited source

- **WHEN** the committed origin is composited
- **THEN** the original layer paints the edited source scaled into its existing rect

#### Scenario: The editor stays open and clean

- **WHEN** saving an editor commits successfully
- **THEN** the editor tab stays open and reports no unsaved changes

#### Scenario: Save writes no user file and shows no dialog

- **WHEN** Save runs on a session editor
- **THEN** no Save dialog is shown, no user-chosen file is written, and the only file written is the session temporary file

#### Scenario: Refusal records nothing

- **WHEN** the edited document cannot be serialized or the commit is refused
- **THEN** the bridge returns `false`, the origin is unchanged, and its history count is unchanged

#### Scenario: Replace Contents still records its own label

- **WHEN** `replace_smart_object_contents` runs on a replaceable layer
- **THEN** it records exactly one `"Replace Contents"` state, unchanged from before

### Requirement: Discarding or closing an Edit Contents session leaves the origin unchanged

Closing an editor tab without saving SHALL leave the origin document unchanged
and SHALL record no history state. Closing an editor SHALL remove its session and
delete its temporary file. Closing the origin SHALL remove every session
targeting it and SHALL leave each orphaned editor open as an ordinary untitled
document; a later Save on an orphaned editor SHALL behave like Save As for an
untitled document and SHALL NOT write the deleted temporary file. Destroying the
window SHALL delete every session's temporary file.

#### Scenario: Closing the editor unsaved leaves the origin unchanged

- **WHEN** an editor tab is closed without saving
- **THEN** the origin document is unchanged and its history count is unchanged

#### Scenario: Closing the editor removes the session and its temp file

- **WHEN** an editor tab is closed
- **THEN** the session temporary file no longer exists and the session is dropped

#### Scenario: Closing the origin orphans the editor as untitled

- **WHEN** the origin document is closed while an editor for it is open
- **THEN** the editor tab stays open with an empty document path and no session

#### Scenario: An orphaned editor does not write the deleted temp file

- **WHEN** Save runs on an orphaned editor
- **THEN** it routes through Save As and no file is written to the deleted temporary path

#### Scenario: Window destruction cleans up temp files

- **WHEN** the main window is destroyed with live sessions
- **THEN** every session's temporary file is deleted
