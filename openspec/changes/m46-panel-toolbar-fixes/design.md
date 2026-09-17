## Context

M45 landed an attempted fix for the widget-panel and Tools-toolbar reports as
`openspec/changes/archive/2026-09-17-m45-panel-fixes`. The source at HEAD
(`fd94128`) contains those attempts, and the running `build/pictura` is built
from that source, yet the user still observes the defects. The M45 C++ self-tests
run offscreen and either call test helpers that bypass the gesture under test or
assert a value the code itself just computed, so they pass while the UI is wrong.

This design records the verified root causes (traced through
`panel_column.cpp`, `panel_group.cpp`, `toolbox.cpp`, `frame.cpp`) and the
minimal decision for each. It also changes two established invariants: the
primary widget column is no longer exempt from empty-column removal, and the
central splitter is no longer child-collapsible.

## Goals / Non-Goals

**Goals:**

- Every reported drop-indicator, toolbar, empty-column, minimize, sizing, and
  compact-popup defect is fixed at its root, not at the tested helper.
- Self-tests exercise the real mouse/geometry path and fail if the behavior
  regresses.
- Keep the diff scoped to the existing Qt widget classes; no new dependencies,
  no new abstraction layers.

**Non-Goals:**

- CS6-exact pixel parity for indicator bands (there is no oracle).
- Reworking the M43/M44/M45 drop grammar, session schema, or panel model.
- Any GPU/render, codec, or document behavior.

## Decisions

### D1 — Tab insertion geometry from visible tabs only

`PanelGroup::tabInsertionX`/`tabInsertionIndexAt` iterate all tabs and read
`QTabBar::tabRect`, which is empty for a hidden tab. A group with a hidden tab
therefore returns `0` from `tabInsertionX` (line drawn at the far left) and
`count` from `tabInsertionIndexAt` (tab inserted far right). Decision: iterate
only visible tabs and map an insertion index into that visible space, falling
back to the nearest visible tab / bar edge. This fixes both the "right drop
shows left" and "rightmost tab shows leftmost" reports with one change.

*Alternative considered:* clamp `tabInsertionX` to `[0, bar->width()]`. Rejected:
it hides the mismatch between index and x, so the line would still not match the
committed slot.

### D2 — Cross-column delegation accepts all non-outside targets

`PanelColumn::resolveDrop` delegates to another column only when the delegated
kind is `IntoGroup`, so body/boundary drops over a sibling column stay
`outside` and clear the indicator while the commit still moves the widget.
Decision: accept any `valid && !outside` delegated target and carry
`owner = other`. `Reorder` cannot occur cross-column because the dragged group
lives in the source column.

### D3 — Boundary indicator clamped to the scroll viewport

The boundary branch computes `y` just past the last group's bottom; with the
group stretched to the viewport, `y == height + 1` and the 3 px widget is
entirely outside. Decision: `y = qBound(0, y, qMax(0, viewport->height() - 3))`.

### D4 — New-column indicator owned by the edge column

`resolveDrop`'s workspace-edge branch sets `owner = this` (the drag source),
but `applyNewColumnDrop` places the new column at the workspace end. Decision:
resolve the edge-adjacent column as owner (or run `columnEdgeAnchorAt` before
`newColumnSideAt`) so the line and the landing slot are the same edge.

### D5 — Floating toolbar gesture observed on the dock, not the title bar

Qt's `QDockWidget` drag calls `grabMouse()` after the first move, so subsequent
moves/releases are delivered to the `QDockWidget`, never to the custom
`titleBar_`. `Toolbox::eventFilter` only emitted for `watched == titleBar_`, so
the resolve/commit path was dead in the real UI. Decision: record a press-origin
flag on `MouseButtonPress` over `titleBar_`, and accept `watched == this` for
move/release while floating; clear the flag on release. The existing test that
drives the resolver directly stays but is labelled as not covering the gesture,
and a real-event test is added.

### D6 — Toolbar resolver falls back to the column under the pointer

`resolveToolboxDrop` only consults `columnEdgeAnchorAt`, a ±26 px band strictly
outside a column, so hovering a column interior resolves nothing. Decision: on
miss, use `columnAtGlobal` and the pointer's half of that column to choose
`PanelSide`, then reuse `showEdgeDropIndicator`. `commitToolboxDrop` already
inserts by splitter index, so "any side of any column regardless of where
columns are docked" needs no new grammar.

### D7 — Shared, non-collapsible width floor

`QSplitter::childrenCollapsible` defaults true, so a handle drag collapses a pane
to zero even with `minimumWidth` set. Decision: call
`setChildrenCollapsible(false)` on both the central horizontal splitter and the
vertical group splitter that hosts groups.

The separate right-side clipping report (content wider than a narrow column
being hidden rather than scrollable) is **deferred**: forcing the scroll
content's minimum width to the group's content width made the column itself
widen instead of engaging the horizontal scrollbar, which violated the shared
floor and regressed the M43 column geometry tests. A safe fix needs the group
content to opt into horizontal scrolling at the group level and will be handled
once it can be reproduced on a real session without moving the column's own
minimum. The existing `ScrollBarAsNeeded`, tab elision, and header-corner
reservation stay in place.

### D8 — Minimize clamps minimum height too

`applyMinimize` clamps only maximum heights; Qt will not shrink a widget below
its layout minimum, so groups whose content minimum exceeds the tab bar stay
expanded. Decision: save/clamp/restore `minimumHeight` symmetric to the maxima.
The menu label is already state-dependent (`Expand Panel`) and only needs
verification.

### D9 — Primary column participates in empty removal

`removeColumnIfEmpty` exempts `panelColumn_`, and a live float blocks removal.
Decision: remove the primary exemption (CS6 behavior), rehome a live float's
wiring instead of bailing, and call `maybeRemoveSelf()` from `destroyFloat` and
`cleanupEmptyGroup` so non-drag paths re-test emptiness. Because the shell
assumes a primary column exists, keep the primary as a persistent host object
that is hidden rather than destroyed, and re-show/re-host it on the next drop;
if that proves to need a broad refactor, stop and report instead.

## Risks / Trade-offs

- [Removing the primary column breaks shell assumptions] → keep a persistent
  primary host, hide it when empty, re-host on next drop; add a self-test that
  closes all panels then drops a widget again.
- [Clamping min height makes headers clip in an exotic theme] → restore the
  saved minima on expand; test with a tall-content group (Color/Swatches).
- [Accepting all cross-column targets could allow an invalid move] → only
  `valid && !outside` is accepted; `Reorder` is impossible cross-column.
- [Real-event tests in offscreen Qt can be flaky] → assert geometry after
  `processEvents`, not timing; keep the helper-driven tests for regression
  detail.
- [Primary export still uses M45 tests as coverage] → relabel/bypass-proof the
  new tests so a future change cannot pass by calling the resolver directly.

## Migration Plan

No data migration. Session schema is unchanged. Rollback is a revert of the
commit; no persisted state depends on the new behavior.
