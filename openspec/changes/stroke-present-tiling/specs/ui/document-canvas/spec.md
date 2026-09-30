# Spec Delta

## ADDED Requirements

### Requirement: Canvas crops level 0 while a GPU stroke presents

While a GPU stroke is live, the canvas SHALL crop level 0 — whose region blits
keep it current — instead of the zoom-selected stored level. The present-level
signal SHALL distinguish this state (`-1`) from the preview level (a positive
stored level) and the idle zoom-selected level (`0`). Releasing or cancelling
the stroke SHALL restore the zoom-selected level once the commit or restore
rebuilds the pyramid.

#### Scenario: A live GPU stroke presents level 0 [cv_deferred_level0]

- **WHEN** the canvas paints while a GPU stroke is live
- **THEN** it crops level 0 rather than the zoom-selected stored level

#### Scenario: The commit restores the zoom-selected level [cv_deferred_commit]

- **WHEN** the GPU stroke is released
- **THEN** the pyramid is rebuilt from the committed level 0 and the canvas
  returns to the zoom-selected level

#### Scenario: An idle refresh is unaffected [cv_deferred_idle]

- **WHEN** no stroke is live
- **THEN** the present level is the zoom-selected level, exactly as before
