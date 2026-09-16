## MODIFIED Requirements

### Requirement: Cursor hotspots

Each SVG cursor SHALL define an action hotspot, and the cursor art SHALL place
its action point at that hotspot. The hotspot SHALL be resolved from the frozen
tool catalogue, so each tool uses its documented action point: crosshair-style
selection, crop, path, shape, type, 3D and rotate tools at the centre
`(12,12)`; the Move tool at its compound cursor's arrowhead tip `(2,2)`;
brush-like and sample tools (Eyedropper, Color Sampler, Ruler, Note,
brush, stamp, eraser, fill, blur, toning, healing) at their lower-left tip
`(2,22)`; pen tools at their upper-left nib `(2,2)`; Hand and Zoom at the
pointing finger `(9,2)`. Every hotspot SHALL lie inside `[0, 24) × [0, 24)`.

#### Scenario: Hotspot matches the artwork

- **WHEN** a tool cursor is created
- **THEN** the hotspot is the catalogue's action point for that tool, and the art places its action point there

#### Scenario: An unknown id falls back to the centre

- **WHEN** a cursor is requested for an id with no catalogue hotspot
- **THEN** it uses the centre `(12,12)`
