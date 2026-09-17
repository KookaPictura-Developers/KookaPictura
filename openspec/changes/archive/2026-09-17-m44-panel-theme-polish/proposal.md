## Why

M41–M43 built and then refined the custom `PanelColumn`/`PanelGroup` workspace:
a multi-column host, session v6, an in-window float overlay, a compact/iconic
strip with flyouts, and per-widget menus. The user has handed back a sixth
lived-in refinement list. Five of the six clusters are chrome: the Tools
toolbar's float height and docking sides, the widget panels' chevrons, default
active tab, tab colours, float-drag continuity, group divider, dock targets and
theme borders, the compact strip's flyout parity, divider colour, drop
placement, drag handle and label eliding, and the document tab strip's borders.
One item is an engine bug: a brand-new document's canvas first paints a
repeating black/white/green/red garbage pattern and only a recomposite makes it
white. M44 roots that out as well. This pass is the sixth panel interruption;
the Layers-panel program shifts again to **M45** filtering/search, **M46**
management, **M47** styles/effects, and **M48** smart objects.

## What Changes

- **New-document canvas render bug (E1).** A new white-background document SHALL
  render pure white on its first present, before any layer move or recomposite.
  The initial composite/readback seam (`new_document` →
  `current_buffer`/`store_composite`/`buffer_to_image` in `cxxqt_object.rs`, and
  the GPU `composite_active`/`composite_gpu` first-use path) SHALL be diagnosed
  empirically first, the root cause named, fixed at the seam, and pinned by one
  runnable regression check (a Rust test on the composite seam) plus the app
  self-test step `m44_newdoc`.
- **Tools toolbar (`Toolbox` dock).** When floating, its height SHALL NOT be
  drag-resizable (it keeps the M42 minimum content height). It SHALL be
  dockable to any side of the workspace and the widget panels, not only the
  workspace left/right dock areas, while keeping the fixed content size (width
  for a left/right dock, height for a top/bottom dock) and the M40
  no-tabification contract.
- **Widget panel chrome.** The collapse-panel toggle's double-chevron icons are
  inverted and SHALL be swapped. The default active tab SHALL be the **first**
  panel, not the last, in whichever artefact is wrong (default group tab,
  default current panel, or default page). The active/inactive tab backgrounds
  are reversed and SHALL be swapped so the active tab matches the widget
  (pane) background and the inactive does not. Dragging a widget tab outward to
  float SHALL keep following the cursor until mouse release, exactly like a
  group/column drag, instead of stopping when the float is created. A whole
  widget column SHALL be dockable to any side of the Tools toolbar, another
  widget panel/column, or the workspace, using the M43 `resolveDrop` grammar and
  the single blue indicator. The divider between widget groups SHALL be thicker
  and darker grey.
- **Compact/iconic panel.** The flyout SHALL match the docked panel widget
  exactly (same tab bar, background, borders). The white compact-strip divider
  SHALL become dark grey. Drag-and-place SHALL improve: drop onto/among a group
  or just above/below a group inserts into that group at that place; drop
  between groups, above the top group, or below the bottom group creates a new
  group; the whole group SHALL be draggable through a small **drag-handle dots**
  affordance above each group of icons; and each group of icon buttons SHALL
  read as one visual unit. Strip labels SHALL elide as soon as any room exists,
  not only at the fixed `>= 120 px` threshold.
- **File bar.** The document tab bar SHALL get a dark grey border on its right,
  and SHALL lose its extra top border (the options bar above already draws a
  dark grey bottom border).
- **Global styling.** All panels/columns (the Tools toolbar, and both normal and
  compact widget panels) SHALL get a darker grey border. A widget SHALL have the
  same colour scheme and style whether docked, popup, or floating, by sharing
  one scoped stylesheet across the three presentations.
- **Self-tests.** New `m44_*` checks at exit codes **153–166**; the driver
  pumps the event loop bounded and forces layout so geometry is real while
  headless.

## Capabilities

### New Capabilities

None. M44 refines requirements already owned by `panel-column`,
`application-shell`, `tool-framework`, and `document-canvas`.

### Modified Capabilities

- `panel-column`: the column host and its drop grammar gain any-side docking for
  a whole widget column and the compact group-relative drop targets; a compact
  drag handle and visual grouping are added; a float drag continues to mouse
  release; the panel tab colours are corrected; the group divider and icon-strip
  divider are restyled; a scoped theme gives docked, flyout, and float widgets
  the same look and a darker border; the first panel is active by default.
- `application-shell`: the CS6 chrome styling gains panel/column and document
  tab-strip borders and the dock/popup/float parity; the Tools dock's float
  height is fixed and its allowed docking sides widen beyond left/right.
- `tool-framework`: the Tools panel's float height is no longer drag-resizable
  and its docking sides widen.
- `document-canvas`: a newly created white-background document's initial canvas
  render is pure white with no garbage frame, with the composite seam pinned by
  a regression check.

No capability is added or removed; the canonical count stays at **60** after
archive.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` — any-side dock targets
  and compact group-relative drop placement, the drag-handle dots affordance and
  group visual grouping, the label-elide threshold, the divider and compact
  divider styling, the corrected tab colours, the float-drag continuation, the
  scoped theme for the flyout/float, and the `m44_*` test hooks.
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}` — the correct
  collapse/expand chevron, the first-panel default active tab, and the shared
  widget styling hook.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` — the no-resize float height and the
  widened allowed docking sides while keeping the fixed content size.
- `crates/pictura-app/cpp/frame.{h,cpp}` — the document tab-strip borders, the
  panel/column border and parity wiring, and the widened Tools dock areas.
- `crates/pictura-app/cpp/theme.cpp` — the scoped panel/column, divider,
  compact-divider, flyout, float, and document-tab-strip selectors using the
  darker `${border}`.
- `crates/pictura-app/src/cxxqt_object.rs` — the new-document initial composite
  seam fix (E1) and its regression test; `--headless` is unchanged.
- `crates/pictura-app/cpp/main.cpp` — the `m44_*` self-test steps, exit codes
  **153–166**.
- `docs/dev/m44-panel-theme-polish.md` (brief). No dependency, PSD, session, or
  bridge-ABI change.
