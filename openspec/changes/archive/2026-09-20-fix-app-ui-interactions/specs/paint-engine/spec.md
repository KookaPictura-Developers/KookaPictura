## ADDED Requirements

### Requirement: Incremental dirty region and dab latency

The paint engine SHALL report a per-dab incremental dirty rectangle rather than
the cumulative dirty rectangle of the whole stroke, so compositing after a dab
touches only the pixels that dab changed. The CPU region compositor SHALL
composite only the requested region rather than fully compositing the document
and slicing the result, and a region blit SHALL update only that region of the
presented image instead of invalidating the whole scaled present cache. The
sustained cost of a brush dab on a 4000×4000 document SHALL meet the canvas-view
budget (`docs/dev/canvas-view-spec.md`): input-to-first-pixel at most 16 ms per
dab and a sustained redraw at or above 60 FPS.

#### Scenario: A dab dirties only its own region [lpe_dirty_per_dab]

- **WHEN** a stroke places a dab after other dabs
- **THEN** the reported dirty rectangle covers the new dab's footprint only, not
  the union of all prior dabs of the stroke

#### Scenario: Region compositing matches the full composite [lpe_region_equal]

- **WHEN** a region is composited through the CPU region compositor
- **THEN** every pixel in that region equals the full-document composite within
  the compositor tolerance

#### Scenario: A dab meets the canvas-view budget [lpe_dab_budget]

- **WHEN** a brush dab is processed on a 4000×4000 document
- **THEN** the input-to-first-pixel latency is at most 16 ms and the sustained
  update rate is at least 60 FPS, measured against the canvas-view budget
