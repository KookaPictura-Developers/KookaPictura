## MODIFIED Requirements

### Requirement: Bridge self-test mode

The system SHALL provide a `--self-test` mode that exercises the Rust↔Qt bridge,
writes a summary to stderr, and exits non-zero when the bridged image is missing.
The self-test SHALL expose each later milestone as an independently addressable
check with a stable exit code, so a failure names the check. The headless
self-test SHALL assert that the application platform is `offscreen` and SHALL
reserve exit code **152** for a platform mismatch; later checks SHALL allocate
codes from **153** upward. The M44 checks SHALL occupy codes **153–166**.

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

- **WHEN** an M44 check fails
- **THEN** the self-test exits with that check's code in the range **153–166** and prints which check failed

### Requirement: CS6-style chrome styling

The system SHALL style the application chrome (menu bar, options bar, Tools
panel, panel docks and their tabs and title bars, status bar, tool buttons, and
scrollbars) with a CS6-style dark stylesheet driven by the same brightness ramp
as the palette, and SHALL rebuild the stylesheet when the brightness level
changes. The bar chrome SHALL be a low-contrast dark surface distinct from the
document canvas, and panel tabs SHALL be flat with a distinctly highlighted
active tab. The Tools toolbar and the normal and compact widget panels SHALL
draw a darker grey border from the shared theme, and a widget SHALL use the same
colour scheme and style whether docked, shown in a compact flyout, or floating.
The document tab strip SHALL draw a dark grey border on its right side and SHALL
NOT draw an extra top border, because the options bar above it already draws a
dark grey bottom border. Every new rule SHALL be scoped to the panel family and
the document tab strip so the rest of the chrome is unaffected.

#### Scenario: Chrome is styled at startup

- **WHEN** the frame is shown
- **THEN** the dock tabs, dock title bars, tool buttons, menu bar, and status bar use the dark stylesheet rather than unstyled Fusion defaults

#### Scenario: Brightness restyles the chrome

- **WHEN** the brightness level changes with `Shift+F1` or `Shift+F2`
- **THEN** the stylesheet is regenerated for the new level and the chrome colours change

#### Scenario: Panels draw the darker grey border [m44_panelborder]

- **WHEN** the Tools toolbar and a normal- or compact-mode widget panel are shown
- **THEN** each draws a darker grey border from the shared theme

#### Scenario: The document tab strip has only a right border [m44_filebar]

- **WHEN** the document tab strip is inspected
- **THEN** it draws a dark grey border on its right side and no extra top border

#### Scenario: A widget looks the same in all three presentations [m44_popupstyle]

- **WHEN** the same widget is docked, shown in a compact flyout, and floating
- **THEN** all three present the same background, borders, and tab-bar styling

### Requirement: Tools panel is a standalone dock

The Tools panel SHALL be allowed in the main-window dock areas on every side of
the workspace and the widget panels — left, right, top, and bottom — and SHALL
support being moved, floated, and closed, but SHALL NOT be grouped with other
panels in a tab group. A drop of the Tools panel onto a tab bar SHALL NOT tabify
it; when a drop still results in tabification, the frame SHALL re-dock the panel
to its previous area as a fallback. When floated, the panel SHALL size to the
minimum height its content needs rather than expanding to fill the window, and
that height SHALL NOT be drag-resizable. The panel's custom title bar SHALL
remain draggable so the panel can be moved and floated. The panel's content size
SHALL be fixed along the dock's major axis: a fixed content width for a left or
right dock and a fixed content height for a top or bottom dock. Dragging the
dock separator SHALL NOT resize it, and the panel SHALL be placeable on any side
of the workspace or beside a panel column without breaking the fixed-size rule or
the no-tabification contract.

#### Scenario: The panel docks on every side [m44_docksides]

- **WHEN** the Tools panel's allowed areas are queried
- **THEN** the left, right, top, and bottom main-window dock areas are permitted

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

#### Scenario: The fixed width holds beside a column [m43_tools]

- **WHEN** the Tools panel is docked beside a panel column in one- or two-column
  mode
- **THEN** its width is the fixed content width and the column can still be laid
  out beside it
