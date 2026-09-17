## MODIFIED Requirements

### Requirement: Layers panel rows

The system SHALL present the full layer tree of the active document as rows in
topmost-first display order, each row showing a visibility toggle, a thumbnail
(or a folder glyph for a group), an editable name, and indicators for clipping,
a layer mask, and an adjustment/style badge. A group row SHALL be expandable
and collapsible, and its children SHALL be shown indented beneath it when
expanded; a layer clipped to the layer below SHALL be indented and its base
layer's name SHALL be underlined. The rows SHALL reflect the document and update
when the document changes or the active document changes.

#### Scenario: Rows reflect the tree [m39_tree]

- **WHEN** a document with a group containing children is active
- **THEN** the panel shows a row per node, topmost-first, with the group's
  children shown indented under it and the group row marked expandable

#### Scenario: A clipped layer is indented under its base [m39_badges]

- **WHEN** a layer is clipped to the layer below it
- **THEN** the clipped row is indented and the base layer's name is underlined

#### Scenario: No document [m39_tree]

- **WHEN** no document is open
- **THEN** the panel shows no rows and does not crash

### Requirement: Layer property editing

The system SHALL apply visibility, name, blend mode, opacity, fill, lock, and
color-label edits to the active document through the bridge, addressed by the
layer's path (a `/`-separated sequence of bottom-first child indices). Every
edit SHALL mark the document modified and be undoable. When more than one layer
is selected, a header edit SHALL apply to every selected layer in a single undo
step, and a layer that the CS6 contract excludes SHALL be skipped rather than
fail the edit. Opacity and fill SHALL each be reported and edited as a value in
`0..=255`, where `255` is fully opaque. A layer's lock state SHALL be reported
and edited per flag (transparency, image pixels, position, or all). A layer's
color label SHALL be one of `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`,
`Violet`, or `Gray`. The system SHALL refuse, rather than apply, an edit that
the CS6 contract excludes: a group has no Fill control, and the Background layer
SHALL expose neither opacity nor fill and SHALL refuse lock and color-label
edits. A fully locked layer SHALL refuse opacity and fill edits; its lock flags
SHALL remain editable so it can be unlocked. A refused edit SHALL leave that
layer unchanged.

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

#### Scenario: A multi-selection edit applies to the eligible layers in one step [m39_multi]

- **WHEN** several layers are selected and a blend mode is chosen
- **THEN** every selected layer that is not the Background is changed, an
  ineligible layer is left unchanged, and the whole edit is one undo step

#### Scenario: A nested layer is edited by path [m39_tree]

- **WHEN** a layer inside a group is renamed
- **THEN** only that nested layer changes, addressed by its path, and the
  operation is one undo step

### Requirement: Layer operations

The system SHALL offer New Layer, New Group, Duplicate Layer(s), Delete
Layer(s), Group Layers, Ungroup Layers, and Move Layer Up/Down from the panel
and its menus, applying them to the active document through the bridge, and the
document's rows SHALL update afterwards. A new layer SHALL be an empty, fully
transparent raster layer at the document size and SHALL NOT change the
composite. A new group SHALL be an empty group with a Normal blend. New
Layer/New Group SHALL insert directly above the selected layer, or, when the
selected row is a group, as that group's topmost child; with no selection they
SHALL insert at the top of the stack, and the new row SHALL become selected.
Duplicating SHALL deep-copy each selected layer — children, mask, adjustment
data, and all attributes included — directly above itself, naming the copy
`"<name> copy"`. Deleting SHALL remove every selected, eligible layer. Move
Layer Up/Down SHALL swap the selected layer with its neighbour within its own
container. Each applied operation SHALL mark the document modified and be one
undoable step.

#### Scenario: Add a layer

- **WHEN** the user clicks New Layer with a layer selected
- **THEN** an empty transparent layer is inserted directly above it, becomes selected, the row count grows by one, and the composite is unchanged

#### Scenario: Add a layer inside a selected group [m39_tree]

- **WHEN** the user clicks New Layer with a group selected
- **THEN** an empty transparent layer is inserted as that group's topmost child
  and becomes the selected row

#### Scenario: Add a group

- **WHEN** the user clicks New Group with no layer selected
- **THEN** an empty group is inserted at the top of the stack, becomes selected, and a new row appears

#### Scenario: Duplicate a layer

- **WHEN** the user duplicates the selected layer
- **THEN** a deep copy named `"<name> copy"` is inserted directly above it, the row count grows by one, and the copy becomes selected

#### Scenario: Delete a layer

- **WHEN** the user deletes the selected layer
- **THEN** the layer is removed and the panel updates

#### Scenario: Move a layer

- **WHEN** the user moves the selected layer up or down from the panel or row menu
- **THEN** its order within its container changes, the rows reorder, and the composite updates

### Requirement: Layer grouping commands

