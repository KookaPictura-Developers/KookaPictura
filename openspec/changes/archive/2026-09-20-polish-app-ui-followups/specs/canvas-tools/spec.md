## ADDED Requirements

### Requirement: Alt press without movement does not duplicate

A Move-tool press with Alt held that does not move the pointer SHALL leave the
document unchanged: no duplicate layer and no history state. The duplicate SHALL
only come into existence once the pointer has actually moved, or SHALL be rolled
back on a zero-offset release. A non-zero Alt drag SHALL continue to insert the
clone, preview it, and commit one state with the clone active.

#### Scenario: A bare Alt click leaves no layer [mv_alt_zero_noop]

- **WHEN** the Move tool is pressed with Alt held and released at the same point
  without moving
- **THEN** no layer is added, the active layer is unchanged, and no history state
  is recorded

#### Scenario: The first movement creates the clone [mv_alt_first_move]

- **WHEN** an Alt Move press is followed by pointer movement before release
- **THEN** the clone is inserted and previewed, and release commits one state
  with the clone active
