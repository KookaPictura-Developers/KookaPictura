# free-transform Specification

## ADDED Requirements

### Requirement: Projective (quad) layer transform op

The system SHALL provide
`pictura_render::transform_layer_quad(doc: &mut Document, path: &str, quad:
[(f64, f64); 4]) -> bool`, where `quad` gives the document-space target of the
source layer rect's four corners in top-left, top-right, bottom-right,
bottom-left order. On success the operation SHALL resample every channel plane
(and the layer mask, about its own rect) with the bilinear inverse of the
projective map sending the source corners to `quad`, SHALL set the layer `rect`
to the integer bounding box of `quad` (`floor(min x)`, `floor(min y)`,
`ceil(max x)`, `ceil(max y)`), and SHALL return `true`. A destination pixel whose
inverse-mapped source point lies outside `[0, w) × [0, h)` SHALL be written as
`0`, and bilinear taps SHALL be edge-clamped. The operation SHALL refuse and MUST
leave `doc` bit-identical for every case `transform_layer` refuses, and also when
any corner is non-finite, the projective map is singular or near-singular, or the
result rect has zero area or exceeds the op's pixel ceiling. It SHALL NOT
recomposite the document and MUST NOT panic on any input.

#### Scenario: An identity quad is the source rect

- **WHEN** `transform_layer_quad` is called with `quad` equal to the source rect corners
- **THEN** it returns true and the layer rect and every plane are unchanged

#### Scenario: A translated quad matches the similarity translation

- **WHEN** `transform_layer_quad` is called with every corner translated by an integer `(dx, dy)`
- **THEN** the resulting layer rect and channel planes equal those produced by `transform_layer` with the same translation

#### Scenario: A perspective quad maps the corners onto the targets

- **WHEN** a projective quad with a non-affine corner is applied
- **THEN** the source corners map onto the requested `quad` within a small tolerance

#### Scenario: Degenerate quads are refused

- **WHEN** `quad` has a non-finite corner, three collinear corners, or zero area
- **THEN** it returns false and every field of `doc` equals its pre-call value

#### Scenario: Pixels outside the source are transparent

- **WHEN** a projective quad pulls part of the destination outside the source rect
- **THEN** every destination pixel whose source point is outside the source rect has all channels equal to `0`

### Requirement: Skew, Distort, and Perspective transform modes

The Free Transform session SHALL expose a mode of `Free` (the existing
similarity), `Skew`, `Distort`, or `Perspective`. In `Free` the existing scalars
drive the transform; in a projective mode the session SHALL hold the live target
quad and the scalars SHALL be unused. A corner drag in `Distort` SHALL set that
corner to the pointer; a corner drag in `Perspective` SHALL set that corner to
the pointer and move the opposite corner by the negated delta about the quad
centre; an edge drag in `Skew` SHALL translate that edge's two endpoints by the
same delta, constrained to the edge axis when Shift is held. A projective-mode
gesture that would make the quad degenerate SHALL be refused. Hit testing in a
projective mode SHALL accept only the eight bounding handles; a press away from a
handle SHALL not move or rotate. Committing SHALL call `transform_layer_quad` for
the projective modes and `transform_layer` for `Free`, recomposite, and record
exactly one `"Free Transform"` history state; an identity commit SHALL record
none; Escape SHALL leave the document bit-identical with no history state added.
The precise gesture semantics are inferred from CS6 behavior (no reference image
oracle); the projective map itself is exact.

#### Scenario: Distort commits the dragged corner

- **WHEN** a `Distort` session drags one corner and commits
- **THEN** the layer's source corner is at the pointer's document position and exactly one `"Free Transform"` state is added

#### Scenario: Perspective mirrors the opposite corner

- **WHEN** a `Perspective` session drags a corner by a delta and commits
- **THEN** the opposite corner moves by the negated delta and the quad centre is unchanged

#### Scenario: Skew slides one edge

- **WHEN** a `Skew` session drags an edge handle and commits
- **THEN** that edge translates by the drag delta, the opposite edge is unchanged, and exactly one `"Free Transform"` state is added

#### Scenario: Escape cancels a projective mode exactly

- **WHEN** a projective-mode session is begun, a gesture is applied, and Escape is pressed
- **THEN** the document equals its pre-session value and the history count is unchanged

#### Scenario: A degenerate drag is refused

- **WHEN** a projective gesture would place three corners on one line
- **THEN** the session's quad is not collapsed to a degenerate shape

### Requirement: Skew, Distort, and Perspective menu commands

The application SHALL provide implemented commands `Edit > Transform > Skew`,
`Edit > Transform > Distort`, and `Edit > Transform > Perspective` that begin a
Free Transform session in the matching mode on the active layer, using the same
active-layer resolution and refusal as `Edit > Free Transform`. Each command
SHALL be enabled exactly when `Edit > Free Transform` is enabled.

#### Scenario: The Distort command enters a Distort session

- **WHEN** `Edit > Transform > Distort` is invoked with exactly one transformable layer active
- **THEN** a session begins in `Distort` mode on that layer

#### Scenario: A non-transformable target leaves the command disabled

- **WHEN** the active layer is a group, an adjustment layer, a Background layer, or position-locked
- **THEN** the Distort/Skew/Perspective commands are disabled and invoking them begins no session
