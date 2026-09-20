# layer-management Specification

## Purpose
TBD - created by archiving change layers-panel-management. Update Purpose after archive.
## Requirements
### Requirement: Merge Down and Merge Layers

The system SHALL provide one selection-dependent merge command bound to
`Ctrl+E`. With exactly one layer selected it SHALL perform **Merge Down**, and
with more than one selected layer it SHALL perform **Merge Layers**. The merge
SHALL validate before mutating: the merge target (the lower layer for Merge
Down, the bottommost selected layer for Merge Layers) MUST NOT be an adjustment
or fill-content layer, and Merge Down MUST have a layer directly below the
selected layer. Merge Down SHALL require exactly one selected layer and MUST be
refused when there is nothing below it. A refused merge MUST leave the document
unchanged and MUST NOT add a history state.

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

### Requirement: Merge Visible

The system SHALL provide a `Merge Visible` command bound to `Shift+Ctrl+E`
whose inputs are exactly the currently eye-visible layers, including visibility
inherited from ancestor groups. Hidden layers MUST be skipped and left in place.
The command SHALL require at least one visible layer and the active layer MUST
be visible; otherwise it SHALL be refused and the document unchanged. The result
SHALL be a single pixel layer at the topmost visible input's position, and the
command SHALL recomposite and record exactly one undo state.

#### Scenario: A hidden layer survives Merge Visible

- **WHEN** layer A is hidden, layer B is visible, and Merge Visible runs with B
  selected
- **THEN** B's content is merged into one pixel layer and hidden A remains in
  place unchanged

#### Scenario: A hidden selection is refused

- **WHEN** the active layer is hidden and Merge Visible is chosen
- **THEN** the command is unavailable and no layer changes

### Requirement: Merge Clipping Mask

The system SHALL provide a `Merge Clipping Mask` command that collapses a
clipping group into its raster base layer. The base layer MUST be a raster
(non-adjustment, non-fill-content) layer; when the base is not raster the
command SHALL be disabled and MUST refuse without changing the document. The
collapsed result SHALL occupy the base layer's position and SHALL have the
clipped layers' coverage applied into its alpha, and the command SHALL
recomposite and record exactly one undo state.

#### Scenario: A clipping group collapses into its base

- **WHEN** a raster base has one or more clipped layers above it and Merge
  Clipping Mask runs
- **THEN** the base and its clipped layers are replaced by one raster layer at
  the base's position whose alpha reflects the clipping coverage

#### Scenario: A non-raster base is refused

- **WHEN** the clipping base is an adjustment or fill-content layer and Merge
  Clipping Mask runs
- **THEN** the command is disabled, the document is unchanged, and no history
  state is added

### Requirement: Flatten Image

The system SHALL provide a `Flatten Image` command. It SHALL discard every
hidden layer without a confirmation prompt, composite all visible layers over an
opaque **white** backdrop the size of the document, and replace the entire layer
tree with a single **Background** node covering the full document with no
transparent pixels and no alpha below full opacity. Groups, clipping, masks, and
vector data are removed by flattening. The command SHALL recomposite and record
exactly one undo state that restores the prior tree and pixels.

#### Scenario: Hidden layers are discarded and transparency becomes white

- **WHEN** a document containing hidden layers and transparent areas is
  flattened
- **THEN** exactly one Background layer remains, every pixel is opaque, and the
  areas that were transparent are white

#### Scenario: Flatten removes structure

- **WHEN** a document with a group, a clipping mask, and a layer mask is
  flattened
- **THEN** the tree is a single Background layer and the structural containers
  no longer exist

#### Scenario: Flatten is one undo step

- **WHEN** the user undoes a flatten
- **THEN** the complete prior layer tree and pixels are restored

### Requirement: New Layer and New Group dialog

The system SHALL open a modal dialog when the user `Alt`-clicks the New
Layer/New Group panel button or chooses `Layer > New > Layer…` /
`Layer > New > Group…`. The dialog SHALL collect a Name, a Color label, a blend
Mode, an Opacity, a **Fill With Mode-Neutral Color** option, and a **Use
Previous Layer to Create Clipping Mask** option, and the clipping option MUST
NOT be offered for a group. On accept the system SHALL create the layer or group
with the chosen attributes directly above the selected layer (or at the top of
the stack with no selection) as one undoable step. The mode-neutral fill SHALL
be chosen from a lookup by blend mode: white for Darken, Multiply, Color Burn,
Linear Burn, Darker Color, and Divide; black for Lighten, Screen, Color Dodge,
Linear Dodge (Add), Lighter Color, Difference, Exclusion, and Subtract; 50 %
gray for Overlay, Soft Light, Hard Light, Vivid Light, Linear Light, and Pin
Light; transparent for Normal, Dissolve, Hard Mix, Hue, Saturation, Color, and
Luminosity; and the documented default for any unlisted mode SHALL be
transparent *(inferred)*.

#### Scenario: The dialog creates a configured layer

