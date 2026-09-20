## ADDED Requirements

### Requirement: Selection from a layer's alpha channel

The system SHALL provide a bridge operation that builds a document-sized
selection from a layer's alpha channel. For each document pixel, the selection
coverage SHALL be the layer's alpha value at that pixel's layer-local coordinate
(`document − layer.rect.top_left`); when the layer has no alpha channel the
coverage SHALL be `255`, and a pixel outside the layer's `rect` SHALL be `0`. The
resulting selection SHALL be combined with the current selection using the New
mode (it replaces the current selection) and SHALL be recorded as one undoable
state. An unresolved layer path SHALL be refused without changing the selection
or the document. The existing `pictura_select::Selection::from_channel`
construction SHALL be reused rather than reimplemented.

#### Scenario: A layer's alpha shape becomes the selection [lst_from_alpha]

- **WHEN** the bridge operation runs for a layer with an alpha channel
- **THEN** the resulting selection covers exactly the pixels where the layer's
  alpha is non-zero, offset to the layer's document position

#### Scenario: A missing alpha channel is fully opaque [lst_from_alpha_missing]

- **WHEN** the bridge operation runs for a layer with no alpha channel
- **THEN** every pixel inside the layer's `rect` has coverage `255` and every
  pixel outside it has coverage `0`

#### Scenario: The result replaces the current selection [lst_from_alpha_new]

- **WHEN** a selection already exists and the operation runs
- **THEN** the current selection is replaced by the layer's alpha shape as one
  undoable state

#### Scenario: An unresolved path is refused [lst_from_alpha_bad_path]

- **WHEN** the bridge operation is given a path that does not resolve to a layer
- **THEN** it returns a refusal and the selection and document are unchanged
