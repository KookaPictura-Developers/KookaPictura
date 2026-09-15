## ADDED Requirements

### Requirement: GPU filter application entry point

The system SHALL provide a way to apply a `pictura_filters::Filter` to a buffer
through `pictura_render::gpu::apply_filter_active(filter, buf, gpu_enabled) ->
Result<Backend, FilterError>`. It SHALL run a wgpu compute pass and return
`Ok(Backend::Gpu)` when GPU compute is enabled, a usable adapter is available,
and a GPU kernel exists for the filter; otherwise it SHALL run
`pictura_filters::apply` on the CPU and return `Ok(Backend::Cpu)`. It MUST NOT
panic and MUST surface invalid filter parameters as `Err(FilterError)` from the
CPU oracle.

#### Scenario: GPU available and enabled

- **WHEN** `apply_filter_active` is called with `gpu_enabled` true on a machine
  with a usable adapter and a filter that has a GPU kernel
- **THEN** it returns `Ok(Backend::Gpu)` and the buffer was produced by the
  compute pass

#### Scenario: GPU disabled or unavailable

- **WHEN** `apply_filter_active` is called with `gpu_enabled` false, or no
  usable adapter exists
- **THEN** it returns `Ok(Backend::Cpu)` and the buffer was produced by
  `pictura_filters::apply`

#### Scenario: Invalid parameters surface an error

- **WHEN** `apply_filter_active` is called with filter parameters the CPU oracle
  rejects
- **THEN** it returns `Err(FilterError)` and does not panic

### Requirement: GPU filter results match the CPU oracle

Every accelerated pointwise and separable-convolution filter SHALL match the CPU
oracle: the GPU result MUST match `pictura_filters::apply` within ±1 LSB per
channel for representative inputs (gradient, flat, and single-colour buffers),
and MUST preserve the alpha channel bit-for-bit on 4-channel buffers.

#### Scenario: Representative kernel parity

- **WHEN** an accelerated filter from the set including Gaussian Blur, Box Blur,
  Motion Blur, Blur, Sharpen, Unsharp Mask, and High Pass is applied on a usable
  adapter to a representative input
- **THEN** every colour channel differs from `pictura_filters::apply` by at most
  1 LSB and alpha is unchanged

### Requirement: Default-on GPU filters

Filter application SHALL use the GPU by default when the M26 `gpuCompute`
preference is enabled and a usable adapter exists, consistent with the default-on
compositing requirement. When `gpuCompute` is disabled the system MUST force the
CPU implementation for every filter.

#### Scenario: Default routes to the GPU

- **WHEN** a supported filter is applied with the default `gpuCompute` value on a
  machine with a usable adapter
- **THEN** the GPU backend runs the filter

#### Scenario: Disabling forces the CPU

- **WHEN** `gpuCompute` is false and a supported filter is applied
- **THEN** `pictura_filters::apply` runs on the CPU and the reported backend is
  `Backend::Cpu`

### Requirement: Unsupported kernels fall back to CPU

A filter with no GPU kernel SHALL fall back to the CPU implementation and return
a result byte-identical to `pictura_filters::apply`, without an error or a
panic.

#### Scenario: Painterly filter falls back

- **WHEN** a filter with no GPU kernel (for example `ColoredPencil` or
  `OilPaint`) is applied while GPU compute is enabled and an adapter is
  available
- **THEN** the CPU implementation runs and the result is byte-identical to
  `pictura_filters::apply`, with no error and no panic

### Requirement: CPU fallback is deterministic

With GPU compute disabled or no adapter available, filter application SHALL be
byte-identical to `pictura_filters::apply` for every filter, including those with
a GPU kernel.

#### Scenario: Disabled GPU matches the oracle

- **WHEN** a supported (accelerated) filter and an unsupported (kernel-less)
  filter are each applied with GPU compute disabled
- **THEN** both results are byte-identical to `pictura_filters::apply`
