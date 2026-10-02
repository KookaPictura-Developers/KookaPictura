# shared-widgets Specification

## Purpose

The reusable CS6 controls shared by the adjustment dialogs and the Properties
panel: a circular angle dial, a colour-ramp-groove slider, a hue spectrum strip,
and a curve editor. Each is self-contained, Qt-only, and carries no engine or
bridge dependency, so panels can use it without a second implementation.

## ADDED Requirements

### Requirement: Angle dial

The system SHALL provide an `AngleDial` that draws a circular face with a radius
line pointing at the current angle and SHALL let the user press or drag anywhere
on the face to aim that line. `setAngle` SHALL wrap its argument into `[0, 360)`
and SHALL leave the angle unchanged when the wrapped value equals the current
one. A press or drag SHALL emit `angleChanged` with the finite wrapped angle,
using the screen convention where 90° points up.

#### Scenario: An angle wraps into the half-open turn [sw_angle_wrap]

- **WHEN** `setAngle` is called with 370, then with −90
- **THEN** the reported angles are 10 and 270, each in `[0, 360)`

#### Scenario: Pressing the face emits a finite angle [sw_angle_press]

- **WHEN** the user presses or moves the mouse on the dial face
- **THEN** `angleChanged` is emitted with a finite value equal to the current
  wrapped angle

### Requirement: Ramp slider

The system SHALL provide a `RampSlider` that is a tracking slider whose groove
is painted as a gradient of evenly spaced colour stops. `setRamp` with at least
two stops SHALL apply the gradient, and `setRamp` with fewer than two stops SHALL
restore the ordinary groove. The control SHALL retain the press-drag tracking of
the shared `JumpSlider`, so pressing the groove jumps to the clicked position and
holds. Rebuilding the same ramp SHALL be a no-op.

#### Scenario: A two-stop ramp paints a gradient [sw_ramp_gradient]

- **WHEN** `setRamp` is called with two or more colours
- **THEN** the slider carries a groove gradient built from those stops

#### Scenario: A short ramp falls back to the groove [sw_ramp_fallback]

- **WHEN** `setRamp` is called with fewer than two colours
- **THEN** the slider's custom groove style is cleared and value tracking is
  unaffected

#### Scenario: Pressing the groove tracks the value [sw_ramp_track]

- **WHEN** the user presses the slider groove at a position away from the current
  value
- **THEN** the value moves to that position rather than page-stepping

### Requirement: Spectrum bar

The system SHALL provide a `SpectrumBar` that paints a horizontal rainbow strip
and SHALL rotate the strip's hues by the amount given to `setHueShift`, so the
bar previews a hue adjustment before it is applied.

#### Scenario: A hue shift repaints the strip [sw_spectrum_shift]

- **WHEN** `setHueShift` changes the shift by a non-zero amount
- **THEN** the rendered pixmap differs from the one before the change

### Requirement: Curve widget

The system SHALL provide a `CurveWidget` that edits a monotone curve through
ordered control points, starting at `(0,0)` and `(1,1)`. It SHALL interpolate
the points with a monotone cubic spline, SHALL expose the current points, and
SHALL fill a 256-entry lookup table whose values are non-decreasing and clamp to
`[0, 255]`. `setPoints` SHALL replace the curve and emit `curveChanged`;
`resetCurve` SHALL restore the two default endpoints. Clicking SHALL add a
control point, dragging SHALL move it, and dragging a non-endpoint off the face
SHALL remove it.

#### Scenario: The default curve is the identity [sw_curve_identity]

- **WHEN** a new `CurveWidget` builds its lookup table
- **THEN** the table has 256 entries, is monotonic, and maps every input to
  itself

#### Scenario: A control point changes the table [sw_curve_bend]

- **WHEN** a point above the diagonal is inserted between the endpoints
- **THEN** the lookup table changes from the identity at that input

#### Scenario: Reset restores the default curve [sw_curve_reset]

- **WHEN** `resetCurve` is called after the curve was edited
- **THEN** the control points are exactly `(0,0)` and `(1,1)` and the lookup
  table is the identity again

### Requirement: Reusable shared-widget contract

Each shared widget SHALL be self-contained and Qt-only: it SHALL declare its
behaviour entirely within `namespace pictura`, SHALL depend on no Rust bridge or
project engine type, and SHALL be usable by more than one panel without
duplication. A control that already exists in Kooka SHALL be reused rather than
re-implemented, and the shared widgets SHALL be covered by a Qt Test suite that
constructs them directly.

#### Scenario: A widget stands alone [sw_contract_selfcontained]

- **WHEN** a shared widget is constructed in a Qt Test without a main window or
  engine
- **THEN** it builds, responds to its public API, and emits its signals

#### Scenario: The suite is registered [sw_contract_suite]

- **WHEN** the app tests are built and run
- **THEN** `tst_shared_widgets` is one of the registered Qt Test suites
