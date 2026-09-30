# Spec Delta

## ADDED Requirements

### Requirement: Reduced-resolution preview for large dabs

When a dab's bounding box would push its rasterization past the
input-to-first-pixel budget, the system SHALL present a reduced-resolution
preview of the in-progress stroke instead of rasterizing it at full resolution.
The preview SHALL cover the stroke's bounding box at a view-pyramid level whose
own rasterization and present fit the same budget as the full-resolution
rasterization it replaces, SHALL be built from a snapshot of that level taken at
stroke start rather than from a fresh scan of the document, and MUST NOT write
level 0. Dabs whose bounding box fits the budget SHALL rasterize at full
resolution exactly as before.

#### Scenario: A small dab is not previewed [pe_preview_below_threshold]

- **WHEN** a stroke runs with a dab bounding box at or under the raster budget
- **THEN** it rasterizes and presents at full resolution, with no reduced
  document and no change to the pixels it produces

#### Scenario: A large dab is previewed [pe_preview_above_threshold]

- **WHEN** a stroke runs with a dab bounding box above the raster budget
- **THEN** the in-progress stroke is presented at a view-pyramid level whose own
  work fits the budget, built from a snapshot taken at stroke start rather than
  a scan of the document, and the number of pixels rasterized per dab no longer
  grows with the bounding box

#### Scenario: The preview does not touch the exact stroke's pixels [pe_preview_is_presentational]

- **WHEN** a previewed stroke is recorded
- **THEN** the reduced-resolution work is discarded, the exact full-resolution
  stroke is computed before the history state is recorded, and every pixel of
  the committed document is byte-identical to the same stroke run without a
  preview

#### Scenario: Cancelling a previewed stroke restores the document [pe_preview_cancel]

- **WHEN** a previewed stroke is cancelled
- **THEN** the recorded samples are dropped without producing a history state
  and the displayed document is the pre-stroke one
