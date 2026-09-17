## Why

M41 shipped the custom `PanelColumn`/`PanelGroup` chrome, and the user has now
handed back a concrete refinement list from living with it: the column has no
minimum width and can be squeezed to nothing, compact mode keeps the old wide
splitter width, the iconic-strip and Tools icons are too small, the floated
Tools dock wastes vertical space, the Tools dock carries stray width, a widget
overlaps the menu bar, floating panels open their own app windows, the compact
strip cannot be rearranged, compact icons open an ungrouped flyout on the wrong
side with no close button, and a group tab header has no panel menu. This is the
third panel interruption; it pins the chrome down before the Layers-panel
program resumes (see Non-Goals for the renumbered program).

## What Changes

- **Normal-mode minimum width.** In normal mode the column enforces a sensible
  minimum width so it cannot be squeezed to nothing; the no-height-minimum and
  scroll behaviour stay.
- **Compact mode starts at the smallest width.** Toggling to `iconic` sets the
  column to the smallest width that shows the strip instead of keeping the prior
  splitter width; toggling back restores the normal width.
- **Bigger icons.** The iconic-strip buttons and the Tools (toolbox) slot and
  screen-mode icons render at a larger pixmap size than the M41/M40 pass.
- **Tools dock geometry.** The floated Tools dock sizes to its content height
  (the `addStretch` that padded it is removed), and the dock uses only the width
  its content needs in one- and two-column modes.
- **Menu-bar overlay removed.** The stray widget that draws text over the
  `File`/`Edit` row is diagnosed and removed; the menu-bar row is kept clear of
  every other widget, and a stale persisted layout that no longer matches the
  current chrome is discarded instead of restoring a widget over the menu bar.
- **In-window floating overlay.** A torn-off group no longer creates a
  `Qt::Tool` top-level OS window; it floats as a movable child overlay inside
  the main window, clipped to it, raised above the columns and canvas, and
  re-docks when dropped back on the column.
- **Compact-strip drag and drop.** Strip icons can be dragged to reorder within
  the strip and dropped onto the normal-mode group stack to move the panel,
  reusing the M41 drag machinery.
- **Compact flyout refinement.** The flyout opens on the inner side of the
  column, the open panel's icon renders active/pressed, the flyout is styled as
  a panel group (tab header + content) with a close icon button (double right
  chevron) at the right end of its header.
- **Per-widget header action button.** Each group's tab header carries a
  context-action button at its far right that shows the current panel's CS6
  menu (per-widget, not per-group), with unimplemented entries disabled under
  the repo's "not implemented yet" tooltip and `Close`/`Close Panel Group` kept
  off the per-widget menu.
- **Foreground/background swap.** The existing default-colour (X) reset stays,
  and the missing CS6 swap (double-headed arrow) control is added; the `X` key
  swaps foreground/background when no tool shortcut claims it.
- **Self-tests.** New `m42_*` checks at exit codes **131–139**; no new session
  schema (compact reorder reuses the v5 `panelGroups` order).

## Capabilities

### New Capabilities

None. M42 refines requirements already owned by `panel-column`,
`tool-framework`, and `application-shell`.

### Modified Capabilities

- `panel-column`: the column gains a normal-mode minimum width and a
  smallest-width compact transition; the iconic strip gains larger icons,
  drag/reorder, an inner-side group-styled flyout with an active icon and a
  header close button; the tab header gains a per-widget action menu; tear-off
  changes from a `Qt::Tool` OS window to an in-window floating overlay.
- `tool-framework`: the Tools panel gains larger icons, a
  foreground/background swap control and the `X` swap key, and the one- and
  two-column widths are pinned to the tight content width.
- `application-shell`: the Tools dock sizes to its content height when floated,
  and the menu-bar row is guaranteed clear of every other widget (with stale
  layouts discarded).

No capability is added or removed; the canonical count stays at **60** after
archive.

## Impact

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}` — minimum width and
  compact-transition width, larger strip buttons, inner-side group-styled
  flyout with header close button, active-icon state, strip drag/reorder, the
  per-widget header action button and menus, and the in-window float overlay
  replacing `PanelFloat`'s `Qt::Tool` window.
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}` — the tab-header action
  button and the per-widget menu hook; the header uses the current panel.
- `crates/pictura-app/cpp/panels/panel_menus.{h,cpp}` (new, or folded into the
  header button) — the frozen per-panel menu tables from
  `docs/dev/m42-panel-menus.md` with implemented flags.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}` — the Layers panel's
  existing `Panel Options…` action is surfaced through the per-widget menu; no
  behaviour change to the M39/M41 Panel Options dialog.
- `crates/pictura-app/cpp/toolbox.{h,cpp}` — larger slot/screen-mode icon
  pixmaps, removed `addStretch`, tight content width, the fg/bg swap control and
  `X` shortcut.
- `crates/pictura-app/cpp/frame.{h,cpp}` — discard a stale session layout, keep
  the menu-bar row clear, host the in-window float overlay, and route the `X`
  swap key.
- `crates/pictura-app/cpp/main.cpp` — the `m42_*` self-test steps, exit codes
  **131–139**.
- `docs/dev/m42-panel-refinements.md` (brief) and `docs/dev/m42-panel-menus.md`
  (per-panel menu research). No Rust, bridge, document, compositor, PSD,
  session-schema, or dependency change.
