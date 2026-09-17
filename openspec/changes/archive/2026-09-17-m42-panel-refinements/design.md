## Context

M41 (archived) replaced the right-hand `QDockWidget` area with a custom
`PanelColumn` (`crates/pictura-app/cpp/panels/panel_column.{h,cpp}`): a vertical
`QSplitter` of `PanelGroup : QWidget` top-tab groups inside a `QScrollArea`,
with a `panelColumnToggle`, an iconic icon strip, `Qt::Popup` flyouts, a
seven-item tab context menu, tab drag/regroup/insert with a blue drop indicator,
and tear-off into a top-level `Qt::Tool` `PanelFloat`. The session store is at
schema v5 with `panelRailMode`, `railWidth`, `autoCollapseIconic`,
`autoShowHidden`, and the per-group `panelGroups` array
(`crates/pictura-app/cpp/session.h:20–28`). The M40 Tools panel
(`crates/pictura-app/cpp/toolbox.{h,cpp}`) is a standalone `QDockWidget` with a
custom title bar (`toolsColumnToggle`, no label), a body `QVBoxLayout` whose
last child is `layout->addStretch(1)` (`toolbox.cpp:404`), slot buttons fixed at
`30×30` with `20×20` icons (`toolbox.cpp:334–335`), and a `ForegroundBackgroundWidget`
that draws two swatches plus a corner reset but no swap control.

The chrome is now stable enough to refine. The user's list (twelve items) is a
lived-in critique of M41, not new scope. This change is docs/proposal only: no
`.rs`, `.cpp`, `.h`, `CMakeLists.txt`, or asset change lands here, and no
dependency is added. The frozen interfaces live in
`docs/dev/m42-panel-refinements.md`; the per-panel menu research lives in
`docs/dev/m42-panel-menus.md`.

## Goals / Non-Goals

**Goals:**

- Give normal-mode panels a sensible minimum width and make the compact
  transition start at the smallest strip width.
- Enlarge the iconic-strip icons and the Tools (toolbox) icons.
- Make the floated Tools dock hug its content height and use only the width its
  content needs in one- and two-column modes.
- Diagnose and remove the widget drawn over the menu bar, and keep the row clear.
- Replace the `Qt::Tool` tear-off window with an in-window floating overlay.
- Let the compact strip be dragged to reorder and dropped into the group stack.
- Refine the compact flyout: inner-side placement, active/pressed open icon,
  group styling (tab header + content), and a header close button.
- Add a per-widget action button to each group tab header with the researched
  CS6 per-panel menus, disabled entries marked, and `Close`/`Close Panel Group`
  excluded.
- Add the CS6 foreground/background swap control and the `X` swap key.
- Ship runnable checks: the `m42_*` self-test steps at exit codes 131–139.

**Non-Goals:**

- Real content for the placeholder panels (Styles, Properties, Gradients,
  Patterns, Libraries) — they stay empty states.
- Pixel-exact CS6 metrics; the strip/tool icon sizes, the normal-mode minimum
  width, and the flyout offsets are chosen constants.
- Multi-monitor floating and OS-window float chrome: the M42 overlay is
  deliberately in-window and clipped, so a group cannot leave the main window.
- Implementing the disabled per-panel menu entries. The research for Gradients,
  Patterns, and Libraries is documented as a **non-goal** (those were CS6 picker
  pop-ups, not panels; Libraries is CC-only).
- A session-schema bump. Compact reorder writes the existing v5 `panelGroups`
  order; no new key is added.
- Any change to the tool catalogue, the implemented 10-tool set, the M40
  flyout/shortcut/dock contracts, or the M41 tab context menu item list.

## Decisions

### 1. Normal-mode minimum width, compact starts at the smallest width (frozen)

`PanelColumn` sets a bounded normal-mode minimum width (a chosen constant sized
to the tab-title/content hint, not a fixed pixel contract) so the splitter
handle cannot squeeze it to nothing. The existing no-height-minimum rule stays:
a short window scrolls rather than growing the main window. `setRailMode(true)`
records the current normal width and then sets the column to the smallest width
that shows the icon strip; `setRailMode(false)` restores the recorded width. The
M41 `m41_minwidth` check (code 121) is amended in place to assert the new width
floor instead of "no width floor", while keeping the scroll and height clauses.

