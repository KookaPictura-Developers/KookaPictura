# numeric-fields Specification

## Purpose
TBD - created by archiving change fix-app-ui-issues. Update Purpose after archive.
## Requirements
### Requirement: Shared numeric field control

The system SHALL provide one reusable numeric input control (`NumericField`) that
presents a text value, an optional leading text label, and an optional slider
popup, and SHALL be configured by minimum, maximum, step, decimal places, suffix,
integer-or-double value type, and whether the popup is offered. The control SHALL
emit a preview while a scrub or slider drag is in progress and a single commit
when the edit finishes, and SHALL emit nothing for a programmatic value sync. The
Layers panel Opacity and Fill fields SHALL be a thin configuration of this control
(`0..=100`, integer, `%` suffix) rather than a separate implementation, and every
control migrated to it SHALL keep its existing stored-value conversion.

#### Scenario: A configured field enforces its range and formatting [lpn_config]

- **WHEN** a `NumericField` is configured with a minimum, maximum, and decimal
  places and the user types an out-of-range value
- **THEN** the committed value is clamped to the range and displayed with the
  configured decimal places and suffix

#### Scenario: PercentField is one configuration [lpn_percent_thin]

- **WHEN** the Layers panel Opacity field is shown and edited
- **THEN** it exhibits the shared control's scrubbing, popup, and conversion
  behavior, and no second numeric implementation exists for it

#### Scenario: A programmatic sync emits no edit [lpn_sync]

- **WHEN** the owning panel reflects an existing value into the control without
  user input
- **THEN** no preview or commit is emitted

### Requirement: Label and field scrubbing

Pressing and dragging horizontally on a `NumericField`'s label or its value SHALL
scrub the value by one step per pixel with a short press dead zone and a
horizontal-resize cursor, and releasing SHALL commit the edit as exactly one undo
step. A modifier SHALL scale the scrub sensitivity: `Shift` for a coarse step and
`Ctrl` for a fine step. The whole press-drag-release SHALL preview live and add
no history state until release.

#### Scenario: Dragging the label scrubs the value [lpn_scrub]

- **WHEN** the user presses a `NumericField`'s label and drags horizontally past
  the dead zone
- **THEN** the value changes by the dragged step count, previews live, and one
  undo step is added on release

#### Scenario: A modifier scales the scrub sensitivity [lpn_scrub_mod]

- **WHEN** the same horizontal drag is made with `Shift` held and then with `Ctrl`
  held
- **THEN** the `Shift` drag changes the value by a coarser amount than the plain
  drag and the `Ctrl` drag by a finer amount

#### Scenario: A drag adds one undo state only at release [lpn_scrub_commit]

- **WHEN** a scrub previews several values and is then released
- **THEN** no undo state is added during the drag and exactly one is added on
  release

### Requirement: Slider popup

A `NumericField` configured with a popup SHALL expose a control that opens a
popup hosting the shared tracking slider, and the popup SHALL position itself
relative to the field. Pressing on the slider groove and dragging SHALL track the
value continuously rather than page-stepping or jumping, and the drag SHALL
preview live and commit once on release.

#### Scenario: The popup slider tracks a press-drag [lpn_popup_track]

- **WHEN** the user presses the popup slider groove and drags without releasing
- **THEN** the value follows the pointer continuously, the canvas or target
  updates live, and one undo step is added on release

#### Scenario: The popup opens relative to its field [lpn_popup_place]

- **WHEN** the user opens a `NumericField`'s slider popup
- **THEN** the popup is positioned relative to the field rather than at an
  unrelated location

### Requirement: Popup keyboard handling

While a `NumericField`'s slider popup is open, `Left` and `Right` SHALL change
the value by one step, `Home` and `End` SHALL move it to the minimum and maximum,
and `PageUp` and `PageDown` SHALL move it by one page, regardless of which widget
in the popup holds focus. The key handling SHALL work even when the popup window
never acquires keyboard focus.

#### Scenario: Arrow keys step the popup value [lpn_keys_arrows]

- **WHEN** the popup is open and the user presses `Right` and then `Left`
- **THEN** the value increases by one step and then decreases by one step, and
  each change previews and commits like a drag

#### Scenario: Home, End, and Page keys navigate the range [lpn_keys_nav]

- **WHEN** the popup is open and the user presses `End`, `Home`, `PageUp`, and
  `PageDown`
- **THEN** the value moves to the maximum, the minimum, and by one page in each
  direction respectively

