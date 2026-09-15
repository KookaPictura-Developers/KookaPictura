## MODIFIED Requirements

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
