# gpu-compositing Specification

## Purpose
TBD - created by archiving change m2-gpu-compositing. Update Purpose after archive.
## Requirements
### Requirement: GPU compositor matches the CPU oracle within ±1 LSB

`pictura_render::composite_gpu(doc)` SHALL composite the document's layer stack
on a wgpu compute shader and return the same 4-channel planar straight-alpha
8-bit `PixelBuffer` shape as `pictura_render::composite_rgba(doc)`. For Normal,
every separable blend mode (including the whole-RGB Darken/Lighten comparisons
`DarkerColor` and `LighterColor`), and the four non-separable modes (`Hue`,
`Saturation`, `Color`, `Luminosity`), the GPU output MUST match the CPU oracle
within ±1 LSB per channel. The GPU path SHALL likewise apply the five supported
adjustment layers (invert, posterize, threshold, brightness/contrast,
hue/saturation) at the layer's stack position and MUST match the CPU oracle
within ±1 LSB. The GPU path MUST reproduce the CPU semantics for per-layer masks,
layer opacity, clipping-free pixel layers, and both pass-through and isolated
group compositing. The CPU compositor remains the oracle and MUST NOT change.

#### Scenario: Separable mode parity

- **WHEN** a document of visible pixel layers using Normal or a separable blend
  mode is composited on a usable Vulkan adapter
- **THEN** every channel of `composite_gpu(doc)` differs from `composite_rgba(doc)` by at most 1 LSB, and the buffer dimensions and channel count are identical

#### Scenario: Non-separable mode parity

- **WHEN** a document of visible pixel layers using `Hue`, `Saturation`, `Color`,
  or `Luminosity` is composited on a usable Vulkan adapter
- **THEN** every channel of `composite_gpu(doc)` differs from `composite_rgba(doc)` by at most 1 LSB

#### Scenario: Adjustment layer parity

- **WHEN** a document contains a visible invert, posterize, threshold,
  brightness/contrast, or hue/saturation adjustment layer
- **THEN** the GPU composite matches `composite_rgba(doc)` within ±1 LSB, with the
  adjustment applied at the same stack position

#### Scenario: Gradients, masks, and opacity

- **WHEN** a layer carries a raster mask, non-255 opacity, or partial alpha, and
  its blend mode is separable or non-separable
- **THEN** the GPU composite matches the CPU composite within ±1 LSB, including
  in the masked-out and partially transparent regions

#### Scenario: Group semantics

- **WHEN** the stack contains a pass-through group or an isolated group blended
  onto the running canvas
- **THEN** the GPU result matches the CPU result within ±1 LSB for the group and
  its children

### Requirement: CPU-only modes are rejected before dispatch

`Dissolve` SHALL be the only CPU-only blend mode: its per-pixel randomness is
not bit-reproducible on the GPU. The four non-separable modes (`Hue`,
`Saturation`, `Color`, `Luminosity`) are supported on the GPU. The five
supported adjustment kinds (invert, posterize, threshold, brightness/contrast,
hue/saturation) are applied on the GPU; any other adjustment kind SHALL make
`composite_gpu` return `GpuError::UnsupportedAdjustment` for that layer without
dispatching it to the shader and without panicking.

#### Scenario: Dissolve mode

- **WHEN** `composite_gpu` is called on a document whose visible layer uses
  `Dissolve`
- **THEN** it returns `Err(GpuError::UnsupportedMode(BlendMode::Dissolve))`

#### Scenario: Supported adjustment composites

- **WHEN** `composite_gpu` is called on a document whose visible adjustment layer
  is one of the five supported kinds
- **THEN** it returns `Ok` and applies the adjustment at the layer's stack
  position rather than erroring

#### Scenario: Unsupported adjustment kind

- **WHEN** `composite_gpu` is called on a document whose visible layer is an
  adjustment layer of any other kind
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

### Requirement: GPU-native compositing data path

The GPU compositor SHALL consume each layer's pixel data as 8-bit planar channel
samples over the layer's clamped rect, uploaded directly, without a host-side
full-canvas floating-point assembly of the source or mask. The compute shader
SHALL sample the source channels and the layer-mask coverage itself, the working
canvas SHALL be kept as packed 8-bit RGBA rather than `f32`, and the compositor
SHALL return packed 8-bit output de-interleaved to the planar `PixelBuffer`.
Output MUST remain identical in shape to `composite_rgba` and within ±1 LSB per
channel of the CPU oracle. The public API (`composite_gpu`, `composite_active`,
`gpu_available`, `Backend`, `GpuError`) SHALL be unchanged.

#### Scenario: Parity after the data-path change

- **WHEN** a document using Normal, a separable or non-separable blend mode, or
  one of the supported adjustment layers is composited through the 8-bit planar
  data path on a usable adapter
- **THEN** every channel of the GPU result differs from `composite_rgba(doc)` by
  at most 1 LSB, and the CPU oracle is unchanged

#### Scenario: Output shape is unchanged

- **WHEN** any document is composited through the GPU-native data path
- **THEN** the returned `PixelBuffer` has the same dimensions and channel count
  as `composite_rgba(doc)`

#### Scenario: Group layer dispatch

- **WHEN** the stack contains a pass-through or isolated group
- **THEN** the group dispatch binds the group's inner packed 8-bit canvas as its
  source without re-materializing it to `f32`, and the GPU result still matches
  the CPU oracle within ±1 LSB

### Requirement: Large-document GPU dispatch

The GPU compositor SHALL dispatch its compute shader as a two-dimensional grid
(an equivalent tiling) so that any document whose pixel count exceeds the
one-dimensional workgroup limit (`max_compute_workgroups_per_dimension`, 65535
workgroups × 64 threads ≈ 4.19 MP) still runs on the GPU rather than falling back
to the CPU. The shader SHALL derive the linear pixel index from a row stride
passed in the uniform, so that no invocation is skipped or double-covered,
preserving the existing ±1 LSB parity with the CPU oracle and the existing
`GpuError` semantics. A document whose pixel count exceeds the two-dimensional
workgroup product (`limit × limit × 64`) SHALL return `GpuError::TooLarge`
without panicking.

#### Scenario: A document above the one-dimensional limit composites on the GPU

- **WHEN** a document whose pixel count exceeds ~4.19 MP (for example 4000×4000)
  is composited with GPU compute enabled on a machine with a usable adapter
- **THEN** the reported backend is `Backend::Gpu`, every channel differs from
  `composite_rgba(doc)` by at most 1 LSB, and the alpha channel is unchanged

#### Scenario: A document above the two-dimensional product limit is rejected

- **WHEN** `composite_gpu` is called on a document whose pixel count exceeds
  `max_compute_workgroups_per_dimension × max_compute_workgroups_per_dimension ×
  64`
- **THEN** it returns `Err(GpuError::TooLarge)` and does not panic

