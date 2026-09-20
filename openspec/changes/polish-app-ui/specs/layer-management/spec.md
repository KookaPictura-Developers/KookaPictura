## MODIFIED Requirements

### Requirement: First-class Background layer

`pictura_core::Layer` SHALL carry an explicit background flag. The codec SHALL
derive the flag on read — the bottom top-level, non-group layer named
`Background` is the Background — and SHALL write the `Background` name for a
flagged layer on save. The `is_background` check SHALL use the flag and MUST NOT
depend on the layer's index or name. `Layer from Background…` SHALL clear the
flag and unlock the layer, and `Background From Layer` SHALL set the flag,
convert transparent pixels to the background color, move the node to the bottom
of the stack, and refuse a group or a layer that is already the Background. **A
Background converted to a normal layer SHALL be renamed to the next free
`Layer N` name** (through `next_layer_name(doc, "Layer")`) rather than keeping
the name `Background`; the conversion SHALL be reachable from the Layers panel
by a double-click on the Background row in its content band and by dropping the
Background on the New Layer button, and each of those panel gestures SHALL
present a **name-and-color dialog** (a name+color-only factory, or the unlock
followed by `set_layer_name_path` and `set_layers_color`) so the converted
layer's name and color label are chosen by the user, defaulting to `Layer N` and
no label. Each conversion SHALL recomposite and record exactly one undo state;
cancelling the dialog SHALL leave the Background unchanged and record nothing.

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
- **THEN** the flag is cleared, the layer is no longer locked, the layer is
  renamed to the next free `Layer N`, and the change is one undo step

#### Scenario: A double-click converts the Background [lmb_background_dblclick]

- **WHEN** the user double-clicks the Background row in its content band
- **THEN** a name-and-color dialog opens, and on accept the Background becomes a
  normal unlocked layer named by the user (default `Layer N`) with the chosen
  color label, in one undo step

#### Scenario: Dropping the Background on New Layer converts it [lmb_background_drop]

- **WHEN** the Background row is dropped on the New Layer strip button
- **THEN** the name-and-color dialog opens, and on accept the Background is
  converted in place to a normal `Layer N` layer with the chosen label, no
  `Background copy` is created, and the change is one undo step

#### Scenario: Cancelling the conversion dialog changes nothing [lmb_background_cancel]

- **WHEN** the conversion dialog is cancelled
- **THEN** the Background keeps its flag, lock, name, and pixels, and no history
  state is recorded

#### Scenario: Convert to Background

- **WHEN** `Background From Layer` runs on a normal pixel layer
- **THEN** the layer becomes the flagged Background at the bottom of the stack,
  its transparent pixels take the background color, and the change is one undo
  step

## ADDED Requirements

### Requirement: Selecting, copying, and moving on an invisible active layer

When the active layer is invisible, the application SHALL still allow
selection, copy, and Move operations on it, while paint and filter edits are
refused. The Move tool SHALL translate an invisible active layer exactly as a
visible one (mouse drag and keyboard nudge), and `compute_move_preview` SHALL
save and restore the active layer's `visible` flag around building its preview
base so a move never makes an invisible layer visible as a side effect. A move
on an invisible layer SHALL still record exactly one history state and SHALL
leave the layer invisible after commit.

#### Scenario: Move works on an invisible layer [lmg_move_invisible]

- **WHEN** the active layer is invisible and the Move tool drags it by a delta
- **THEN** the layer's content is translated, the layer stays invisible after
  commit, and one history state is recorded

#### Scenario: The preview does not reveal the layer [lmg_move_invisible_preview]

- **WHEN** the Move drag begins on an invisible active layer
- **THEN** `compute_move_preview` preserves the prior `visible` value and the
  document's layer visibility is unchanged by starting the drag

#### Scenario: Selection and copy stay available [lmg_select_invisible]

- **WHEN** the active layer is invisible and a selection tool or a copy command
  runs
- **THEN** the selection and copy operate on the layer's content and are not
  refused
