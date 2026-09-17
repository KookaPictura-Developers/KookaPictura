## Context

M38 (archived) rebuilt `crates/pictura-app/cpp/toolbox.{h,cpp}` as one
`QToolButton` per group (23 slots) in catalogue order
(`toolbox.cpp:213–268`), with a private `ToolSlotButton` that holds a 350 ms
`QTimer` to open the menu (`toolbox.cpp:31–88`), `MenuButtonPopup` for the corner
marker (`toolbox.cpp:234`), a single `QVBoxLayout` column, and
`setMinimumWidth(66)` (`toolbox.cpp:290`). `refreshSlot()` gives each implemented
slot a `setShortcut()` except Brush/Pencil (`toolbox.cpp:334–338`).
`PicturaMainWindow::buildTools()` (`frame.cpp:870–950`) hard-codes the only
group-cycling shortcut, the `B`/`Shift+B` `cyclePaintTool` lambda
(`frame.cpp:884–893`); `registerPanel()` (`frame.cpp:163–171`) adds every dock to
an area and the frame uses `AnimatedDocks | AllowTabbedDocks` (`frame.cpp:72`).
`session.{h,cpp}` is at schema v3 (`session.h:10–19`); the M39 fix makes
`saveSession()` load before writing so new fields are not clobbered.

`docs/02-ui-ux/toolbox-and-options-bar.md` (`UI-004`) is the CS6 toolbox contract:
one column by default with a top double arrow to two columns, a lower-right
hidden-tool triangle, hold/`Alt`-click, `Shift`+letter cycling gated by
`Use Shift Key For Tool Switch`. `docs/02-ui-ux/preferences.md`
(`UI-010`, General pane, line 78) fixes that preference's default to **on**.
`docs/02-ui-ux/workspace-and-docks.md` (`UI-003`) governs docks; the Tools panel
is special (it cannot be docked bottom or grouped in tab groups — the teachucomp
CS6 source). The frozen M40 interfaces live in `docs/dev/m40-tools-panel.md`.

Constraint: docs/proposal only in this change. No `.rs`, `.cpp`, `.h`,
`CMakeLists.txt`, or asset change lands here; the Rust bridge, document model,
compositor, and PSD I/O are untouched, and no dependency is added.

## Goals / Non-Goals

**Goals:**

- Freeze the flyout indicator, opening, and key-display contracts and the
  deterministic test hook.
- Freeze the one/two-column toggle (custom title bar, icons, reflow rule,
  persistence).
- Freeze the standalone-dock constraints and the tabify-refusal fallback.
- Replace the Brush/Pencil special case with a generic per-group `Shift`+letter
  contract gated by a session preference, and extend the session store to v4.
- Reconcile the canonical `tool-framework` and `application-shell` specs so
  `openspec/specs/` is consistent after archive.
- Ship runnable checks: the `m40_*` C++ self-test steps with fresh exit codes.

**Non-Goals:**

- The panel dock/rail refactor, compact/iconic docks, popup flyouts for panels,
  and `Auto-Collapse Iconic Panels` (M41).
- The Preferences dialog (General + Interface panes) and the UI for the new
  preference (M41); M40 persists the key only.
- Temporary (spring-loaded) tool switching, tool-preset pickers, and any change
  to the 71-tool catalogue or the 10-tool implemented set.
- A visual pixel-exact match of the CS6 triangle or title-bar metrics.

## Decisions

### 1. The flyout indicator is custom paint, not `MenuButtonPopup` (frozen)

`ToolSlotButton::paintEvent` calls `QToolButton::paintEvent` and then paints a
small filled triangle in the button's lower-right corner when the slot has two or
more group members (including all-unimplemented groups). The icon rect stays
centred; the triangle is a ~5 px right-triangle with the same foreground as the
button palette. `setPopupMode(QToolButton::MenuButtonPopup)` is removed, so no
stock arrow or reserved arrow width remains.

*Alternative considered:* keep `MenuButtonPopup` so Qt draws the marker.
Rejected: its arrow is not CS6's lower-right triangle, it reserves layout width,
and the arrow column is also a click target with semantics we do not want.

### 2. Flyout opens on hold past ~300 ms or right-click, below the button (frozen)

