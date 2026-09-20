## ADDED Requirements

### Requirement: Empty document area draws no ghost canvas

When no document is open the application SHALL hide the document tab pane (and
therefore the `QTabWidget#documentTabs` pane background that reads as a ghost
canvas), so the workspace shows only chrome and panel panes until a document is
created or opened. The visibility SHALL be driven from the shared
`frame.cpp::refresh` funnel so every path that changes the document set
(create, open, import, close, close all, restore) takes the same branch and no
code path can leave the pane visible with an empty `docs_`. When a document
exists the pane SHALL be shown and sized as before.

#### Scenario: No document shows no ghost canvas [las_empty_pane]

- **WHEN** the frame starts with no document open
- **THEN** the document tab pane is hidden and no canvas-like pane background is
  drawn between the panel columns

#### Scenario: The first document reveals the pane [las_empty_pane_show]

- **WHEN** a document is created or opened with no document previously open
- **THEN** the document tab pane is shown and hosts the new canvas

#### Scenario: Closing the last document hides the pane again [las_empty_pane_hide]

- **WHEN** the last open document is closed
- **THEN** the document tab pane is hidden again through the same refresh path
