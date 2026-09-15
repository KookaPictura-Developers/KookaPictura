## ADDED Requirements

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
