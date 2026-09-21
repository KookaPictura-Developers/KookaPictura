# Design

## Root causes

**Centring.** `Toolbox::updateContentMetrics()` sets the dock body's size policy
to `Fixed/Fixed`. Qt's `QDockWidgetLayout::setGeometry` puts the body into the
space below the title bar; the `QWidgetItem` for that body then runs
`QWidgetItem::setGeometry`, which shrinks the target rect to the widget's maximum
size (the size hint for a `Fixed` axis) and, when the item carries no `AlignTop`,
centres what is left:

```cpp
QSize s = r.size().boundedTo(maximumSize());
...
else if (!(align & Qt::AlignTop))
    y = y + (r.height() - s.height()) / 2;
```

While a docked panel was itself `setFixedHeight(contentH)` the gap was zero, so
the centring was invisible. Round 8 made a docked/pane dock free-height, so the
body is now shorter than the content rect and gets centred.

The fix is to let the body expand vertically when the dock is not floating, so
the `Fixed` size hint no longer caps it and the body's trailing stretch keeps the
slots at the top. Horizontal stays `Fixed` (the width lock).

## Outer-band preview

`resolveToolboxDrop` must keep declining the outer band so the release takes the
dock path (`dockToolbox`), and it must keep returning a null anchor and `side ==
-1` (tests 407/415 and `frame_test.cpp` assert this). But declining currently also
clears the shared indicator, so the user sees nothing.

`PanelColumn::updateColumnDrag` already has the pattern for a bare workspace
edge with no anchor: it approximates the line on the outermost visible column and
marks it `ponytail: visual-only anchor`. The Tools drop reuses exactly that: show
the outermost visible column's edge indicator on the band's side, store it in
`toolboxDropAnchor_`, and still return `false`. `commitToolboxDrop` must then
clear that visual-only anchor before it returns `false`, or a declined commit
leaves the line on screen.

## Floating

`resolveToolboxDrop` currently falls back to the horizontally nearest visible
column anywhere in the central area, so a release over the empty workspace always
becomes a pane. That makes it impossible to leave the panel floating by dragging
and contradicts the `tool-framework` "keep its current state" clause. Removing
that fallback lets the release handler's existing fall-through run: not a column,
not the outer band, inside the frame -> `floatToolboxAt(pos)`.

The order of checks matters: the outer band is tested before the column under
the pointer, so an edge-adjacent column cannot swallow the dock band.

## Ceilings

- The outer-band preview is anchored to the outermost visible column, so it is
  exact only while a column sits at the splitter extreme; with none, no line is
  drawn (the column drag has the same ceiling).
- No mouse-grab change: the drag still only reports during the gesture and the
  panel is (re)placed on release, so Qt's dock drag stays suppressed.
