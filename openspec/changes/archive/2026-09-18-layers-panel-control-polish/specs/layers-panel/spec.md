## MODIFIED Requirements

### Requirement: Layers panel rows

The system SHALL present the full layer tree of the active document as rows in
topmost-first display order, each row showing a visibility toggle, a thumbnail
(or a folder glyph for a group), an editable name, and indicators for clipping,
a layer mask, and an adjustment/style badge. The visibility toggle SHALL be the
row's only visibility control: the row SHALL NOT paint a separate checkbox or
check indicator. A group row SHALL be expandable and collapsible, and its
children SHALL be shown indented beneath it when expanded; a layer clipped to
the layer below SHALL be indented and its base layer's name SHALL be underlined.
The rows SHALL reflect the document and update when the document changes or the
active document changes.

#### Scenario: Rows reflect the tree [m39_tree]

- **WHEN** a document with a group containing children is active
- **THEN** the panel shows a row per node, topmost-first, with the group's
  children shown indented under it and the group row marked expandable

#### Scenario: A clipped layer is indented under its base [m39_badges]

- **WHEN** a layer is clipped to the layer below it
- **THEN** the clipped row is indented and the base layer's name is underlined

#### Scenario: No document [m39_tree]

- **WHEN** no document is open
- **THEN** the panel shows no rows and does not crash

#### Scenario: A row paints no checkbox indicator [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its only visibility control is the eye icon and the model reports no
  check state for the row

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, the
color-label swatch, a clipping-mask indicator for a clipped layer, the clipping
indentation and base underline, the layer-mask thumbnail when a mask is present,
and an adjustment/style badge when adjustment content is present. A layer whose
lock state has any flag set SHALL also show a lock badge at the right side of its
row; an unlocked layer SHALL show none. The visibility toggle SHALL be an eye
icon (`layers.eyeOn`/`layers.eyeOff`) drawn slightly inset from the panel's left
edge and at the same x for every row, independent of nesting depth; the nesting
indentation SHALL apply to the thumbnail and name, not to the visibility toggle.
A group with at least one child SHALL show a disclosure icon — right when
collapsed, down when expanded — at its indented position, and clicking that icon
SHALL expand or collapse the group. If an expected icon asset is unavailable, the
delegate SHALL omit that badge while keeping the row legible rather than fail.

#### Scenario: The visibility toggle is an eye icon [lpr_eye]

- **WHEN** a layer row is shown
- **THEN** its visibility toggle is drawn from the eye icon asset, and a hidden
  layer uses the off variant

#### Scenario: The eye is left-anchored for every depth [lpr_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its eye icon is at the same x as a top-level row's, while its
  thumbnail and name are indented

#### Scenario: A group shows a disclosure icon [lpr_chevron]

- **WHEN** a group with children is shown collapsed and then expanded
- **THEN** it shows the right-pointing icon when collapsed and the down-pointing
  icon when expanded, and clicking the icon toggles it

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A clipped layer shows the clipping indicator [lpc_clip]

- **WHEN** a layer is clipped to the layer below it
- **THEN** its row draws the clipping-mask indicator in addition to the
  indentation and the base layer's underline

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A locked layer shows a right-side lock badge [lpc_lockbadge]

- **WHEN** a layer has any lock flag set
- **THEN** its row draws a lock badge on the right side, and an unlocked layer
  draws no lock badge

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

### Requirement: Opacity and Fill percent controls

The Layers panel SHALL present Opacity and Fill as percentages in the range
`0..=100`, each through one reusable control that offers a text value, a popup
slider, and scrubbing by pressing and dragging horizontally on the label or the
field. Each field's `%` sign SHALL be rendered inside the value box rather than
beside it. The control SHALL convert between the displayed percentage and the
stored `0..=255` value with `round(pct * 255 / 100)` on edit and `round(value *
100 / 255)` on display, so `100`% is `255`, `50`% is `128`, and `0`% is `0`. The
control SHALL distinguish a live preview from a commit: while a scrub or slider
drag is in progress it SHALL emit preview changes that update the canvas without
adding a history state, and when the edit finishes it SHALL emit one commit that
adds exactly one undo state for the whole edit. A completed text edit SHALL also
add one state. The control SHALL emit nothing for a programmatic sync, so a
programmatic sync does not feed back. The existing enable/refusal rules SHALL be
unchanged: Fill is unavailable for a group, and the Background and a fully locked
layer disable both controls.

#### Scenario: A percentage maps to the stored byte [lpc_percent]

- **WHEN** Opacity is set to 50 % on an eligible layer
- **THEN** the layer's stored opacity becomes 128 and the document is modified

#### Scenario: A stored byte displays as a percentage [lpc_percent]

- **WHEN** a layer with stored opacity 128 becomes the current row
- **THEN** the Opacity control displays 50 %

#### Scenario: The popup slider and label drag both change the value [lpc_percent]

- **WHEN** the user drags the Opacity popup slider or drags horizontally on the
  Opacity field
- **THEN** the percentage changes, the canvas updates live, and the whole drag is
  applied in one undo step

#### Scenario: A drag adds one undo state only at release [lpc_preview]

- **WHEN** an Opacity or Fill drag previews several values and is then released
- **THEN** no undo state was added during the drag and exactly one undo state is
  added on release

#### Scenario: The percent sign is inside the value box [lpr_percent]

- **WHEN** the Opacity or Fill field is shown
- **THEN** its `%` sign is drawn inside the value box rather than beside it

#### Scenario: A programmatic sync does not re-apply the edit [lpc_percent]

- **WHEN** the panel reflects an existing value into the control without user input
- **THEN** no edit is sent and no history state is added

### Requirement: Layers panel header fields

The Opacity and Fill fields SHALL each carry their label as part of the field,
and pressing and dragging that label horizontally SHALL scrub the value exactly
as dragging the field does. Each field's `%` sign SHALL be shown inside the
value box. The lock strip SHALL hold five toggles; the toggles for alpha, paint,
position, and nesting SHALL be semantic icons describing what they lock
(transparency, image pixels, position, nesting), and only the full-lock toggle
SHALL be a padlock icon.

#### Scenario: Dragging the label scrubs the value [lpr_label]

- **WHEN** the user presses the `Opacity` label and drags horizontally
- **THEN** the opacity percentage changes and is applied in one undo step, the
  same as dragging the field itself

#### Scenario: The value shows a percent sign inside the box [lpr_percent]

- **WHEN** the Opacity or Fill field is shown
- **THEN** its value's `%` sign is drawn inside the value box

#### Scenario: Only Lock All is a padlock [lpr_locks]

- **WHEN** the lock strip is shown
- **THEN** the alpha, paint, position, and nesting toggles use their semantic
  icons and the full-lock toggle is the only padlock
