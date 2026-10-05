# Proposal: workspace-ui-polish

## Why

The workspace renders CS6 layout structure but not CS6 surface polish: tool
buttons keep an idle outline, chrome separators are the wrong weight, the
panel/bar/workspace background steps are flat, tool-slot metrics ignore DPI,
and several interaction bugs (left/right docking, first-click menu geometry,
long-press flyout placement, one-click tool switching) remain. Issue #156
collects the whole polish pass.

## What Changes

- Introduce a cross-cutting polish capability: one global idle-less button
  style (tool-panel and icon buttons), 2px chrome vs 1px in-body separators,
  one-step background shade steps for panel/bar/widget/group and the workspace,
  and DPI-aware tool-slot metrics defined once and scaled from the base metrics.
- Replace the derived shade ramp with an explicit CS6 dark palette (panel
  `#4d4d4d`, in-body separator/table `#404040`, headers/file bar `#363636`,
  outer border/active button `#2e2e2e`, workspace `#1f1f1f`, inputs/text buttons
  `#3b3b3b`, outlines `#595959`, pressed text button `#303030`), exact at the
  default brightness level and shifted uniformly for the other levels; draw a
  3px 2px-panel + 1px-border frame around the central band; make the Tools
  column a fixed flush left slot (no resize seam) while widget columns stay
  draggable; and give the iconic strip 1px group-bottom lines instead of the
  grip line and inter-group divider, with both-side width resizing.
- Tools panel: half-open flyouts open to the side, a second tool can be picked
  while a menu is open, the 3D and Camera slots are hidden from the panel (their
  catalogue entries stay), the swap icon is
  redrawn as top-left down+left arrows, the screen-mode button opens the mode
  menu, and a Paint Mask toggle button sits left of it — button only, no Quick
  Mask engine (#172).
- Options bar: drop the idle brush-settings button style and shrink the body so
  the bar is not over-tall.
- Application shell: screen modes selectable from the Tools button menu,
  workspace colour darker and not black with an image, empty footer with no
  document, 4-chevron "Arrows" hint, consistent zoom field, no corner triangle.
- Panel column: remove normal-mode tab icons (icon row keeps them), header band
  matches the inactive tab and loses its lighter rule, 3px column header lines,
  Lucide `menu` corner glyph, docking a widget/group/panel to either side of a
  column on the other side works.
- Layers panel: lighter row background, two-column row with a non-highlightable
  visibility toggle, gray selection replacing blue, 2px right-column padding.
- Info panel: the readout menu opens fully on the first click.
- Numeric fields: input values left-aligned.

## Capabilities

### New Capabilities

- `ui/workspace-shell-polish`: the cross-cutting chrome layer — global
  idle-less button styling, chrome/body separator thickness, background shade
  steps, DPI-aware tool-slot metrics, and the toolbar Paint Mask toggle stub.

### Modified Capabilities

- `ui/application-shell`: screen-mode button menu, workspace colour, status bar
  empty state and chrome, theme shading.
- `ui/panel-column`: tab icons, header band, column separator widths, corner
  menu glyph, and left/right docking.
- `ui/layers-panel`: row background, two-column layout, visibility toggle and
  selection colour.
- `ui/info-histogram-panel`: first-open menu geometry.
- `ui/numeric-fields`: input alignment.
- `ui/icon-assets`: the `panel.menu` corner glyph.
- `tools/tool-framework`: flyout placement, one-click switching, options-bar
  compactness, swap icon, hidden 3D/Camera, Paint Mask placement.
- `tools/tool-hint-bar`: four-chevron arrows hint.
