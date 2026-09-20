# filter-application Specification

## Purpose
TBD - created by archiving change m6-filter-integration. Update Purpose after archive.
## Requirements
### Requirement: Destructive filter application entry point

`pictura-render` SHALL expose `pub fn apply_filter(layer: &mut pictura_core::Layer, filter: &pictura_filters::Filter, mask: Option<&pictura_core::LayerMask>) -> Result<(), pictura_filters::FilterError>`. It SHALL apply `filter` destructively to the layer's color channels and gate the write by the optional document-coordinate `mask`. When `layer.rect.width()` or `layer.rect.height()` is `<= 0` it MUST return `Ok(())` without modifying the layer. When the layer's lock state includes the pixel lock, it MUST return a refusal error without modifying the layer.

#### Scenario: A non-empty layer is filtered in place

- **WHEN** `apply_filter` is called on a layer with a non-empty `rect` and a valid filter
- **THEN** the layer's color channels are rewritten in place and `Ok(())` is returned

#### Scenario: A degenerate rect is a no-op

- **WHEN** `apply_filter` is called on a layer whose `rect` has zero or negative width or height
- **THEN** it returns `Ok(())` and the layer is bit-identical to its input

#### Scenario: A pixel-locked layer is refused [lfa_pixel_locked]

- **WHEN** `apply_filter` is called on a layer whose lock state includes the pixel lock
- **THEN** it returns a refusal error and the layer's channels are bit-identical to their input

### Requirement: Color-channel extraction and validation

The function SHALL extract the layer's color channels with ids `0`, `1`, and `2`, in that order, into a planar 8-bit `PixelBuffer` of `rect.width() × rect.height()` with 3 channels. A missing color channel, or a color channel whose `data.len()` is not `rect.width() * rect.height()`, MUST return `FilterError::InvalidParams` and MUST NOT panic. Filter-level validation errors MUST propagate to the caller unchanged, and no error path may partially write the layer.

#### Scenario: Missing color channel errors

- **WHEN** the layer has no channel with id `1`
- **THEN** `apply_filter` returns `FilterError::InvalidParams` and does not panic

#### Scenario: Wrong channel length errors

- **WHEN** a color channel's `data.len()` is not `rect.width() * rect.height()`
- **THEN** `apply_filter` returns `FilterError::InvalidParams` and the layer is unchanged

#### Scenario: Filter parameter errors propagate

- **WHEN** the filter rejects its own parameters (for example a negative Gaussian radius)
- **THEN** `apply_filter` returns that filter's `FilterError` and the layer is unchanged

### Requirement: Mask-confined write

For each local pixel `(lx, ly)`, the coverage SHALL be the mask value at document coordinates `(rect.left + lx, rect.top + ly)`, or `mask.default_color` when that point lies outside the mask rect; when `mask` is `None` the coverage SHALL be `255`. The function SHALL write `round(orig + (filtered - orig) * coverage / 255)` into color channels `0`, `1`, and `2`, where `orig` is the pre-filter sample and `filtered` the filter's output. Coverage `255` MUST reproduce the filter output, coverage `0` MUST leave the pixel unchanged, and intermediate coverage MUST interpolate.

#### Scenario: No mask filters the full frame

- **WHEN** `apply_filter` is called with `mask` `None`
- **THEN** every pixel receives the filter output as if coverage were 255

#### Scenario: Full coverage equals the filter output

- **WHEN** every pixel under the layer rect has mask coverage 255
- **THEN** the written pixels equal the filter output

#### Scenario: Zero coverage leaves the pixel unchanged

- **WHEN** a pixel under the layer rect has mask coverage 0
- **THEN** that pixel is bit-identical to its pre-filter value

#### Scenario: Partial coverage interpolates

- **WHEN** a pixel has mask coverage between 1 and 254
- **THEN** the written sample is the rounded linear interpolation of `orig` and `filtered` by `coverage / 255`

#### Scenario: Outside the mask rect uses the default color

- **WHEN** the layer rect extends beyond the mask rect
- **THEN** pixels outside the mask rect use `mask.default_color` as their coverage

### Requirement: Alpha and layer metadata preservation

The transparency channel (`id == -1`) MUST NOT be modified. `apply_filter` MUST NOT change `layer.rect`, `layer.mask`, `layer.opacity`, or `layer.blend`.

#### Scenario: Transparency is bit-identical

- **WHEN** `apply_filter` runs on a layer that carries a channel with id `-1`
- **THEN** that channel's data equals the input bit for bit

#### Scenario: Layer metadata is unchanged

- **WHEN** `apply_filter` returns success
- **THEN** `rect`, `mask`, `opacity`, and `blend` equal their pre-call values

### Requirement: Errors instead of panics on bad layer data

`apply_filter` MUST return a `pictura_filters::FilterError` — never panic — for any malformed layer color data, and MUST leave the layer unchanged on every error path.

#### Scenario: Malformed layer data returns an error without panicking

- **WHEN** `apply_filter` receives a layer with a missing or wrong-length color channel
- **THEN** it returns a `FilterError` and does not panic

#### Scenario: The layer is unchanged after an error

- **WHEN** `apply_filter` returns an error
- **THEN** every channel of the layer is bit-identical to its input

