## ADDED Requirements

### Requirement: Mask edit target

The bridge SHALL expose a per-view mask edit target naming the layer whose raster
mask is the current edit target, empty when no mask is active. Setting the target
SHALL require a path that resolves to a layer carrying a raster mask and SHALL
otherwise refuse; clearing SHALL always succeed. The target SHALL be independent
of the document's pixels, SHALL record no history, and SHALL read back empty once
the layer or its mask no longer resolves. `pictura_render` SHALL expose
`mask_document(doc, path)` returning the mask as a mask-sized grayscale document
(its coverage replicated across the layer's colour channels, opacity locked) and
`write_mask_back(doc, path, edited, region)` copying the edited document's colour
plane into the real mask over `region`, materialising a data-less mask from its
default colour first. A missing mask, a zero-sized mask rectangle, or a malformed
edited document SHALL refuse both.

#### Scenario: Activating and clearing the target

- **WHEN** the target is set to a layer with a raster mask and then cleared
- **THEN** the first call reports success and the target reads back that path, and
  the second leaves the target empty; a path without a mask refuses and records no
  state

#### Scenario: A mask round-trips through the adapter

- **WHEN** `mask_document` builds the document for a mask whose coverage is a
  left-black/right-white step, the document's layer colour plane is edited, and
  `write_mask_back` runs
- **THEN** the real mask's coverage carries the edited plane and the layer's
  pixels are untouched

#### Scenario: A deleted mask empties the target

- **WHEN** the target names a layer whose mask is then deleted
- **THEN** the target reads back empty

### Requirement: Painting, filling, and filtering the active mask

With a mask edit target active, the Brush (and Pencil) SHALL paint the mask's
coverage instead of the layer's pixels: the foreground colour's Rec.601 luma is
the painted value, so a black foreground hides the layer, white reveals it, and
gray is partial; opacity, flow, hardness, spacing, and the tip shape SHALL behave
as for layer painting. `Edit ▸ Fill` SHALL fill the mask's coverage with the fill
colour's luma through the selection, and a destructive filter SHALL filter the
coverage. Each operation SHALL record exactly one history state with its normal
label ("Brush" / "Pencil", "Fill", "Filter") and SHALL leave the layer's pixels
unchanged. Without an active mask target the three operations SHALL keep editing
the layer's pixels exactly as before. A mask stroke or fill SHALL be refused when
the mask rectangle is empty.

#### Scenario: Painting white and black on the mask

- **WHEN** the mask target is active and the Brush paints a white foreground over
  a masked pixel and a black foreground over a revealed pixel
- **THEN** the first pixel's coverage rises toward 255, the second falls toward 0,
  the composite follows (revealing and hiding the layer), and one "Brush" state is
  recorded

#### Scenario: Filling the mask

- **WHEN** the mask target is active and `Edit ▸ Fill` runs with a black
  foreground
- **THEN** the selection's coverage becomes the black luma and one "Fill" state is
  recorded, while the layer's pixels are unchanged

#### Scenario: Filtering the mask

- **WHEN** the mask target is active and a destructive filter commits
- **THEN** the mask's coverage is filtered over the mask rectangle and one
  "Filter" state is recorded; a preview cancelled before commit restores the
  original coverage bit-identically

#### Scenario: No target keeps layer editing

- **WHEN** no mask edit target is set and the Brush paints
- **THEN** the layer's pixels change and the mask is untouched
