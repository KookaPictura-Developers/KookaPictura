# Spec Delta

## ADDED Requirements

### Requirement: Arrange layers within the container

The system SHALL provide the `Layer > Arrange` commands **Bring to Front**,
**Bring Forward**, **Send Backward**, and **Send to Back**. Each SHALL move the
active layer within its own container only — never into a sibling container or
a group — where the top of the stack is the front. Bring to Front SHALL move
the layer to the top of its container, Bring Forward SHALL move it one position
towards the front, Send Backward SHALL move it one position towards the back,
and Send to Back SHALL move it to the bottom of its container. A move that
would leave the layer in its current position SHALL be refused and record no
history state, and the command SHALL be disabled when no move is possible. The
Background layer and a fully-locked layer MUST NOT reorder. Every applied
arrange SHALL recomposite and record exactly one undo state.

#### Scenario: Bring to Front moves the active layer to the top

- **WHEN** a layer is selected in a container with other layers above it and
  Bring to Front runs
- **THEN** the layer is the topmost node of its container, the relative order of
  the other nodes is unchanged, and the change is one undo step

#### Scenario: Send to Back moves the active layer to the bottom

- **WHEN** a layer is selected and Send to Back runs
- **THEN** the layer is the bottom-most node of its container and the change is
  one undo step

#### Scenario: Bring Forward and Send Backward move one position

- **WHEN** Bring Forward runs on a layer with a node above it
- **THEN** the layer and that node swap positions, and Send Backward performs
  the inverse swap

#### Scenario: A boundary move is refused and disabled

- **WHEN** the active layer is already at the front and Bring Forward or Bring
  to Front is invoked
- **THEN** the command is disabled, the document is unchanged, and no history
  state is recorded

#### Scenario: The Background does not reorder

- **WHEN** the Background layer is the active layer and any Arrange command runs
- **THEN** the command is disabled and the document is unchanged

### Requirement: Reverse selected layers

The system SHALL provide `Layer > Arrange > Reverse`, which SHALL reverse the
stacking order of the selected layers. The selection MUST contain at least two
layers that share one container and occupy a contiguous run of positions in
that container; otherwise the command SHALL be refused, the document SHALL be
unchanged, and no history state SHALL be recorded. The command MUST refuse when
any selected layer is the Background or is fully locked. A successful reverse
SHALL recomposite and record exactly one undo state.

#### Scenario: A contiguous run reverses in place

- **WHEN** three adjacent layers in one container are selected and Reverse runs
- **THEN** their order within the run is reversed, the nodes before and after
  the run keep their positions, and the change is one undo step

#### Scenario: A non-contiguous selection is refused

- **WHEN** the selected layers share a container but are not contiguous
- **THEN** the command is refused, the document is unchanged, and no history
  state is recorded

#### Scenario: A cross-container selection is refused

- **WHEN** the selected layers span more than one container and Reverse runs
- **THEN** the command is refused, the document is unchanged, and no history
  state is recorded

### Requirement: Delete Layer

The system SHALL provide a `Layer > Delete Layer` command that deletes every
selected layer. The Background and fully-locked layers MUST NOT be deleted. A
selected node whose ancestor is also selected SHALL be deleted once, with its
ancestor. When at least one layer is deleted the command SHALL recomposite and
record exactly one undo state; when nothing is deletable it SHALL record no
history state. The command SHALL be disabled when no document is open or no
layer is selected.

#### Scenario: Delete removes the selected layers

- **WHEN** one or more ordinary layers are selected and Delete Layer runs
- **THEN** those layers are removed and the removal is one undo step

#### Scenario: The Background is not deletable

- **WHEN** the only selected layer is the Background and Delete Layer runs
- **THEN** the command is disabled, the document is unchanged, and no history
  state is recorded

### Requirement: Merge Down as a distinct command

The system SHALL provide a distinct `Layer > Merge Down` command that merges
the active layer with the layer directly below it within the same container,
reusing the document compositor and the Merge Down rules (raster target, a
layer must exist below, one undo state). This leaf SHALL be separate from the
`Layer > Merge Layers` command; `Ctrl+E` SHALL remain bound to Merge Layers.
Merge Down SHALL be disabled when the active layer has no layer directly below
it or either layer is an adjustment or fill-content layer.

#### Scenario: Merge Down is reachable as its own leaf

- **WHEN** the `Layer` menu is opened with a pixel layer sitting directly above
  another pixel layer
- **THEN** `Merge Down` is present, enabled, and distinct from `Merge Layers`

#### Scenario: Merge Down replaces the pair in place

- **WHEN** Merge Down runs on a pixel layer with a raster layer directly below
- **THEN** the two layers are replaced by one pixel layer at the lower layer's
  position that inherits the lower layer's name, blend mode, and opacity, and
  the change is one undo step

#### Scenario: Merge Down is disabled without a layer below

