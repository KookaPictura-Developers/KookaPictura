# Spec Delta

## ADDED Requirements

### Requirement: Remove Black Matte and Remove White Matte

The system SHALL provide the `Layer > Matting > Remove Black Matte` and
`Layer > Matting > Remove White Matte` commands. Each SHALL recover the layer's
un-composited colour from its straight-alpha pixels by solving
`out = (px − bg·(1−a)) / a` for the red, green, and blue channels, where `a` is
the pixel's normalised alpha, `bg` is `0` for Remove Black Matte and `255` for
Remove White Matte, and the result is clamped to the 0–255 sample range. The
operation MUST leave the alpha channel unchanged and MUST NOT alter a pixel
whose alpha is zero. The commands SHALL operate on the active layer, and when a
selection is active SHALL confine their writes to the selection. A successful
command SHALL record exactly one undo state.

#### Scenario: Remove Black Matte divides out a black background

- **WHEN** a layer holds a colour that was composited over black and Remove
  Black Matte runs
- **THEN** each colour sample becomes `px / a` (clamped), the alpha channel is
  unchanged, and the change is one undo step

#### Scenario: Remove White Matte subtracts a white background

- **WHEN** a layer holds a colour that was composited over white and Remove
  White Matte runs
- **THEN** each colour sample becomes `(px − 255·(1−a)) / a` (clamped), the alpha
  channel is unchanged, and the change is one undo step

#### Scenario: Fully transparent pixels are untouched

- **WHEN** a pixel has zero alpha and either Remove Matte command runs
- **THEN** that pixel's colour and alpha are unchanged

### Requirement: Matting refuses a locked or empty layer

The system SHALL refuse every `Layer > Matting` command when the active layer
is pixel-locked, when the active layer has no editable pixel content, or when
no single pixel layer is active. A refused command MUST leave the document
unchanged and MUST NOT record a history state; the corresponding menu command
SHALL be disabled.

#### Scenario: A locked layer is refused

- **WHEN** the active layer is pixel-locked and a Matting command runs
- **THEN** the command is disabled, the document is unchanged, and no history
  state is recorded

#### Scenario: A non-pixel layer is refused

- **WHEN** the active layer is a group or an adjustment layer and a Matting
  command runs
- **THEN** the command is disabled, the document is unchanged, and no history
  state is recorded

### Requirement: Defringe replaces edge colour

The system SHALL provide the `Layer > Matting > Defringe…` command, which SHALL
replace the colour of the layer's edge pixels — the solid pixels within a given
`Width` of the transparent region — with the colour of the nearest interior
pixel, leaving alpha unchanged. The command SHALL present a dialog with a
`Width` field in pixels whose default is `1`. When a selection is active the
command SHALL confine its writes to the selection. A successful Defringe SHALL
record exactly one undo state.

#### Scenario: A fringe colour is replaced by the interior colour

- **WHEN** a layer has a one-pixel band of off-colour pixels against its
  transparency and Defringe runs with width 1
- **THEN** the band's colour becomes the colour of the nearest interior pixel,
  the alpha channel is unchanged, and the change is one undo step

#### Scenario: Width bounds how deep the replacement reaches

- **WHEN** the same layer is defringed with a larger width
- **THEN** a wider band of edge pixels takes the nearest interior colour

#### Scenario: The dialog defaults to one pixel

- **WHEN** the Defringe dialog opens
- **THEN** its `Width` field shows `1` and the command applies the entered width
  on OK and nothing on Cancel

#### Scenario: An opaque layer is unchanged

- **WHEN** a layer has no transparent pixels and Defringe runs
- **THEN** the command succeeds with no pixel change and records one undo state
