## ADDED Requirements

### Requirement: GPU compositor matches the CPU oracle within ±1 LSB

`pictura_render::composite_gpu(doc)` SHALL composite the document's layer stack
on a wgpu compute shader and return the same 4-channel planar straight-alpha
8-bit `PixelBuffer` shape as `pictura_render::composite_rgba(doc)`. For Normal
and every separable blend mode, including the whole-RGB Darken/Lighten
comparisons (`DarkerColor`, `LighterColor`), the GPU output MUST match the CPU
oracle within ±1 LSB per channel. The GPU path MUST reproduce the CPU semantics
for per-layer masks, layer opacity, clipping-free pixel layers, and both
pass-through and isolated group compositing. The CPU compositor remains the
oracle and MUST NOT change.

#### Scenario: Separable mode parity

- **WHEN** a document of visible pixel layers using Normal or a separable blend
  mode is composited on a usable Vulkan adapter
- **THEN** every channel of `composite_gpu(doc)` differs from `composite_rgba(doc)` by at most 1 LSB, and the buffer dimensions and channel count are identical

#### Scenario: Gradients, masks, and opacity

- **WHEN** a layer carries a raster mask, non-255 opacity, or partial alpha, and
  its blend mode is separable
- **THEN** the GPU composite matches the CPU composite within ±1 LSB, including
  in the masked-out and partially transparent regions

#### Scenario: Group semantics

- **WHEN** the stack contains a pass-through group or an isolated group blended
  onto the running canvas
- **THEN** the GPU result matches the CPU result within ±1 LSB for the group and
  its children

### Requirement: CPU-only modes are rejected before dispatch

The GPU compositor SHALL treat Hue, Saturation, Color, Luminosity, and Dissolve
as CPU-only. `composite_gpu` MUST return `GpuError::UnsupportedMode(mode)` for a
visible layer using one of them, and MUST return
`GpuError::UnsupportedAdjustment` for a visible adjustment layer, without
dispatching that layer to the shader and without panicking.

#### Scenario: Non-separable mode

- **WHEN** `composite_gpu` is called on a document whose visible layer uses
  `Hue`, `Saturation`, `Color`, or `Luminosity`
- **THEN** it returns `Err(GpuError::UnsupportedMode(mode))` for that mode

#### Scenario: Dissolve mode

- **WHEN** `composite_gpu` is called on a document whose visible layer uses
  `Dissolve`
- **THEN** it returns `Err(GpuError::UnsupportedMode(BlendMode::Dissolve))`

#### Scenario: Adjustment layer

- **WHEN** `composite_gpu` is called on a document whose visible layer is an
  adjustment layer
- **THEN** it returns `Err(GpuError::UnsupportedAdjustment)`

### Requirement: Graceful CPU fallback when no GPU is available

The GPU path MUST NOT panic when no usable adapter exists. Adapter/device
creation failure, documents exceeding device buffer or workgroup limits, and
readback failure SHALL each produce a `GpuError` variant
(`Unavailable`, `TooLarge`, `Readback`). `composite_gpu_or_cpu(doc)` MUST return
the CPU compositor's result whenever `composite_gpu` returns any `GpuError`, and
its output for a CPU-only stack MUST be byte-identical to
`composite_rgba(doc)`.

#### Scenario: No Vulkan adapter

- **WHEN** `composite_gpu` runs on a machine with no usable Vulkan adapter
- **THEN** it returns `Err(GpuError::Unavailable)` and does not panic, and
  `composite_gpu_or_cpu` returns the CPU composite

#### Scenario: Document exceeds device limits

- **WHEN** the document's buffers or dispatch count exceed the device limits
- **THEN** `composite_gpu` returns `Err(GpuError::TooLarge)` and does not panic

#### Scenario: Fallback for a CPU-only stack

- **WHEN** a document contains a color-luminosity or Dissolve layer and is passed
  to `composite_gpu_or_cpu`
- **THEN** the returned buffer equals `composite_rgba(doc)` byte for byte
