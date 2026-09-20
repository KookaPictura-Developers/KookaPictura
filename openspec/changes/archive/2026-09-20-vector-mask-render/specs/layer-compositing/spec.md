## ADDED Requirements

### Requirement: Vector mask coverage multiplies source alpha

An enabled layer vector mask SHALL multiply the layer's effective source alpha by
the vector coverage sampled at the canvas pixel, in addition to the layer's
opacity, fill, and any raster mask, in both the CPU oracle and the GPU
compositor. The vector coverage and the raster coverage SHALL combine by
multiplication, so a pixel with either coverage at 0 SHALL suppress the layer's
contribution there. An absent or `disabled` vector mask SHALL leave the coverage
unchanged.

The coverage SHALL be sampled at pixel centres and evaluated in document
coordinates, because the vector mask path is document-relative. Closed subpaths
SHALL be combined under the mask's fill rule: even-odd unless a subpath declares
`non-zero`, in which case non-zero winding SHALL be used. An open subpath SHALL
contribute no coverage. When the mask's `invert` flag is set, the sampled
coverage SHALL be `255 − c`. When the path has no closed subpath, the coverage
SHALL be `255` (no clipping).

A vector mask SHALL make the GPU build its per-pixel mask plane rather than the
constant-coverage fill, so a scene with a vector mask SHALL remain within the
existing ±1 LSB CPU/GPU parity.

#### Scenario: A vector mask clips a fill layer

- **WHEN** a solid-fill layer covering the canvas carries a closed-rectangle
  vector mask over an opaque base
- **THEN** the pixels inside the rectangle carry the fill and the pixels outside
  it carry the base

#### Scenario: An inverted vector mask hides its inside

- **WHEN** a layer's vector mask has the `invert` flag set
- **THEN** the pixels inside the path are suppressed and the pixels outside it
  carry the layer's contribution

#### Scenario: A disabled vector mask is ignored

- **WHEN** a layer's vector mask has the `disable` flag set
- **THEN** the layer's contribution is unchanged by the vector mask

#### Scenario: Vector and raster masks combine

- **WHEN** a layer carries both an enabled raster mask and an enabled vector
  mask
- **THEN** the layer's source alpha is scaled by the product of the two
  coverages

#### Scenario: A vector mask with no closed subpath does not clip

- **WHEN** a layer's vector mask has no closed subpath
- **THEN** the layer renders unmasked by the vector mask

#### Scenario: The GPU agrees with the CPU with a vector mask

- **WHEN** a scene whose layer carries a vector mask is composited on the GPU
  and on the CPU
- **THEN** the two results agree within ±1 LSB per channel
