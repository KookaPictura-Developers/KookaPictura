## ADDED Requirements

### Requirement: Declarative command table
The system SHALL define every menu command once in a declarative table keyed by a
stable string identifier, carrying the menu path, label, default shortcut, and an
implemented flag. The menu bar SHALL be built from this table rather than from
hand-assembled `QAction`s.

#### Scenario: Menu bar is generated from the table
- **WHEN** the application frame is constructed
- **THEN** each top-level menu and child item corresponds to an entry in the command table

#### Scenario: Command identifiers are stable
- **WHEN** a command's label or shortcut is changed in the table
- **THEN** its identifier is unchanged and dispatch by identifier still resolves it

### Requirement: Full documented top-level menu tree
The system SHALL present the ten standard CS6 top-level menus -- File, Edit,
Image, Layer, Type, Select, Filter, View, Window, Help -- in that order.
Submenus SHALL follow the documented CS6 ordering. Commands without a handler
SHALL appear disabled rather than be omitted.

#### Scenario: All top-level menus are present
- **WHEN** the frame is shown
- **THEN** the menu bar contains File, Edit, Image, Layer, Type, Select, Filter, View, Window, and Help in that order

#### Scenario: Unimplemented leaf is disabled
- **WHEN** a documented command has no registered handler
- **THEN** its menu item is visible and disabled

### Requirement: Dispatch by command identifier
The system SHALL dispatch a triggered command to a handler registered against
that command's identifier. Triggering a command with no registered handler SHALL
be a no-op that does not crash.

#### Scenario: Registered handler runs
- **WHEN** a menu item for a command with a registered handler is triggered
- **THEN** the handler runs and performs the command's action

#### Scenario: Unregistered command is inert
- **WHEN** a command with no registered handler is triggered
- **THEN** the application does not report an error and remains usable

### Requirement: Enablement recomputed on menu open
The system SHALL evaluate each command's enablement predicate when its menu is
opened, so document-dependent commands are greyed rather than hidden when they do
not apply, and become enabled without rebuilding the menu.

#### Scenario: No document open
- **WHEN** no document is loaded and a document-requiring menu is opened
- **THEN** the document-requiring commands are disabled

#### Scenario: Document becomes available
- **WHEN** a document is loaded and the same menu is reopened
- **THEN** the document-requiring commands are enabled

### Requirement: Dynamic command labels
The system SHALL update labels whose meaning depends on state without rebuilding
the menu; in particular the undo command SHALL read "Undo" or "Redo" according to
the current history state.

#### Scenario: Undo label tracks history
- **WHEN** the document has no undoable step
- **THEN** the edit-history menu item reflects that no undo is available

#### Scenario: Label updates after an edit
- **WHEN** a document edit is recorded and the Edit menu is reopened
- **THEN** the history command label reflects the new state
