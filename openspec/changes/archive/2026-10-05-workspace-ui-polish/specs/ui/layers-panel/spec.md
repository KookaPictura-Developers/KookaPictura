## ADDED Requirements

### Requirement: Layer row surface and two-column layout

Each layer row's background SHALL be one shade step lighter than the list/base
background. Each row SHALL be split into two columns — a fixed visibility-toggle
column and a content column holding the rest of the row — so a click in the
visibility column toggles visibility and never selects or highlights the row. The
selected-row highlight SHALL be a lighter grey painted directly by the row
delegate rather than the palette's blue highlight, and the delegate SHALL be the
single source of that selection colour with no scoped stylesheet selection rule
for the tree view. The content column SHALL carry a 2 px left padding so the
thumbnail and name are not flush against the visibility gutter. The visibility
toggle's label tint and its 1 px gutter separator SHALL be unchanged, and the
gutter separator SHALL stay 1 px.

#### Scenario: The row background steps lighter than the list [lpr_row_surface]

- **WHEN** a layer row is shown against the list background
- **THEN** the row background is one shade step lighter than the list background

#### Scenario: The visibility toggle cannot be highlighted [lpr_eye_no_select]

- **WHEN** a click lands in the row's visibility-toggle column
- **THEN** visibility toggles and the row is neither selected nor highlighted

#### Scenario: Selection uses a lighter grey [lpr_selection_grey]

- **WHEN** one or more layer rows are selected
- **THEN** the row delegate paints the selected-row background as a lighter grey
  rather than the blue highlight, with no separate stylesheet rule overriding it

#### Scenario: The content column has a 2 px left padding [lpr_content_padding]

- **WHEN** a layer row is shown
- **THEN** the content column begins 2 px to the right of the visibility gutter,
  with the gutter separator still 1 px
