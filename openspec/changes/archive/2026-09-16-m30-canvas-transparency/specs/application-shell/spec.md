## ADDED Requirements

### Requirement: Canvas transparency checkerboard

The document canvas SHALL draw a two-tone checkerboard behind the document image,
clipped to the document rect, so that any pixel with alpha less than 255 reveals
it. A fully transparent document SHALL show the checkerboard, and the
checkerboard SHALL NOT be drawn outside the document rect.

#### Scenario: A transparent document reveals both checker tones

- **WHEN** a fully transparent document is displayed on the canvas
- **THEN** pixels inside the document rect show the two checkerboard tones and
  pixels outside the document rect show the canvas colour

#### Scenario: Opaque content covers the checkerboard

- **WHEN** a document region is fully opaque
- **THEN** that region shows the document pixels and the checkerboard is not
  visible through it

### Requirement: Canvas clips content to the document bounds

The canvas SHALL clip the composited document and the live move preview to the
document rect, and content outside the canvas SHALL NOT be drawn.

#### Scenario: A layer preview dragged outside the document is cropped

- **WHEN** the move preview draws a layer whose position puts part of it outside
  the document rect
- **THEN** the part inside the document rect shows the layer and the part outside
  shows the canvas colour

### Requirement: Checkerboard is screen-space and document-anchored

The checkerboard cell size SHALL be constant in screen space, independent of
zoom, and the checkerboard SHALL be anchored to the document origin so that
panning does not move the pattern relative to the document.

#### Scenario: Zoom does not change the cell size

- **WHEN** the canvas zoom changes
- **THEN** the checkerboard cell size in screen pixels stays the same

#### Scenario: Panning does not shift the pattern

- **WHEN** the canvas is panned
- **THEN** the checkerboard stays aligned to the document origin rather than to
  the viewport
