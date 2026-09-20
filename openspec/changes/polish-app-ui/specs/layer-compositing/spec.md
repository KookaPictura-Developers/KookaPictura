## ADDED Requirements

### Requirement: Layer visibility toggles use the region fast path

A layer-visibility change SHALL refresh through the **region fast path** — the
Layers panel eye toggle and any `set_layers_visible` call composite the changed
layer's clamped document region and blit that region rather than always running
a full `recomposite()` and recording a full-document snapshot. The path SHALL fall back to `recomposite` only when the layer's region
is not representable (a `None` region, for example a channel-less or unbounded
layer), and the visible result SHALL be byte-identical to a full recomposite
either way. The change SHALL still record exactly one undo state and update the
row projection. On a large reference document (≈4000²) the visibility toggle to
first correct pixel SHALL complete in under one second on the reference run.

#### Scenario: Eye toggle uses the region path [lvc_visibility_region]

- **WHEN** the eye toggle flips a regular raster layer on a large document
- **THEN** a region refresh fires for the layer's clamped region, no full
  `changed` recomposite runs, and the canvas shows the toggled state

#### Scenario: The region path matches a full recomposite [lvc_visibility_parity]

- **WHEN** a visibility toggle completes through the region path
- **THEN** the displayed document equals a full recomposite of the same state
  byte for byte

#### Scenario: An unrepresentable region falls back [lvc_visibility_fallback]

- **WHEN** a visibility change targets a layer with no representable region
- **THEN** the system falls back to the full `recomposite` and still shows the
  correct state

#### Scenario: One undo state per toggle [lvc_visibility_undo]

- **WHEN** a visibility toggle completes
- **THEN** exactly one undo state is recorded and one undo restores the prior
  visibility

#### Scenario: Large-image toggle is sub-second [lvc_visibility_latency]

- **WHEN** the eye toggle runs on the ≈4000² reference document
- **THEN** the first correct pixel is shown in under one second
