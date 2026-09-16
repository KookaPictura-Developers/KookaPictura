## ADDED Requirements

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
