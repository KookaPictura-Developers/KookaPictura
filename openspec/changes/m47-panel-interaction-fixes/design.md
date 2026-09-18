## Context

M46 fixed drop-indicator geometry, the floating-Tools gesture, primary-column
removal, minimize collapse, and splitter non-collapsibility, but a second batch
of interaction defects remains. Root causes were traced through `panel_column.cpp`,
`panel_group.cpp`, `toolbox.cpp`, `frame.cpp`, and `theme.cpp`; the key ones are
recorded below so the fixes target the cause, not the symptom.

## Goals / Non-Goals

**Goals:** fix the nine reported interaction defects; add runnable self-tests
for each; keep the diff scoped to the existing widgets; no new dependencies.

**Non-Goals:** OS-window floats; multi-monitor chrome; CS6-exact pixel parity.

## Decisions

### D1 — Empty column is removed even with a live float
`frame.cpp removeColumnIfEmpty` early-returns while `floatCountForTest() > 0`,
so a torn-off last group keeps its empty source column. Decision: fold the
emptiness scan ahead of the float guard; add
`PanelColumn::rehomeFloatsTo(PanelColumn*)` that disconnects each float's group
from this column, rewires it to the target, moves the `PanelFloat*` between
`floats_` lists, and copies panel visibility. Dynamic columns re-home floats to
the primary then delete; the primary hides (object survives, so its floats stay
wired).

### D2 — Ghost groups are hidden at cleanup
`PanelGroup::takePanel` removes the last visible tab but does not update group
visibility, leaving a visible shell with an empty tab bar. Decision: centralise
on `PanelColumn::cleanupEmptyGroup`, which now hides (not deletes) a group whose
`titleCountForTest() > 0` but `visibleTitles().isEmpty()`, and calls
`maybeRemoveSelf()`. Broaden the source-cleanup predicates in `commitDrop` /
`cancelDrag` from `titleCountForTest()==0` to `visibleTitles().isEmpty()` so the
panel-drag source group is also cleaned.

### D3 — Compact icon drag survives the float creation
`createFloat` calls `buildIconStrip()` in rail mode, deleting the strip button
that holds the implicit mouse grab, so no further move/release arrives. Decision:
remove that rebuild from `createFloat`; `commitDrop`/`cancelDrag` already rebuild
the strip. Accept a stale strip row during the drag (`ponytail:` comment).

### D4 — Compact grip creates a group above
`resolveIconicDrop` classifies the group container (including the grip) as
`OnStrip` for panel drags. Decision: check each group's `panelIconGroupGrip`
rect before the container case and return `AboveGroup` at that group's index;
the existing commit already inserts a new one-panel group at the boundary.

### D5 — Stable shared width floor, no horizontal scroll
The column's minimum is a flat 180 while the scroll widget's content minimum can
be larger, so the `AsNeeded` bar appears and the right side clips. Decision:
compute a process-wide, monotonically-growing floor from
`splitter_->minimumSizeHint().width()` plus the vertical scrollbar extent and
frame, cap it (`kPanelMaxWidth`), and apply it to the **column** via
`setMinimumWidth` — never to `splitter_` (the earlier regression). Set the
horizontal policy to `ScrollBarAlwaysOff`; `minimumWidthFloorForTest` returns the
shared floor. A wider column is the intended fix for the reported clipping.

### D6 — Iconic column has a fixed width
A splitter handle drag resizes both neighbours; the iconic pane has no maximum,
so it grows/shrinks. Decision: while iconic, `setFixedWidth(kIconStripMinWidth)`
plus horizontal `QSizePolicy::Fixed`; on exit, clear the maximum
(`setMaximumWidth(QWIDGETSIZE_MAX)`) and restore the policy before recomputing
the normal floor. This supersedes the old "strip width is user-resizable"
wording for the docked strip.

### D7 — Compact chrome shades
`QWidget#panelIconGroup` uses `${base}` (darker than the pane); change to
`${window}`. Add `color: ${disabledText}` to `QWidget#panelIconGroupGrip` so the
`•••` dots are dark gray instead of inheriting near-white.

### D8 — Tools panel is a movable splitter pane placeable among columns
The title-bar gesture only arms while already floating, and the commit is gated
on `isFloating()` after a deferred timer, so a docked drag-out never reaches the
column grammar. Decision: arm on a title-bar press regardless of the float
state, emit move/release for the whole gesture, and commit without the
`isFloating()` re-check. `resolveToolboxDrop` gains a workspace-edge fallback and
treats the Tools pane itself as an anchor for its neighbouring column. The
Toolbox gains an explicit splitter-pane state so `updateContentMetrics` keeps a
fixed width (not a fixed height) while it is a pane.

### D9 — Widget drags cross the toolbar
`floatBounds` clamps the overlay to the central rect, so a floating widget
cannot pass over a docked toolbar. Decision: union the central rect with the
tools-dock rect (menu bar stays clear). Toolbar-side drops resolve to the
neighbouring column so the indicator and the landing slot agree.

### D10 — Floating widget close control
A float hosts a `PanelGroup` with only the per-widget `▾` corner button. Put a
close `QToolButton` in a container with the existing corner button (rightmost),
visible only while the group is hosted in a float. Closing re-homes the group
into the column, calls `closeGroup` (hide tabs, keep the group so
`Window > Panels` restores it), then destroys the overlay shell.

## Risks / Trade-offs

- [A process-wide growing floor widens all columns] → capped at `kPanelMaxWidth`
  and only grows to a content minimum; accepted ceiling.
- [Removing the strip rebuild in `createFloat` leaves a stale strip row during
  the drag] → cosmetic; rebuilt on commit/cancel.
- [Fixing the iconic width removes in-place label widening] → labels widen via
  `setIconStripWidthForTest` only; the spec wording is amended.
- [D8/D9 are the riskiest, touching the dock/splitter interplay] → the Toolbox
  keeps its dock features and only the pane state changes layout branching.
- [Ghost groups hidden rather than deleted could accumulate] → they are found by
  `groupForPanel` and re-shown by `Window > Panels`, matching `closeGroup`.

## Migration Plan

No data migration; session schema unchanged. Rollback is a revert.
