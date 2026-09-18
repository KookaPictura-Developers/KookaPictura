## Context

Follow-up to `layers-panel-controls` / `layers-panel-chrome-fixes`. The percent
fields own no label (the label is a sibling `QLabel`), the lock/eye/chevron art
is either generic or barely visible, and the tree has `setDragEnabled(false)` so
a press-and-move starts a rubber-band selection. Reordering needs a reparent op
the bridge does not have.

## Decisions

### D1 — `PercentField` owns its label and `%`
`PercentField(const QString& label, QWidget* parent)` builds
`[QLabel label][QLineEdit][QLabel "%"][arrow]`. The scrub event filter is
installed on the label and the `%` label as well as the edit, so dragging the
label changes the value. `labelText()` exposes the label; the edit still holds
the bare number and the `%` is a suffix label.

### D2 — Semantic lock icons
Only `layers.lockAll` is a padlock. `layers.lockAlpha` is a transparency
checkerboard, `layers.lockPaint` a brush, `layers.lockPosition` a move cross,
`layers.lockNesting` nested squares. Asset names unchanged, so no code change.

### D3 — Eye and chevron as icons
`layers.eyeOn`/`layers.eyeOff` are drawn by the delegate at
`rect.left() + kEyeInset` (`kEyeInset = 6`), size `min(18, rowHeight)`; `eyeRect`
matches. The group disclosure is `layers.disclosureRight`/`layers.disclosureDown`
drawn in the reserved chevron slot; the painted `paintChevron` is replaced by
these pixmaps (null-safe).

### D4 — Kind buttons carry icons
Each Kind toggle gets `layers.kindPixel`/`kindAdjustment`/`kindGroup`/
`kindBackground`, `ToolButtonIconOnly`, `iconSize 16`, tooltip = the kind name.

### D5 — Drag and drop
`LayersTreeView` enables drag/drop, overrides `startDrag` to build a
`QDrag` carrying the dragged path in `application/x-pictura-layer`, and
overrides `dragEnterEvent`/`dragMoveEvent`/`dropEvent`. On drop it resolves the
row under the cursor to a target path and a mode (`0` above, `1` below, `2`
into a group) and calls a panel-supplied handler. Only the dragged row (the
current index) moves, so a drag never extends the selection. The panel's
handler calls `view_->move_layer_to(path, target, mode)`.

Engine (`properties.rs::move_path_to`): clone-and-remove the source, compute the
destination container/index from the target and mode (Into appends to the
target group's children; Above inserts one index above; Below at the target
index), decrement the destination index when it shares the source's container
and sits above the removed node, then insert. Refuses the Background, a
fully-locked or nesting-locked source, a malformed/empty target other than the
root, `target == path`, and a target inside the source. The bridge
`move_layer_to` recomposites and records one `"Move Layer"` state.

Bottom-strip drop: each strip button is a drop target with a
`layerDropAction` property; the panel's event filter accepts the layer MIME and
runs the button's action on the dragged paths (`delete_layers`,
`duplicate_layers`, `group_layers`; mask/link/fx stay inert until their engine
lands).

## Risks / Trade-offs

- **Single-row drag** is the scope; a multi-selection drag moves only the
  current row (`// ponytail:` note). Multi-drag needs path re-sequencing.
- **Reparent index math** is the hazard; the engine test covers above/below/
  into, same-container adjustment, self-drop, and descendant-drop refusal.
- Dropping on a strip button needs the MIME accepted by a `QToolButton` the
  panel already owns, so the existing panel event filter is extended rather than
  adding a new drop widget.