The hold timer interval becomes **300 ms** (was 350). On timeout the slot opens
its `QMenu` with `menu->popup(button->mapToGlobal(QPoint(0, button->height())))`
(clamped to `screen()->availableGeometry()`), so the menu sits below the button.
Right-click opens immediately on press. A left release before the timeout stops
the timer and runs the normal tool activation; it never opens the menu. A
single-member slot has no menu and shows no triangle.

*Alternative considered:* `QToolButton::DelayedPopup` (used by M38's open
question). Rejected: it pops the menu on press with no release handling and gives
no control over menu position or the triangle.

### 3. Flyout keys are displayed, not registered (frozen)

Each flyout `QAction` gets `setShortcut(slotLetter)`,
`setShortcutVisibleInContextMenu(true)`, and
`setShortcutContext(Qt::WidgetWithChildrenShortcut)` while the action is a child
of the flyout `QMenu`. This shows the key right-aligned on every item, including
disabled items. Qt skips disabled actions and only scopes the shortcut to the
open popup; **no window-global shortcut is registered by a flyout item and the
slot button no longer calls `setShortcut()` at all**, so a group's shared letter
(including on a disabled member) cannot steal the key from the frame's generic
handler. The plain letter is owned by exactly one `QShortcut` in the frame.

*Alternative considered:* an explicit key column via `QWidgetAction`. Rejected:
it re-implements menu row painting, disabled styling, hover, and keyboard
navigation that the stock `QMenu` already does; the scoped-shortcut form meets
the "display, do not steal" contract with less code. If a future Qt change makes
menu-action shortcuts window-global, this is the fallback.

### 4. One/two columns live behind a custom title bar (frozen)

`Toolbox` installs a custom title bar with `setTitleBarWidget()`: a `Tools` label
and a flat double-arrow `QToolButton`. The button shows the icon for the layout
it switches **to** — `panel.columnsTwo` while in one column, `panel.columnsOne`
while in two. Toggling re-adds the 23 slot widgets to the `QGridLayout`:
one column at `(i, 0)`; two columns **row-major** at `(i / 2, i % 2)`. Two
columns widen the dock (a two-column minimum of two button widths plus margins);
one column restores the 66 px minimum. The foreground/background widget and the
Screen Mode button stay after the grid in the vertical layout, so they are pinned
at the bottom in both layouts.

*Alternative considered:* `QToolBox`/two stacked containers. Rejected: a single
`QGridLayout` with a recomputed position is the smallest reflow and keeps the
existing button objects and `slotButtons()`.

*Decided where CS6 is silent:* the two-column reading order (row-major) and that
the button shows the target rather than the current layout. Both are recorded in
the brief's open questions for a screenshot pass.

### 5. The Tools panel is a standalone dock with a reactive tabify fallback (frozen)

`Toolbox` sets `setAllowedAreas(Qt::LeftDockWidgetArea | Qt::RightDockWidgetArea)`
and `setFeatures(Movable | Floatable | Closable)`. Because Qt offers no clean
"veto tabification" API (there is no per-dock `setTabbable(false)`), the frame
installs an event filter that rejects a drag/drop whose target is a dock tab bar,
and connects `QDockWidget::dockLocationChanged`: if
`tabifiedDockWidgets(toolbox)` is non-empty after a change, the frame calls
`setFloating(true)` and re-adds the dock to its previous side. The custom title
bar stays draggable so move/float still work; the `Window > Panels > Tools`
toggle re-shows the panel after a close.

