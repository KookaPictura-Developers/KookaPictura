# Spec Delta

## ADDED Requirements

### Requirement: Canvas presents the reduced level during a preview stroke

While a preview stroke is in progress the canvas SHALL crop the view-pyramid
level the preview wrote rather than the level the zoom would select, so the
whole in-progress image — not only the painted pixels — is shown at the preview
resolution. The preview SHALL repair that stored level directly and MUST NOT
rebuild it from level 0, because level 0 is deliberately left untouched until
the stroke ends. Releasing or cancelling the stroke SHALL restore the
zoom-selected level: the commit patches level 0 and rebuilds the stored levels
from it, which overwrites the preview.

#### Scenario: The canvas crops the preview level while previewing [cv_preview_present]

- **WHEN** a preview stroke presents a region
- **THEN** the canvas crops the view-pyramid level the preview wrote, that
  level is repaired in place rather than rebuilt from level 0, and level 0 is
  unchanged

#### Scenario: The commit restores the zoom-selected level [cv_preview_commit_restores_level]

- **WHEN** a previewed stroke is released
- **THEN** the exact region patches level 0, the stored levels are rebuilt from
  it, and the canvas returns to the level the zoom selects

#### Scenario: A cancelled preview restores the zoom-selected level [cv_preview_cancel_restores_level]

- **WHEN** a previewed stroke is cancelled
- **THEN** the pre-stroke document is presented and the canvas returns to the
  level the zoom selects
