## ADDED Requirements

### Requirement: Toolbox catalogue and flyout groups

The system SHALL define a frozen toolbox catalogue of the CS6 tools in
single-column flyout slots, and SHALL show every catalogue tool, implemented or
not, in its slot's flyout. Each catalogue entry SHALL carry a stable asset id,
a display label, a shortcut (or none), its flyout group and slot, whether it is
implemented, a `Qt::CursorShape` fallback, and a cursor hotspot. The toolbox
SHALL present one button per slot, SHALL mark a slot with more than one member
with a corner triangle, and SHALL reveal the slot's members on hold (with
`Alt`-click or the slot shortcut cycling the enabled members). The slot's
visible tool SHALL be its last-used member. A tool that is not implemented SHALL
be shown disabled with the tooltip `<label> — not implemented yet`. The 10
implemented tools SHALL keep their existing shortcut, checked state, cursor, and
canvas behaviour.

#### Scenario: The catalogue is complete

- **WHEN** the toolbox catalogue is enumerated
- **THEN** it contains 71 tools in 23 slots, with exactly 10 marked implemented

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
