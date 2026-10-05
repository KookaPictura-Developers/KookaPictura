# Panel column vertical sizing

The widget column body is a `PanelColumn` → `QScrollArea` (`panelColumnScroll`,
`setWidgetResizable(true)`) → `QSplitter` (`panelColumnSplitter`) of `PanelGroup`s.

## The trap

The splitter's `minimumSizeHint` is the sum of each group's content minimum
(tab bar + the tallest QTabWidget page). At the default `frame.resize(1100,700)`
the four default visible groups are already ~732 px (color 190, adjustments 65,
layers 256, navigator 221) plus 12 px of dividers = 744 px, more than the
~684 px viewport. Result: a vertical scrollbar appears and **every group is
pinned at its minimum, so the dividers cannot be dragged**.

Hidden groups (`setVisible(false)`) contribute 0 to the splitter minimum even
though their own `minimumSizeHint` can be huge (the Character group is ~603 px).

## Why the obvious fixes fail

- `PanelGroup::setSizePolicy(Preferred, Ignored)` (or `minimumSizeHint` → tab
  bar) lets the splitter shrink groups, but the panels have **no internal
  scroll area**, so shrinking clips their stacked control rows. Self-test checks
  `lpr_percent` (215) and `indicator_hidden_tab_shown` (180) then fail because
  the layers panel's opacity field/geometry never lays out.
- The real CS6 behaviour needs each panel page to scroll internally (or the
  panel minimum reduced). That is a per-panel refactor, not a one-liner.

## If picking this up

Give each panel page a `QScrollArea` (frame-less, no horizontal scroll) inside
the `QTabWidget`, so the group minimum becomes the tab bar height while content
scrolls; then the splitter fits and stays draggable. Update the panel-geometry
tests to the wrapped hierarchy.
