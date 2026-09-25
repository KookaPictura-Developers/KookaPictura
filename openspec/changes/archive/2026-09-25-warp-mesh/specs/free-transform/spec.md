# free-transform Specification

## ADDED Requirements

### Requirement: Mesh warp op

The system SHALL provide
`pictura_render::transform_layer_warp(doc: &mut Document, path: &str, mesh:
&WarpMesh, params: WarpParams) -> bool`, where `WarpMesh` is a row-major control
net of `rows × cols` points in the source rect's local space and `WarpParams`
carries `distort_h`/`distort_v` percentages. It SHALL resample every channel
plane (and the layer mask, about its own rect) through the tensor-product cubic
Bézier surface defined by the control net, after scaling each row about its edge
midpoint by `1 + (2v−1)·distort_v/100` and each column likewise by `distort_h`.
A destination pixel whose source point lies outside the source rect SHALL be `0`,
and the result `rect` SHALL be the integer bounding box of the forward-mapped
lattice. The op SHALL refuse and leave `doc` bit-identical for every case
`transform_layer` refuses, and additionally when a mesh point is non-finite,
`rows` or `cols` is below 2, or the point count is not `rows·cols`. It SHALL NOT
recomposite and MUST NOT panic. A uniform control net SHALL map a point to
itself (the identity), so an identity mesh leaves the document byte-identical.
The named Adobe warp presets and their `Bend`/`X`/`Y` geometry are out of scope
(no Photoshop oracle); parity is behavioral.

#### Scenario: An identity mesh is a no-op

- **WHEN** `transform_layer_warp` is called with `identity_mesh` over the source rect and zero distortion
- **THEN** it returns true and the layer rect and every plane are unchanged

#### Scenario: An interior control point deforms the interior

- **WHEN** an interior control point of a 4×4 identity mesh is moved and the op runs
- **THEN** the layer rect grows to the deformed bounding box and the four corner pixels are unchanged

#### Scenario: Degenerate meshes are refused

- **WHEN** a mesh point is non-finite, `rows` or `cols` is below 2, or the point count does not equal `rows·cols`
- **THEN** the op returns false and every field of `doc` equals its pre-call value

#### Scenario: Out-of-source pixels are transparent

- **WHEN** the mesh pulls part of the destination outside the source rect
- **THEN** every such destination pixel has all channels equal to `0`
