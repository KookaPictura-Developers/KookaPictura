## MODIFIED Requirements

### Requirement: Layer row badges and delegate

The system SHALL draw each row with a delegate that paints, in CS6 order, the
visibility toggle, the thumbnail (a folder glyph for a group), the name, the
color-label swatch, a clipping-mask indicator for a clipped layer, the clipping
indentation and base underline, the layer-mask thumbnail when a mask is present,
and an adjustment/style badge when adjustment content is present. The visibility
toggle SHALL be anchored at the panel's left edge for every row, independent of
nesting depth; the nesting indentation SHALL apply to the thumbnail and name,
not to the visibility toggle. A group with at least one child SHALL show an
expand/collapse chevron at its indented position, and clicking that chevron
SHALL expand or collapse the group. If an expected icon asset is unavailable,
the delegate SHALL omit that badge while keeping the row legible rather than
fail.

#### Scenario: A masked layer shows a mask thumbnail [m39_badges]

- **WHEN** a layer carries a layer mask
- **THEN** its row shows a mask thumbnail and the mask-presence flag is set

#### Scenario: A clipped layer shows the clipping indicator [lpc_clip]

- **WHEN** a layer is clipped to the layer below it
- **THEN** its row draws the clipping-mask indicator in addition to the
  indentation and the base layer's underline

#### Scenario: The visibility toggle is left-anchored [lpc_eye]

- **WHEN** a nested layer is shown under its group
- **THEN** its visibility toggle is at the same left position as a top-level
  row's, while its thumbnail and name are indented under the group

#### Scenario: Clicking the chevron expands a group [lpc_chevron]

- **WHEN** the user clicks a group's expand/collapse chevron
- **THEN** the group expands or collapses and its expansion state is recorded

#### Scenario: A group row is expandable [m39_badges]

- **WHEN** a group has children
- **THEN** its row is marked expandable and shows an expand/collapse control

#### Scenario: A missing badge asset does not break the row [m39_badges]

- **WHEN** the adjustment badge's icon asset is missing
- **THEN** the row is still drawn and the badge is simply omitted

## ADDED Requirements

### Requirement: Layers panel header layout

The panel SHALL stack its header controls in this order, top to bottom: the
filter/search row; a row containing the blend-mode popup and a labeled Opacity
field; a row containing the five lock toggles and a labeled Fill field; the
layer list; and the action strip. The Opacity and Fill fields SHALL each carry a
leading text label, and each field's popup slider SHALL open centred
horizontally under the field rather than aligned to its right edge. The panel
SHALL NOT provide its own menu button; the wired Layer commands SHALL remain
reachable from the panel-group widget menu. The five lock toggles SHALL all sit
in the single lock row above the layer list, not below it.

#### Scenario: The header order is filter / blend+Opacity / locks+Fill / list [lpc_order]

- **WHEN** the Layers panel is shown
- **THEN** the filter row is above the blend and Opacity row, which is above the
  lock and Fill row, which is above the layer list, with the action strip below
  the list

#### Scenario: Opacity and Fill are labeled [lpc_labels]

- **WHEN** the Layers panel is shown
- **THEN** the Opacity field and the Fill field each have a text label directly
  to their left

#### Scenario: The percent picker opens centered [lpc_popup]

- **WHEN** the user opens the Opacity or Fill slider popup
- **THEN** the popup is centered horizontally under its field

#### Scenario: The panel has no separate menu button [lpc_nomenu]

- **WHEN** the Layers panel header is inspected
- **THEN** it contains no panel-menu button, and the Layer commands are offered
  by the panel-group widget menu
