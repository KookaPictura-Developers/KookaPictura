# layers-panel Specification

## Purpose
Layer rows, property editing, layer operations, thumbnails, grouping, and the Layers panel dock.

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
context menu, the `Layer` menu, and the `Ctrl+G`/`Shift+Ctrl+G` shortcuts, acting
on the active document's currently selected layers. Group Layers SHALL wrap the
selection in one new group placed at the topmost selected layer's position,
preserving the selected layers' order, and SHALL name the group `"Group N"` where
N is one more than the highest existing `Group <number>` name. Group Layers SHALL
be refused, leaving the document unchanged, when the selection is empty, contains
the Background layer, contains a fully locked layer, or spans more than one
container. Ungroup Layers SHALL splice each selected group's children into the
parent at the group's position, preserving their order, and SHALL skip a selected
layer that is not a group. Each applied operation SHALL be one undoable step. The
keyboard shortcut path SHALL use the same selection-aware operation as the menu
path; it SHALL NOT operate on only the current row.

#### Scenario: Group a layer

- **WHEN** the user runs Group Layers on a selected layer
- **THEN** the layer is wrapped in a new group that occupies the layer's position and the wrapped layer becomes its only child

#### Scenario: Group a multi-selection [m39_multi]

- **WHEN** the user runs Group Layers on several selected layers that share a
  container
- **THEN** one group is created at the topmost selected position containing all
  of them in their existing order, in one undo step

#### Scenario: Ctrl+G groups the selection [lpr_ctrl_g]

- **WHEN** several layers are selected and `Ctrl+G` is pressed
- **THEN** the whole selection is wrapped in one group in one undo step, exactly
  as the menu Group Layers command does

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
visibility toggle, the thumbnail (a folder glyph for a group), the name, a
clipping-mask indicator for a clipped layer, the clipping indentation and base
underline, the layer-mask thumbnail when a mask is present, and an
adjustment/style badge when adjustment content is present. A layer whose lock
state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A layer whose color label is not `None` SHALL tint the **visibility toggle's own
background** (`eyeRect`) with that label color behind the eye glyph, and the
delegate SHALL NOT paint a color swatch after the name; the tint SHALL keep the
eye glyph and any selection highlight legible, and the label color elsewhere on
the row SHALL fall back to the row background. A group with at least one child
SHALL show a disclosure icon — right when collapsed, down when expanded — at its
indented position, and clicking that icon SHALL expand or collapse the group. A
**regular (non-group) layer's thumbnail SHALL be drawn over a cached two-tone
checkerboard** so transparency reads under it, while a group keeps its folder
glyph and no checkerboard. **Every thumbnail SHALL carry a 1 px black outline**,
and **when exactly one layer is active** (the same singular active-layer
resolution tool edits use) its thumbnail SHALL additionally show white 1 px
corner brackets drawn one pixel outside the outline; with zero or multiple active
layers no brackets are drawn. Row typography SHALL derive from the layer: a
`background` layer's name SHALL be italic/cursive, every other name normal, and
a layer that is a member of the frame's link set or a placed/external smart
object SHALL be underlined through the new `LayerRowLinkedRole` and
`LayerRowPlacedRole` projections. The delegate SHALL report a row height of at
least **28 px** through one named constant used by both `sizeHint` and the
vertical centring math. If an expected icon asset is unavailable, the delegate
SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: The color label tints only the eye toggle [lpr_label_tint]

- **WHEN** a layer with a non-`None` color label is shown
- **THEN** the eye toggle's background is filled with that label color behind
  the eye glyph, no swatch is painted after the name, and the rest of the row
  background is unchanged

#### Scenario: An unlabeled layer has no gutter tint [lpr_label_tint_none]

- **WHEN** a layer with color label `None` is shown
- **THEN** its eye gutter uses the normal row background

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

#### Scenario: A regular thumbnail shows a checkerboard [lpr_thumb_checker]

