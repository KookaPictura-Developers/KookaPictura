## ADDED Requirements

### Requirement: Standard brush tip coverage

The system SHALL render a procedural round/elliptical tip whose coverage is a
radial profile: fully opaque inside `hardness%` of the tip radius, falling off
smoothly to zero at the radius. Size SHALL be the tip diameter in pixels,
roundness SHALL squash the short axis, and angle SHALL rotate the profile. The
Brush SHALL keep a minimum anti-aliasing ramp so that hardness 100% still
produces a soft edge; coverage values SHALL be accumulated per pixel over a
stroke.

#### Scenario: Dabs are stamped along a stroke

- **WHEN** a stroke is painted with size `d` and fixed spacing `s%`
- **THEN** consecutive dab centers are spaced `s% × d` apart along the path

#### Scenario: Brush edge is anti-aliased at full hardness

- **WHEN** the Brush paints with hardness 100% on a hard-edged test document
- **THEN** at least one pixel at the stroke boundary has partial coverage, not only full or zero

#### Scenario: Elliptical tip

- **WHEN** roundness is below 100%
- **THEN** the painted footprint is narrower across the short axis than across the long axis

### Requirement: Stroke spacing

The system SHALL place dabs at a fixed arc-length interval when spacing is a
percentage of the diameter, and SHALL place dabs according to pointer speed when
spacing is disabled. Resampling SHALL carry the residual distance across
samples so spacing stays uniform on fast strokes, and SHALL clamp to a minimum
geometric step so coincident samples do not multiply dabs without bound.

#### Scenario: Increasing spacing separates dabs

- **WHEN** the same path is painted at a small spacing and then at a large spacing
- **THEN** the large-spacing stroke leaves visibly separated marks

#### Scenario: Coincident samples do not explode the dab count

- **WHEN** many samples arrive at the same position with a very low spacing
- **THEN** the number of placed dabs stays bounded and painting does not stall

### Requirement: Flow and opacity accumulation

The system SHALL treat opacity as a per-stroke ceiling on the paint applied to
any pixel and flow as the rate at which coverage builds toward that ceiling.
Repeated passes within one held stroke SHALL NOT exceed the opacity value, and
releasing and painting a new stroke SHALL apply an additional independent step
of colour.

#### Scenario: Opacity caps a single stroke

- **WHEN** a stroke with opacity `O` backtracks over the same pixels repeatedly before release
- **THEN** the resulting coverage at those pixels does not exceed `O` plus rounding

#### Scenario: A second stroke adds coverage

- **WHEN** a pixel is painted by one stroke and then by a second stroke with the same settings
- **THEN** the coverage after the second stroke is greater than after the first and does not exceed the opacity ceiling

#### Scenario: Flow is the build-up rate

- **WHEN** two strokes use the same opacity but different flow values
- **THEN** the lower-flow stroke reaches less coverage in a single pass than the higher-flow stroke

### Requirement: Pencil aliased edge and Auto Erase

The system SHALL paint the Pencil tool with a hard, non-anti-aliased edge, so
every covered pixel is fully painted and boundary pixels are either fully
painted or untouched. When Auto Erase is enabled and the stroke begins over a
pixel matching the foreground colour, the Pencil SHALL erase toward the
background colour or transparency instead of painting.

#### Scenario: Pencil edge is aliased

- **WHEN** the Pencil paints a stroke at any hardness
- **THEN** no boundary pixel has partial coverage

#### Scenario: Auto Erase begins over the foreground colour

- **WHEN** Auto Erase is on and the Pencil starts a stroke on a pixel equal to the foreground colour
- **THEN** that stroke removes colour instead of adding the foreground colour

### Requirement: Paint modes

The system SHALL support the paint modes `Normal`, `Dissolve`, `Behind`, and
`Clear`. `Normal` interpolates the destination toward the paint colour by the
composited coverage; `Dissolve` selects the paint colour stochastically per
pixel in proportion to coverage; `Behind` writes only where the destination is
not fully opaque; `Clear` reduces the destination alpha by the coverage.

#### Scenario: Normal blends by coverage

- **WHEN** a Normal stroke paints at full opacity over an opaque base
- **THEN** every fully covered pixel becomes the paint colour

#### Scenario: Clear reduces alpha

- **WHEN** a Clear stroke paints at full coverage on an opaque pixel
- **THEN** that pixel's alpha becomes zero

#### Scenario: Behind leaves opaque pixels untouched

- **WHEN** a Behind stroke paints over an opaque pixel
- **THEN** that pixel's colour is unchanged

### Requirement: Per-stroke commit and undo

The system SHALL commit a completed stroke as exactly one undoable history state
and SHALL make undo restore every touched pixel bit-exactly. A stroke that
painted no pixel SHALL add no history state and SHALL NOT mark the document
dirty. Undo requested while a stroke is in progress SHALL be ignored or cancel
the stroke without committing it.

#### Scenario: One history state per stroke

- **WHEN** a stroke is completed
- **THEN** the history gains exactly one labelled state and the document is dirty

#### Scenario: Undo restores the pixels

- **WHEN** a completed stroke is undone
- **THEN** every pixel the stroke touched returns to its pre-stroke value

#### Scenario: Empty stroke is a no-op

- **WHEN** a stroke completes without covering any pixel
- **THEN** no history state is added and the document is not marked dirty

#### Scenario: Undo mid-stroke is ignored

- **WHEN** undo is requested while a paint stroke is still in progress
- **THEN** the in-progress stroke is not committed by the undo
