# layers-panel Specification

## Purpose
TBD - created by archiving change m20-panels. Update Purpose after archive.
## Requirements
### Requirement: Layers panel rows

The system SHALL present the full layer tree of the active document as rows in
topmost-first display order, each row showing a visibility toggle, a thumbnail
(or a folder glyph for a group), an editable name, and indicators for clipping,
a layer mask, and an adjustment/style badge. The visibility toggle SHALL be the
row's only visibility control: the row SHALL NOT paint a separate checkbox or
check indicator. A group row SHALL be expandable and collapsible, and its
children SHALL be shown indented beneath it when expanded; a layer clipped to
the layer below SHALL be indented and its base layer's name SHALL be underlined.
The rows SHALL reflect the document and update when the document changes or the
active document changes.

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

#### Scenario: A row paints no checkbox indicator [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its only visibility control is the eye icon and the model reports no
  check state for the row

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

### Requirement: Layers panel docking and toggle
The system SHALL host the Layers panel in a registered dock with a stable
`objectName` and SHALL expose a `Window > Panels > Layers` toggle.

#### Scenario: Toggle the Layers panel
- **WHEN** the user toggles Layers from the Window menu
- **THEN** the panel is shown or hidden without changing the document

### Requirement: Thumbnails downsample without a full-size intermediate

Layer thumbnails SHALL be produced by downsampling the layer's channel data
directly, without first materializing a full-resolution RGBA image, so that
thumbnail cost is not proportional to the layer's full pixel buffer. The
thumbnail's visual result SHALL be unchanged.

#### Scenario: A thumbnail of a large layer avoids a full-resolution image

- **WHEN** a 24 px thumbnail is generated for a 4000×4000 layer
- **THEN** no full-resolution RGBA image is allocated, the thumbnail is visually
  unchanged, and generation completes within a small time budget

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

### Requirement: Layers panel action strip icons

The system SHALL give each Layers-panel action-strip button an icon from the
frozen layers asset set: `layers.link` (Link Layers), `layers.fx` (Layer Style),
`layers.mask` (Add Layer Mask), `layers.fillAdjustment` (New Fill / Adjustment
Layer), `layers.group` (New Group), `layers.newLayer` (New Layer), and
`layers.delete` (Delete). The buttons SHALL keep their text labels, and adding
an icon SHALL NOT change what each button does. Buttons whose operation is not
yet implemented (link, fx, mask) SHALL be shown disabled until their operation
lands.

#### Scenario: The implemented strip buttons carry icons

- **WHEN** the Layers panel is shown
- **THEN** its fill/adjustment, group, new-layer, and delete buttons each carry their documented icon

#### Scenario: A deferred button is disabled

- **WHEN** the Layers panel is shown before the link, fx, or mask operation exists
- **THEN** that button is disabled

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
color-label swatch, a clipping-mask indicator for a clipped layer, the clipping
indentation and base underline, the layer-mask thumbnail when a mask is present,
and an adjustment/style badge when adjustment content is present. A layer whose
lock state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A group with at least one child SHALL show a disclosure icon — right when
collapsed, down when expanded — at its indented position, and clicking that icon
SHALL expand or collapse the group. If an expected icon asset is unavailable, the
delegate SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: A group shows a disclosure icon [lpr_chevron]

- **WHEN** a group with children is shown collapsed and then expanded
- **THEN** it shows the right-pointing icon when collapsed and the down-pointing
  icon when expanded, and clicking the icon toggles it

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

#### Scenario: A locked layer shows a right-side lock badge [lpc_lockbadge]

- **WHEN** a layer has any lock flag set
- **THEN** its row draws a lock badge on the right side, and an unlocked layer
  draws no lock badge

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

### Requirement: Opacity and Fill percent controls

