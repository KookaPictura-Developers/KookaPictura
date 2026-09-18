## ADDED Requirements

### Requirement: Central splitter hosts the Tools pane

The application shell SHALL allow the Tools panel to be re-hosted from its dock
into the central splitter as a fixed-width pane, on either side of a widget
column or between two widget columns, through the same column drop grammar and
indicator the widget columns use. While it is a splitter pane the Tools panel
SHALL keep its fixed content width, fill the splitter height, remain re-draggable
by its title bar, and SHALL NOT be tabified. A docked or pane-hosted Tools panel
SHALL NOT block a floating widget overlay from being dragged across it.

#### Scenario: The Tools panel is hosted between columns [m47_tools_pane]

- **WHEN** the floating Tools panel is dropped between two widget columns
- **THEN** it becomes a fixed-width splitter pane at that boundary and is not
  tabified

#### Scenario: A widget overlay crosses the Tools pane [m47_float_over_tools]

- **WHEN** a widget overlay is dragged over the Tools panel
- **THEN** it continues to follow the cursor instead of stopping at the central
  area edge

### Requirement: Compact group chrome shading

The compact/iconic group container SHALL use the panel surface shade rather than
the darker base shade, and its drag-handle dots SHALL be dark gray, distinct
from the near-white window text, so the dots read as a handle.

#### Scenario: Compact group background and dots [m47_compact_shade]

- **WHEN** the compact strip is built
- **THEN** each group container uses the panel surface shade and its drag dots
  render in dark gray
