# Spec Delta

## ADDED Requirements

### Requirement: Frame-bounded in-stroke present

While a paint stroke is in progress the app SHALL bound the present rate
instead of presenting once per input event. The view SHALL accumulate the dirty
rectangles of the dabs it receives after the most recent present into one
pending region, and SHALL present that pending region only when
`PictureView::flush_present()` runs. While a stroke has a pending region the app
SHALL invoke `flush_present()` at least once per frame interval. Committing a
stroke SHALL flush the pending region before the history state is recorded, and
cancelling a stroke SHALL drop it without presenting. A region refresh outside a
stroke SHALL still present immediately. The coalescing MUST NOT change the
pixels: after the flush the canvas SHALL equal a full recomposite of the stroke's
working document over the presented region.

#### Scenario: Dabs between two presents are presented together
[cv_dabs_between_presents_present_once]

- **WHEN** several dabs arrive after a present and before the frame's
  `flush_present()` during a stroke
- **THEN** none of them presents on arrival, and `flush_present()` presents one
  region covering all of them

#### Scenario: A consumer can force a present [cv_flush_present]

- **WHEN** `flush_present()` is called while a stroke has a pending region
- **THEN** the pending region is presented before the call returns and the
  pending region is cleared

#### Scenario: The commit flushes before recording [cv_commit_flushes_pending]

- **WHEN** a stroke with a pending region is released
- **THEN** the pending region is presented before the history state is recorded

#### Scenario: A cancelled stroke presents nothing pending [cv_cancel_drops_pending]

- **WHEN** a stroke with a pending region is cancelled
- **THEN** the pending region is not presented and the canvas is restored to the
  pre-stroke image

#### Scenario: An idle region refresh is unaffected [cv_idle_region_immediate]

- **WHEN** a region refresh runs outside a stroke
- **THEN** it presents immediately, exactly as it does without coalescing