- **WHEN** the bottom-most layer of the stack is the active layer
- **THEN** Merge Down is disabled and the document is unchanged

### Requirement: Stamp Visible and Stamp Selected

The system SHALL provide `Layer > Stamp Visible` and `Layer > Stamp Selected`,
which composite the currently eye-visible (respecting ancestor visibility) or
selected layers respectively into a NEW raster pixel layer inserted directly
above the active layer. The source layers MUST be left intact — the command is
non-destructive and MUST NOT remove, merge, or hide any existing layer. Each
command SHALL refuse, without changing the document and without recording
history, when the active layer does not resolve, when no input layer is
eligible, or (for Stamp Visible) when the active layer is hidden. A successful
stamp SHALL recomposite and record exactly one undo state.

#### Scenario: Stamp Visible adds one layer and keeps the originals

- **WHEN** a document has several visible layers and Stamp Visible runs with an
  active layer
- **THEN** exactly one new raster layer is inserted directly above the active
  layer whose pixels equal the composite of the visible layers, and every
  original layer still exists and is unchanged

#### Scenario: A hidden layer is excluded and survives

- **WHEN** a hidden layer is present and Stamp Visible runs
- **THEN** the hidden layer is not part of the composite and remains in place
  unchanged

#### Scenario: Stamp Selected composites only the selection

- **WHEN** two layers are selected and Stamp Selected runs
- **THEN** one new raster layer above the active layer holds the composite of
  just those selected layers and both originals remain

#### Scenario: A hidden active layer is refused for Stamp Visible

- **WHEN** the active layer is hidden and Stamp Visible is invoked
- **THEN** the command is disabled, the document is unchanged, and no history
  state is recorded

#### Scenario: A stamp is one undo step

- **WHEN** the user undoes a stamp
- **THEN** the added layer is removed and the prior layer tree and pixels are
  restored

## MODIFIED Requirements

### Requirement: Merge Down and Merge Layers

The system SHALL provide one selection-dependent merge command **Merge Layers**
bound to `Ctrl+E`. With more than one selected layer it SHALL perform **Merge
Layers**, and with exactly one selected layer it SHALL perform **Merge Down**.
A separate `Layer > Merge Down` command SHALL expose Merge Down directly for the
active layer. The merge SHALL validate before mutating: the merge target (the
lower layer for Merge Down, the bottommost selected layer for Merge Layers)
MUST NOT be an adjustment or fill-content layer, and Merge Down MUST have a
layer directly below the selected layer. Merge Down SHALL require exactly one
selected layer and MUST be refused when there is nothing below it. A refused
merge MUST leave the document unchanged and MUST NOT add a history state.

The command SHALL composite the inputs in stacking order through the document
compositor over the union of their content rectangles, apply each input's mask
into the resulting alpha, and replace the inputs with one pixel node. Clipping
coverage is folded into the base alpha only by `Merge Clipping Mask`; the
compositor has no clipping pass yet, so Merge Down / Merge Layers / Merge
Visible share that documented gap rather than diverging. For Merge Down the
result SHALL inherit the lower layer's name, blend mode, and opacity
*(inferred)*. For Merge Layers the result SHALL occupy the topmost selected
layer's position and its blend mode and opacity SHALL be reset to `Normal` and
`255` *(inferred)*. Sibling order and parent-group references SHALL be repaired
so the result sits where the inputs were removed. Every merge SHALL recomposite
and record exactly one undo state.

#### Scenario: Merge Down composites two layers

- **WHEN** a pixel layer sits directly above another pixel layer and Merge Down
  runs
- **THEN** the two layers are replaced by one pixel layer whose pixels equal the
  compositor's composite of the pair within the compositor tolerance, whose
  alpha is the union of their nontransparent areas, and which inherits the lower
  layer's name, blend mode, and opacity

#### Scenario: Merge Layers replaces the selection at the topmost position

- **WHEN** more than one layer is selected and `Ctrl+E` runs
- **THEN** the selected layers are replaced by one pixel layer at the topmost
  selected position whose pixels equal their composite and whose blend mode and
  opacity are `Normal` and `255`

#### Scenario: The distinct Merge Down leaf is available

- **WHEN** a pixel layer sits directly above a raster layer and `Layer > Merge
  Down` is chosen
- **THEN** the pair is merged even though only one layer is selected, and the
  change is one undo step

#### Scenario: An adjustment target is refused

- **WHEN** the bottommost selected layer is an adjustment layer and merge runs
- **THEN** the merge is refused, the document is unchanged, and no history state
  is added

#### Scenario: Merge Down needs a layer below

- **WHEN** the bottom layer of the stack is the only selected layer and Merge
  Down runs
- **THEN** the merge is refused, the document is unchanged, and no history state
  is added

#### Scenario: A merge is one undo step

- **WHEN** a merge completes and the user undoes it
- **THEN** the exact pre-merge layer tree, kinds, names, masks, and pixels are
  restored