The system SHALL offer Group Layers and Ungroup Layers from the panel, the row
context menu, and the Layer menu, acting on the active document's currently
selected layers. Group Layers SHALL wrap the selection in one new group placed
at the topmost selected layer's position, preserving the selected layers' order,
and SHALL name the group `"Group N"` where N is one more than the highest
existing `Group <number>` name. Group Layers SHALL be refused, leaving the
document unchanged, when the selection is empty, contains the Background layer,
contains a fully locked layer, or spans more than one container. Ungroup Layers
SHALL splice each selected group's children into the parent at the group's
position, preserving their order, and SHALL skip a selected layer that is not a
group. Each applied operation SHALL be one undoable step.

#### Scenario: Group a layer

- **WHEN** the user runs Group Layers on a selected layer
- **THEN** the layer is wrapped in a new group that occupies the layer's position and the wrapped layer becomes its only child

#### Scenario: Group a multi-selection [m39_multi]

- **WHEN** the user runs Group Layers on several selected layers that share a
  container
- **THEN** one group is created at the topmost selected position containing all
  of them in their existing order, in one undo step

#### Scenario: Group refuses the Background

- **WHEN** the selection contains the Background layer
- **THEN** Group Layers is refused and the document is unchanged

#### Scenario: Ungroup a group with children [m39_multi]

- **WHEN** the user runs Ungroup Layers on one or more selected groups with children
- **THEN** each group is replaced in place by its children in their existing order, in one undo step

#### Scenario: Ungroup skips a non-group

- **WHEN** the user runs Ungroup Layers on a selection that includes a pixel or adjustment layer
- **THEN** the non-group is left unchanged and remains selected

## ADDED Requirements

### Requirement: Layer tree projection and paths

The system SHALL address every layer by a path: a `/`-separated sequence of
non-negative decimal indices, each an index into the bottom-first `children`
vector of the node named by the preceding segments, starting from the
document's layer list. The empty string SHALL be invalid, a segment SHALL have
no leading zeros, and a path SHALL be positional rather than a stable
identifier. The bridge SHALL project the whole tree as a depth-first,
topmost-first row list with, for each row, its path, depth, name, kind,
visibility, blend mode, opacity, fill, lock flags, color label, clipping flag,
mask presence, adjustment presence, expandability, and child count. A
path-based operation on an out-of-range or malformed path SHALL fail or no-op
without panicking and without changing the document.

#### Scenario: A nested node resolves by path [unit: path_resolve_nested; m39_tree]

- **WHEN** a path `"0/2/1"` is resolved
- **THEN** it names `doc.layers[0].children[2].children[1]` and its depth is 2

#### Scenario: The projection is depth-first and topmost-first [m39_tree]

- **WHEN** the row list of a document whose bottom layer is a group with two children is read
- **THEN** the rows are the group followed by both children, the group's row
  precedes its children, and the topmost node is the first row

#### Scenario: An out-of-range path is refused [unit: path_resolve_rejects]

- **WHEN** a setter is called with a path whose final segment is past the end of its container
- **THEN** the call fails, no history state is added, and the document is unchanged

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, the
color-label swatch, the clipping indentation and base underline, the layer-mask
thumbnail when a mask is present, and an adjustment/style badge when adjustment
content is present. A group with at least one child SHALL show an
expand/collapse control. If an expected icon asset is unavailable, the delegate
SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

### Requirement: Multi-selection edits and refusals

The system SHALL support contiguous and non-contiguous selection in the tree and
SHALL apply a header edit to all selected rows as one undo step. The eligibility
of each selected layer SHALL follow the CS6 contract: the Background and a fully
locked layer are skipped for opacity and fill; a group is skipped for fill; the
Background is skipped for lock, blend, and color; the Background and a fully
locked layer are skipped for delete. Group Layers SHALL refuse the whole
operation when the selection contains the Background, contains a fully locked
layer, or spans more than one container. A batch that changes no layer SHALL
record no history and emit no change.

#### Scenario: A multi-selection blend is one undo step [m39_multi]

- **WHEN** a visible layer and the Background are selected and Multiply is chosen
- **THEN** the visible layer becomes Multiply, the Background is unchanged, and
  one undo restores the prior blend mode

#### Scenario: Group fill is skipped [m39_multi]

- **WHEN** a group and a pixel layer are selected and the fill value is changed
- **THEN** the pixel layer's fill changes, the group's fill is unchanged, and the
  edit is one undo step

#### Scenario: Delete skips a fully locked layer [m39_multi]

- **WHEN** a fully locked layer and an unlocked layer are selected and Delete is invoked
- **THEN** the unlocked layer is removed, the fully locked layer remains, and the
  deletion is one undo step

### Requirement: Solo visibility

The system SHALL implement `Alt`-click solo visibility on a row's visibility
toggle: the first `Alt`-click SHALL snapshot the visibility of every row and
then show only the clicked layer or group (with its ancestors), and a second
`Alt`-click SHALL restore the snapshot exactly, including a prior partial-hidden
state. The snapshot SHALL be transient panel state and SHALL NOT be serialized
with the document. Each direction SHALL be exactly one undoable step, and the
snapshot SHALL be discarded when the active document changes or a structural
operation runs.

#### Scenario: Alt-click solos a layer [m39_solo]