The Layers panel SHALL present Opacity and Fill as percentages in the range
`0..=100`, each through one reusable control that offers a text value, a popup
slider, and scrubbing by pressing and dragging horizontally on the label or the
field. Each field's `%` sign SHALL be rendered inside the value box rather than
beside it. The control SHALL convert between the displayed percentage and the
stored `0..=255` value with `round(pct * 255 / 100)` on edit and `round(value *
100 / 255)` on display, so `100`% is `255`, `50`% is `128`, and `0`% is `0`. The
control SHALL distinguish a live preview from a commit: while a scrub or slider
drag is in progress it SHALL emit preview changes that update the canvas without
adding a history state, and when the edit finishes it SHALL emit one commit that
adds exactly one undo state for the whole edit. A completed text edit SHALL also
add one state. The control SHALL emit nothing for a programmatic sync, so a
programmatic sync does not feed back. The existing enable/refusal rules SHALL be
unchanged: Fill is unavailable for a group, and the Background and a fully locked
layer disable both controls.

#### Scenario: A percentage maps to the stored byte [lpc_percent]

- **WHEN** Opacity is set to 50 % on an eligible layer
- **THEN** the layer's stored opacity becomes 128 and the document is modified

#### Scenario: A stored byte displays as a percentage [lpc_percent]

- **WHEN** a layer with stored opacity 128 becomes the current row
- **THEN** the Opacity control displays 50 %

#### Scenario: The popup slider and label drag both change the value [lpc_percent]

- **WHEN** the user drags the Opacity popup slider or drags horizontally on the
  Opacity field
- **THEN** the percentage changes, the canvas updates live, and the whole drag is
  applied in one undo step

#### Scenario: A drag adds one undo state only at release [lpc_preview]

- **WHEN** an Opacity or Fill drag previews several values and is then released
- **THEN** no undo state was added during the drag and exactly one undo state is
  added on release

#### Scenario: The percent sign is inside the value box [lpr_percent]

- **WHEN** the Opacity or Fill field is shown
- **THEN** its `%` sign is drawn inside the value box rather than beside it

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

### Requirement: Layers panel header layout

The panel SHALL stack its header controls in this order, top to bottom: the
filter/search row; a row containing the blend-mode popup and a labeled Opacity
field; a row containing the five lock toggles and a labeled Fill field; the
layer list; and the action strip. The Opacity and Fill fields SHALL each carry a
leading text label, and each field's popup slider SHALL open centred
horizontally under the field rather than aligned to its right edge. The panel
SHALL NOT provide its own menu button; the wired Layer commands SHALL remain
reachable from the panel-group widget menu. The five lock toggles SHALL all sit
in the single lock row above the layer list, not below it.

#### Scenario: The header order is filter / blend+Opacity / locks+Fill / list [lpc_order]

- **WHEN** the Layers panel is shown
- **THEN** the filter row is above the blend and Opacity row, which is above the
  lock and Fill row, which is above the layer list, with the action strip below
  the list

#### Scenario: Opacity and Fill are labeled [lpc_labels]

- **WHEN** the Layers panel is shown
- **THEN** the Opacity field and the Fill field each have a text label directly
  to their left

#### Scenario: The percent picker opens centered [lpc_popup]

- **WHEN** the user opens the Opacity or Fill slider popup
- **THEN** the popup is centered horizontally under its field

#### Scenario: The panel has no separate menu button [lpc_nomenu]

- **WHEN** the Layers panel header is inspected
- **THEN** it contains no panel-menu button, and the Layer commands are offered
  by the panel-group widget menu

### Requirement: Layers panel header fields

The Opacity and Fill fields SHALL each carry their label as part of the field,
and pressing and dragging that label horizontally SHALL scrub the value exactly
as dragging the field does. Each field's `%` sign SHALL be shown inside the
value box. The lock strip SHALL hold five toggles; the toggles for alpha, paint,
position, and nesting SHALL be semantic icons describing what they lock
(transparency, image pixels, position, nesting), and only the full-lock toggle
SHALL be a padlock icon.

#### Scenario: Dragging the label scrubs the value [lpr_label]

- **WHEN** the user presses the `Opacity` label and drags horizontally
- **THEN** the opacity percentage changes and is applied in one undo step, the
  same as dragging the field itself

#### Scenario: The value shows a percent sign inside the box [lpr_percent]

- **WHEN** the Opacity or Fill field is shown
- **THEN** its value's `%` sign is drawn inside the value box

#### Scenario: Only Lock All is a padlock [lpr_locks]

- **WHEN** the lock strip is shown
- **THEN** the alpha, paint, position, and nesting toggles use their semantic
  icons and the full-lock toggle is the only padlock

