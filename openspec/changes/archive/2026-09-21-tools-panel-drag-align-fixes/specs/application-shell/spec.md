## ADDED Requirements

### Requirement: Docked or pane-hosted Tools panel content is top-aligned

The content body of the Tools panel SHALL fill the height the dock is given and
SHALL start at the top while the panel is docked in a left or right dock area or
hosted as a central-splitter pane, so the slot column begins directly below the
custom title bar. The body SHALL expand vertically rather than taking its fixed
content height and being centred in the taller dock. Only a floating panel SHALL
keep the body at its fixed content height.

#### Scenario: The content fills and starts at the top [las_tools_top_align]

- **WHEN** the Tools panel is hosted as a central-splitter pane or docked and the
  frame is taller than the panel's content
- **THEN** the content body's height equals the space below the title bar and its
  top edge sits at the title bar's bottom, not centred in the dock
