## ADDED Requirements

### Requirement: Tools panel height is pinned only while floating

The Tools panel SHALL pin its height to its content only while it is floating.
While docked in a left or right dock area, or hosted as a central-splitter pane,
its height SHALL be free: the panel SHALL fill the height it is given and its
content minimum SHALL NOT force the main window's central workspace shorter than
the window. Re-docking a floating panel SHALL release the pinned height, and
floating it again SHALL re-pin it.

#### Scenario: A pane-hosted panel does not shrink the workspace [las_tools_free_height]

- **WHEN** the Tools panel is hosted as a central-splitter pane
- **THEN** the panel's height bounds are free, the central splitter keeps its
  height, and the document pane fills the splitter

#### Scenario: Floating pins and re-docking releases [las_tools_float_height]

- **WHEN** the Tools panel is floated and then re-docked
- **THEN** the floating panel is pinned to its content height and the re-docked
  panel is free again
