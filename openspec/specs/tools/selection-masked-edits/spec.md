# selection-masked-edits Specification

## Purpose
PictureView selection state that masks an adjustment layer, with UI controls and a self-test.
## Requirements
### Requirement: PictureView selection state

`PictureView` MUST expose `select_all()`, `deselect()`, and
`magic_wand(x, y, tolerance)`. `select_all()` MUST make the active selection
cover the whole document, `deselect()` MUST clear it, and the active selection
MUST be queryable through `has_selection()` and `selection_count()` (the number
of pixels with non-zero coverage). `magic_wand` MUST flood-select the contiguous
region around `(x, y)` whose pixels are within `tolerance` of the seed, return
false without a document or when the point is out of bounds, and MUST clamp the
tolerance to `0..=255`.

#### Scenario: Select all covers the document
- **WHEN** `select_all()` is called with a document loaded
- **THEN** `has_selection()` is true and `selection_count()` equals the document area

#### Scenario: Deselect clears the selection
- **WHEN** `deselect()` is called
- **THEN** `has_selection()` is false and `selection_count()` is 0

#### Scenario: Magic wand selects a region
- **WHEN** `magic_wand` is called at an in-bounds point within tolerance
- **THEN** it returns true and the selection covers a non-empty proper subset of the document

#### Scenario: Magic wand rejects out-of-bounds and missing document
- **WHEN** `magic_wand` is called at a negative coordinate or with no document
- **THEN** it returns false and the selection is unchanged

### Requirement: Selection masks an adjustment layer

While a selection is active, `add_adjustment(kind)` MUST create the adjustment
layer with a raster layer mask built from the selection coverage, so the effect
is confined to selected pixels and unselected pixels are unchanged. With no
selection, `add_adjustment` MUST create a full-frame effect. `add_adjustment`
MUST return false for an unknown kind or with no document.

#### Scenario: Effect is confined to the selection
- **WHEN** an adjustment is added while a selection covers part of the document
- **THEN** the masked pixels change and the unselected pixels do not

#### Scenario: No selection applies full-frame
- **WHEN** an adjustment is added with no active selection
- **THEN** the effect applies to the whole document

#### Scenario: Unknown kind is refused
- **WHEN** `add_adjustment` is called with an unrecognised kind
- **THEN** it returns false and no layer is added

### Requirement: Selection UI controls

The app UI MUST provide buttons for Select all, Magic wand (center), and
Deselect. Select all MUST call `select_all()`, Magic wand (center) MUST call
`magic_wand` at the center of the current image, and Deselect MUST call
`deselect()`. The UI MUST display the current selection pixel count and refresh
after any selection change.

#### Scenario: Buttons drive selection state
- **WHEN** the Select all / Magic wand (center) / Deselect buttons are clicked
- **THEN** the corresponding selection command runs and the image refreshes

### Requirement: Masked adjustment self-test

The headless `--self-test` MUST, after loading a layered PSD, wand-select one
quadrant, add an Invert adjustment, and assert that inverted pixels are confined
to that quadrant while the other quadrant is unchanged. The self-test MUST exit
non-zero on failure and clean up the added layer and selection before the
remaining checks.

#### Scenario: Quadrant-local inversion
- **WHEN** `--self-test` runs against a layered PSD
- **THEN** the wand selects a proper subset, the Invert adjustment inverts only the selected quadrant, and the run exits 0

