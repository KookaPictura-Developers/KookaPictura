## MODIFIED Requirements

### Requirement: Layer effects are rejected before GPU dispatch

A visible layer carrying a decodable object-based layer effect SHALL make
`composite_gpu` return `GpuError::UnsupportedLayerEffect` before dispatching that
layer to the GPU, without panicking. A layer counts as effect-bearing when its
`lfx2` block decodes to an enabled and present `DropShadow`, an enabled and
present `OuterGlow`, or an enabled and present `InnerShadow`; a disabled, absent,
or malformed effect SHALL NOT reject the document.
`composite_active` and `composite_gpu_or_cpu` SHALL fall back to the CPU
composite for a document with such a layer, and the fallback output SHALL be
byte-identical to `composite_rgba` of the same document.

#### Scenario: A drop-shadow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present drop shadow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An outer-glow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present outer glow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: An inner-shadow layer is rejected before dispatch

- **WHEN** `composite_gpu` is called on a document whose visible layer carries an enabled and present inner shadow
- **THEN** it returns `Err(GpuError::UnsupportedLayerEffect)` and does not panic

#### Scenario: The effect document falls back to the CPU composite

- **WHEN** `composite_gpu_or_cpu` is called on a document whose visible layer carries an enabled and present drop shadow, outer glow, or inner shadow
- **THEN** it returns the same buffer as `composite_rgba` for that document

#### Scenario: A disabled effect does not reject the GPU

- **WHEN** `composite_gpu` is called on a document whose only effect is disabled
- **THEN** the disabled effect does not by itself produce `UnsupportedLayerEffect`