- **WHEN** the user `Alt`-clicks a layer's eye in a document with several visible layers
- **THEN** only that layer (and its ancestor groups) is visible, one undo state
  is added, and the prior visibility is remembered

#### Scenario: A second Alt-click restores exactly [m39_solo]

- **WHEN** the user `Alt`-clicks again while solo is active
- **THEN** every row's prior visibility is restored exactly, including layers
  that were hidden before the first `Alt`-click

#### Scenario: Solo survives undo via its history state [m39_solo]

- **WHEN** the user undoes a solo operation
- **THEN** the prior visibility of every layer is restored

### Requirement: Inline rename with Tab navigation

The system SHALL begin inline name editing when the user double-clicks a row's
name, and while editing SHALL commit the edit and move to the next visible row
when the user presses `Tab`, and to the previous visible row when the user
presses `Shift+Tab`. At the last or first visible row, the commit SHALL occur
without wrapping.

#### Scenario: Double-click starts editing [m39_rename]

- **WHEN** the user double-clicks a row's name
- **THEN** an editor opens on that row and the document is unchanged until the edit commits

#### Scenario: Tab commits and moves down [m39_rename]

- **WHEN** the user edits a name and presses `Tab`
- **THEN** the name changes, the edit is one undo state, and the editor moves to the next visible row

#### Scenario: Shift+Tab moves up without wrapping [m39_rename]

- **WHEN** the user is editing the first visible row and presses `Shift+Tab`
- **THEN** the edit commits and editing ends without wrapping to the last row

### Requirement: Panel Options

The system SHALL provide a `Panel Options…` dialog from the panel menu with a
thumbnail size (`None`, `Small`, `Medium`, `Large`), thumbnail contents
(`Entire Document`, `Layer Bounds`), and an `Expand New Effects` toggle. The
defaults SHALL be `Medium`, `Entire Document`, and enabled respectively, and the
three values SHALL persist in the session store, with an older or missing value
loading the default. Thumbnail contents SHALL be resolved when the thumbnail is
built: `Layer Bounds` fills the thumbnail from the layer's own bounds, while
`Entire Document` places the layer's content at its document position scaled
into a document-sized thumbnail. `Expand New Effects` SHALL have no visible
effect until effect rows exist.

#### Scenario: Defaults on a fresh install [m39_options]

- **WHEN** the panel options are read with no saved session values
- **THEN** the thumbnail size is Medium, the thumbnail contents is Entire Document, and Expand New Effects is enabled

#### Scenario: Thumbnail contents changes the thumbnail [m39_options]

- **WHEN** a small layer in a large document is shown with Layer Bounds and then with Entire Document
- **THEN** the Layer Bounds thumbnail is filled by the layer while the Entire Document thumbnail shows the layer smaller in its document context

#### Scenario: Options persist [m39_options]

- **WHEN** the user changes the thumbnail size and expands-new-effects setting and the session is saved and reloaded
- **THEN** the panel restores those values

### Requirement: Panel and row menus

The system SHALL provide a panel menu and a row context menu. The panel menu
SHALL offer `Panel Options…`, New Layer, New Group, Duplicate Layer(s), Delete
Layer(s), Group Layers, Ungroup Layers, Move Layer Up, and Move Layer Down. The
row context menu SHALL offer those row commands plus Rename and a color-label
submenu containing `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`,
and `Gray`. Right-clicking the visibility toggle SHALL offer show/hide this
layer only and show/hide all. The bottom action strip SHALL remain exactly the
CS6 seven buttons, so Move Up/Down SHALL NOT appear in the strip. Commands that
M39 does not implement SHALL NOT be shown in these menus.

#### Scenario: The panel menu contains the wired commands [m39_menus]

- **WHEN** the panel menu is opened
- **THEN** it contains Panel Options, New Layer, New Group, Duplicate, Delete,
  Group, Ungroup, Move Up, and Move Down, and no unimplemented command

#### Scenario: The row menu sets a color label [m39_menus]

- **WHEN** the user right-clicks a row and picks Red from the color-label submenu
- **THEN** that layer's color label becomes Red

#### Scenario: Move Up/Down reorders from the menu [m39_menus]

- **WHEN** the user picks Move Layer Up for a row that is not the top of its container
- **THEN** the layer swaps with the layer above it within its container in one undo step

#### Scenario: The action strip has no Move buttons [m39_strip]

- **WHEN** the Layers panel is shown
- **THEN** its bottom strip contains exactly the seven CS6 buttons and no Move Up
  or Move Down button

### Requirement: Layer row tooltips

The system SHALL provide a tooltip for each row that includes the layer's name
and its kind, where the kind is one of pixel, group, adjustment, or background.

#### Scenario: A group row tooltip names the kind [m39_tooltip]

- **WHEN** the pointer rests on a group row
- **THEN** the tooltip contains the group's name and the word `group`

#### Scenario: A pixel row tooltip names the kind [m39_tooltip]

- **WHEN** the pointer rests on a pixel layer row
- **THEN** the tooltip contains the layer's name and the word `pixel`