- **WHEN** a regular (non-group) layer with translucent pixels is shown
- **THEN** its thumbnail is drawn over a two-tone checkerboard, while a group
  row keeps its folder glyph and draws no checkerboard

#### Scenario: Thumbnails are outlined and the active layer is bracketed [lpr_thumb_bracket]

- **WHEN** a regular layer row is shown with a 1 px black thumbnail outline
- **THEN** the singular active layer's thumbnail additionally shows white
  corner brackets one pixel outside the outline, and a multi- or zero-selection
  row shows no brackets

#### Scenario: Row typography follows the layer kind [lpr_row_fonts]

- **WHEN** the rows for the Background, a linked layer, and a placed smart
  object are inspected
- **THEN** the Background name is italic/cursive, ordinary names are normal, and
  the linked and placed rows are underlined through their roles

#### Scenario: The row height has a floor [lpr_row_height]

- **WHEN** a row's size hint is read
- **THEN** its height is at least 28 px from the single named row-height
  constant

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
**name/label content band**, and SHALL NOT begin editing on the visibility,
chevron, thumbnail, lock, fx, or mask regions. The content band SHALL be the
`nameRect` geometry the delegate paints, corrected so it mirrors paint exactly
(including the +4 gap after the thumbnail and the right-edge badge/mask caps)
and is floored to a non-zero minimum width, so a click on an empty label area
still resolves to the name and not to a zero-width rect. A double-click that is
not on the eye/chevron/thumbnail/lock/fx/mask controls and not on a masked or
clipped indicator SHALL be treated as a rename for a normal layer. When the
double-clicked row is the Background layer, the system SHALL convert it to a
normal layer (through the name-and-color dialog) instead of renaming; for every
other layer it SHALL invoke the layer-style affordance, which is a documented
no-op while no Layer Style dialog exists. While editing, the system SHALL commit
the edit and move to the next visible row when the user presses `Tab`, and to
the previous visible row when the user presses `Shift+Tab`. At the last or first
visible row, the commit SHALL occur without wrapping.

#### Scenario: Double-click starts editing [m39_rename]

- **WHEN** the user double-clicks a row's name/label content band
- **THEN** an editor opens on that row and the document is unchanged until the
  edit commits

#### Scenario: Double-click anywhere in the label band renames [lpr_rename_band]

- **WHEN** the user double-clicks a non-Background row anywhere in the content
  band outside the eye/chevron/thumbnail/lock/fx/mask controls, including a
  zero-width-name row when thumbnails are off
- **THEN** the inline editor opens for that row

#### Scenario: Double-click on a control does not rename [lpr_rename_name_only]

- **WHEN** the user double-clicks the eye, chevron, thumbnail, lock, fx, or mask
  control
- **THEN** no inline editor opens and the control's own action runs

#### Scenario: Double-clicking the Background converts it [lpr_background_dblclick]

- **WHEN** the user double-clicks the Background row in the content band
- **THEN** the Background is converted through the name-and-color dialog rather
  than opening an inline editor

#### Scenario: Tab commits and moves down [m39_rename]

- **WHEN** the user edits a name and presses `Tab`
- **THEN** the name changes, the edit is one undo state, and the editor moves to
  the next visible row

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
and `Gray`. For pixel and smart-object rows the row context menu SHALL also
offer `Export As…` and `Quick Export as PNG`; those two entries SHALL be absent
or disabled on group, adjustment, and type rows. Right-clicking the visibility
toggle SHALL offer show/hide this layer only and show/hide all. The bottom action
strip SHALL remain exactly the CS6 seven buttons, so Move Up/Down SHALL NOT
appear in the strip. Commands that M39 does not implement SHALL NOT be shown in
these menus.

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

#### Scenario: Pixel and smart-object rows offer export

- **WHEN** the user opens the row menu for a pixel layer or a smart-object layer
- **THEN** it contains Export As… and Quick Export as PNG