- **WHEN** the user Alt-clicks New Layer, enters a name, sets Multiply, 50 %
  opacity, and enables the neutral fill, and accepts
- **THEN** a pixel layer with that name, blend mode, and opacity is inserted
  above the selected layer filled with white

#### Scenario: The neutral color follows the mode

- **WHEN** the neutral fill is enabled with Screen selected
- **THEN** the created layer is filled black

#### Scenario: Clipping is not offered for a group

- **WHEN** the New Group dialog is opened
- **THEN** it shows no Use Previous Layer to Create Clipping Mask option and the
  created group is not clipped

#### Scenario: An unlisted mode defaults to transparent

- **WHEN** the neutral fill is enabled with a blend mode outside the documented
  lookup
- **THEN** the created layer is fully transparent

### Requirement: First-class Background layer

`pictura_core::Layer` SHALL carry an explicit background flag. The codec SHALL
derive the flag on read — the bottom top-level, non-group layer named
`Background` is the Background — and SHALL write the `Background` name for a
flagged layer on save. The `is_background` check SHALL use the flag and MUST NOT
depend on the layer's index or name. `Layer from Background…` SHALL clear the
flag and unlock the layer, and `Background From Layer` SHALL set the flag,
convert transparent pixels to the background color, move the node to the bottom
of the stack, and refuse a group or a layer that is already the Background. The
conversion from a Background SHALL also be reachable from the Layers panel: a
double-click on the Background row outside its name, and dropping the Background
on the New Layer button, SHALL each run `Layer from Background…` in place rather
than cloning a locked `Background copy`. Each conversion SHALL recomposite and
record exactly one undo state.

#### Scenario: The flag round-trips through PSD

- **WHEN** a document whose bottom layer is the Background is saved and read
  back
- **THEN** the re-read bottom layer reports the background flag

#### Scenario: The flag is independent of position and name

- **WHEN** a non-bottom layer named `Background` is present and a flagged layer
  sits at the bottom under another name
- **THEN** only the flagged layer is reported as the Background

#### Scenario: Convert from Background

- **WHEN** `Layer from Background…` runs on the Background layer
- **THEN** the flag is cleared, the layer is no longer locked, and the change is
  one undo step

#### Scenario: A double-click converts the Background [lmb_background_dblclick]

- **WHEN** the user double-clicks the Background row outside its name region
- **THEN** the Background is converted to a normal, unlocked layer in one undo
  step

#### Scenario: Dropping the Background on New Layer converts it [lmb_background_drop]

- **WHEN** the Background row is dropped on the New Layer strip button
- **THEN** the Background is converted in place to a normal layer, no
  `Background copy` is created, and the change is one undo step

#### Scenario: Convert to Background

- **WHEN** `Background From Layer` runs on a normal pixel layer
- **THEN** the layer becomes the flagged Background at the bottom of the stack,
  its transparent pixels take the background color, and the change is one undo
  step

### Requirement: Layer via Copy and Layer via Cut

The system SHALL provide `Layer via Copy` (`Ctrl+J`) and `Layer via Cut`
(`Shift+Ctrl+J`). Each SHALL require an active selection; with no selection the
command SHALL be refused and the document unchanged. `Layer via Copy` SHALL
create a new pixel layer directly above the active layer containing only the
pixels inside the active selection. `Layer via Cut` SHALL do the same and SHALL
additionally clear those pixels from the source layer. Each SHALL preserve
every pixel outside the selection and SHALL recomposite and record exactly one
undo state.

#### Scenario: Layer via Copy extracts the selection

- **WHEN** a raster layer has an active selection and Layer via Copy runs
- **THEN** a new layer above it contains only the selected pixels and the source
  layer is unchanged

#### Scenario: Layer via Cut clears the source

- **WHEN** a raster layer has an active selection and Layer via Cut runs
- **THEN** a new layer above it contains the selected pixels and those pixels
  are cleared to transparency in the source layer

#### Scenario: No selection is refused

- **WHEN** Layer via Copy or Cut is invoked with no active selection
- **THEN** the command is unavailable, no layer is added, and the document is
  unchanged

### Requirement: Select Similar and Select Linked Layers

The system SHALL provide `Select Similar` and `Select Linked Layers`. `Select
Similar` SHALL select every layer that matches the active layer's kind and
attributes. `Select Linked Layers` SHALL select every member of the active
layer's link set. `Link Layers` SHALL add the selected layers to a link set,
joining existing sets when members already belong to one, and `Unlink Layers`
SHALL remove the selected layers from their sets and drop an emptied set. All
four SHALL be view/selection operations that do not change layer pixels and do
not add a history state. Link sets SHALL be transient session state: they SHALL
NOT be serialized, and a structural layer operation SHALL clear them (documented
limitation; the PSD link-set representation is unsourced).

#### Scenario: Select Similar matches the active layer

- **WHEN** Select Similar runs on a pixel layer
- **THEN** every layer of the same kind and matching attributes is selected and
  no pixels change

#### Scenario: Link and Select Linked

