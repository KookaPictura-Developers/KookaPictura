## MODIFIED Requirements

### Requirement: Layer property editing

The system SHALL apply visibility, name, blend mode, opacity, fill, lock, and
color-label edits to the active document through the bridge, addressed by the
layer's path (a `/`-separated sequence of bottom-first child indices). Every
edit SHALL mark the document modified and be undoable. When more than one layer
is selected, a header edit SHALL apply to every selected layer in a single undo
step, and a layer that the CS6 contract excludes SHALL be skipped rather than
fail the edit. The system SHALL report and edit opacity and fill through the
bridge as a value in `0..=255`, where `255` is fully opaque; the panel SHALL
present both as percentages per the Opacity and Fill percent controls
requirement. A layer's lock state SHALL be reported and edited per flag
(transparency, image pixels, position, nesting, or all). A layer's color label
SHALL be one of `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, or
`Gray`. The system SHALL refuse, rather than apply, an edit that the CS6
contract excludes: a group has no Fill control, and the Background layer SHALL
expose neither opacity nor fill and SHALL refuse lock and color-label edits. A
fully locked layer SHALL refuse opacity and fill edits; its lock flags SHALL
remain editable so it can be unlocked. A refused edit SHALL leave that layer
unchanged.

#### Scenario: Change opacity

- **WHEN** the user changes a layer's opacity in the panel
- **THEN** the document's layer opacity changes, the composite updates, and the document reports modified

#### Scenario: Change blend mode

- **WHEN** the user selects a blend mode for a layer
- **THEN** the layer's blend mode changes and the composite updates

#### Scenario: Rename a layer [m39_rename]

- **WHEN** the user edits a layer's name
- **THEN** the layer name changes and the tab title reflects an untitled-name change where applicable

#### Scenario: Change fill opacity

- **WHEN** the user changes a layer's fill opacity to 128 in the panel
- **THEN** the layer's fill becomes 128, the composite updates with the layer's pixels at half strength, and the edit is undoable as one step

#### Scenario: Toggle a lock flag

- **WHEN** the user enables Lock Position for a layer and then enables Lock All
- **THEN** the layer reports the position bit and then all four lock bits, the panel disables opacity and fill, and each toggle is one undo state

#### Scenario: Set a color label

- **WHEN** the user picks Red from the row's color-label menu
- **THEN** the layer's color label becomes Red and the row reflects it

#### Scenario: Fill is unavailable for a group

- **WHEN** a group is selected
- **THEN** the Fill control is disabled and a fill edit is refused, while Opacity remains editable

#### Scenario: Background and fully locked layers refuse edits

- **WHEN** the Background layer or a fully locked layer is selected and opacity or fill is changed
- **THEN** the edit is refused, the document is unchanged, and the control is disabled

#### Scenario: A multi-selection edit applies to the eligible layers in one step [m39_multi]

- **WHEN** several layers are selected and a blend mode is chosen
- **THEN** every selected layer that is not the Background is changed, an
  ineligible layer is left unchanged, and the whole edit is one undo step

#### Scenario: A nested layer is edited by path [m39_tree]

- **WHEN** a layer inside a group is renamed
- **THEN** only that nested layer changes, addressed by its path, and the
  operation is one undo step

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, the
color-label swatch, a clipping-mask indicator for a clipped layer, the clipping
indentation and base underline, the layer-mask thumbnail when a mask is present,
and an adjustment/style badge when adjustment content is present. A group with
at least one child SHALL show an expand/collapse control. If an expected icon
asset is unavailable, the delegate SHALL omit that badge while keeping the row
legible rather than fail.

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A clipped layer shows the clipping indicator [lpc_clip]

- **WHEN** a layer is clipped to the layer below it
- **THEN** its row draws the clipping-mask indicator in addition to the
  indentation and the base layer's underline

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

## ADDED Requirements

### Requirement: Opacity and Fill percent controls

The Layers panel SHALL present Opacity and Fill as percentages in the range
`0..=100`, each through one reusable control that offers a text value, a popup
slider, and scrubbing by pressing and dragging horizontally on the field. The
control SHALL convert between the displayed percentage and the stored `0..=255`
value with `round(pct * 255 / 100)` on edit and `round(value * 100 / 255)` on
display, so `100`% is `255`, `50`% is `128`, and `0`% is `0`. The control SHALL
emit a change only for user input, so a programmatic sync does not feed back.
The existing enable/refusal rules SHALL be unchanged: Fill is unavailable for a
group, and the Background and a fully locked layer disable both controls.

#### Scenario: A percentage maps to the stored byte [lpc_percent]

- **WHEN** Opacity is set to 50 % on an eligible layer
- **THEN** the layer's stored opacity becomes 128 and the document is modified

#### Scenario: A stored byte displays as a percentage [lpc_percent]

- **WHEN** a layer with stored opacity 128 becomes the current row
- **THEN** the Opacity control displays 50 %

#### Scenario: The popup slider and label drag both change the value [lpc_percent]

- **WHEN** the user drags the Opacity popup slider or drags horizontally on the
  Opacity field
- **THEN** the percentage changes and the new value is applied in one undo step

#### Scenario: A programmatic sync does not re-apply the edit [lpc_percent]

- **WHEN** the panel reflects an existing value into the control without user input
- **THEN** no edit is sent and no history state is added

### Requirement: Nesting lock

The system SHALL support a fifth lock flag, `nesting`, alongside transparency,
image pixels, position, and all. The flag SHALL be part of the layer's lock
state, reported in the row projection, and included in the "all" set, so that
Lock All also sets nesting and a fully locked layer is one whose four lower bits
are all set. A nesting-locked layer SHALL keep its structural parent: the system
SHALL refuse Group Layers and Ungroup Layers when the selection contains a
nesting-locked layer, leaving the document unchanged, while a within-container
reorder (Move Layer Up/Down) SHALL remain allowed. The flag SHALL round-trip
through the PSD `lspf` block.

#### Scenario: The nesting flag round-trips to the row and the bridge [lpc_nesting]

- **WHEN** the nesting lock is enabled for a layer and the layer's row is read
- **THEN** the row reports the nesting bit and `all` reports the four-bit set

#### Scenario: Grouping a nesting-locked layer is refused [lpc_nesting]

- **WHEN** Group Layers is run with a nesting-locked layer selected
- **THEN** the operation is refused, the layer's parent is unchanged, and no
  history state is added

#### Scenario: Ungrouping a nesting-locked group is refused [lpc_nesting]

- **WHEN** Ungroup Layers is run with a nesting-locked group selected
- **THEN** the operation is refused and the group and its children are unchanged

#### Scenario: Reordering within the container still works [lpc_nesting]

- **WHEN** Move Layer Up is run on a nesting-locked layer inside a group
- **THEN** the layer swaps with its sibling above it in one undo step

#### Scenario: The nesting bit survives a PSD round-trip [lpc_nesting]

- **WHEN** a layer with the nesting lock is written and read back
- **THEN** the nesting bit is set on the re-read layer
