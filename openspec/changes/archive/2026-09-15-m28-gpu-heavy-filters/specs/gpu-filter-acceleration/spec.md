## MODIFIED Requirements

### Requirement: GPU filter results match the CPU oracle

The accelerated set SHALL include the deterministic high-cost kernels Surface
Blur, Median, Maximum, Minimum, Custom (5×5), and Oil Paint. Every accelerated
pointwise, separable-convolution, window, and neighbourhood/effect filter MUST
match the CPU oracle: the GPU result MUST match `pictura_filters::apply` within
±1 LSB per channel for representative inputs (gradient, flat, and single-colour
buffers), and MUST preserve the alpha channel bit-for-bit on 4-channel buffers.

#### Scenario: Newly accelerated window kernels match the oracle

- **WHEN** Surface Blur or Median from the accelerated set is applied on a usable
  adapter to a representative input
- **THEN** every colour channel differs from `pictura_filters::apply` by at most
  1 LSB and alpha is unchanged on 4-channel buffers

#### Scenario: Oil Paint matches the oracle

- **WHEN** Oil Paint is applied on a usable adapter to a representative input
- **THEN** every colour channel differs from `pictura_filters::apply` by at most
  1 LSB and alpha is unchanged on 4-channel buffers

#### Scenario: Maximum and Minimum morphology matches the oracle

- **WHEN** Maximum or Minimum is applied on a usable adapter to a representative
  input
- **THEN** every colour channel differs from `pictura_filters::apply` by at most
  1 LSB and alpha is unchanged on 4-channel buffers

#### Scenario: Custom 5×5 matches the oracle

- **WHEN** Custom is applied with a 5×5 kernel on a usable adapter to a
  representative input
- **THEN** every colour channel differs from `pictura_filters::apply` by at most
  1 LSB and alpha is unchanged on 4-channel buffers

## ADDED Requirements

### Requirement: Profile-driven accelerated-kernel selection

A filter SHALL be GPU-accelerated only when it is high-cost at 1024² (a profiling
baseline of at least ~150 ms recorded in
`crates/pictura-filters/tests/profile.rs`) AND a GPU kernel reaches ±1 LSB parity
with `pictura_filters::apply`. A kernel that cannot reach parity MUST stay on the
CPU with a byte-identical fallback, the tolerance MUST NOT be widened to admit it,
and every stochastic (seeded) filter — `Crystallize`, `Watercolor`,
`ConteCrayon`, `PaintDaubs`, `DryBrush`, `OceanRipple`, `Spatter`, `Sponge`,
`PaletteKnife`, `AddNoise`, and `ColoredPencil` — SHALL remain on the CPU with a
byte-identical fallback.

#### Scenario: Stochastic filter falls back byte-identically

- **WHEN** a stochastic (seeded) filter such as `Crystallize` or `AddNoise` is
  applied while GPU compute is enabled and an adapter is available
- **THEN** the CPU implementation runs via `pictura_filters::apply` and the
  result is byte-identical to the oracle, with no error and no panic

#### Scenario: High-cost filter with a working kernel runs on the GPU

- **WHEN** a filter whose profiled 1024² baseline is at least ~150 ms and whose
  GPU kernel reaches ±1 LSB parity is applied with GPU compute enabled on a
  machine with a usable adapter
- **THEN** the reported backend is `Backend::Gpu` and the result matches the
  oracle within ±1 LSB

#### Scenario: Kernel without parity stays on the CPU

- **WHEN** a kernel is admitted on cost but cannot reach ±1 LSB parity with the
  CPU oracle on the parity corpus
- **THEN** the filter stays on the CPU, its fallback is byte-identical to
  `pictura_filters::apply`, and the tolerance is not widened to admit it

### Requirement: Heavy-kernel performance evidence

The GPU path for the accelerated heavy kernels SHALL be materially faster than
the CPU baseline. Specifically, a 1024×1024 RGB Surface Blur SHALL complete at
least ~10× faster on the GPU than the CPU baseline recorded at ~7.1 s.

#### Scenario: Surface Blur speedup is recorded

- **WHEN** Surface Blur is timed at 1024×1024 on the GPU and against the CPU
  baseline and the evidence is recorded in `STATE.md` / the M28 brief
- **THEN** the recorded timing shows the GPU completing the filter at least ~10×
  faster than the CPU baseline
