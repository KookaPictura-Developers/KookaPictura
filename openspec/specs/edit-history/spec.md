# edit-history Specification

## Purpose
TBD - created by archiving change m14-undo-history. Update Purpose after archive.
## Requirements
### Requirement: Snapshot capture on mutating commands

The system SHALL capture a snapshot of the full editing state — the loaded `Document` and the active `Option<Selection>` — before each successful mutating command. The commands that capture SHALL be: `apply_filter`, `add_adjustment`, `remove_layer`, `set_layer_visible`, `resize_image`, `resize_canvas`, `rotate_doc`, `flip_doc`, `select_all`, `deselect`, and `magic_wand`. A command that fails validation or whose engine call errors SHALL NOT capture, leaving the history stack unchanged.

#### Scenario: Successful command adds a history state

- **WHEN** any listed command completes successfully
- **THEN** the pre-command document and selection are recoverable via undo and the history depth increased by one

#### Scenario: Failed command leaves history untouched

- **WHEN** a command is rejected (invalid parameters, unknown kind, no document loaded)
- **THEN** the history depth and stack contents are unchanged

### Requirement: Undo and redo restore the captured state

The system SHALL provide `undo() -> bool` and `redo() -> bool`. `undo()` SHALL restore the most recent captured snapshot (document and selection), decrement the history position, recompute the composite, and return `true`; with no state to undo it SHALL return `false` and leave the document untouched. `redo()` SHALL re-apply the state that was undone, return `true`, and recompute the composite; with nothing to redo it SHALL return `false`. Both SHALL leave the document bit-identical to the snapshot being restored.

#### Scenario: Undo reverts a filter bit-exactly

- **WHEN** a filter is applied to a document and `undo()` is called
- **THEN** it returns `true` and every document field, including the composite and the selection, equals its pre-filter value

#### Scenario: Redo re-applies the undone op bit-exactly

- **WHEN** `undo()` succeeded and `redo()` is called
- **THEN** the document equals its post-filter value

#### Scenario: Undo and redo at the stack ends are no-ops

- **WHEN** `undo()` is called with an empty history, or `redo()` is called with no undone state
- **THEN** each returns `false` and the document is unchanged

### Requirement: Bounded depth of 20 states

The history SHALL retain at most 20 captured states, matching the Photoshop CS6 default. When a capture occurs on a full stack, the oldest state SHALL be dropped. Redo information SHALL be discarded when a new successful mutating command captures after one or more undos.

#### Scenario: Oldest state falls off the bounded stack

- **WHEN** 21 successful mutating commands have captured snapshots in sequence
- **THEN** the first command's state is no longer recoverable via undo, and `undo()` returns the state before the second command

#### Scenario: New op invalidates redo

- **WHEN** after one or more `undo()` calls a new successful mutating command runs
- **THEN** `redo()` returns `false` for the previously undone state

### Requirement: History resets on document open

Opening a document (`PictureView::open`) SHALL clear the history stack regardless of whether the load succeeds, so undo never crosses a file boundary.

#### Scenario: Undo is unavailable after opening

- **WHEN** a document is opened after any number of commands
- **THEN** `can_undo()` is `false` and `undo()` returns `false`

### Requirement: App controls and accessors

The system SHALL expose `can_undo() -> bool`, `can_redo() -> bool`, and `history_depth() -> i32` on `PictureView`, and the Qt shell SHALL provide Undo and Redo controls wired to `undo()`/`redo()` with Ctrl+Z and Ctrl+Y shortcuts, disabled when the corresponding availability accessor is `false`, refreshing after every state change.

#### Scenario: Controls track availability

- **WHEN** the stack is empty, after a captured op, and after an undo
- **THEN** the Undo control is disabled, enabled, and enabled-with-Redo-enabled respectively

### Requirement: Headless self-test coverage

The app `--self-test` SHALL exercise undo/redo end-to-end: capture the image before a mutating command, run it, undo, and assert the displayed image is bit-identical to the pre-command image; redo and assert bit-identical to the post-command image; and assert a following command invalidates redo.

#### Scenario: Self-test proves undo and redo in the app

- **WHEN** the app runs `--self-test` with the layered fixture
- **THEN** the undo/redo assertions pass and the self-test exits successfully

