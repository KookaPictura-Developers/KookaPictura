## ADDED Requirements

### Requirement: A stroke is continuously visible while dragging

While a paint stroke is in progress, every dab SHALL become visible on the
canvas as it is applied, before the mouse button is released, at the correct
document location under the current pan and zoom. The live path
(`paint_dab` → region refresh → canvas region blit) SHALL keep the scaled
present cache correct at every zoom, not only at 100 %, so a large document
whose initial view is zoomed to fit still shows the growing stroke. The stroke
SHALL write its pixel data and record exactly one history state only on release;
releasing SHALL NOT be the first moment the stroke is visible.

#### Scenario: Dabs show during the drag [lbt_live_visible]

- **WHEN** the Brush drags across the canvas with the button held
- **THEN** each dab is visible on the canvas before release, at the cursor's
  document position

#### Scenario: Live visibility at a non-100% zoom [lbt_live_visible_zoom]

- **WHEN** a large document is shown zoomed to fit (zoom below 100 %) and the
  Brush drags
- **THEN** the dabs are still visible at the correct on-screen position while
  the button is held

#### Scenario: One history state only on release [lbt_live_commit]

- **WHEN** a multi-dab stroke completes
- **THEN** exactly one history state is recorded by `end_paint`, and an undo
  restores the pre-stroke image

### Requirement: Invisible active layer refuses paint but allows selection and copy

When the active layer is invisible, paint and filter edits SHALL be refused with
a user-visible refusal and the canvas SHALL show a Block/Forbidden cursor over
the paint target, while selection, copy, and Move remain available (see the
`layer-management` requirement for the Move contract). The refusal SHALL leave
the document unchanged and record no history state. The policy SHALL be resolved
through one shared `active_layer_visible` helper so paint, filters, and the
cursor branch agree.

#### Scenario: Painting an invisible layer is refused [lbt_invisible_refuse]

- **WHEN** the Brush presses on the canvas with an invisible active layer
- **THEN** no stroke begins, the document is unchanged, no history state is
  added, and the refusal is reported

#### Scenario: The cursor marks the refusal [lbt_invisible_cursor]

- **WHEN** a paint tool hovers with an invisible active layer
- **THEN** the canvas shows a Block/Forbidden cursor instead of the blank paint
  cursor

#### Scenario: Selection and copy still work [lbt_invisible_select]

- **WHEN** the active layer is invisible and a selection or copy command runs
- **THEN** it operates on the layer's content and is not refused

### Requirement: Transient Eyedropper on Alt

Holding Alt while a paint tool is active SHALL transiently switch to the
Eyedropper: the canvas SHALL sample the composited colour under the pointer and
show the Alt/eyedropper cursor, and the sample SHALL update the foreground
colour. Releasing Alt SHALL restore the previous paint tool, its options, and
its cursor without permanently changing the active tool or recording history.

#### Scenario: Alt samples without switching tools [lbt_alt_sample]

- **WHEN** Alt is held with the Brush active and the pointer is over the canvas
- **THEN** the foreground colour becomes the sampled colour and the active tool
  remains the Brush

#### Scenario: Alt shows the eyedropper cursor [lbt_alt_cursor]

- **WHEN** Alt is held with a paint tool active over the canvas
- **THEN** the canvas shows the eyedropper/Alt cursor rather than the blank paint
  cursor

#### Scenario: Releasing Alt restores the tool [lbt_alt_restore]

- **WHEN** Alt is released
- **THEN** the paint tool's blank cursor and ring behaviour are restored and no
  stroke or history state was created

### Requirement: Brush size shortcuts bind by native scan code

The `[` and `]` brush-size shortcuts SHALL work on EU/Scandinavian keyboard
layouts as well as US layouts, by matching `QKeyEvent::nativeScanCode` for the
evdev scan codes for `[` (34) and `]` (35) and their Shift variants, in addition
to the US key values, and SHALL be guarded to the Brush and Pencil tools only.
The mapping from a key event to a brush-size delta SHALL be a single pure
helper so it is unit-testable without a Qt event loop.

#### Scenario: Bracket on a US layout changes the size [lbt_bracket_us]

- **WHEN** `]` is pressed with the Brush active on a US layout
- **THEN** the brush diameter increases by one step

#### Scenario: Bracket on a Nordic layout changes the size [lbt_bracket_native]

- **WHEN** the physical `]` key (evdev scan code 35) is pressed with the Brush
  active on an EU/Scandinavian layout
- **THEN** the brush diameter changes by the same step as on a US layout

#### Scenario: Shift bracket changes hardness [lbt_bracket_shift]

- **WHEN** `Shift` and the physical `[` key are pressed with the Brush active
- **THEN** the brush hardness decreases by one step

#### Scenario: Other tools ignore the bracket keys [lbt_bracket_guard]

- **WHEN** `[` or `]` is pressed with a non-paint tool active
- **THEN** the brush size is unchanged

### Requirement: Brush size shortcut mapping is a testable helper

The native-scan-code and key-value to brush-size delta mapping SHALL be a pure
function with unit tests covering the US and Nordic key codes, the Shift
variants, and the non-paint guard, so the keyboard-layout behaviour does not
depend on a running Qt application.

#### Scenario: The helper maps both layouts [lbt_bracket_helper]

- **WHEN** the helper is called with a US `]` key event and with an evdev-35
  event
- **THEN** both return the same positive diameter delta