#### Scenario: Group and adjustment rows hide export

- **WHEN** the user opens the row menu for a group or an adjustment layer
- **THEN** it does not offer Export As… or Quick Export as PNG

#### Scenario: Type rows hide export

- **WHEN** the user opens the row menu for a type layer
- **THEN** it does not offer Export As… or Quick Export as PNG

### Requirement: Layer row tooltips

The system SHALL provide a tooltip for each row that includes the layer's name
and its kind, where the kind is one of pixel, group, adjustment, background,
or type.

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
or begin a rubber-band selection. The row model SHALL implement the drag-and-drop
capability virtuals — `mimeTypes()`, `canDropMimeData()`, and
`supportedDropActions()` — so the view's `canDrop()` is true and a real drop
indicator position is computed, and the view SHALL enter the dragging state on
drag enter so the indicator paints. **The model SHALL return
`Qt::ItemIsDropEnabled` for the invalid parent index**, so a top-level row is a
valid drop target surface and Qt's `AboveItem`/`BelowItem` indicator is
available for the top level as well as for nested rows; without it the drop
indicator is empty and reordering a top-level row shows nothing. **The view SHALL
set a closed-hand cursor for the duration of `startDrag`** and restore the
previous cursor when the drag ends. The system SHALL validate a candidate drop
during the drag and SHALL show a drop indicator only for a valid target, and it
SHALL reject an invalid drop without calling the bridge or changing the document.
Releasing a dragged row above or below another row SHALL reorder it at that
position, releasing it onto a group row SHALL reparent it as a child of that
group, and releasing it on the empty viewport below the last row SHALL move it to
the document root (out of any group), each as one undoable step. The system SHALL
refuse, leaving the document unchanged, a move of the Background layer, of a
fully-locked or nesting-locked layer, of a row onto itself, or of a row into its
own descendant. Dropping a dragged row on a bottom-strip button SHALL apply that
button's action to the dragged row: Delete deletes it, New Layer duplicates it
unless the dragged row is the Background — in which case New Layer converts the
Background to a normal layer in place — and New Group groups it; buttons whose
operation is not yet implemented (mask, link, fx) SHALL be inert. A sibling
reorder SHALL resolve to an above/below drop and SHALL NOT be mistaken for a drop
into the row.

#### Scenario: Dragging reorders a row [lpr_drag]

- **WHEN** a row is dragged and released above a sibling
- **THEN** the row moves to that position in one undo step and the drag does not
  select any other row

#### Scenario: A top-level sibling shows the above/below indicator [lpr_drag_top_level]

- **WHEN** a top-level row is dragged over the gap above or below a top-level
  sibling
- **THEN** the model is drop-enabled for the invalid parent, the drop indicator
  is above/below, and the reorder is accepted

#### Scenario: The drag cursor is a closed hand [lpr_drag_cursor]

- **WHEN** a row drag starts
- **THEN** the viewport cursor is a closed hand until the drag ends, then the
  previous cursor is restored

#### Scenario: A sibling reorder resolves at the drop [lpr_drag_reorder_mode]

- **WHEN** a row is dragged over the gap above or below a sibling
- **THEN** the drop indicator is set to the above/below position and the reorder
  is accepted rather than being treated as a drop into the sibling

#### Scenario: The view enters the dragging state [lpr_drag_state]

- **WHEN** a drag enters the layer tree's viewport
- **THEN** the view sets its dragging state so the drop indicator is painted, and
  an invalid target shows no indicator

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

#### Scenario: A press-move starts a drag rather than extending a selection [lpr_drag_enabled]

- **WHEN** the user presses a draggable row and moves the pointer
- **THEN** the drag pipeline starts, the selection is not extended, and no
  rubber-band selection appears

#### Scenario: Dropping on Delete deletes the dragged row [lpr_drop_button]

