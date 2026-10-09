## ADDED Requirements

### Requirement: GPU composites move pixels without host copies

A GPU composite SHALL write each layer's source planes and mask coverage
straight into the queue's staging memory, in parallel runs, without first
assembling them in a host buffer; SHALL read its result back into one
allocation taken from zeroed pages; and SHALL reuse its readback buffer across
composites, so a large readback does not fault a fresh mapping in every time.
The uploaded bytes SHALL equal the per-pixel reference assembly and the
composite SHALL keep matching the CPU oracle within ±1 LSB.

#### Scenario: The parallel writers equal the per-pixel reference [gt_writers]

- **WHEN** the source and mask writers fill a buffer for a covering RGB layer, a
  layer without alpha, a monochrome layer, a gray layer, and masks with and
  without data
- **THEN** the bytes equal the per-pixel reference assembly

#### Scenario: Large documents still match the CPU [gt_parity]

- **WHEN** the `gpu_parity` suite runs with `PICTURA_GPU_BENCH=1` (4000²
  three-layer and over-the-1-D-limit documents)
- **THEN** every composite matches the CPU within ±1 LSB
