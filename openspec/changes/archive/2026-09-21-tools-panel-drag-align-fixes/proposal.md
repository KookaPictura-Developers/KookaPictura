# Tools panel drag, align, and float fixes

## Why

Three regressions in the Tools panel:

1. **Content vertically centred.** A docked or pane-hosted Tools panel must fill
   the height it is given, but its content is centred in the dock instead of
   starting at the top. Round 8 made a docked/pane dock free-height; the body
   widget is still `Fixed` vertically, and Qt's `QWidgetItem::setGeometry`
   centres a non-expanding child when the item is shorter than the content
   rect.
2. **Outer-edge drag preview lost.** Dragging the panel to the workspace left or
   right outer band shows no drop preview. The band is a dock target, and the
   resolve declines it while clearing the shared edge indicator, so the user
   gets no feedback before the release docks the panel.
3. **Cannot float.** A release over the empty workspace snaps the panel to the
   nearest widget column (a pane) instead of floating, so a floating panel
   cannot stay floating and a docked panel cannot be floated by dragging. This
   also contradicts `tool-framework`'s "when no column is under the pointer the
   panel SHALL keep its current state".

## What Changes

- A docked or pane-hosted Tools panel's content fills the dock height and is
  top-aligned; only a floating panel keeps its fixed content height.
- The workspace outer band shows the shared blue edge indicator as a
  visual-only drop preview while the pointer is in the band; the release still
  docks the panel to that side.
- A title-bar release with no widget column under the pointer and outside the
  outer band floats the panel at the cursor instead of snapping to the nearest
  column.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-framework`: the title-bar drag's release fall-through and the outer-band
  preview (the "Tools panel" requirement's no-column clause).
- `application-shell`: the docked/pane-hosted Tools panel's content alignment.

## Impact

- **C++ app**: `toolbox.cpp`, `frame_columns.cpp`, `selftest_shell_round3.cpp`.
- **No document-format change, no new dependency, no `docs/` edit.**