- **WHEN** a row is dragged onto the Delete strip button and released
- **THEN** that row is deleted in one undo step

#### Scenario: Dropping on New Layer duplicates the dragged row [lpr_drop_button]

- **WHEN** a non-Background row is dragged onto the New Layer strip button and
  released
- **THEN** the row is duplicated in one undo step

#### Scenario: Dropping the Background on New Layer converts it [lpr_drop_background_convert]

- **WHEN** the Background is dragged onto the New Layer strip button and released
- **THEN** the Background is converted to a normal layer in place in one undo
  step, rather than cloned as a locked `Background copy`

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

### Requirement: Thumbnail Ctrl-click selects layer pixels

The layer row delegate SHALL expose the thumbnail's hit rectangle, and a
`Ctrl`-click inside that rectangle SHALL select that layer's pixels: the bridge
SHALL build a document-sized selection from the layer's alpha channel (`-1`,
treating a missing alpha channel as fully opaque `255`), offset by the layer's
`rect`, and combine it with the current selection using the New mode. The click
SHALL be consumed by the thumbnail and SHALL NOT start a drag, begin rename, or
toggle visibility.

#### Scenario: Ctrl-clicking a thumbnail selects its pixels [lpr_thumbnail_select]

- **WHEN** the user `Ctrl`-clicks a layer's thumbnail
- **THEN** a document-sized selection matching that layer's alpha shape becomes
  the current selection and the layer's row stays selected

#### Scenario: A thumbnail Ctrl-click is not a drag or rename [lpr_thumbnail_consumes]

- **WHEN** the user `Ctrl`-clicks a layer's thumbnail
- **THEN** no drag starts, no editor opens, and the visibility is unchanged

### Requirement: Nesting lock button is hidden in the panel

The Layers panel SHALL hide its nesting-lock toggle button (`lockNesting_`), so
the panel exposes only the transparency, pixel, position, and all lock controls.
Hiding the button SHALL NOT change the engine: the `NESTING` flag SHALL remain
part of the layer lock state, SHALL still refuse Group Layers and Ungroup Layers
on a nesting-locked layer, SHALL still round-trip through the PSD `lspf` block,
and SHALL still be settable and readable through the bridge. A layer already
carrying the flag SHALL still report it in its row and keep its structural
parent; the button is a presentation removal, not a feature removal.

#### Scenario: The panel has no nesting lock button [lpr_nesting_hidden]

- **WHEN** the Layers panel header lock controls are shown
- **THEN** the nesting-lock button is hidden and the other lock controls remain

#### Scenario: The engine nesting rules still hold [lpr_nesting_engine]

- **WHEN** a layer reports the `NESTING` bit programmatically and Group Layers
  is attempted
- **THEN** the operation is still refused and the PSD round-trip still preserves
  the bit

### Requirement: Layer row visibility gutter and content spacing

The visibility toggle SHALL be centred horizontally inside its fixed left
gutter, with equal padding on its left and right sides, rather than being
left-anchored with all padding on one side. The delegate SHALL draw a 1 px
separator in a slightly darker grey at the gutter's right edge, between the
visibility toggle and the row content. The expand/collapse chevron slot SHALL be
reserved only for rows that are expandable, so a regular (non-group) layer's
thumbnail begins immediately after the gutter instead of leaving an empty
chevron-width gap. The eye hit-target, the label tint, and the thumbnail/name
hit-targets SHALL all mirror this layout.

#### Scenario: The eye is centred in its gutter [lpr_eye_gutter]

- **WHEN** a row is shown at any depth
- **THEN** the eye glyph's own rect has equal left and right padding inside the
  fixed visibility gutter

#### Scenario: A separator divides the gutter from the content [lpr_eye_sep]

- **WHEN** a layer row is shown
- **THEN** a 1 px darker-grey vertical separator is drawn at the gutter's right
  edge between the eye and the content

