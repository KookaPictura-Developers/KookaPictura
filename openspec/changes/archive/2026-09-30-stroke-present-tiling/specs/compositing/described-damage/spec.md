# Spec Delta

## ADDED Requirements

### Requirement: A paint commit composites its dirty tiles

A paint stroke's commit SHALL track its dirty area on the 64×64 tile grid and
composite the area as disjoint rectangles rather than the stroke's bounding
box. It SHALL fall back to the bounding box when the area collapses to one
rectangle, exceeds 64 rectangles, or the rectangles fill at least three quarters
of their bounding box. The recomposited canvas MUST be byte-identical to a full
recomposite, and pixels outside the dirty area MUST keep their previous value.

#### Scenario: A diagonal commits its tiles, not its bounding box [dd_commit_tiles]

- **WHEN** a thin diagonal stroke is committed
- **THEN** the dirty area is composited as multiple disjoint tile rectangles
  covering less than the stroke's bounding box

#### Scenario: A filled blob collapses to its union [dd_commit_collapses]

- **WHEN** a stroke fills most of its bounding box
- **THEN** the commit composites the bounding box as a single rectangle

#### Scenario: The tiled commit equals a full recomposite [dd_commit_equal]

- **WHEN** a stroke is committed through the tiled path
- **THEN** `doc.composite` equals a full recomposite of the committed document

### Requirement: A collapsed commit blits the region buffer it already composited

When a committed dirty area collapses to a single rectangle, the commit SHALL
build its one region blit image from the sRGB region buffer that its region
composite already produced, rather than a second crop of level 0. It SHALL still
emit exactly one region blit per commit, so the shell's region handler — and the
command registry it refreshes — runs once. The emitted image MUST be
byte-identical to the level-0 crop it replaces.

#### Scenario: The region image is emitted, not a second crop [dd_commit_blit_buffer]

- **WHEN** a stroke whose dirty area collapses to one rectangle is committed
- **THEN** the emitted region image is the premultiplied region buffer and no
  level-0 crop is built

#### Scenario: The region image equals the level-0 crop [dd_commit_blit_equal]

- **WHEN** the collapsed commit's emitted image is compared pixel for pixel with
  the level-0 crop of the same rectangle
- **THEN** every channel is identical

#### Scenario: One blit per commit [dd_commit_one_blit]

- **WHEN** any paint stroke is committed
- **THEN** exactly one region blit is emitted, whatever the tile count
