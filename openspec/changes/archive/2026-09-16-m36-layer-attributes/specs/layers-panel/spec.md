## MODIFIED Requirements

### Requirement: Layer property editing

The system SHALL apply visibility, name, blend mode, opacity, fill, lock, and
color-label edits to the active document through the bridge, and each applied
edit SHALL mark the document modified and be undoable. Opacity and fill SHALL
each be reported and edited as a value in `0..=255`, where `255` is fully
opaque. A layer's lock state SHALL be reported and edited per flag
(transparency, image pixels, position, or all). A layer's color label SHALL be
one of `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, or `Gray`.
The system SHALL refuse, rather than apply, an edit that the CS6 contract
excludes: a group has no Fill control, and the Background layer SHALL expose
neither opacity nor fill and SHALL refuse lock and color-label edits. A fully
locked layer SHALL refuse opacity and fill edits; its lock flags SHALL remain
editable so it can be unlocked. A refused edit SHALL return failure and leave
the document unchanged.

#### Scenario: Change opacity

- **WHEN** the user changes a layer's opacity in the panel
- **THEN** the document's layer opacity changes, the composite updates, and the document reports modified

#### Scenario: Change blend mode

- **WHEN** the user selects a blend mode for a layer
- **THEN** the layer's blend mode changes and the composite updates

#### Scenario: Rename a layer

- **WHEN** the user edits a layer's name
- **THEN** the layer name changes and the tab title reflects an untitled-name change where applicable

#### Scenario: Change fill opacity

- **WHEN** the user changes a layer's fill opacity to 128 in the panel
- **THEN** the layer's fill becomes 128, the composite updates with the layer's pixels at half strength, and the edit is undoable as one step

#### Scenario: Toggle a lock flag

- **WHEN** the user enables Lock Position for a layer and then enables Lock All
- **THEN** the layer reports the position bit and then all three lock bits, the panel disables opacity and fill, and each toggle is one undo state

#### Scenario: Set a color label

- **WHEN** the user picks Red from the row's color-label menu
- **THEN** the layer's color label becomes Red and the row reflects it

#### Scenario: Fill is unavailable for a group

- **WHEN** a group is selected
- **THEN** the Fill control is disabled and a fill edit is refused, while Opacity remains editable

#### Scenario: Background and fully locked layers refuse edits

- **WHEN** the Background layer or a fully locked layer is selected and opacity or fill is changed
- **THEN** the edit is refused, the document is unchanged, and the control is disabled
