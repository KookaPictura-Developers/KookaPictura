## ADDED Requirements

### Requirement: Panel width mode survives a restart for every column

Each panel column's rail mode (normal vs icon/label width) and its normal width SHALL
survive a save/restart, including the primary/leftmost column and including
a column that was flipped from icon mode to normal mode just before quitting. The
persisted normal width SHALL be the remembered normal width, not the transient
icon-strip width captured while a mode flip is still being applied.

#### Scenario: Primary icon to normal survives a restart [wsp_primary_rail_restart]

- **WHEN** the primary column is in icon mode, flipped to normal mode, and the
  app is restarted
- **THEN** the column restores to normal mode at its remembered normal width

#### Scenario: Other columns keep their own mode [wsp_other_rail_restart]

- **WHEN** one column is normal and another is icon, and the app is restarted
- **THEN** each column restores its own mode and width