*Alternative considered:* make the minimum width equal to a panel's `sizeHint`.
Rejected: panel hints are placeholder-sized and would over-constrain the window;
a small constant answers the "squeezed to nothing" complaint without dictating
the workspace.

### 2. Bigger strip and tool icons (frozen)

The strip button constant `kIconButtonSize`/`kIconPixmapSize`
(`panel_column.cpp:30–31`, currently 24/16) and the Tools slot button/icon sizes
(`toolbox.cpp:334–335`, currently 30/20) are raised to larger chosen values. The
icons ride the existing `icons.h` asset ids, so the change is a size constant,
not new art.

*Alternative considered:* a user-adjustable size. Rejected: not requested, and
one more persisted value for no observed need.

### 3. Tools dock hugs its content width and floated height (frozen)

Remove `layout->addStretch(1)` from the `Toolbox` body (`toolbox.cpp:404`) so
the body's size hint is the sum of its children; a floated dock then sizes to
that hint instead of filling the window height. `updateContentMetrics()` keeps
setting the minimum width from `contentWidth(columns_)`, and the body margins are
trimmed to the layout's own margins so neither the one- nor the two-column dock
carries extra horizontal space.

*Alternative considered:* set a maximum height while keeping the stretch.
Rejected: the stretch is the source of the padding; deleting it is the smaller
change and lets the layout compute the height.

### 4. Menu-bar overlay: discard stale layouts, keep the row clear (frozen)

The diagnosis order is (a) inspect `menuBar()->cornerWidget()` and every
`findChildren<QToolBar*>()` for one whose geometry intersects the menu bar's
global rect; (b) treat the persisted `state.layout` as the prime suspect —
`frame.cpp:113–115` calls `restoreState()` unconditionally, and M41 deleted the
M24 `PanelRail` toolbar, so a layout saved before M41 can restore a toolbar/dock
into a position that no longer matches the current chrome. The fix removes the
stray widget and discards a persisted layout whose schema/version no longer
matches the current chrome (falling back to the default arrangement), so nothing
is restored over the menu bar. The `m42_menubar` check asserts that only the
menu bar occupies the menu-bar row and that no other child widget's global
geometry intersects it.

*Alternative considered:* keep `restoreState` and patch only the named widget.
Rejected: the stale-layout path can reintroduce the ghost on the next run, so
the layout guard is the root-cause fix.

### 5. In-window floating overlay replaces `Qt::Tool` (frozen)

`PanelFloat` stops being a top-level `Qt::Tool` window and becomes a child
widget of the main window (or of the central widget) with a raised stacking
order. It is movable by its tab bar using the existing group-drag machinery,
clipped to the main window's rect, and re-docks into the `PanelColumn` when
dropped on it (the M41 `applyGroupDrop` path is reused). The overlay is not a
window, so it never appears in the task list and never opens a second app
window.

*Alternative considered:* keep the OS window and restyle it. Rejected: the user
explicitly chose an in-window overlay; an OS window is the complaint.

*Trade-off:* an overlay cannot be moved to another monitor and has no window
decorations. Recorded as a non-goal.

### 6. Compact-strip drag reuses the M41 drag machinery (frozen)

The strip icon buttons become drag sources that call the same
`beginPanelDrag`/`updateDrag`/`commitDrop` path the tab bar uses. Reordering a
strip icon writes the panel order back through `PanelGroup::setPanelOrder` (the
v5 `panelGroups` order), and dropping a strip icon on the normal-mode group
stack resolves through the existing `resolveDrop` group/boundary targets so the
panel joins the target group. No `QDrag`/MIME system is added.

*Alternative considered:* `QListWidget` internal move for the strip. Rejected: it
does not bridge to the group stack and would duplicate the drop resolver.

### 7. Compact flyout: inner side, active icon, group styling, header close (frozen)

`openIconFlyout` positions the `Qt::Popup` on the inner side of the column: for
a right-hand column the popup's right edge meets the strip's left edge, and for
a left-hand column the popup's left edge meets the strip's right edge (the
column is on the right today). The currently open icon button is checkable and
drawn pressed while its flyout is visible. The flyout is restyled as a group: a
header row whose left is the panel title (the tab) and whose right is a
close icon button (double right chevron, a new icon id or the existing chevron
reused); the body is the detached panel content. Reusing the one `PanelFlyout`
keeps a single flyout system.

