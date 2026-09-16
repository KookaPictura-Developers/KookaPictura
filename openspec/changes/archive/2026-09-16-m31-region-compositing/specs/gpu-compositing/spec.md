## ADDED Requirements

### Requirement: Region compositing

`pictura_render::composite_region_active(doc, rect, gpu_enabled)` SHALL composite
only the requested document rectangle through the active backend and return a
rectangle-sized buffer. Its full signature is
`(doc: &Document, rect: PsdRect, gpu_enabled: bool) -> (PixelBuffer, Backend)`.
The `rect` SHALL be clamped to the document bounds, and the returned
`PixelBuffer` SHALL be sized `rect.width() × rect.height()` over the clamped
rectangle. For separable modes, non-separable modes, adjustment layers, groups,
and masked layers, the returned buffer MUST be byte-identical to the
corresponding sub-rectangle of
`composite_active(doc, gpu_enabled)`. The full-document `composite_active` SHALL
remain the oracle and MUST NOT change. A `rect` whose intersection with the
document is empty SHALL return a zero-dimension buffer and MUST NOT panic.

#### Scenario: A region matching the full canvas equals the full composite

- **WHEN** `composite_region_active` is called with a rect equal to the whole
  document rect
- **THEN** the returned buffer is byte-identical to
  `composite_active(doc, gpu_enabled)` and reports the same `Backend`

#### Scenario: A sub-rect equals the corresponding slice of the full composite

- **WHEN** a document using a separable mode, a non-separable mode, an adjustment
  layer, a group, or a masked layer is composited over a sub-rectangle smaller
  than the canvas
- **THEN** every pixel of the returned buffer equals the same pixel of
  `composite_active(doc, gpu_enabled)` at the rect offset

#### Scenario: Out-of-bounds and empty regions are clamped without panicking

- **WHEN** `composite_region_active` is called with a rect that extends outside
  the document, or that has zero or negative area
- **THEN** it clamps the rect to the document intersection and returns a buffer
  of the intersection's size, or a zero-dimension buffer when the intersection is
  empty, and does not panic