- **WHEN** two layers are linked and Select Linked Layers then runs on one of
  them
- **THEN** both members of the set are selected

#### Scenario: Unlink removes membership

- **WHEN** Unlink Layers runs on a linked layer
- **THEN** that layer is no longer in the set and linking/selecting no longer
  includes it

#### Scenario: Link sets are session-only

- **WHEN** a document with linked layers is saved and reopened, or a structural
  layer operation runs
- **THEN** the link sets are not persisted and are cleared by the structural
  operation

### Requirement: Delete Hidden Layers and Hide Layers

The system SHALL provide `Delete Hidden Layers`, which SHALL remove every layer
whose effective visibility is off, and `Hide Layers`, which SHALL set the
selected layers hidden. Each SHALL recomposite and record exactly one undo
state; `Delete Hidden Layers` on a document with no hidden layers SHALL record
no history state.

#### Scenario: Delete Hidden Layers removes hidden layers

- **WHEN** a document with hidden and visible layers runs Delete Hidden Layers
- **THEN** every hidden layer is removed, the visible layers remain, and the
  removal is one undo step

#### Scenario: Hide Layers hides the selection

- **WHEN** Hide Layers runs on one or more selected layers
- **THEN** each selected layer becomes hidden and the change is one undo step

### Requirement: Rasterize subset

The system SHALL provide a `Rasterize` submenu. `Fill Content` SHALL be
implemented: for a fill-content layer (a layer whose opaque adjustment block is
a fill-content key — `SoCo`, `GdFl`, or `PtFl` — and whose payload decodes to
`Adjustment::SolidFill`, `Adjustment::GradientFill`, or
`Adjustment::PatternFill`) it SHALL render the fill content to a full-layer
pixel node and clear the fill/adjustment data. A solid fill SHALL bake its
decoded colour across the layer rect; a gradient fill SHALL bake the generated
five-kind ramp with opaque alpha across the layer rect; a pattern fill SHALL
bake the pattern the document's `Patt` resource tiles over the layer rect (with
the pattern's own alpha, or the grey placeholder when the referenced pattern is
absent or was skipped as non-8-bit/malformed). `Rasterize Layer` SHALL rasterize the active layer only when
it is a fill-content layer, and SHALL otherwise refuse without changing the
document. `Rasterize All Layers` SHALL rasterize every fill-content layer in the
document. The `Type`, `Shape`, `Vector Mask`, `Smart Object`, `Video`, and `3D`
variants SHALL remain visible and disabled, with a documented reason that those
layer kinds do not exist in the model. A rasterization whose content is not
decodable SHALL be disabled and SHALL refuse without changing the document, and
every applied rasterization SHALL recomposite and record exactly one undo state.

#### Scenario: Fill Content becomes pixels

- **WHEN** Rasterize Fill Content runs on a solid fill-content layer
- **THEN** the layer becomes a pixel layer holding the rendered fill, its fill /
  adjustment data is cleared, and the change is one undo step

#### Scenario: A descriptor-form fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `SoCo` payload is the
  standard Photoshop descriptor
- **THEN** the layer becomes a pixel layer holding the descriptor's colour and
  the change is one undo step

#### Scenario: A gradient fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `GdFl` payload decodes to
  `Adjustment::GradientFill`
- **THEN** the layer becomes a pixel layer holding the generated gradient over
  its rect, its fill / adjustment data is cleared, and the change is one undo
  step

#### Scenario: A pattern fill is rasterizable

- **WHEN** Rasterize Fill Content runs on a layer whose `PtFl` payload decodes to
  `Adjustment::PatternFill`
- **THEN** the layer becomes a pixel layer holding the tiled pattern over its
  rect, its fill / adjustment data is cleared, and the change is one undo step

#### Scenario: A pattern fill with a missing pattern is still rasterizable

- **WHEN** Rasterize Fill Content runs on a `PtFl` layer whose `pattern_id` is
  not present in the document's `Patt` resource
- **THEN** the layer becomes a pixel layer holding the grey placeholder and
  the change is one undo step

#### Scenario: A kind-less variant stays disabled

- **WHEN** the Rasterize submenu is shown
- **THEN** Type, Shape, Vector Mask, Smart Object, Video, and 3D are visible but
  disabled, and their reason is documented

#### Scenario: Rasterize Layer refuses a non-fill layer

- **WHEN** Rasterize Layer runs on a group, adjustment, or plain pixel layer
- **THEN** the document is unchanged and no undo state is recorded

### Requirement: Management commands recomposite and record one state

Every management command in this capability SHALL run through the bridge, SHALL
recompute the document composite before recording, and SHALL add exactly one
undo state per invocation. A command that changes no layer SHALL record no
history state and emit no document change.

#### Scenario: A no-op records no history

- **WHEN** a management command is invoked and its validation finds nothing to
  change
- **THEN** no history state is added and the document is unchanged

#### Scenario: One state per command

- **WHEN** a management command completes
- **THEN** exactly one undo state is recorded and one undo restores the prior
  tree and pixels