*Alternative considered:* build a real `PanelGroup` inside the flyout. Rejected:
it would re-parent panels through the tab stack and complicate restore; a
styled header + content is the visual contract without the extra structure.

### 8. Per-widget header action button and menus (frozen)

`PanelGroup` adds a small context-action `QToolButton` at the far right of its
tab-bar row (a corner widget on the `QTabBar`/`QTabWidget`, or a button strip
beside it). Activating it shows the menu of the group's **current panel**
(`currentPanelName()`), so the menu is per-widget, not per-group. The menu
tables come from `docs/dev/m42-panel-menus.md`; each entry carries an implemented
flag. Unimplemented entries are `setEnabled(false)` with the tooltip
`<label> — not implemented yet` (the M38/M40 convention); submenu parents whose
children are all disabled are disabled too. `Close` and `Close Panel Group` are
not in the per-widget menu; they stay on the tab context menu. The Layers
`Panel Options…` entry invokes the existing `LayersPanel::openPanelOptions`
behaviour, so M39/M41 Panel Options keep working.

*Alternative considered:* a menu per group aggregating its panels. Rejected: the
user clarified the actions are per-widget.

### 9. Tools foreground/background swap and `X` (frozen)

`ForegroundBackgroundWidget` gains a swap affordance (a double-headed arrow
drawn between the two swatches, or a small adjacent button) that calls the
`ColorState` setters to exchange foreground and background. The frame wires `X`
to swap when no tool letter claims it (`X` is unassigned in the toolbox
catalogue), matching CS6. The existing corner reset (X) stays.

*Alternative considered:* a full CS6 swatch cloud (black/white default plus
swap plus reset as three micro-icons). Rejected: the reset already exists; only
the swap is missing.

## Risks / Trade-offs

- **The menu-bar overlay root cause is unconfirmed by a fresh screenshot.** →
  The diagnosis order is explicit and the fix removes the candidates *and*
  guards the stale-layout path; the `m42_menubar` check asserts the invariant
  (nothing intersects the menu bar), so a regression fails the self-test even if
  the original widget was never named.
- **The in-window overlay changes tear-off geometry semantics.** → The re-dock
  path is the existing `applyGroupDrop`; the overlay reuses `PanelFloat`'s group
  host and drag routing, so only the window flags/clipping change.
- **Researched menu ordering is best-effort.** → Adobe's Help groups entries by
  function rather than printing menu order; `docs/dev/m42-panel-menus.md` states
  this and tags each panel with its source. A screenshot can reorder entries
  later without a code structure change.
- **Most per-panel entries ship disabled.** → They are honest stubs under the
  existing tooltip convention; the menus are a chrome contract until the panel
  features land.
- **Icon-size and minimum-width constants are unsourced.** → Chosen constants,
  marked `ponytail:` where relevant, adjustable in one place.
- **Strip drag adds a second drag source.** → It calls the same begin/update/
  commit path; no new drop resolver, so the behaviour stays testable with the
  existing hooks.

## Migration Plan

Additive and app-local, except the tear-off container change. Rollback: restore
`PanelFloat`'s `Qt::Tool` flags, the `addStretch`, the M41 icon sizes, the
unconditional `restoreState`, the old flyout placement, and remove the header
button. No document, codec, bridge, compositor, PSD, session-schema, or on-disk
format change, so no data migration is needed. Sequence: freeze the brief and
the per-panel menu tables → Phase A chrome fixes → Phase B strip drag → Phase C
flyout refinement → Phase D per-widget menus → Phase E in-window overlay →
Phase F self-tests → STATE.

## Open Questions

- **Exact menu order per panel.** Resolved as best-effort; a CS6 screenshot can
  refine `docs/dev/m42-panel-menus.md` without code-structure change.
- **The precise stray widget over the menu bar.** The diagnosis order above
  names the candidates; if none is found, the stale-layout guard is still the
  fix and the self-test freezes the invariant.
- **Whether `X` should also be exposed as a Tools flyout entry.** CS6 documents
  the key; M42 wires the key and the swap control, and leaves the flyout
  unchanged.
- **Gradients, Patterns, Libraries menus.** Documented as non-goals; their
  placeholder panels keep no per-widget menu beyond disabled stubs.
