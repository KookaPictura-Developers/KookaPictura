# gpu-compute-backend Specification

## Purpose
GPU compute as the default compositing backend with a capability probe, a disable switch, and persisted preference.
## Requirements
### Requirement: GPU compute is the default compositing backend

The system SHALL composite documents through
`pictura_render::gpu::composite_active(doc, gpu_enabled) -> (PixelBuffer, Backend)`.
When `gpu_enabled` is true and `pictura_render::gpu::gpu_available()` reports a
usable adapter, `composite_active` SHALL return the GPU composite and
`Backend::Gpu`; otherwise it SHALL return the CPU composite and `Backend::Cpu`.
The returned buffer MUST match `pictura_render::composite_rgba(doc)` within ±1 LSB
per channel regardless of the chosen backend.

#### Scenario: Adapter present and GPU enabled

- **WHEN** a document is composited with `gpu_enabled` true on a machine with a usable adapter
- **THEN** `composite_active` returns `Backend::Gpu` and a buffer within ±1 LSB of `composite_rgba(doc)`

#### Scenario: GPU disabled or unavailable

- **WHEN** `gpu_enabled` is false, or no usable adapter exists
- **THEN** `composite_active` returns `Backend::Cpu` and a buffer byte-equal to `composite_rgba(doc)`

### Requirement: Capability probe

`pictura_render::gpu::gpu_available()` SHALL report whether a usable Vulkan
adapter and device exist. The probe MUST be cheap to call repeatedly (its result
is cached) and MUST NOT panic. It SHALL return false on a machine with no usable
adapter.

#### Scenario: Probe reports a usable adapter

- **WHEN** `gpu_available()` is called on a machine with a working Vulkan adapter
- **THEN** it returns true, and repeated calls return the same cached value without re-creating the device

#### Scenario: Probe reports no adapter

- **WHEN** `gpu_available()` is called on a machine with no usable adapter
- **THEN** it returns false and does not panic

### Requirement: User can disable GPU compute

A boolean session preference `gpuCompute` SHALL control GPU compositing. Its
default SHALL be true, and a missing value SHALL be read as true. When `gpuCompute`
is false the system MUST force the CPU backend for every composite while keeping
results within the parity tolerance. A checkable, implemented command SHALL toggle
the preference and reflect the persisted value.

#### Scenario: Disabling forces the CPU backend

- **WHEN** `gpuCompute` is false and a document is composited
- **THEN** every call returns `Backend::Cpu` with a buffer within ±1 LSB of `composite_rgba(doc)`

#### Scenario: Toggle command reflects the value

- **WHEN** the GPU-compute command is invoked
- **THEN** the preference flips, the document recomposites on the newly selected backend, and the command's checked state matches the persisted `gpuCompute` value

### Requirement: GPU compute preference persists

The `gpuCompute` value SHALL round-trip through the XDG session store
(`crates/pictura-app/cpp/session.{h,cpp}`). The session store's schema version
SHALL be bumped to 2. A store without the field MUST load `gpuCompute` as true.

#### Scenario: Round-trip

- **WHEN** a session with `gpuCompute` false is saved and then loaded
- **THEN** the loaded `gpuCompute` is false, with the store reporting schema version 2

#### Scenario: Missing field defaults true

- **WHEN** a schema-1 store without a `gpuCompute` field is loaded
- **THEN** `gpuCompute` is true

### Requirement: Status bar reports the active backend

The status bar SHALL show the active compositing backend: `GPU` when the GPU
backend is active, `CPU` when the user disabled GPU compute, and `CPU (no GPU)`
when no adapter is available.

#### Scenario: Backend indicator values

- **WHEN** the status bar is updated with GPU compute active, with GPU compute user-disabled, and with no adapter available
- **THEN** it shows `GPU`, `CPU`, and `CPU (no GPU)` respectively

### Requirement: No GPU never blocks a document

When the capability probe fails, the system SHALL fall back to the CPU backend,
the toggle command SHALL be disabled/greyed, and compositing SHALL continue to
work without a panic or an error dialog, consistent with the existing `Graceful
CPU fallback` requirement.

#### Scenario: Document opens without a GPU

- **WHEN** a document is opened on a machine where `gpu_available()` is false
- **THEN** it composites on the CPU without a panic or error dialog, and the toggle command is disabled

#### Scenario: Toggle stays disabled without an adapter

- **WHEN** the GPU-compute command is offered while no adapter is usable
- **THEN** it is disabled/greyed and invoking it does not enable the GPU backend

