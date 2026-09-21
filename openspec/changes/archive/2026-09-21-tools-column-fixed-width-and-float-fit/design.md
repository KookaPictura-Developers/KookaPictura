## Context

The Tools toolbar is an atomic `PanelColumn` (`toolsContent_`) hosted in the
central splitter. Three behaviours were specified but not enforced in code.

## Decisions

### Docked fixed width: disable the separator handle

`PanelColumn::updateMinimumWidth` already floors the tools column at
`max(kIconStripMinWidth, contentWidth)`, but `QSplitter` still offers the handle
between it and its neighbour, and dragging it resizes the toolbar. The spec
already says the separator drag "SHALL NOT resize it".

The handle is disabled in `reapplyColumnStretch`, which already walks the
splitter to set stretch factors and is called on every structural change
(create, move, remove, clear). The disabled state is a property of the
`QSplitterHandle`, so it must be re-applied after `applyPanelSession` re-inserts
the tools column — `insertWidget` creates fresh, enabled handles.

`QSplitter` still reserves the handle's width, so the layout is unchanged; only
the resize affordance is gone.

### Floating re-fit: reuse `syncToContent`

`PanelFloat::syncToContent` is the one "size the overlay to its content" path.
It bailed out for the tools column. It now snaps both axes to the content
minimum (width from `minimumWidth()`, height from `minimumSizeHint()`), matching
what `setResizable(false)` does at float time. `refreshToolsWidth` — the
`columnsChanged` handler — calls it when the column is floating instead of only
resizing a splitter pane.

### Edge mark above the overlay: a frame-level widget

The column-move / new-column mark used the column's viewport `indicator_`. A
following overlay is an in-window child of the frame raised above the splitter,
so on child-mode platforms it covers the line wherever the cursor is. The mark
is now a separate `edgeIndicator_` widget parented to the owning frame and
raised on every update, so it paints above the overlay. It is deleted with the
column (its QObject parent is the frame, which outlives the column).

The mark is suppressed on the atomic Tools column: the existing checks assert
that a widget drag over the tools body draws no line, and the tools column never
shows a widget drop line.

## Alternatives

- **Pin min == max on the tools column** instead of disabling the handle. This
  shrinks the column to its content width and changes the resolved drop geometry
  (the narrow column then falls inside the outer new-column band), breaking
  existing atomicity checks. Disabling the handle is the smaller, targeted fix.
- **Reorder the atomic check before the outer band** so no new column is offered
  over the tools column. This breaks the legitimate "new column left of Tools"
  drop, which the tests exercise. Keep the band and suppress only the mark.
- **Draw the mark inside the floating overlay** instead of at the target edge.
  This changes the established "the line is drawn in the column that owns the
  target" contract and would need every indicator check rewritten. The
  frame-level widget keeps the geometry and only fixes the stacking.
