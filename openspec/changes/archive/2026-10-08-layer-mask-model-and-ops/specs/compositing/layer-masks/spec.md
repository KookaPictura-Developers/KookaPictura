# Spec Delta

## Purpose

Authoring of a raster layer mask: creating, deleting, applying, enabling, and
linking one, plus the PSD link-flag semantics the Layers panel needs.

## ADDED Requirements

### Requirement: Layer mask creation

The system SHALL create a raster layer mask for a target layer from a
`LayerMaskKind` of Reveal All, Hide All, Reveal Selection, Hide Selection, or
From Transparency. A mask created for Reveal All, Hide All, or From Transparency
SHALL be sized to the layer's content rectangle; a selection-driven mask SHALL be
sized to the supplied selection's rectangle. The reveal variants SHALL produce
maximal coverage (255 inside the mask rectangle) and the hide variants SHALL
produce minimal coverage (0 inside it), with the region outside the mask
rectangle contributing the mask's default color so the result is reveal-everything
or hide-everything respectively. Creation SHALL be refused, leaving the document
and history unchanged, when the target layer already has a mask or when the path
does not resolve. A refused creation SHALL report failure and a created mask
SHALL report success.

#### Scenario: Reveal All creates a full mask

- **WHEN** Reveal All is applied to a pixel layer that has no mask
- **THEN** the layer gains one mask sized to the layer's content rectangle whose
  samples are 255 and whose outside-rectangle default color is 255, and the
  composite is unchanged

#### Scenario: Hide All creates an empty mask

- **WHEN** Hide All is applied to a pixel layer that has no mask
- **THEN** the layer gains one mask whose samples and outside-rectangle default
  color are 0, and the layer contributes nothing to the composite

#### Scenario: A second mask is refused

- **WHEN** a layer-mask operation is applied to a layer that already has a mask
- **THEN** the operation is refused, the existing mask is unchanged, and no
  history state is added

#### Scenario: An unknown path is refused

- **WHEN** a layer-mask operation targets a path that does not resolve
- **THEN** the operation is refused and the document is unchanged

### Requirement: Selection-driven mask variants

Reveal Selection and Hide Selection SHALL derive their coverage from a selection
coverage value supplied by the caller, which the engine SHALL accept without
depending on any application type. Reveal Selection SHALL use the selection
coverage directly, so selected pixels are revealed; Hide Selection SHALL use the
inverse, so selected pixels are hidden. When no selection coverage is supplied to
a selection-driven variant, the operation SHALL be refused and the document left
unchanged.

#### Scenario: Reveal Selection keeps only the selected pixels

- **WHEN** Reveal Selection runs with a selection covering part of the layer
- **THEN** the mask reveals the selected pixels and hides the unselected pixels,
  and the composite shows the layer only where the selection covered it

#### Scenario: Hide Selection removes the selected pixels

- **WHEN** Hide Selection runs with a selection covering part of the layer
- **THEN** the mask hides the selected pixels and reveals the unselected pixels,
  and the composite omits the layer where the selection covered it

#### Scenario: A selection variant without a selection is refused

- **WHEN** a selection-driven variant runs with no selection coverage supplied
- **THEN** the operation is refused and no mask is created

### Requirement: Mask from transparency

From Transparency SHALL create a mask whose coverage is derived from the target
layer's alpha: fully revealed where the layer is opaque and fully hidden where the
layer is transparent. A layer with no alpha channel SHALL be treated as fully
opaque.

#### Scenario: Coverage follows the alpha channel

- **WHEN** From Transparency runs on a layer with transparent and opaque pixels
- **THEN** the mask is hidden at the transparent pixels and revealed at the
  opaque pixels, and the composite matches the layer's own alpha

### Requirement: Mask deletion

The system SHALL delete the raster layer mask of a target layer, reporting
success when a mask was removed and failure when the layer had no mask or the
path does not resolve. Deletion SHALL leave the layer's pixels unchanged and the
composite SHALL return to the unmasked result.

#### Scenario: Deleting restores the unmasked composite

