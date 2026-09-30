# document/history-retention Specification

## Purpose
Bounded undo/redo that stores each state's changed 64×64 region tiles against
the previous state, so retained history memory scales with the edited area
rather than the document area.

## Requirements

### Requirement: Region-delta undo retention

The history SHALL store the oldest state as a materialized base and each later
state as the 64×64 tiles whose bytes differ from the previous state, for the
layer channel planes, the document composite, and the layer mask planes, plus
the state's selection and non-pixel metadata. The retained history memory SHALL
scale with the edited area rather than with the document area. The public
`History` API (`capture`, `undo`, `redo`, `jump`, `can_undo`/`can_redo`, the
label/index/depth/count accessors, and the named-snapshot accessors) and the
20-state and 10-snapshot bounds SHALL be unchanged.

#### Scenario: Small edits retain small deltas [hr_bounded_retention]

- **WHEN** many small edits are captured on a large document
- **THEN** the retained delta bytes are bounded by the changed tile area and the
  history keeps a single materialized base, not one full state per edit

#### Scenario: A geometry change stores a full state [hr_geometry_anchor]

- **WHEN** a captured state changes the tracked-plane count, stride, or height
  (resize, crop, flatten, or a layer/mask structure change)
- **THEN** that state is stored as a full snapshot and becomes a new anchor, and
  tile deltas only span a stable geometry

### Requirement: Byte-identical state materialization

`undo`, `redo`, and `jump` SHALL return a `Snapshot` whose `Document` and
selection are byte-identical to the state that was captured, including states
reachable only across a full anchor. The label order, cursor movement, depth
bound, and redo truncation SHALL be unchanged.

#### Scenario: Every state round-trips exactly [hr_exact_materialize]

- **WHEN** undo, redo, or jump restores any captured state
- **THEN** the restored document equals the captured document field for field,
  including pixels stored as tiles and pixels left unchanged since the base

#### Scenario: A paint commit undoes to the prior pixels [hr_paint_undo]

- **WHEN** a paint commit is captured and then undone
- **THEN** the edited layer channel plane and the composite equal their bytes
  before the commit, and redo restores the committed bytes

#### Scenario: Undo across a geometry change is exact [hr_geometry_undo]

- **WHEN** a geometry-changing state is captured and then undone
- **THEN** the restored document equals the pre-change document exactly
