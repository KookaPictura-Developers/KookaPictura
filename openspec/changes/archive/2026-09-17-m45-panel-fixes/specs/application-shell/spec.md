## MODIFIED Requirements

### Requirement: Bridge self-test mode

The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.
The self-test SHALL expose each later milestone as an independently addressable
check with a stable exit code, so a failure names the check. The headless
self-test SHALL assert that the application platform is `offscreen` and SHALL
reserve exit code **152** for a platform mismatch; later checks SHALL allocate
codes from **153** upward. The M44 checks SHALL occupy codes **153–166** and the
M45 checks SHALL occupy codes **167–179**.

#### Scenario: Self-test succeeds

- **WHEN** the executable is started with `--self-test` and the bridge returns an image
- **THEN** it prints a self-test summary and exits 0

#### Scenario: Self-test fails on a null image

- **WHEN** the bridge returns no image
- **THEN** the self-test prints a failure and exits with a non-zero status

#### Scenario: The headless platform is asserted

- **WHEN** `--headless --self-test` runs while the application platform is not `offscreen`
- **THEN** the self-test prints a failure and exits **152**

#### Scenario: Milestone checks are independently addressable

- **WHEN** an M45 check fails
- **THEN** the self-test exits with that check's code in the range **167–179** and prints which check failed

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed in the main-window dock areas only on the left
and right sides of the workspace — not on the top or bottom — and SHALL support
being moved, floated, and closed, but SHALL NOT be grouped with other panels in
a tab group. A drop of the Tools panel onto a tab bar SHALL NOT tabify it; when
a drop still results in tabification, the frame SHALL re-dock the panel to its
previous area as a fallback. When floated, the panel SHALL size to the minimum
height its content needs rather than expanding to fill the window, and that
height SHALL NOT be drag-resizable. The panel's custom title bar SHALL remain
draggable so the panel can be moved and floated. The panel's content size SHALL
be fixed along the dock's major axis: a fixed content width for a left or right
dock. Dragging the dock separator SHALL NOT resize it. The panel SHALL be
placeable on any side of any widget panel or column, wherever the columns are
docked — to the left of a right column, between two columns, or at the outer
edge — through the same column drop grammar and the same single insertion
indicator the widget columns use, without breaking the fixed-size rule or the
no-tabification contract.

#### Scenario: The panel docks only on the left or right [m45_tools_sides]

- **WHEN** the Tools panel's allowed areas are queried
- **THEN** only the left and right main-window dock areas are permitted, and the
  top and bottom areas are refused

#### Scenario: The panel can float [m40_dock]

- **WHEN** the Tools panel is dragged out of its dock area
- **THEN** it floats as an independent window and can be docked back to a side

#### Scenario: The floated dock hugs its content height [m42_tools]

- **WHEN** the Tools panel is floated
- **THEN** its height is the minimum its content needs and it does not expand to
  fill the window

#### Scenario: The floated height cannot be dragged [m44_toolsfloat]

- **WHEN** a resize is attempted on the floating Tools panel
- **THEN** it keeps its fixed content height and does not change

#### Scenario: Tabification is refused [m40_dock]

- **WHEN** the Tools panel is dropped onto another panel's tab bar
- **THEN** it does not become a tab in that group

#### Scenario: A tabified drop falls back to a side dock [m40_dock]

- **WHEN** a drop nonetheless leaves the Tools panel tabified with another panel
- **THEN** the frame re-docks it to its previous dock area

#### Scenario: The width cannot be dragged [m43_tools]

- **WHEN** the dock separator beside the Tools panel is dragged
- **THEN** the Tools panel's width does not change and stays at its content width

#### Scenario: The panel docks beside a widget column [m45_tools_beside_column]

- **WHEN** the floating Tools panel is dragged to a side of a widget column,
  including the left of a right-hand column or between two columns
- **THEN** the single blue indicator marks that boundary and the panel is placed
  there without being tabified and without losing its fixed content width