- **WHEN** a mask that hides part of a layer is deleted
- **THEN** the layer has no mask and the composite reveals the previously hidden
  pixels

#### Scenario: Deleting without a mask reports failure

- **WHEN** deletion targets a layer that has no mask
- **THEN** it reports failure and the document is unchanged

### Requirement: Applying a mask permanently

Applying a mask SHALL fold the mask coverage permanently into the layer's pixel
alpha, multiply the layer's existing alpha by the mask's coverage at each pixel,
and then clear the layer's mask. The composite after applying SHALL match the
composite before applying within the compositor's one-unit-per-channel tolerance.
Applying SHALL be refused on a smart-object layer, since CS6 cannot apply a mask
permanently to a smart object; a refused apply SHALL leave the mask and the pixels
unchanged.

#### Scenario: Applying folds coverage into alpha and clears the mask

- **WHEN** a mask that hides part of a layer is applied
- **THEN** the layer has no mask, its alpha is the previous alpha multiplied by
  the mask coverage, and the composite is unchanged within tolerance

#### Scenario: Applying a disabled mask does not change pixels

- **WHEN** a disabled mask is applied
- **THEN** the layer's alpha is unchanged and the mask is cleared

#### Scenario: Applying to a smart object is refused

- **WHEN** a mask on a smart-object layer is applied
- **THEN** the operation is refused and both the mask and the layer pixels remain
  unchanged

### Requirement: Enabling and disabling a mask

The system SHALL set a mask's enabled state. Enabling SHALL clear the mask's
disabled state and disabling SHALL set it. A disabled mask SHALL have no effect on
the composite, and setting the state to its current value SHALL be reported as no
change.

#### Scenario: Disabling reveals the whole layer

- **WHEN** a mask that hides part of a layer is disabled
- **THEN** the composite reveals the whole layer and the mask is retained but
  marked disabled

#### Scenario: Re-enabling re-applies the mask

- **WHEN** a disabled mask is re-enabled
- **THEN** the composite hides the same pixels it hid before being disabled

### Requirement: Mask link flag

The system SHALL expose the PSD layer-mask "position relative to layer" flag as
the mask's linked state. The link flag SHALL be bit 0 of the mask's flags byte;
setting the linked state SHALL set or clear that bit while preserving every other
bit, so the raw flags byte keeps round-tripping through PSD save and read. The
linked state SHALL default to linked for a newly created mask, matching CS6.

#### Scenario: Linking sets only bit 0

- **WHEN** a mask whose flags byte has other bits set is linked
- **THEN** bit 0 becomes set, every other bit is preserved, and the mask reads as
  linked

#### Scenario: Unlinking clears only bit 0

- **WHEN** a linked mask is unlinked
- **THEN** bit 0 is cleared, every other bit is preserved, and the mask reads as
  unlinked

#### Scenario: A new mask is linked

- **WHEN** a mask is created
- **THEN** it reads as linked

### Requirement: Mask read predicates

The system SHALL report whether a target layer currently has a raster layer mask
and whether that mask is linked, without mutating the document. A path that does
not resolve, or a layer with no mask, SHALL report no mask and unlinked.

#### Scenario: Reads reflect the model

- **WHEN** a mask is present and linked on a layer
- **THEN** the presence and link reads report present and linked, and a layer
  without a mask reports absent and unlinked

### Requirement: Qt bridge for layer-mask operations

The application bridge SHALL expose the mask operations over the active layer,
taking the operation kind as a string for creation and the selection coverage from
the current selection. Each mutation that changes the document SHALL recomposite
and record exactly one history state; a mutation that changes nothing SHALL record
no state. When there is no document or no single active layer, every operation
SHALL report failure and leave the document unchanged.

#### Scenario: A bridge mutation is one undo step

- **WHEN** the active layer receives a mask through the bridge
- **THEN** the canvas is recomposited and exactly one history state is recorded

#### Scenario: A no-op bridge call records nothing

- **WHEN** a bridge mask operation is refused because the layer already has a mask
  or no active layer exists
- **THEN** no history state is recorded and the document is unchanged
