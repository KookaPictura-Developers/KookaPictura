# canvas-scrollbars Specification

## Purpose
Scrollbars that project the view state and follow pan, zoom, fit, and the Navigator.
## Requirements
### Requirement: Scrollbars project the view state

The workspace SHALL wrap the canvas in a thin container carrying a horizontal and
a vertical scrollbar. Each scrollbar's range SHALL be derived from the same range
helper that clamps the canvas pan, and its value SHALL be derived from the
canvas's current offset and zoom, so the scrollbars are a pure projection. The
canvas offset SHALL remain the single source of truth and the scrollbars SHALL
NOT store pan or zoom. Dragging a scrollbar SHALL pan the canvas, and the canvas
SHALL clamp the resulting offset through the shared range.

#### Scenario: A scrollbar drag pans the canvas [lcs_drag]

- **WHEN** the user drags the horizontal scrollbar
- **THEN** the canvas pans horizontally and the vertical scrollbar and the
  Navigator proxy reflect the new visible region

#### Scenario: Range and value round-trip [lcs_roundtrip]

- **WHEN** the canvas is panned and zoomed to a known state and the scrollbar
  value is set to a derived position
- **THEN** the canvas offset equals the clamped derived position and the
  scrollbar reads back that value

#### Scenario: The offset stays authoritative [lcs_single_source]

- **WHEN** the canvas offset is changed by a pan, zoom, fit, or the Navigator
  rather than by a scrollbar
- **THEN** the scrollbars update to match and no second stored pan state is used
  to compute the canvas transform

### Requirement: Scrollbars follow pan, zoom, fit, and the Navigator

The scrollbars SHALL update whenever the canvas view changes by any path — a pan,
a zoom, a fit or 100 % command, or a Navigator proxy drag — through a view-change
signal emitted by the canvas, so the bars never lag the actual view.

#### Scenario: Zoom updates the bars [lcs_zoom]

- **WHEN** the canvas zoom changes
- **THEN** the scrollbar ranges and visible-handle size are recomputed from the
  new zoom

#### Scenario: The Navigator and the bars agree [lcs_navigator]

- **WHEN** the user drags the Navigator's proxy rectangle
- **THEN** the scrollbars move to the corresponding range positions and the
  canvas offset matches the proxy region

### Requirement: Scrollbars are always visible

Both the horizontal and the vertical scrollbar SHALL be visible at all times,
regardless of whether the document fits the viewport, so the canvas is freely
pannable. Each bar SHALL remain a pure projection of the canvas offset and zoom
through the shared range helper, and the canvas SHALL remain the single source of
truth. Hiding a bar by an as-needed policy is removed; the bars SHALL NOT collapse
to hidden when the document fits.

#### Scenario: Bars stay visible when the document fits [lcs_always_visible]

- **WHEN** the document is smaller than the viewport along both axes
- **THEN** both scrollbars are still shown and the canvas can still be panned
  within the shared range

#### Scenario: Bars are visible when the document overflows

- **WHEN** the canvas is zoomed so the document is larger than the viewport along
  an axis
- **THEN** both scrollbars are shown with that axis's pan range

