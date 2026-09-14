# canvas-operations Specification

## Purpose
TBD - created by archiving change m10-image-ops. Update Purpose after archive.
## Requirements
### Requirement: Canvas resize entry point and error contract

The system SHALL provide `pictura_ops::resize_canvas(buf: &PixelBuffer, width: u32, height: u32, anchor: Anchor, background: [u8; 4]) -> Result<PixelBuffer, OpsError>`. It SHALL return a newly allocated `PixelBuffer` whose `width` and `height` equal the requested values and whose `channels` equals the input's, and it MUST leave `buf` bit-identical. `width` or `height` below 1, or an input whose `data.len()` does not equal `width * height * channels`, SHALL be rejected with `OpsError::InvalidParams`. The function MUST NOT panic for any input, including 1×1 and 1-pixel images.

#### Scenario: Canvas resize returns a new buffer and leaves the input untouched

- **WHEN** `resize_canvas` is called on a valid `PixelBuffer`
- **THEN** it returns `Ok` with a buffer of the requested canvas size, the same channel count, and the input buffer still equals its pre-call bytes

#### Scenario: Invalid dimensions are rejected

- **WHEN** `resize_canvas` is called with `width` 0 or `height` 0
- **THEN** it returns `OpsError::InvalidParams` and the input buffer is unchanged

#### Scenario: A malformed input buffer errors instead of panicking

- **WHEN** `resize_canvas` is called on a buffer whose `data.len()` is not `width * height * channels`
- **THEN** it returns `OpsError::InvalidParams` and does not panic

### Requirement: Nine-anchor placement

`Anchor` SHALL have exactly the nine variants `TopLeft`, `TopCenter`, `TopRight`,
`MiddleLeft`, `Center`, `MiddleRight`, `BottomLeft`, `BottomCenter`, and
`BottomRight`. On growth, the existing image SHALL be placed in the new canvas at
the position selected by `anchor`; on shrinkage, the output SHALL be the
rectangle of the requested size aligned to the same relative position within the
source. A `Center` anchor SHALL center the image, a top-left anchor with a
larger canvas SHALL add the new area to the right and bottom only, and a
bottom-right anchor SHALL add it to the left and top only. Placement MUST use the
same anchor math for grow and shrink.

#### Scenario: Center anchor grows symmetrically

- **WHEN** a 1000×1000 buffer is grown to 1200×1200 with `Anchor::Center`
- **THEN** the original content is centered with a 100-pixel border on all four sides

#### Scenario: Top-left anchor adds on the right and bottom

- **WHEN** a 1000×1000 buffer is grown to 1200×1200 with `Anchor::TopLeft`
- **THEN** the new space appears only to the right and below the original content

#### Scenario: Shrink crops to the anchor-aligned rectangle

- **WHEN** `resize_canvas` is called with a smaller width and height
- **THEN** the output is the rectangle of that size positioned within the source according to `anchor` and no other source pixels remain

### Requirement: Grow fill, background alpha, and shrink discard

On growth, every pixel in the added region SHALL equal `background`, whose first
three components are the color and whose 4th component is alpha; for a 4-channel
buffer the alpha channel SHALL be written from `background[3]`, and for a
3-channel buffer the 4th component SHALL be ignored. Growth MUST NOT interpolate
or blend, so the copied source pixels are bit-identical to the input and the added
region is a flat fill. On shrinkage, pixels outside the requested rectangle SHALL
be absent from the output and MUST NOT be resampled.

#### Scenario: Added pixels equal the background including alpha

- **WHEN** a 4-channel buffer is grown with a background whose alpha is 0
- **THEN** every added pixel equals the background color with alpha 0 and every copied source pixel is bit-identical to the input

#### Scenario: Shrink discards pixels outside the new rectangle

- **WHEN** a buffer is shrunk below its size
- **THEN** the output contains only pixels inside the anchor-aligned rectangle and no pixel outside it

#### Scenario: Tiny canvases do not panic

- **WHEN** `resize_canvas` grows or shrinks a 1×1 buffer
- **THEN** no call panics and each returns `Ok` or `OpsError::InvalidParams`

### Requirement: Canvas ImageMagick `-extent` oracle

The system SHALL extend `scripts/ops_oracle.py` with an ImageMagick `-extent
<W>x<H> -gravity <gravity>` operator and diff `resize_canvas` against it in
`crates/pictura-ops/tests/oracle.rs`. The mapping SHALL map each `Anchor` to its
ImageMagick gravity (`Center`, `NorthWest`, `North`, `NorthEast`, `West`, `East`,
`SouthWest`, `South`, `SouthEast`). The differential SHALL use only 3-channel
buffers, where the mapping is exact (measured max delta 0) for every anchor on
both grow and shrink; growth SHALL be diffed with the matching background color
and shrinkage as the gravity-aligned crop. A 4-channel buffer SHALL NOT be part
of the differential because ImageMagick's `-extent` composites and rounds the
background alpha differently (measured deltas 17–189); the alpha fill contract
shall instead be covered by the `src/canvas.rs` unit tests. The tests SHALL skip
with a message when `magick` is absent and MUST NOT be marked `#[ignore]`.

#### Scenario: Anchor maps to an ImageMagick gravity row

- **WHEN** the oracle mapping table is checked
- **THEN** all nine `Anchor` variants have a row naming the matching gravity and a tolerance justified by the measured delta

#### Scenario: A centered 3-channel growth matches the oracle

- **WHEN** a 3-channel buffer is grown with `Anchor::Center` and ImageMagick extends the same image with `-gravity center`
- **THEN** no sample differs by more than the recorded tolerance (measured max delta 0)

#### Scenario: Alpha fill is covered by unit tests, not the differential

- **WHEN** the oracle mapping is checked for 4-channel buffers
- **THEN** the differential is documented as 3-channel-only because ImageMagick composites and rounds the background alpha differently, and the alpha fill is asserted by the `canvas.rs` unit tests

#### Scenario: Missing ImageMagick skips cleanly

- **WHEN** `magick` is not on `PATH`
- **THEN** the differential tests print a skip message and the suite still passes

