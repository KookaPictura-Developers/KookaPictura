# layer-compositing Specification

## Purpose
TBD - created by archiving change m2-layer-compositing. Update Purpose after archive.
## Requirements
### Requirement: Composited output format

The compositor SHALL return a `PixelBuffer` with four planar channels (R, G, B, A), 8-bit, at the document's width and height, in straight (unpremultiplied) alpha.

#### Scenario: Empty stack is fully transparent

- **WHEN** `composite_rgba` is called on a document with no layers
- **THEN** every sample of the returned buffer MUST be 0

#### Scenario: Output matches document resolution and channel count

- **WHEN** the document is `W × H`
- **THEN** the returned buffer MUST be `W × H` with exactly four channels

### Requirement: Bottom-to-top stack walk

The compositor SHALL composite visible layers in array order, treating index 0 as the bottom, each blended into the running backdrop, and MUST skip any layer whose `visible` flag is false.

#### Scenario: Order determines the result

- **WHEN** layers `[A, B, C]` are composited
- **THEN** each layer MUST be blended over the accumulated composite of every preceding layer

#### Scenario: Hidden layers are skipped

- **WHEN** a layer has `visible == false`
- **THEN** it MUST NOT change any output sample

### Requirement: W3C source-over compositing equation

The compositor SHALL combine a source sample `(Cs, αs)` with a backdrop `(Cb, αb)` using the W3C *Compositing and Blending Level 1* equation `Co = ((1 − αb)·αs·Cs + αs·αb·B(Cb, Cs) + (1 − αs)·αb·Cb) / αo` with `αo = αs + αb·(1 − αs)`, evaluated in normalized `f32` with premultiplied intermediates and converted to 8-bit exactly once at write-back.

#### Scenario: Blend mode is ignored over a transparent backdrop

- **WHEN** the backdrop alpha is 0
- **THEN** the result color MUST equal the source color `Cs` regardless of the blend mode

#### Scenario: Partial source alpha mixes with the backdrop

- **WHEN** a source sample with `0 < αs < 1` is composited in `Normal` mode over an opaque backdrop
- **THEN** the result MUST be the source-over mix of `Cs` and `Cb` weighted by `αs`

#### Scenario: Output alpha is Porter-Duff source-over

- **WHEN** source alpha `αs` is composited over backdrop alpha `αb`
- **THEN** the output alpha MUST be `αs + αb·(1 − αs)`

### Requirement: Layer opacity scales source alpha

Each layer's `opacity` (0..=255) SHALL scale the source alpha before compositing; `255` is fully opaque and `0` contributes nothing.

#### Scenario: Partial opacity blends proportionally

- **WHEN** a full-alpha source layer is composited at `opacity == 128` over an opaque backdrop
- **THEN** the result MUST be the source-over mix with effective source alpha `128/255`

### Requirement: Raster mask multiplies source alpha

An enabled layer raster mask SHALL multiply the source alpha by the mask sample at the corresponding pixel. A missing mask, a disabled mask, or a sample outside the mask rectangle MUST leave alpha unchanged when the mask default color is 255; a mask sample of 0 MUST suppress the layer's contribution at that pixel.

#### Scenario: Black mask hides the layer

- **WHEN** the mask sample is 0 at a pixel
- **THEN** that pixel MUST be left unchanged by the layer

#### Scenario: White mask reveals the layer

- **WHEN** the mask sample is 255
- **THEN** the layer MUST contribute at its full pre-opacity alpha

#### Scenario: Disabled mask is ignored

- **WHEN** `mask.disabled` is true
- **THEN** the mask MUST have no effect on the composite

### Requirement: Isolated group semantics

A group whose blend mode is not `PassThrough`, or a `PassThrough` group with opacity other than 255 or a mask, SHALL composite its children into a private fully transparent buffer and then blend that buffer into the parent as a single layer using the group's blend mode, opacity, and mask.

#### Scenario: Isolated group blends as one layer

- **WHEN** a `Multiply` group containing an opaque child sits over an opaque backdrop
- **THEN** the child MUST be composited into a transparent group buffer and that buffer MUST be multiplied into the backdrop as one layer

#### Scenario: Group opacity scales the isolated result

- **WHEN** an isolated group has `opacity == 128`
- **THEN** the group buffer's alpha MUST be scaled by `128/255` before it is blended

### Requirement: Pass-through group semantics

A `PassThrough` group with opacity 255 and no mask SHALL composite its children directly onto the running backdrop so that child blend modes observe content outside the group.

#### Scenario: Pass-through equals the flattened stack

- **WHEN** the same layers are composited with and without a `PassThrough` group wrapper
- **THEN** the two output buffers MUST be identical

#### Scenario: Pass-through with opacity falls back to isolated

- **WHEN** a `PassThrough` group has opacity other than 255, or a mask
- **THEN** it MUST be composited as an isolated group with the group blend mode treated as `Normal`

### Requirement: Off-canvas layers are clipped

The compositor SHALL clip every layer to the intersection of its `PsdRect` (in canvas coordinates) with the canvas. Zero-area and fully off-canvas layers MUST contribute nothing and MUST NOT panic.

#### Scenario: Partly off-canvas layer is clipped

- **WHEN** a layer rectangle extends beyond one or more canvas edges
- **THEN** only pixels inside the canvas MUST be written and no out-of-bounds index MUST be read

#### Scenario: Fully off-canvas and zero-area layers are inert

- **WHEN** a layer rectangle does not intersect the canvas, or has zero width or height
- **THEN** the output MUST be unchanged

