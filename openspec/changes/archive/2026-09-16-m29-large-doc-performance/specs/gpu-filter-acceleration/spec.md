## ADDED Requirements

### Requirement: Large-buffer GPU filter dispatch

The GPU filter path SHALL dispatch its compute shader as a two-dimensional grid
(an equivalent tiling) so that a filter applied to a buffer whose pixel count
exceeds the one-dimensional workgroup limit (65535 × 64 ≈ 4.19 MP) runs on the
GPU. It SHALL preserve byte-identical CPU fallback semantics and the existing
±1 LSB parity with `pictura_filters::apply` on the large buffer.

#### Scenario: A filter above the one-dimensional limit runs on the GPU

- **WHEN** an accelerated filter such as Surface Blur is applied to a buffer
  whose pixel count exceeds ~4.19 MP with GPU compute enabled on a machine with a
  usable adapter
- **THEN** the reported backend is `Backend::Gpu` and every colour channel
  differs from `pictura_filters::apply` by at most 1 LSB, with alpha unchanged on
  4-channel buffers
