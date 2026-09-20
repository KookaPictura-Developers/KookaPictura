## ADDED Requirements

### Requirement: Layer row visibility gutter and content spacing

The visibility toggle SHALL be centred horizontally inside its fixed left
gutter, with equal padding on its left and right sides, rather than being
left-anchored with all padding on one side. The delegate SHALL draw a 1 px
separator in a slightly darker grey at the gutter's right edge, between the
visibility toggle and the row content. The expand/collapse chevron slot SHALL be
reserved only for rows that are expandable, so a regular (non-group) layer's
thumbnail begins immediately after the gutter instead of leaving an empty
chevron-width gap. The eye hit-target, the label tint, and the thumbnail/name
hit-targets SHALL all mirror this layout.

#### Scenario: The eye is centred in its gutter [lpr_eye_gutter]

- **WHEN** a row is shown at any depth
- **THEN** the eye glyph's own rect has equal left and right padding inside the
  fixed visibility gutter

#### Scenario: A separator divides the gutter from the content [lpr_eye_sep]

- **WHEN** a layer row is shown
- **THEN** a 1 px darker-grey vertical separator is drawn at the gutter's right
  edge between the eye and the content

#### Scenario: A non-group thumbnail sits close to the gutter [lpr_thumb_gap]

- **WHEN** a regular non-expandable layer and a group row are shown at the same
  depth
- **THEN** the regular layer's thumbnail starts immediately after the gutter,
  while the group row still reserves the chevron slot for its disclosure icon

### Requirement: Layer thumbnail preserves the canvas aspect ratio

A layer row thumbnail SHALL be drawn at the document's aspect ratio inside the
row's thumbnail box, letterboxed rather than stretched into a square, so a wide
or tall document is not distorted. The thumbnail outline, the transparency
checkerboard, the active bracket, and the thumbnail hit-target SHALL all use the
letterboxed rect. A group's folder glyph keeps its square slot.

#### Scenario: A non-square document is not stretched [lpr_thumb_aspect]

- **WHEN** a regular layer of a document whose width and height differ is shown
- **THEN** the drawn thumbnail's aspect ratio matches the document, with the
  unused part of the row's thumbnail box left empty

### Requirement: Layer drag shows a CS6 drop indicator

While a layer drag is over a valid target, the view SHALL draw the CS6 drop
indicator itself rather than the stock style primitive: a thin blue line above
or below the target row for a sibling reorder, and a thin blue outline around a
group row for a drop-into. An invalid target SHALL continue to show no
indicator, and the closed-hand cursor and the above/below resolution SHALL be
unchanged.

#### Scenario: A sibling reorder shows a thin blue line [lpr_drop_line]

- **WHEN** a row is dragged over the gap above or below a valid sibling target
- **THEN** a thin blue line is drawn at that edge and no other indicator is shown

#### Scenario: A drop into a group outlines the group [lpr_drop_group_outline]

- **WHEN** a row is dragged over a group row that is a valid target
- **THEN** a thin blue outline is drawn around the group row

#### Scenario: An invalid target shows no indicator [lpr_drop_line_invalid]

- **WHEN** a row is dragged over its own descendant or another refused target
- **THEN** no drop indicator is drawn

### Requirement: Raised layer row height

The single named row-height constant SHALL be raised so a Medium-thumbnail row
is at least 34 px tall, giving layer rows slightly more vertical breathing room
while keeping the same constant used by both `sizeHint` and the delegate's
centring math. The existing "at least 28 px" floor remains satisfied.

#### Scenario: A Medium row is taller [lpr_row_height_raised]

- **WHEN** a row's size hint is read with the Medium thumbnail size
- **THEN** its height is at least 34 px from the one named row-height constant