### Requirement: Layer drag and drop

A layer row SHALL be draggable. Starting a drag SHALL NOT extend the selection
or begin a rubber-band selection. The system SHALL validate a candidate drop
during the drag and SHALL show a drop indicator only for a valid target, and it
SHALL reject an invalid drop without calling the bridge or changing the
document. Releasing a dragged row above or below another row SHALL reorder it at
that position, releasing it onto a group row SHALL reparent it as a child of
that group, and releasing it on the empty viewport below the last row SHALL move
it to the document root (out of any group), each as one undoable step. The
system SHALL refuse, leaving the document unchanged, a move of the Background
layer, of a fully-locked or nesting-locked layer, of a row onto itself, or of a
row into its own descendant. Dropping a dragged row on a bottom-strip button
SHALL apply that button's action to the dragged row: Delete deletes it, New
Layer duplicates it, and New Group groups it; buttons whose operation is not yet
implemented (mask, link, fx) SHALL be inert.

#### Scenario: Dragging reorders a row [lpr_drag]

- **WHEN** a row is dragged and released above a sibling
- **THEN** the row moves to that position in one undo step and the drag does not
  select any other row

#### Scenario: Dragging onto a group reparents [lpr_drag]

- **WHEN** a row is dragged and released onto a group row
- **THEN** the row becomes a child of that group in one undo step

#### Scenario: Dragging out of a group to the viewport [lpr_drop_out]

- **WHEN** a row inside a group is dragged and released on the empty viewport
  below the last row
- **THEN** the row is reparented to the document root in one undo step

#### Scenario: An invalid target is rejected before commit [lpr_drop_rules]

- **WHEN** a row is dragged over its own descendant, onto itself, or over a
  refusal target such as the Background
- **THEN** no valid drop indicator is shown and releasing the drag changes
  nothing and adds no history state

#### Scenario: A locked or Background row is refused [lpr_drag]

- **WHEN** the Background, a fully-locked, or a nesting-locked row is dragged
  onto another row
- **THEN** the move is refused and the document is unchanged

#### Scenario: Dropping on Delete deletes the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the Delete strip button and released
- **THEN** that row is deleted in one undo step

#### Scenario: Dropping on New Layer duplicates the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Layer strip button and released
- **THEN** the row is duplicated in one undo step

#### Scenario: Dropping on New Group groups the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the New Group strip button and released
- **THEN** the row is wrapped in a new group in one undo step

### Requirement: Layer management commands

The system SHALL expose the layer-management commands from the Layers panel
menus and the `Layer` menu, wired to the active document through the bridge.
The set SHALL include Merge Down / Merge Layers (`Ctrl+E`), Merge Visible
(`Shift+Ctrl+E`), Merge Clipping Mask, Flatten Image, Layer from Background…,
Background From Layer, Layer via Copy (`Ctrl+J`), Layer via Cut
(`Shift+Ctrl+J`), Delete Hidden Layers, Hide Layers, Select Similar, Select
Linked Layers, Link Layers, Unlink Layers, and the Rasterize entries. Each
available command SHALL apply as exactly one undoable step and the panel rows
SHALL update afterwards. A command that does not apply to the current selection
SHALL be disabled rather than shown enabled and refused, and the `Rasterize`
Type, Shape, Vector Mask, Smart Object, Video, and 3D entries SHALL stay visible
and disabled. `Alt`-clicking the New Layer/New Group panel button SHALL open
the New Layer/New Group dialog instead of creating immediately.

#### Scenario: Merge is reachable from the Layer menu and the panel menu

- **WHEN** the `Layer` menu or the panel menu is opened with a document active
- **THEN** Merge Down / Merge Layers, Merge Visible, Merge Clipping Mask, and
  Flatten Image are present and wired to the bridge

#### Scenario: A selection-dependent command is enabled correctly

- **WHEN** one layer is selected and the Layer menu is opened
- **THEN** the merge command is enabled for Merge Down, and with multiple
  selected layers it is enabled for Merge Layers

#### Scenario: Alt-click opens the dialog

- **WHEN** the user Alt-clicks the New Layer panel button
- **THEN** the New Layer/New Group dialog opens instead of a layer being created
  immediately

