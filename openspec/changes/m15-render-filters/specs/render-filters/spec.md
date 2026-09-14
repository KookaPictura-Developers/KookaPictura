# render-filters Specification (Delta)

## ADDED Requirements

### Requirement: Clouds replaces RGB with a seeded two-color noise field

The system SHALL provide `Filter::Clouds { color_a: [u8; 3], color_b: [u8; 3], starker: bool, seed: u64 }`. It SHALL replace every pixel's RGB on the layer with a deterministic fractal noise field interpolated between `color_a` and `color_b`, driven by `ChaCha8Rng` seeded with `seed`, leaving alpha untouched. The `starker` variant SHALL produce a higher-contrast field than the non-starker variant for the same seed. It MUST NOT read the pre-existing pixel colors.

#### Scenario: Clouds replaces pixels deterministically

- **WHEN** `Clouds` is applied twice to the same buffer with the same seed and colors
- **THEN** both results are bit-identical, every RGB value lies within the per-channel range spanned by `color_a` and `color_b`, and alpha equals its pre-call value

#### Scenario: Starker variant differs in contrast

- **WHEN** `Clouds` runs with `starker: false` and `starker: true` at the same seed
- **THEN** the two outputs differ and the starker output has the higher per-channel standard deviation

### Requirement: Difference Clouds blends the noise field via Difference

The system SHALL provide `Filter::DifferenceClouds { color_a: [u8; 3], color_b: [u8; 3], starker: bool, seed: u64 }`. It SHALL generate the Clouds noise field for the same parameters and set each pixel's RGB to the per-channel absolute difference between the pre-existing color and the field color, leaving alpha untouched. Applying it repeatedly SHALL change the result (cumulative marble patterning).

#### Scenario: Difference formula holds per pixel

- **WHEN** `DifferenceClouds` is applied to a known buffer
- **THEN** every resulting RGB channel equals `|existing − field|` computed from the Clouds field for the same seed, and alpha is unchanged

#### Scenario: Repeated application keeps changing the image

- **WHEN** `DifferenceClouds` runs twice in sequence on a non-uniform buffer
- **THEN** the second result differs from the first

### Requirement: Fibers renders seeded directional fibers

The system SHALL provide `Filter::Fibers { variance: f64, strength: f64, color_a: [u8; 3], color_b: [u8; 3], seed: u64 }`. It SHALL replace the layer's RGB with a deterministic noise field elongated along the horizontal axis and interpolated between `color_a` and `color_b`, leaving alpha untouched. `variance` SHALL control per-fiber color variation and `strength` the weave tightness. Values of `variance` outside 0..=100 or `strength` outside 1..=100 SHALL be rejected with `FilterError::InvalidParams` before any mutation.

#### Scenario: Fibers parameters shape the field

- **WHEN** `Fibers` is applied with minimum variance and with maximum variance at the same seed
- **THEN** both outputs are bit-identical across repeated runs at their own settings, all RGB values lie within the color range, and the low-variance run varies less along each row than the high-variance run

#### Scenario: Out-of-range parameters are rejected untouched

- **WHEN** `Fibers` is called with `variance` below 0 or above 100, or `strength` below 1 or above 100
- **THEN** it returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

### Requirement: Lens Flare renders a deterministic additive light pass

The system SHALL provide `Filter::LensFlare { brightness: f64, center: (f64, f64), lens: LensType }` with `LensType` values `Zoom`, `Prime35`, `Prime105`, and `MoviePrime`. It SHALL additively composite a bright core at `center` (unit coordinates, clamped into 0..=1), ghost reflections whose geometry depends on `lens`, and starburst rays onto the existing RGB, clamping results to 0..=255 and leaving alpha untouched. `brightness` outside 10..=300 SHALL be rejected with `FilterError::InvalidParams` before any mutation. The filter SHALL be fully deterministic (no seed).

#### Scenario: Brightness is monotone at the core

- **WHEN** `LensFlare` runs at brightness 10, 100, and 300 with the same center and lens on identical buffers
- **THEN** the core-region luminance is non-decreasing in brightness and the buffer is bit-identical across repeated runs at the same settings

#### Scenario: Lens type and center change the geometry

- **WHEN** `LensFlare` runs with each of the four `LensType` values, and with two different centers
- **THEN** the four lens outputs all differ, and moving the center moves the brightest pixel accordingly

#### Scenario: Out-of-range brightness is rejected untouched

- **WHEN** `LensFlare` is called with brightness below 10 or above 300
- **THEN** it returns `FilterError::InvalidParams` and the buffer is bit-identical to its prior state

### Requirement: Dispatch integration and app kinds

The `Filter` enum SHALL carry the four new variants and `pictura_filters::apply` SHALL dispatch them. The app SHALL expose kinds `clouds`, `difference-clouds`, `fibers`, and `lens-flare` through `filter_from_kind` with fixed defaults (black/white colors, `starker: false`, seed 1, variance 16, strength 4, brightness 100, unit center, Zoom lens), and the Qt shell filter combo SHALL list them. `pictura_render::apply_filter` SHALL apply them through the existing layer path unchanged.

#### Scenario: App applies a Render filter through the layer path

- **WHEN** `apply_filter("clouds")` is invoked in the app with no selection
- **THEN** the topmost pixel layer's RGB changes within its rect, the rest of the composite is unchanged, and a second application with the same kind is bit-identical to the first result

### Requirement: Headless self-test coverage

The app `--self-test` SHALL apply at least one Render filter end-to-end: assert the command returns `true`, the composite changes only inside the target layer's rect, alpha outside the rect is unchanged, and a repeated application is bit-identical (seeded replacement semantics).

#### Scenario: Self-test proves a Render filter in the app

- **WHEN** the app runs `--self-test` with the layered fixture
- **THEN** the Render-filter assertions pass and the self-test exits successfully
