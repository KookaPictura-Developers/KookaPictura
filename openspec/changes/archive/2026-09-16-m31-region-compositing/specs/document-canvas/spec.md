## ADDED Requirements

### Requirement: Dirty-region canvas refresh

The canvas SHALL cache the composited document as a full-document image and,
when a mutation reports a dirty rectangle, SHALL recomposite only that rectangle
through the active backend and blit it into the cached image at the rectangle's
origin, then emit `changed`. A Move commit SHALL invalidate
`old_layer_rect ∪ new_layer_rect`; a paint dab SHALL invalidate the dab's
bounding box. A canvas updated only through such region refreshes SHALL be
byte-identical to a full recomposite after any sequence of those mutations. A
mutation that does not report a dirty rectangle SHALL keep the full recomposite.

#### Scenario: After several Move commits the cached canvas equals a full recomposite

- **WHEN** several Move commits are applied to a document, with GPU compute
  enabled or falling back to the CPU oracle
- **THEN** the cached canvas is byte-identical to a full recomposite of the final
  document

#### Scenario: A region update does not disturb pixels outside the region

- **WHEN** a dirty rectangle is refreshed through the active backend
- **THEN** every pixel outside the rectangle keeps its previous value and every
  pixel inside the rectangle equals the corresponding pixel of a full recomposite