*Alternative considered:* `setDockNestingEnabled(false)` + disabling tabbed docks
globally. Rejected: the rest of the workspace depends on `AllowTabbedDocks`
(`application-shell`'s default grouping), so the restriction must be per-dock.

### 6. One generic per-group letter handler replaces the Brush/Pencil case (frozen)

`frame.cpp`'s `cyclePaintTool` lambda and its two `QShortcut`s (`B`, `Shift+B`)
are deleted. Instead a helper resolves a key letter to its (unique) group from
the catalogue, and two shortcuts per distinct letter are registered: the plain
letter and `Shift`+letter. With `useShiftKeyForToolSwitch` **on** (default), the
plain letter activates the slot's current member and `Shift`+letter calls the
toolbox's `cycleGroup()` (implemented members only, wrapping); with the
preference **off**, the plain letter calls `cycleGroup()`. A group with no
implemented member does nothing. The unlettered Blur/Sharpen/Smudge slot is
unaffected (no letter). `refreshSlot()` no longer sets a per-button shortcut, so
the frame owns every letter.

*Alternative considered:* keep per-slot `setShortcut()` and add shift handling in
the button. Rejected: it leaves two owners for one key, keeps the Brush/Pencil
special case, and cannot express "plain selects, Shift cycles" uniformly.

### 7. Session schema v4 (frozen)

`SessionState` gains `int toolsColumns = 1;` and `bool useShiftKeyForToolSwitch =
true;`, with `schemaVersion = 4`. `loadSession()` reads both with those defaults;
`saveSession()` writes them. The M39 load-before-write path is unchanged, so the
existing v3 fields still round-trip. The `Toolbox` receives its initial column
count from the frame after `loadSession()`; the frame persists
`tools->columns()` on save. **No Preferences dialog in M40**; the preference is
session-persisted only and M41 gives it a General-pane checkbox.

### 8. Test hooks (frozen)

`Toolbox` exposes: `int columns() const` / `void setColumns(int)` (1 or 2),
`void openSlotFlyoutForTest(int group)`, `QMenu* slotMenuForTest(int group)`,
`QList<QAction*> slotMenuActionsForTest(int group)`, `void cycleGroupForTest(int
group)`, `QToolButton* titleBarToggleForTest()`, and `bool
hasFlyoutTriangleForTest(int group)` (paints the button to a pixmap and samples
the corner, or answers from the same predicate the paint uses). `frame` exposes
`void handleToolShortcutForTest(const QKeySequence&, bool shift)` or the generic
handler directly. These keep the self-test deterministic without faking click
timing.

## Risks / Trade-offs

- **No tabify-veto API.** → The event filter plus `dockLocationChanged` fallback
  is a best-effort veto; a drop could briefly tabify before the fallback
  re-docks. Recorded as a known limitation in the brief; the fallback restores
  left/right in all cases.
- **Custom title bar loses the default float/close buttons.** → The dock is
  `Movable | Floatable | Closable` and `Window > Panels > Tools` re-shows it; the
  bar is draggable. No float/close glyphs are drawn in M40.
- **Scoped flyout shortcuts rely on Qt's context semantics.** → The design sets
  `Qt::WidgetWithChildrenShortcut` and never registers a window-global shortcut;
  the self-test proves a disabled item's shared key does not change the active
  tool and that the plain letter still reaches the frame handler.
- **Empty/stale stores.** → v4 reads both fields with defaults; an older or
  missing field loads 1 / true and the existing fields are untouched.
- **The two-column order and icon direction are unsourced.** → Decided
  (row-major; target-layout icon) and recorded; either is a small change with no
  interface impact.
- **`panel.columnsOne`/`panel.columnsTwo` assets do not exist yet.** → The asset
  task adds them and the qrc entries; the toggle falls back to a text arrow if an
  icon is null (guarded, like M38's badge fallback).

## Migration Plan

Additive and app-local, except the `Toolbox` rewrite and the `frame.cpp` handler
replacement. Rollback: restore the `MenuButtonPopup` slot, the 350 ms timer, the
single-column layout, the `B`/`Shift+B` case, and drop the two session fields.
No document, codec, bridge, or compositor change, so no on-disk migration is
needed. Sequence: freeze the brief → the two column icons + qrc →
triangle/hold/right-click/keys + hooks → title bar + toggle → dock constraints +
tabify fallback → generic shortcut handler + preference + session v4 →
self-test → STATE.

## Open Questions

- **Two-column reading order.** CS6 fills the two-column toolbox left-to-right
  top-to-bottom, top-to-bottom left-to-right, or in a fixed pairing is not
  captured in the sources; this design picks row-major. A CS6 screenshot would
  settle it.
- **Toggle icon direction.** Whether CS6 shows the current or the target layout
  on the double arrow is unverified; this design shows the target.
- **Hold delay.** The sources say "hold"; the exact timeout (this design: 300 ms)
  is a chosen constant.
- **Title-bar content.** Whether CS6's Tools title bar carries a panel menu or a
  float/close control beyond the double arrow is unsourced; M40 draws only the
  label and the double arrow.
- **Per-workspace vs global column choice.** CS6 workspaces remember panel
  arrangement; M40 stores one global `toolsColumns` in the session (matching
  M39's Panel Options), not per workspace.
