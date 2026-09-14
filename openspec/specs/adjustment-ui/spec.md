# adjustment-ui Specification

## Purpose
TBD - created by archiving change m4-adjustment-ui. Update Purpose after archive.
## Requirements
### Requirement: Layer stack introspection

The `PictureView` QObject SHALL expose `layer_count`, `layer_name(i)`,
`layer_kind(i)`, and `layer_visible(i)` for the top-level layers of the loaded
document. `layer_kind` SHALL return `"pixel"`, `"group"`, or `"adjustment"`.
Without a document, or for an out-of-range index, each accessor MUST return a
safe default (`0`, empty string, and `false`) and MUST NOT panic.

#### Scenario: Introspection reports the loaded layer stack

- **WHEN** a PSD with top-level pixel, group, and adjustment layers is opened
- **THEN** `layer_count` equals the number of top-level layers and, for each `i`,
  `layer_name(i)`, `layer_kind(i)`, and `layer_visible(i)` match that layer

#### Scenario: Kind is classified per layer

- **WHEN** `layer_kind(i)` is queried for a pixel layer, a group, and an
  adjustment layer
- **THEN** it returns `"pixel"`, `"group"`, and `"adjustment"` respectively

#### Scenario: Out-of-range access returns defaults

- **WHEN** any introspection accessor is called with an index outside
  `0..layer_count`, or before a document is loaded
- **THEN** it returns `0`, an empty string, or `false` without panicking

### Requirement: Toggle layer visibility command

`set_layer_visible(i, visible)` SHALL set the visibility flag of layer `i`,
recomposite the document image, and emit the `changed` signal. An out-of-range
index MUST be a no-op that neither recomposites nor emits `changed`.

#### Scenario: Toggling visibility recomposites and signals

- **WHEN** `set_layer_visible(i, false)` is called for a visible layer
- **THEN** `layer_visible(i)` is false, the displayed image is recomposited, and
  `changed` is emitted

#### Scenario: Out-of-range toggle is a no-op

- **WHEN** `set_layer_visible` is called with an index outside
  `0..layer_count`
- **THEN** the document is unchanged and no `changed` signal is emitted

### Requirement: Add adjustment layer command

`add_adjustment(kind)` SHALL append an adjustment layer for a supported kind
(`invert`, `posterize`, `threshold`, `brightness-contrast`,
`hue-saturation`), recomposite, emit `changed`, and return `true`. An unknown
kind or a missing document MUST return `false` without mutating the stack. When
a selection is active, the new layer MUST carry a raster mask built from that
selection so only selected pixels change.

#### Scenario: Supported kind appends and recomposites

- **WHEN** `add_adjustment("invert")` is called on a loaded document
- **THEN** `true` is returned, `layer_count` increases by one, the last layer's
  kind is `"adjustment"`, and the composited pixels change

#### Scenario: Unknown kind is rejected

- **WHEN** `add_adjustment` is called with an unsupported kind or without a
  document
- **THEN** it returns `false` and `layer_count` is unchanged

#### Scenario: Active selection confines the adjustment

- **WHEN** a selection is active and an adjustment is added
- **THEN** the new layer has a mask whose coverage is the selection, and only
  covered pixels differ from the pre-adjustment composite

### Requirement: Remove layer command

`remove_layer(i)` SHALL remove top-level layer `i`, recomposite, and emit
`changed`. An out-of-range index MUST be a no-op that neither changes the stack
nor emits `changed`.

#### Scenario: Removing a layer shrinks the stack and changes output

- **WHEN** `remove_layer(i)` is called with a valid index
- **THEN** `layer_count` decreases by one, the remaining layers keep their
  order, and the composited image is refreshed

#### Scenario: Out-of-range removal is a no-op

- **WHEN** `remove_layer` is called with an index outside `0..layer_count`
- **THEN** the layer stack is unchanged and no `changed` signal is emitted

### Requirement: Adjustment encoders in pictura-render

`pictura-render` SHALL provide encoder functions that build the raw
`AdjustmentData` for the adjustment subset it decodes, next to the decoder:
Invert (`nvrt`), Posterize (`post`), Threshold (`thrs`), BrightnessContrast
(`brit`), and HueSaturation (`hue2`). Each encoder's output MUST round-trip
through `decode_adjustment`, and out-of-range inputs MUST be clamped to the
decoder's accepted range. The encoders MUST NOT introduce a dependency on Qt or
any GUI crate, so `pictura-core` remains dependency-free.

#### Scenario: Each encoder round-trips through the decoder

- **WHEN** each encoder is called with in-range parameters and its output is
  passed to `decode_adjustment`
- **THEN** the decoded `Adjustment` matches the intended variant and parameters

#### Scenario: Out-of-range inputs are clamped

- **WHEN** an encoder is called with values outside its accepted range
- **THEN** the emitted `AdjustmentData` decodes to the range's boundary value
  rather than being rejected

### Requirement: Layers dock UI

The C++ Qt shell SHALL present a Layers dock containing a layer list with a
visibility checkbox per row, an adjustment-type combo box, an Add button, and a
Remove button. The dock MUST rebuild its list and re-display the composite
whenever `PictureView::changed` is emitted. Toggling a checkbox MUST call
`set_layer_visible`; Add MUST call `add_adjustment` with the combo's kind. The
dock is a view over the QObject and MUST NOT hold authoritative document state.

#### Scenario: Dock lists layers with visibility checkboxes

- **WHEN** a document is loaded and the dock refreshes
- **THEN** one row appears per top-level layer, showing its name and kind, with
  a checkbox reflecting `layer_visible(i)`

#### Scenario: Dock re-renders on change

- **WHEN** any command emits `changed`
- **THEN** the dock rebuilds the list and the displayed image is refreshed from
  `PictureView::image`

#### Scenario: Add and Remove buttons drive commands

- **WHEN** the Add button is clicked with a kind selected, or the Remove button
  is clicked with a row selected
- **THEN** `add_adjustment` or `remove_layer` is called for that kind/index and
  the resulting `changed` signal refreshes the dock

### Requirement: Headless self-test

The app's `--self-test` mode SHALL, after loading a layered PSD, add an Invert
adjustment layer and assert the composited pixels changed as expected, then
toggle a layer's visibility and assert the output changed, and exit `0` on
success. It MUST exit non-zero when the adjustment or visibility toggle fails
to change the output.

#### Scenario: Self-test passes on a layered fixture

- **WHEN** `pictura --self-test <layered.psd>` runs headless
- **THEN** the Invert adjustment changes the composited pixels, the visibility
  toggle changes the output, and the process exits `0`

#### Scenario: Self-test fails when the composite does not change

- **WHEN** adding the adjustment or toggling visibility leaves the output
  unchanged
- **THEN** the self-test reports the failure and exits with a non-zero code

