## ADDED Requirements

### Requirement: Color samplers

Color samplers SHALL be stored on `Document::annotations` as document pixel
positions, at most four; adding a fifth SHALL be refused without change. The
Info panel SHALL list each sampler, numbered from 1 in placement order, with its
position and the composite RGB at that pixel, and SHALL refresh after edits.

#### Scenario: The fifth sampler is refused

- **WHEN** four samplers are placed and a fifth placement is attempted
- **THEN** the document still holds four samplers and no history state is added

#### Scenario: The Info panel reads a sampler

- **WHEN** a sampler is placed on a black pixel
- **THEN** the Info panel lists `#1` with its position and R 0 G 0 B 0

### Requirement: Color Sampler tool

The Color Sampler tool SHALL place a sampler at the clicked pixel as one "Color
Sampler" state, move a dragged sampler as one "Move Color Sampler" state on
release, delete a sampler on Alt-click or when dragged off the canvas as one
"Delete Color Sampler" state, and remove all samplers from the options bar's
Clear as one "Clear Color Samplers" state. The canvas SHALL show every sampler
as a numbered crosshair with any tool active.

#### Scenario: Placing, moving, and deleting samplers

- **WHEN** the `annotation_tools` self-test places, drags, Alt-clicks, drags off, and clears samplers
- **THEN** each step records its one named state and the overlay count follows

#### Scenario: Samplers survive undo and tool changes

- **WHEN** Clear is undone and the Move tool is selected
- **THEN** the samplers are back on the document and still drawn on the canvas
