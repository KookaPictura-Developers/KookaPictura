## REMOVED Requirements

### Requirement: Empty document area draws no ghost canvas

### Requirement: First show with no document draws no ghost canvas

## ADDED Requirements

### Requirement: Empty document area keeps its workspace space

When the frame is shown with no document open the document pane SHALL stay in
the central splitter with a minimum width and the splitter stretch, so the
widget columns can never absorb the workspace and the splitter keeps a grabbable
handle between the workspace and each adjacent column. Only the empty tab strip
SHALL be hidden; the pane itself SHALL remain visible and SHALL become the
workspace background. The visibility SHALL be driven from the shared
`frame.cpp::refresh` funnel so every path that changes the document set (create,
open, import, close, close all, restore) takes the same branch. When a document
exists the strip SHALL be shown and the pane SHALL host the canvas as before.

#### Scenario: No document keeps the workspace space [las_empty_pane]

- **WHEN** the frame starts with no document open
- **THEN** the document pane is visible with at least its minimum width, the
  widget columns together take less than the splitter width, an empty tab strip
  is not drawn, and the splitter exposes a handle between the workspace and each
  adjacent column

#### Scenario: The first document reveals the tab strip [las_empty_pane_show]

- **WHEN** a document is created or opened with no document previously open
- **THEN** the document tab strip is shown in the pane and hosts the new canvas

#### Scenario: Closing the last document hides the strip again [las_empty_pane_hide]

- **WHEN** the last open document is closed
- **THEN** the tab strip is hidden again through the same refresh path while the
  pane keeps its workspace space

#### Scenario: An icon column keeps its strip against the workspace [las_icon_workspace]

- **WHEN** a widget column is in icon mode beside the workspace
- **THEN** the column keeps its fixed strip width and the workspace, not the
  column, absorbs the splitter slack
