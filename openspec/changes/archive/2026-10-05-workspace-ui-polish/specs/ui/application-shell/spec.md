## MODIFIED Requirements

### Requirement: Screen modes and canvas colour

The system SHALL support three screen modes -- Standard, Full Screen With Menu
Bar, and Full Screen. The Tools panel's screen-mode button SHALL open a menu
listing the three modes with the active mode checked, and choosing an entry SHALL
switch to that mode; the button's icon SHALL reflect the active mode. `F` SHALL
cycle the modes forward and `Shift+F` backward. `Space+F` SHALL cycle the canvas
background colour.

#### Scenario: Cycle screen modes

- **WHEN** `F` is pressed from Standard mode
- **THEN** the frame enters Full Screen With Menu Bar, and pressing `F` again enters Full Screen

#### Scenario: Reverse the cycle

- **WHEN** `Shift+F` is pressed
- **THEN** the screen mode steps backward through the cycle

#### Scenario: Cycle canvas colour

- **WHEN** `Space+F` is pressed
- **THEN** the canvas background colour advances to the next value

#### Scenario: The screen-mode button opens the mode menu [las_screen_mode_menu]

- **WHEN** the Tools panel's screen-mode button is activated
- **THEN** a menu lists Standard, Full Screen With Menu Bar, and Full Screen with the active mode checked, and choosing an entry switches the frame to that mode

### Requirement: Status bar readouts

The system SHALL show the active document's magnification and file size in a
status bar, along with a tool-hint field, and SHALL expose a view-options popup
from the status bar for selecting which readout is displayed. When no document
is open the status bar SHALL show no readouts and no tool hint — only its
background — and SHALL show them again once a document opens. The status bar
SHALL use a small left padding and SHALL NOT draw the right-side size-grip
corner triangle. The status-bar magnification readout and any zoom control SHALL
show the same value in the same format.

#### Scenario: Magnification and size shown

- **WHEN** a document is displayed
- **THEN** the status bar shows the current magnification and document size

#### Scenario: View options popup

- **WHEN** the status-bar options control is activated
- **THEN** a popup lists the selectable readouts and the chosen readout is displayed

#### Scenario: No document shows an empty status bar [las_status_empty]

- **WHEN** no document is open
- **THEN** the status bar shows no magnification, size, or tool-hint text, only its background, and restores them when a document opens

#### Scenario: No corner size grip [las_status_no_grip]

- **WHEN** the status bar is shown
- **THEN** it draws no right-side size-grip corner triangle and uses a small left padding

### Requirement: Toolbox catalogue and flyout groups

The system SHALL define a frozen toolbox catalogue of the CS6 tools in
single-column flyout slots, and SHALL show every catalogue tool, implemented or
not, in its slot's flyout. The catalogue MAY retain the Object (3D) and Camera
tool entries — the enum and table stay stable — but the Tools panel SHALL NOT
present their slots, so the panel shows 21 slots. Each catalogue entry
SHALL carry a stable asset id, a display label, a shortcut (or none), its flyout
group and slot, whether it is implemented, a `Qt::CursorShape` fallback, and a
cursor hotspot. The toolbox SHALL present one button per slot, SHALL mark a slot
with more than one member with a corner triangle, and SHALL reveal the slot's
members on hold (with `Alt`-click or the slot shortcut cycling the enabled
members). The slot's visible tool SHALL be its last-used member. A tool that is
not implemented SHALL be shown disabled with the tooltip `<label> — not
implemented yet`. The 10 implemented tools SHALL keep their existing shortcut,
checked state, cursor, and canvas behaviour.

#### Scenario: The catalogue is complete

- **WHEN** the toolbox catalogue is enumerated
- **THEN** it retains every CS6 tool including Object (3D) and Camera, while the
  Tools panel presents 21 slots with exactly 10 marked implemented

#### Scenario: The 3D and Camera slots are absent [las_no_3d_camera]

- **WHEN** the toolbox's presented slots are inspected
- **THEN** no Object (3D) or Camera slot or button is present, although the
  catalogue retains their entries

#### Scenario: A slot with hidden tools reveals them

- **WHEN** the user holds the mouse on a slot whose group has more than one member
- **THEN** a flyout lists every member of that group

#### Scenario: An unimplemented tool is visible but disabled

- **WHEN** the flyout of a group containing a tool with no engine is opened
- **THEN** that tool's entry is disabled and its tooltip is `<label> — not implemented yet`

#### Scenario: An implemented tool is selected from a flyout

- **WHEN** the user picks an implemented tool from a flyout
- **THEN** that tool becomes active and the slot shows its icon as the last-used member

#### Scenario: Cycling a slot's enabled members

- **WHEN** the user `Alt`-clicks a slot, or presses `Shift` plus the slot's shortcut
- **THEN** the active tool advances to the next enabled member of that group, wrapping deterministically

#### Scenario: Implemented tools behave as before

- **WHEN** an implemented tool is activated by shortcut or by clicking its button
- **THEN** it is marked active, its cursor is applied, and its options bar is shown, exactly as before this change
