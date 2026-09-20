## MODIFIED Requirements

### Requirement: Hand and Zoom tools

The system SHALL provide Hand and Zoom as first-class tools. Hand SHALL pan the
canvas on drag; Zoom SHALL magnify on click and reduce on modified click. The
canvas offset SHALL be clamped by one shared range helper at every mutation point
— pan, zoom anchoring, centre, fit, actual pixels, initial view, resize, and the
Navigator proxy — so that at least one display inch of 96 logical pixels of the
canvas remains visible on each axis whenever the document is larger than that
axis. The clamp SHALL slide each axis by the minimum needed rather than
recentering, and the canvas SHALL remain the single source of the offset.
**Zoom SHALL target the pointer: the Zoom tool's click SHALL anchor the zoom at
the clicked image point rather than the canvas centre, and the Navigator slider
SHALL anchor at the click/cursor point on the proxy, so the point under the
cursor stays fixed. The wheel SHALL apply one shared modifier precedence: a
horizontal side-wheel (`angleDelta().x() != 0`) SHALL pan horizontally; a
`Ctrl+Alt` vertical wheel SHALL pan vertically; an `Alt` (without `Ctrl`)
vertical wheel SHALL pan horizontally; otherwise the vertical wheel SHALL zoom
at the cursor, with `Shift` doubling the wheel zoom step.**

#### Scenario: Zoom in by click

- **WHEN** the Zoom tool is clicked on the canvas
- **THEN** the canvas magnification increases

#### Scenario: Zoom out by modified click

- **WHEN** the Zoom tool is clicked while a modifier is held
- **THEN** the canvas magnification decreases

#### Scenario: Zoom click targets the cursor [lct_zoom_cursor]

- **WHEN** the Zoom tool clicks a point away from the canvas centre
- **THEN** the zoom is anchored at that point so it stays under the cursor rather
  than the view zooming about the centre

#### Scenario: Shift doubles the wheel step [lct_wheel_shift]

- **WHEN** the vertical wheel is scrolled with Shift held
- **THEN** the zoom step is twice the unmodified step, still anchored at the
  cursor

#### Scenario: Alt pans horizontally [lct_wheel_alt_pan]

- **WHEN** the vertical wheel is scrolled with Alt held (without Ctrl)
- **THEN** the canvas pans horizontally and does not zoom

#### Scenario: Ctrl+Alt pans vertically [lct_wheel_ctrl_alt_pan]

- **WHEN** the vertical wheel is scrolled with Ctrl+Alt held
- **THEN** the canvas pans vertically and does not zoom

#### Scenario: A side-wheel pans horizontally [lct_wheel_side_pan]

- **WHEN** a horizontal side-wheel event arrives
- **THEN** the canvas pans horizontally and does not zoom

#### Scenario: Panning cannot push the canvas fully off-screen [lct_pan_margin]

- **WHEN** the canvas is panned in any direction past the reveal margin
- **THEN** the offset is clamped so at least 96 logical pixels of the canvas stay
  visible on that axis and the canvas does not jump to the other side

#### Scenario: Zoom anchoring is clamped [lct_zoom_clamp]

- **WHEN** a zoom-out at a corner would place the canvas outside the reveal margin
- **THEN** the resulting offset is clamped by the same shared range helper

#### Scenario: Fit and centre remain fully visible [lct_fit_within]

- **WHEN** Fit on Screen or centre is applied
- **THEN** the resulting offset is inside the shared range and the canvas is not
  pushed off-screen

## ADDED Requirements

### Requirement: Keyboard nudge of the active layer

With the Move tool active, a plain arrow key SHALL translate the active layer by
1 document pixel on that axis and `Shift+Arrow` SHALL translate it by 10 pixels,
through the existing `translate_layer` path, recording one history state per
nudge and refreshing the display. The nudge SHALL be refused, leaving the
document unchanged with no history state, for a position-locked layer, a
Background layer, a group or adjustment layer, or when there is no active
document or active layer. The nudge SHALL use the same translate path the mouse
drag uses so the two cannot disagree; no separate nudge handler exists today and
none may be added that bypasses `translate_layer`.

#### Scenario: Plain arrow nudges one pixel [lct_nudge_one]

- **WHEN** the Move tool is active with a translatable active layer and an arrow
  key is pressed
- **THEN** the layer translates by one pixel on that axis and one history state
  is recorded

#### Scenario: Shift arrow nudges ten pixels [lct_nudge_ten]

- **WHEN** `Shift+Arrow` is pressed with the Move tool active
- **THEN** the layer translates by ten pixels on that axis

#### Scenario: A refused nudge changes nothing [lct_nudge_refused]

- **WHEN** an arrow key is pressed on a position-locked, Background, group, or
  adjustment layer, or with no active layer
- **THEN** the document is unchanged and no history state is recorded

### Requirement: Alt-drag duplicates the active layer

Holding Alt while the Move tool drags a whole layer (no active selection) SHALL
duplicate the layer, preview the duplicate moving under the cursor during the
drag, and make the duplicate the active layer on commit, leaving the source layer
in place. The bridge SHALL expose a `begin_move_duplicate` that duplicates the
active layer, recomposites, and builds the move preview without recording
history; the drag SHALL record exactly one undo state on `commit_move`. With an
active selection the behaviour is the selected-pixel duplicate of the
`selection-content-move` capability, not a whole-layer clone.

#### Scenario: Alt-drag clones the layer [lct_alt_clone]

- **WHEN** the Move tool drags a whole layer with Alt held
- **THEN** a duplicate layer is created and moves with the drag, the source layer
  keeps its pixels, and one history state is recorded on release

#### Scenario: The clone becomes active [lct_alt_clone_active]

- **WHEN** the Alt-drag is released
- **THEN** the duplicate is the active layer and the source remains in the stack

#### Scenario: No record during the drag [lct_alt_clone_record]

- **WHEN** `begin_move_duplicate` runs at press
- **THEN** no history state is recorded until the drag commits, which records
  exactly one
