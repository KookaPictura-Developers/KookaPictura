## MODIFIED Requirements

### Requirement: Open imports supported raster images as a new document

The application SHALL expose `PictureView::open_image(path: &QString) -> bool`.
On success it SHALL build a document through
`pictura_core::Document::from_rgba` using the file's base name, replace the view's
document and displayed image with it, reset the edit state, capture exactly one
history state labelled `"Open"`, leave the view's path empty so the document is
untitled, and mark the document unmodified. It SHALL record a display name on the
view so the tab title reads `<file name> (Mode/Bits)`, where the file name
includes the file extension, rather than a generated `Untitled-N` name. It SHALL
select the imported layer as the view's active layer. A fully opaque decoded image
SHALL become exactly one layer named `Background` with `background = true`, a lock
state of `TRANSPARENCY | POSITION`, and the redundant opaque transparency channel
dropped; an image with any non-opaque pixel SHALL remain a regular alpha layer
named from the file stem. It SHALL return `false` without mutating the view on any
refusal. `File > Open` SHALL offer an `Images (…)` filter beside the existing
`Photoshop files (*.psd *.psb)` filter. A PSD/PSB file SHALL continue through the
native `read_psd` path and MUST NOT be routed through Qt.

#### Scenario: A supported image opens as an untitled document

- **WHEN** `open_image` is called on a supported raster file
- **THEN** it returns true, the view holds a document sized to the image with the imported pixels, exactly one `"Open"` history state is added, and the view's path is empty

#### Scenario: The imported tab keeps a display name [lim_display_name]

- **WHEN** `open_image` succeeds for a file such as `image01.jpg`
- **THEN** the view's display name is the file name including its extension and
  its tab title reads `image01.jpg (Mode/Bits)` rather than `Untitled-N`

#### Scenario: An opaque image becomes the locked Background [lim_opaque_background]

- **WHEN** `open_image` decodes a file every pixel of which is fully opaque
- **THEN** the document has exactly one layer named `Background` with the
  background flag set and a lock state of transparency and position only, and no
  separate transparency channel is required

#### Scenario: The imported layer becomes active [lim_activate]

- **WHEN** `open_image` succeeds
- **THEN** the imported layer is the view's active layer and is the single layer a
  subsequent tool edit targets

#### Scenario: A non-opaque image stays a regular alpha layer [lim_transparent_layer]

- **WHEN** `open_image` decodes a file with at least one non-opaque pixel
- **THEN** the document has one regular alpha layer named from the file stem and
  is not a locked Background

#### Scenario: A refusal mutates nothing

- **WHEN** `open_image` is called with a missing, undecodable, or over-budget file
- **THEN** it returns false, the view's document is unchanged, and no history state is added

#### Scenario: The dialog accepts images and keeps PSD native

- **WHEN** `File > Open` is invoked
- **THEN** the dialog offers an `Images (…)` filter and a Photoshop filter, and a chosen `*.psd`/`*.psb` file is opened through `read_psd` rather than Qt
