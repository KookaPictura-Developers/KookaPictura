# M42 — panel refinements (widths, icons, flyout, per-widget menus, in-window float)

- **Status:** proposed (`openspec/changes/m42-panel-refinements`); not
  implemented.
- **Type:** a user-requested refinement pass on the M41 `PanelColumn` chrome.
  It is the **third panel interruption** of the Layers-panel program; the program
  is renumbered behind it (see §7). STATE is handled separately.
- **Contract:** `openspec/specs/panel-column/`, `tool-framework/`, and
  `application-shell/` are the canonical requirements this change reconciles.
  `docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) governs docks;
  `docs/02-ui-ux/panels/layers-panel.md` (`PAN-001`) and
  `docs/05-layers/layer-management-ui.md` (`LAY-002`) stay the Layers-panel
  contract. The per-panel menu tables are frozen in
  `docs/dev/m42-panel-menus.md`.
- **Consumers:** `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`,
  `panels/panel_group.{h,cpp}`, a new `panels/panel_menus.{h,cpp}` (or folded
  into the header button), `panels/layers_panel.{h,cpp}`,
  `cpp/toolbox.{h,cpp}`, `cpp/frame.{h,cpp}`, and `cpp/main.cpp`.
- **Non-goals:** placeholder-panel content, pixel-exact CS6 metrics,
  multi-monitor floating, OS-window float chrome, a session-schema bump,
  implementing the disabled per-panel menu entries, and any change to the tool
  catalogue, the M40 flyout/shortcut/dock contracts, or the M41 tab context menu
  item list. Gradients/Patterns/Libraries are explicitly non-goals (see
  `docs/dev/m42-panel-menus.md`).

## 1. The user's report (twelve items, all in scope)

1. Widget panels have no minimum width in normal mode; the column can be
   squeezed to nothing.
2. Toggling to compact/iconic mode keeps the prior splitter width instead of
   starting at the smallest possible width.
3. The iconic-strip icon buttons are too small.
4. The Tools (toolbox) icons should be slightly bigger.
5. When the Tools dock floats, it wastes vertical space (the `addStretch`).
6. Floating widget panels must not create their own app windows. Chosen
   semantics: an **in-window floating overlay**, clipped to the main window and
   re-docked on drop. Replaces M41's `Qt::Tool` `PanelFloat`.
7. The Tools dock should take as little width as possible in both 1- and
   2-column modes.
8. A stray widget is drawn over the menu bar (text like `File Act…` overlapping
   `File`/`Edit`). Diagnose and remove it so nothing overlays the menu bar.
9. Compact-mode drag & drop: the iconic-strip icons can be dragged to reorder
   and dropped into the normal-mode group stack.
10. Clicking a compact-mode icon must: open the popup on the **inner side**
    (right column ⇒ popup left); render the active widget's icon
    active/pressed; look like a widget group (tab header + content); and show a
    close icon button (double right chevron) at the right end of the popup
    header.
11. Each widget group gets a context-action button at the far right of its tab
    header. The actions are **per-widget (per-panel)**, using the researched CS6
    menus in `docs/dev/m42-panel-menus.md`, with unimplemented entries disabled
    and a "not implemented yet" tooltip; `Close`/`Close Panel Group` stay off
    the per-widget menu.
12. The default-colors (X) button looks inert (fg/bg are already black/white).
    Keep it and add the missing CS6 swap (double-headed arrow) button; wire `X`
    too if free.

## 2. Current state (inventory)

- `crates/pictura-app/cpp/panels/panel_column.{h,cpp}`: `PanelColumn` with a
  `QSplitter` of `PanelGroup`s in a `QScrollArea`, the `panelColumnToggle`, the
  iconic strip (`kIconButtonSize = 24`, `kIconPixmapSize = 16`,
  `panel_column.cpp:29–31`), the single `Qt::Popup` `PanelFlyout`
  (`:45–57`, `:537–614`), the seven-item tab menu (`:616–676`), the drag/drop
  and blue indicator (`:789–1026`), and the `Qt::Tool` `PanelFloat`
  (`:59–77`, `:1085–1125`). `setRailMode` (`:297–315`) swaps the scroll area and
  the strip but does not change the column width; `setMinimumWidth(0)` (`:130`).
- `crates/pictura-app/cpp/panels/panel_group.{h,cpp}`: `PanelGroup` with
  `QTabWidget::North`, minimize/collapse, detach/attach for the flyout, and the
  tab-drag signals; no tab-header action button.
- `crates/pictura-app/cpp/panels/layers_panel.{h,cpp}`: the M39 panel menu
  (`:728–745`) wires `Panel Options…` to `openPanelOptions` plus eight layer
  commands; the per-widget menu must surface these and not regress them.
- `crates/pictura-app/cpp/toolbox.{h,cpp}`: slot buttons `30×30` with `20×20`
  icons (`:334–335`), `screenMode_` also `20×20` (`:395`), a body
  `layout->addStretch(1)` (`:404`), `contentWidth()`/`updateContentMetrics()`
  for the minimum width (`:444–466`), and a `ForegroundBackgroundWidget` with
  two swatches plus a corner reset but no swap control (`:162–272`).
- `crates/pictura-app/cpp/frame.cpp`: `restoreState(state.layout)`
  unconditionally when a layout exists (`:113–115`); the `optionsBar_` toolbar
  is added with `addToolBar` (`:1060–1061`); the screen-mode/menu-bar visibility
  lives at `:529–562`.
- `crates/pictura-app/cpp/session.h`: schema v5 with `panelRailMode`,
  `railWidth`, `autoCollapseIconic`, `autoShowHidden`, and `panelGroups`
  (`:20–28`).
- `crates/pictura-app/cpp/main.cpp`: the M41 checks use exit codes 120–130
  (129 `m41_rail` folded into 120; the `m41_minwidth` check is code 121 at
  `main.cpp:3473–3505`). M42 allocates **131–139**.
- **No** per-widget menu button, compact-strip drag, inner-side flyout, active
  icon, in-window overlay, fg/bg swap, menu-bar guard, or normal-mode minimum
  width exists today.

## 3. Frozen design

### 3.1 Normal-mode minimum width and compact transition

The column enforces a bounded normal-mode minimum width (a chosen constant based
on content/tab hints). Entering `iconic` records the current normal width and
sets the column to the smallest strip width; leaving `iconic` restores the
recorded width. No height minimum: a short window scrolls. The `m41_minwidth`
check (121) is amended in place to the new width floor while keeping its
height/scroll clauses.

### 3.2 Icon sizes

The iconic-strip button/pixmap constants and the Tools slot and screen-mode icon
pixmap sizes are raised to larger chosen values. Existing `icons.h` asset ids
are reused; no new art.

### 3.3 Tools dock geometry

`Toolbox` drops the body `addStretch(1)` so a floated dock hugs its content
height, and keeps `updateContentMetrics()` as the width source with body margins
trimmed to the layout margins, so the one- and two-column dock has no extra
horizontal space.

### 3.4 Menu-bar overlay

The diagnosis order: inspect `menuBar()->cornerWidget()` and every
`findChildren<QToolBar*>()` whose global geometry intersects the menu-bar row;
remove the stray widget. The prime suspect is the persisted `state.layout`
(M41 deleted the M24 `PanelRail`, so a stale layout can restore chrome into a
position that no longer matches) — the fix discards a layout whose
schema/version does not match the current chrome and falls back to the default.
The `m42_menubar` check freezes the invariant.

### 3.5 In-window floating overlay

`PanelFloat` becomes a frameless, raised child widget of the main window
(not a `Qt::Tool` top-level). It moves by its tab bar using the existing group
drag path, is clipped to the main window, and re-docks via the existing
`applyGroupDrop` when dropped on the column. It never appears in the window
manager's task list.

### 3.6 Compact-strip drag and reorder

Strip icon buttons reuse `beginPanelDrag`/`updateDrag`/`commitDrop`. A reorder
writes through `PanelGroup::setPanelOrder` (the v5 `panelGroups` order); a drop
on the group stack resolves through the existing `resolveDrop` group/boundary
targets.

### 3.7 Compact flyout

The flyout opens on the inner side of the column (left of a right-hand strip,
right of a left-hand strip), the open icon is checkable/drawn pressed, and the
single `PanelFlyout` is restyled as a group: a header naming the panel above the
detached content, with a close icon button (double right chevron) at the right
end. No second flyout system.

### 3.8 Per-widget header action button

`PanelGroup` adds a small action `QToolButton` at the far right of its tab-bar
row. It shows the menu of `currentPanelName()` from
`docs/dev/m42-panel-menus.md`. Unimplemented entries are disabled with the
`<label> — not implemented yet` tooltip; submenu parents whose children are all
disabled are disabled. `Close`/`Close Panel Group` stay on the tab context menu.
Layers `Panel Options…` calls the existing `openPanelOptions`.

### 3.9 Foreground/background swap

`ForegroundBackgroundWidget` gains a swap control (double-headed arrow)
exchanging fg/bg; the frame wires `X` to swap when no tool letter claims it. The
corner reset stays.

## 4. Frozen self-test exit codes (131–139)

| Code | Check |
|---|---|
| 131 | `m42_minwidth` — normal-mode minimum width; compact starts at the smallest strip width |
| 132 | `m42_iconic` — strip icons larger than M41; the open panel's icon is active/pressed |
| 133 | `m42_dragstrip` — a strip icon reorders within the strip and persists; dropping it on the group stack moves the panel |
| 134 | `m42_flyout` — the flyout opens on the inner side; header names the panel and carries the close button |
| 135 | `m42_widgetmenu` — the header button shows the current panel's menu; disabled entries have the tooltip; `Close`/`Close Panel Group` are absent; Layers `Panel Options…` opens |
| 136 | `m42_float_overlay` — a torn-off group is an in-window child overlay (not a top-level window), stays within the main window, and re-docks |
| 137 | `m42_fgbg` — the swap control exchanges fg/bg; the `X` key swaps |
| 138 | `m42_menubar` — no child geometry intersects the menu-bar row; a stale layout is discarded |
| 139 | `m42_tools` — larger tool icons; one-/two-column widths differ from content only by the body margins; a floated dock hugs its content height |

Suggested test hooks: `beginStripDragForTest`, `stripOrderForTest`,
`dropStripOnGroupForTest`, `flyoutSideForTest`, `activeIconForTest`,
`flyoutHeaderTitleForTest`, `flyoutHeaderCloseForTest`, `headerMenuButtonForTest`,
`panelMenuTextsForTest`, `panelMenuEnabledForTest`, `triggerPanelMenuForTest`,
`floatIsWindowForTest`, `floatClampedForTest`, plus the amended
`minimumWidthForTest`. Pump the event loop bounded before reading
geometry/visibility (the M40 Wayland-popup lesson; the driver pins xcb for
`--self-test`) and `std::fflush(stderr)`.

## 5. Session

No schema bump. Strip reorder writes the existing v5 `panelGroups` order; the
in-window overlay's position is not persisted. Persisting a float position would
be a v6.

## 6. Honest limits

- The researched menu ordering is best-effort: Adobe groups by function rather
  than printing menu order, and the ordering screenshots are CC-era. A CS6
  screenshot can reorder rows without code-structure change.
- Most per-panel entries ship disabled under the "not implemented yet" tooltip;
  the menus are a chrome contract until the features land.
- The in-window overlay is intentionally clipped to the main window: it cannot
  be moved to another monitor, has no window decorations, and does not appear in
  the task list. Multi-monitor and OS-window float are non-goals.
- The exact stray widget over the menu bar is not confirmed by a fresh
  screenshot; the fix removes the candidates and guards the stale-layout path,
  and the `m42_menubar` check freezes the invariant.
- The normal-mode minimum width, the icon sizes, and the flyout offsets are
  chosen constants, not sourced CS6 metrics.
- The `m41_minwidth` check (code 121) is amended, not removed; this is the one
  permitted change to an existing M41 self-test.

## 7. Layers-panel program shift (record only)

The M40/M41/M42 interruptions push the Layers-panel program's stages back by
three. The renumbered program is:

- **M43** — layer filtering/search
- **M44** — remaining layer management operations
- **M45** — layer styles / effects
- **M46** — smart objects / vector masks / artboards / layer comps

STATE's program line is updated separately; this brief is the record.

## 8. Non-goals

Placeholder-panel content, workspace presets, pixel-exact CS6 metrics,
multi-monitor floating, OS-window float chrome, a session-schema bump, the
disabled per-panel menu features, and any change to the tool catalogue, the M40
flyout/shortcut/dock contracts, or the M41 tab context menu item list. No Rust,
bridge, document, compositor, PSD, or dependency change.

## 9. Verification

- `openspec validate m42-panel-refinements --strict` and
  `openspec validate --all --strict`.
- `git status --porcelain` shows docs + OpenSpec changes only for this proposal.
- Implementation (later) is verified by `cmake --build build`, both app
  self-tests, `cargo fmt`/`clippy`/`test` (expected unchanged), and the `m42_*`
  steps at codes 131–139.
