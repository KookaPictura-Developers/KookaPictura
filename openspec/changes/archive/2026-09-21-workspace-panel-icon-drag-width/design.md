# Design — workspace panel icon mode, drop reach, and minimum width

## Context

Panels are `PanelColumn` panes of a central `QSplitter`; each hosts a stack of
`PanelGroup`s. A group can be torn off into a `PanelFloat` overlay, and can be
collapsed to an icon row. The Tools panel is an atomic `PanelColumn`
(`toolsContent_`) hosted in the same splitter. Drag/drop runs one grammar:
`beginPanelDrag`/`beginGroupDrag` → `updateDrag`/`resolveDrop` →
`showIndicatorFor` → `commitDrop`.

## Decisions

### 1. The drop mark beside the Tools column

`resolveDrop` resolves, in order: a float overlay under the pointer, the iconic
strip, the outer workspace-edge band (`newColumnSideAt`), a band beside a
column (`columnEdgeAnchorAt`), then the atomic Tools guard, then the local
grammar. `showColumnEdgeIndicator` returned early for `isToolsColumn()`, so a
target *beside* the Tools column drew nothing.

The atomic rule is "the Tools column never grows groups and a drag over its
body resolves no target". That is narrower than "never draw a mark beside it".
The suppression moves from the drawing method to the resolution: the outer
workspace band whose owner is the Tools column keeps its mark suppressed (the
existing checks 423/424 drop a panel on the Tools column centre, which lies
inside the 28px outer band and must stay mark-free), while a target with an
`anchorColumn` — a new column beside the Tools column, resolved by
`columnEdgeAnchorAt` — draws the mark like any other column.

This keeps `newColumnDropForTest("tools")` (the point is just outside the Tools
column, so it still resolves the outer band and commits a sibling column) and
the atomic body checks green, and fixes the visible "no line beside the
toolbar" defect.

### 2. Floating icon mode: shrink-wrap, no grip, vertical icons, draggable

`pictura::PanelGroup::setCollapsedToIcons` shows `iconRow_` and hides the tabs.
`PanelFloat::syncToContent` already snaps the collapsed height; it now also
drops the overlay's minimum width to the icon row's own and resizes both axes,
and `setResizable`-style grip visibility is gated on the collapsed state so the
diagonal grip disappears in icon mode and returns on expand.

`rebuildIconRow` puts the buttons in `iconRowLayout_`. It is changed from a
`QHBoxLayout` to a `QVBoxLayout`, so the floating icon row stacks one icon per
row exactly like the docked strip's vertical column. That also makes the
shrink-wrapped width one icon wide.

`makeIconButton` currently only connects `clicked`. It gains the same
press/move/release gesture the docked strip's `panelIcon_*` buttons use, so an
icon in a floating row starts a panel drag through
`tabDragStarted`/`dragMoved`/`dragFinished` — the same signals the tab bar and
grip already emit, wired by `PanelColumn::wireGroup`. A click below the drag
threshold still emits `panelActivated` and opens the flyout.

### 3. Group-on-group outline for body drops

`updateDrag` sets `groupTabify` only when the target is `onTabBar`. A
whole-group drag over another group's body resolves `AboveGroup`/`BelowGroup`,
so only the thin boundary line shows. The rule becomes: a whole-group drag whose
resolved target group is not the dragged group outlines that group, whether the
pointer is on the tab bar or the body, and the drop merges. A drop on the
dragged group's own body keeps the above/below reorder boundary, so in-column
reordering is unchanged. `resolveLocalDrop` returns the target group with
`onTabBar` semantics for the body case so `applyGroupDrop` takes the existing
`mergeGroupInto` path.

### 4. Layers control row proportions

The blend combo is added with a stretch factor of 1 and the Opacity percent
field with none, so the combo absorbs the whole row. The row is changed to split
the slack between the blend combo and the opacity field (the combo still takes
the larger share, matching CS6), so the blend input no longer dominates.

### 5. Shared minimum width 300

`kPanelMinWidth` (and the `kMinNormalWidth`/`kDefaultNormalWidth` bounds derived
from it), plus the floating overlay's `kFloatMinWidth`, rise from 180 to 300.
The floor stays shared and content-raised, capped at `kMaxNormalWidth` (400), so
existing checks that only assert `180 <= floor <= 400` and `iconic < normal`
still hold.

## Alternatives

- **Remove the Tools-column mark suppression entirely.** The Tools column is
  ~40px wide with a 28px outer band, so a panel dropped on its centre resolves
  the outer band and would draw a mark, breaking checks 423/424 and the
  `fp_edge_mark_not_on_tools` contract. Suppressing only the outer-band case
  keeps the atomic rule and fixes the beside-column case.
- **Give the floating icon row a fixed one-icon size instead of shrink-wrap.**
  The overlay already has a `syncToContent` path; extending it is smaller than a
  second sizing rule.
- **Make the icons drag by starting the drag from the group grip only.** The
  grip drags the whole group; the report asks for the individual widgets to be
  draggable, which is the docked strip's existing `panelIcon_*` behaviour, so
  the floating row reuses it rather than inventing a new gesture.

## Risks

- The Tools-column mark rule is delicate: the implementation must assert both
  the beside-column mark and the unchanged atomic body case.
- Changing the icon row to a vertical layout changes the collapsed overlay's
  size hint; the existing `float_icon_width_snap` (436) assertion is an upper
  bound, so it stays valid, but the new size must be checked on both axes.