#### Scenario: A non-group thumbnail sits close to the gutter [lpr_thumb_gap]

- **WHEN** a regular non-expandable layer and a group row are shown at the same
  depth
- **THEN** the regular layer's thumbnail starts immediately after the gutter,
  while the group row still reserves the chevron slot for its disclosure icon

### Requirement: Layer thumbnail preserves the canvas aspect ratio

A layer row thumbnail SHALL be drawn at the document's aspect ratio inside the
row's thumbnail box, letterboxed rather than stretched into a square, so a wide
or tall document is not distorted. The thumbnail outline, the transparency
checkerboard, the active bracket, and the thumbnail hit-target SHALL all use the
letterboxed rect. A group's folder glyph keeps its square slot.

#### Scenario: A non-square document is not stretched [lpr_thumb_aspect]

- **WHEN** a regular layer of a document whose width and height differ is shown
- **THEN** the drawn thumbnail's aspect ratio matches the document, with the
  unused part of the row's thumbnail box left empty

### Requirement: Layer drag shows a CS6 drop indicator

While a layer drag is over a valid target, the view SHALL draw the CS6 drop
indicator itself rather than the stock style primitive: a thin blue line above
or below the target row for a sibling reorder, and a thin blue outline around a
group row for a drop-into. An invalid target SHALL continue to show no
indicator, and the closed-hand cursor and the above/below resolution SHALL be
unchanged.

#### Scenario: A sibling reorder shows a thin blue line [lpr_drop_line]

- **WHEN** a row is dragged over the gap above or below a valid sibling target
- **THEN** a thin blue line is drawn at that edge and no other indicator is shown

#### Scenario: A drop into a group outlines the group [lpr_drop_group_outline]

- **WHEN** a row is dragged over a group row that is a valid target
- **THEN** a thin blue outline is drawn around the group row

#### Scenario: An invalid target shows no indicator [lpr_drop_line_invalid]

- **WHEN** a row is dragged over its own descendant or another refused target
- **THEN** no drop indicator is drawn

### Requirement: Raised layer row height

The single named row-height constant SHALL be raised so a Medium-thumbnail row
is at least 34 px tall, giving layer rows slightly more vertical breathing room
while keeping the same constant used by both `sizeHint` and the delegate's
centring math. The existing "at least 28 px" floor remains satisfied.

#### Scenario: A Medium row is taller [lpr_row_height_raised]

- **WHEN** a row's size hint is read with the Medium thumbnail size
- **THEN** its height is at least 34 px from the one named row-height constant

### Requirement: Reparent into a group resolves post-removal indices

A layer drag SHALL be able to reparent a layer into a group and out of a group
regardless of the dragged layer's position relative to the target. The
destination parent SHALL be resolved in post-removal coordinates, so a layer that
sits above the target group is not mis-resolved after its own removal. The move
SHALL insert the layer as the last child of the group and keep every other row at
its original position; a successful move is one undoable step. All existing
refusals SHALL remain: the Background, a fully- or nesting-locked source, a drop
onto the source or into its own descendant, and an Into target that is not a
group.

#### Scenario: A layer above a group can be dropped into it [lpr_drag_into_above]

- **WHEN** the root has `[A, G]` (G a group) and A is dragged onto G
- **THEN** A becomes G's last child and the move succeeds as one undo step

#### Scenario: A layer above a group with siblings below does not mis-nest [lpr_drag_into_above_siblings]

- **WHEN** the root has `[A, G, B]` and A is dragged onto G
- **THEN** A becomes G's child, A no longer appears at the root, and B stays at
  the root in its original position

#### Scenario: A group child can be moved out of its group [lpr_drag_out]

- **WHEN** a child of a group is dragged to the empty viewport below the last row
  or above/below a root sibling
