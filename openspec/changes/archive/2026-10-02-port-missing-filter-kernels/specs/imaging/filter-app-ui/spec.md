## MODIFIED Requirements

### Requirement: Filter-kind mapping and defaults

The command SHALL map each recognized `kind` to a concrete `pictura_filters::Filter` with fixed defaults: `gaussian-blur` → `GaussianBlur { radius: 5.0 }`, `box-blur` → `BoxBlur { radius: 3 }`, `motion-blur` → `MotionBlur { angle: 0.0, distance: 15 }`, `median` → `Median { radius: 2 }`, `despeckle` → `Despeckle`, `sharpen` → `Sharpen`, `sharpen-more` → `SharpenMore`, `unsharp-mask` → `UnsharpMask { amount: 150.0, radius: 1.0, threshold: 0 }`, `add-noise` → `AddNoise { amount: 25.0, distribution: Uniform, monochromatic: false, seed: 1 }`, `dust-and-scratches` → `DustAndScratches { radius: 1, threshold: 0 }`, `extrude` → `Extrude { kind: Blocks, size: 30, depth: 30.0, level_based: true, solid_front: false, mask_incomplete: false }`, `tiles` → `Tiles { count: 10, offset: 10, fill: BackgroundColor, foreground: [0, 0, 0], background: [255, 255, 255] }`, `trace-contour` → `TraceContour { level: 128, edge: Lower }`, `wind` → `Wind { method: Wind, from_right: true }`, and `smart-sharpen` → `SmartSharpen { amount: 100.0, radius: 1.0, reduce_noise: 0.0, remove: GaussianBlur, angle: 0.0 }`. An unrecognized `kind` SHALL map to no filter and the command MUST return false without changing the document.

#### Scenario: Each known kind maps to its default filter

- **WHEN** `apply_filter` is called with each supported kind
- **THEN** the corresponding `Filter` with the listed defaults is applied

#### Scenario: Unknown kind is refused

- **WHEN** `apply_filter` is called with an unrecognized kind
- **THEN** it returns false and no layer is modified
