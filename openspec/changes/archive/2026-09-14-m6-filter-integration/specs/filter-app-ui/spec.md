## ADDED Requirements

### Requirement: PictureView filter command

`PictureView` MUST expose `apply_filter(kind: &QString) -> bool`. It SHALL apply the mapped filter to the topmost pixel layer — the last layer in the bottom-first `doc.layers` for which `adjustment.is_none() && !is_group` — using the active selection as the mask, then recomposite and emit `changed`. With no active selection it MUST apply over the full frame. It MUST return false when there is no document or when there is no pixel layer.

#### Scenario: Applies to the topmost pixel layer

- **WHEN** a document has several pixel layers and `apply_filter` is called
- **THEN** the last bottom-first pixel layer (skipping adjustment and group layers) is the one filtered

#### Scenario: A selection confines the change

- **WHEN** an active selection covers part of the document and `apply_filter` is called
- **THEN** selected pixels change and unselected pixels are unchanged after recompositing

#### Scenario: No selection applies full-frame

- **WHEN** `apply_filter` is called with no active selection
- **THEN** the filter applies to the whole layer rect

#### Scenario: No document or no pixel layer returns false

- **WHEN** `apply_filter` is called with no document, or with a document that has no pixel layer
- **THEN** it returns false and the document is unchanged

### Requirement: Filter-kind mapping and defaults

The command SHALL map each recognized `kind` to a concrete `pictura_filters::Filter` with fixed defaults: `gaussian-blur` → `GaussianBlur { radius: 5.0 }`, `box-blur` → `BoxBlur { radius: 3 }`, `motion-blur` → `MotionBlur { angle: 0.0, distance: 15 }`, `median` → `Median { radius: 2 }`, `despeckle` → `Despeckle`, `sharpen` → `Sharpen`, `sharpen-more` → `SharpenMore`, `unsharp-mask` → `UnsharpMask { amount: 150.0, radius: 1.0, threshold: 0 }`, and `add-noise` → `AddNoise { amount: 25.0, distribution: Uniform, monochromatic: false, seed: 1 }`. An unrecognized `kind` SHALL map to no filter and the command MUST return false without changing the document.

#### Scenario: Each known kind maps to its default filter

- **WHEN** `apply_filter` is called with each supported kind
- **THEN** the corresponding `Filter` with the listed defaults is applied

#### Scenario: Unknown kind is refused

- **WHEN** `apply_filter` is called with an unrecognized kind
- **THEN** it returns false and no layer is modified

### Requirement: Filter dock controls

The Qt dock MUST provide a filter combo box listing the supported kinds and an "Apply Filter" button that calls `apply_filter` with the combo's current kind. The image MUST refresh when the command succeeds.

#### Scenario: The Apply Filter button runs the selected filter

- **WHEN** a kind is chosen in the combo and "Apply Filter" is clicked
- **THEN** `apply_filter` runs with that kind and the displayed image refreshes

### Requirement: Filter confinement self-test

The headless `--self-test` MUST load a layered PSD, select a proper subset of the document, apply a filter to the topmost pixel layer confined by that selection, and assert that pixels outside the selection are unchanged while the selected region changed. It MUST exit non-zero on failure and MUST clean up before the remaining checks.

#### Scenario: Filtering is confined to the selection

- **WHEN** `--self-test` runs against a layered PSD
- **THEN** the selected region changes, pixels outside it are unchanged, and the run exits 0

#### Scenario: A failure exits non-zero

- **WHEN** the filtered result differs outside the selection or does not change inside it
- **THEN** the self-test exits non-zero