- **THEN** it is reparented to the root (or the sibling's container) in one undo
  step and no other row changes position

#### Scenario: Invalid reparents are still refused [lpr_drag_into_refused]

- **WHEN** a layer is dropped into its own descendant, onto itself, or a
  Background/locked source is dragged
- **THEN** the move is refused and the document is unchanged

### Requirement: Layers property control row layout

The Layers panel SHALL lay out the blend-mode selection and the Opacity control
in one row that fractions the available slack between them, with the blend-mode
input taking the larger share and neither control dominating the row or pushing
the other out. The row SHALL stay usable at the panel's shared minimum width.

#### Scenario: The blend and opacity controls share the row [wpx_blend_row]

- **WHEN** the Layers panel is laid out at its normal width
- **THEN** the blend-mode input and the Opacity field each keep a usable width,
  and the blend-mode input does not absorb the whole row

### Requirement: Smart Filters tree rows

When a smart-object layer carries one or more smart filters, the Layers panel SHALL show a `Smart Filters` parent row directly under that layer, with one child row per filter carrying that filter's name. The parent row SHALL expose a visibility toggle for the group enable flag and each child row SHALL expose a visibility toggle for that filter's enable flag. Toggling either SHALL update the composite, record one history state, and persist the flag to the layer's preserved smart-object descriptor so it survives a write and re-read; the synthetic rows SHALL be non-editable and non-draggable and SHALL NOT appear as selectable layers to layer operations.

#### Scenario: The parent and one child per filter appear

- **WHEN** a smart-object layer with a filterFX chain is present
- **THEN** the panel shows a `Smart Filters` parent with one child row per filter, named for each filter

#### Scenario: Toggling a filter eye updates the composite

- **WHEN** a child row's visibility toggle is flipped
- **THEN** that filter's enable flag flips, the composite updates, and one history state is recorded

#### Scenario: Toggling the group eye bypasses the chain

- **WHEN** the parent row's visibility toggle is flipped off
- **THEN** the group enable flag is cleared and the composite falls back to the unfiltered source

#### Scenario: A toggle survives a write and re-read

- **WHEN** a filtered document whose filter or group visibility was toggled is written and read back
- **THEN** the re-read smart object reports the toggled flag

#### Scenario: Synthetic rows are not real layers

- **WHEN** a smart-filter row is double-clicked to rename or dragged
- **THEN** it is not renamed or moved and no layer operation targets it

### Requirement: Layer row surface and two-column layout

Each layer row's background SHALL be one shade step lighter than the list/base
background. Each row SHALL be split into two columns — a fixed visibility-toggle
column and a content column holding the rest of the row — so a click in the
visibility column toggles visibility and never selects or highlights the row. The
selected-row highlight SHALL be a lighter grey painted directly by the row
delegate rather than the palette's blue highlight, and the delegate SHALL be the
single source of that selection colour with no scoped stylesheet selection rule
for the tree view. The content column SHALL carry a 2 px left padding so the
thumbnail and name are not flush against the visibility gutter. The visibility
toggle's label tint and its 1 px gutter separator SHALL be unchanged, and the
gutter separator SHALL stay 1 px.

#### Scenario: The row background steps lighter than the list [lpr_row_surface]

- **WHEN** a layer row is shown against the list background
- **THEN** the row background is one shade step lighter than the list background

#### Scenario: The visibility toggle cannot be highlighted [lpr_eye_no_select]

- **WHEN** a click lands in the row's visibility-toggle column
- **THEN** visibility toggles and the row is neither selected nor highlighted

#### Scenario: Selection uses a lighter grey [lpr_selection_grey]

- **WHEN** one or more layer rows are selected
- **THEN** the row delegate paints the selected-row background as a lighter grey
  rather than the blue highlight, with no separate stylesheet rule overriding it

#### Scenario: The content column has a 2 px left padding [lpr_content_padding]

- **WHEN** a layer row is shown
- **THEN** the content column begins 2 px to the right of the visibility gutter,
  with the gutter separator still 1 px
