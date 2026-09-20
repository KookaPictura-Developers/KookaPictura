## MODIFIED Requirements

### Requirement: First-class Background layer

`pictura_core::Layer` SHALL carry an explicit background flag. The codec SHALL
derive the flag on read — the bottom top-level, non-group layer named
`Background` is the Background — and SHALL write the `Background` name for a
flagged layer on save. The `is_background` check SHALL use the flag and MUST NOT
depend on the layer's index or name. `Layer from Background…` SHALL clear the
flag and unlock the layer, and `Background From Layer` SHALL set the flag,
convert transparent pixels to the background color, move the node to the bottom
of the stack, and refuse a group or a layer that is already the Background. The
conversion from a Background SHALL also be reachable from the Layers panel: a
double-click on the Background row outside its name, and dropping the Background
on the New Layer button, SHALL each run `Layer from Background…` in place rather
than cloning a locked `Background copy`. Each conversion SHALL recomposite and
record exactly one undo state.

#### Scenario: The flag round-trips through PSD

- **WHEN** a document whose bottom layer is the Background is saved and read
  back
- **THEN** the re-read bottom layer reports the background flag

#### Scenario: The flag is independent of position and name

- **WHEN** a non-bottom layer named `Background` is present and a flagged layer
  sits at the bottom under another name
- **THEN** only the flagged layer is reported as the Background

#### Scenario: Convert from Background

- **WHEN** `Layer from Background…` runs on the Background layer
- **THEN** the flag is cleared, the layer is no longer locked, and the change is
  one undo step

#### Scenario: A double-click converts the Background [lmb_background_dblclick]

- **WHEN** the user double-clicks the Background row outside its name region
- **THEN** the Background is converted to a normal, unlocked layer in one undo
  step

#### Scenario: Dropping the Background on New Layer converts it [lmb_background_drop]

- **WHEN** the Background row is dropped on the New Layer strip button
- **THEN** the Background is converted in place to a normal layer, no
  `Background copy` is created, and the change is one undo step

#### Scenario: Convert to Background

- **WHEN** `Background From Layer` runs on a normal pixel layer
- **THEN** the layer becomes the flagged Background at the bottom of the stack,
  its transparent pixels take the background color, and the change is one undo
  step
