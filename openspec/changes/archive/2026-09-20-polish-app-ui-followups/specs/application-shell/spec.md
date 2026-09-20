## ADDED Requirements

### Requirement: First show with no document draws no ghost canvas

The shell SHALL hide the document tab pane when the main window is first shown
with zero documents, matching the empty-workspace rule that already applies to
every later document-set transition. The first show SHALL route through the same
single visibility writer as the create/open/close transitions, so the two paths
cannot diverge. When the first document is opened the pane SHALL become visible
and show its tab.

#### Scenario: Launch with no document hides the tab pane

- **WHEN** the main window is constructed and shown with no document open
- **THEN** the document tab pane is hidden, so no empty tab widget or its
  ghost-canvas background is drawn

#### Scenario: Opening the first document shows the pane

- **WHEN** the first document is opened in a window that was shown empty
- **THEN** the document tab pane becomes visible and shows the document's tab

#### Scenario: Closing the last document hides the pane again

- **WHEN** the last open document is closed
- **THEN** the document tab pane is hidden again through the same writer
