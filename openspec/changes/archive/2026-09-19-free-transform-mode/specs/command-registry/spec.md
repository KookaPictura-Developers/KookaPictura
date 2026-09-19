## ADDED Requirements

### Requirement: Free Transform command in the command table

The system SHALL declare the `Edit > Free Transform` command in the declarative
command table under the stable identifier `edit.freeTransform`, with the label
`Free Transform` and the default shortcut `Ctrl+T`, and SHALL register a handler
for it. The command's enablement predicate SHALL be true only when a document is
open and the current layer is a transformable target as defined by
`layer_can_free_transform` (a raster layer, or a smart-object layer whose
embedded source can be materialized), and false for a group, an adjustment
layer, a Background layer, a position-locked layer, or no layer. Triggering it
SHALL begin a Free Transform session on the current layer and leave the document
unchanged until the session commits. The remaining `Edit > Transform > …` menu
leaves SHALL stay in the table with no handler and remain visible and disabled.
Changing the label or shortcut SHALL NOT change the identifier.

#### Scenario: The command is enabled for a transformable layer

- **WHEN** a raster layer is the current layer and the Edit menu is opened
- **THEN** `Edit > Free Transform` is enabled and shows `Ctrl+T`

#### Scenario: The command is disabled for a non-transformable layer

- **WHEN** the current layer is a group, an adjustment layer, a Background layer, or a position-locked layer, or no document is open
- **THEN** `Edit > Free Transform` is disabled

#### Scenario: Triggering begins a session

- **WHEN** `edit.freeTransform` is triggered on a transformable layer
- **THEN** a Free Transform session begins on that layer and the document and history are unchanged until commit

#### Scenario: The other Transform leaves stay placeholders

- **WHEN** the `Edit > Transform` submenu is opened
- **THEN** `Scale`, `Rotate`, `Skew`, `Distort`, `Perspective`, `Warp`, and `Again` appear with no handler and are disabled
