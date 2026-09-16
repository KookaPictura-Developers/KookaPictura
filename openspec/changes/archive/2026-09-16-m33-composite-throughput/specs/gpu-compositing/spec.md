## ADDED Requirements

### Requirement: Row-wise planar source assembly

`Gpu::build_source` SHALL assemble a pixel layer's planar 8-bit source planes
with whole-row copies (`copy_from_slice`/`clone_from_slice`) when each channel
plane covers the layer-rect ∩ region row intersection, instead of a per-pixel
loop. It MUST preserve the existing semantics exactly: planar straight-alpha
planes; a grayscale document produces two planes with colour read from channel 0;
an absent green or blue channel aliases channel 0; an absent alpha plane reads
255; an absent colour channel reads 0. When a channel plane does not cover the
intersection (a short or absent plane, or channel 0 absent) the assembly SHALL
fall back to the existing per-pixel path. The assembled plane bytes MUST be
byte-identical (0 LSB) to the per-pixel assembly.

#### Scenario: A covering plane assembles row-wise and is byte-identical

- **WHEN** `build_source` assembles a layer whose channel data covers the whole
  clamped row intersection
- **THEN** the planes are filled with whole-row copies and are byte-identical to
  the per-pixel assembly of the same layer

#### Scenario: A short or absent plane falls back without changing bytes

- **WHEN** a channel plane is short over the clamped row intersection, or the
  green/blue channel is absent (aliasing channel 0), or the alpha plane is absent
- **THEN** the assembly falls back to the per-pixel path, the absent alpha plane
  reads 255, and the planes remain byte-identical to the per-pixel assembly

### Requirement: Row-wise mask coverage assembly

`Gpu::build_mask` SHALL fill the coverage plane with row-wise operations when the
layer has no enabled data-carrying mask (`mask` is absent, `disabled`, or carries
no data): coverage 255 inside the layer/group/adjustment influence rectangle and 0
outside. When the mask is enabled and carries data it SHALL keep the per-pixel
`mask_alpha` path. The coverage plane MUST be byte-identical (0 LSB) to the
per-pixel coverage.

#### Scenario: A maskless layer is filled row-wise and is byte-identical

- **WHEN** `build_mask` runs for a layer with no enabled data-carrying mask
- **THEN** the influence rectangle is filled 255 and the remainder stays 0, and
  the plane is byte-identical to the per-pixel `mask_alpha` coverage

#### Scenario: A data-carrying mask keeps the per-pixel path

- **WHEN** `build_mask` runs for a layer whose enabled mask carries pixel data
- **THEN** the per-pixel `mask_alpha` path is used and the plane is byte-identical
  to the previous coverage

### Requirement: Fused planar readback

The canvas readback SHALL de-interleave the mapped staging bytes directly into
the planar straight-alpha `PixelBuffer` and MUST NOT allocate or copy a host-side
packed RGBA intermediate. The returned `PixelBuffer` MUST be byte-identical
(0 LSB) to the previous packed readback followed by the planar de-interleave.

#### Scenario: The readback equals the packed-then-de-interleaved output

- **WHEN** a composite is read back through the fused path
- **THEN** the returned `PixelBuffer` is byte-identical to the output of the
  previous packed readback and `to_pixel_buffer` de-interleave

### Requirement: GPU-side canvas initialization

The working canvas SHALL be initialized on the GPU with a command-buffer clear
over a storage buffer that carries `COPY_DST` usage, rather than by allocating and
uploading a full-canvas host zero buffer. The clear SHALL be submitted before the
first composite dispatch that reads that canvas, so no dispatch observes
uninitialized canvas memory and the canvas bytes MUST be identical to the
previously uploaded zero canvas.

#### Scenario: A GPU-cleared canvas composites identically

- **WHEN** a composite runs on a canvas initialized by the command-buffer clear,
  including a group whose inner canvas is also cleared on the GPU
- **THEN** the composited output is byte-identical to the same composite on an
  uploaded zero canvas

### Requirement: Full-composite throughput evidence

`composite_gpu` MUST remain byte-identical (0 LSB) to itself across repeated
composites and to the region path, and within ±1 LSB per channel of
`composite_rgba(doc)` with the alpha channel unchanged. An `#[ignore]` profile
test SHALL print the full-composite phase timings (canvas clear, source assembly,
mask assembly, readback de-interleave, and the total) for a 4000×4000
two-pixel-layer document so the improvement is measurable, and SHALL skip with a
printed note when no usable adapter exists. The profile test SHALL NOT assert a
throughput ratio.

#### Scenario: The parity contract holds after the data-path change

- **WHEN** the `gpu_parity` scenes are composited through the row-wise assembly,
  fused readback, and GPU-side clear on a usable adapter
- **THEN** repeated GPU composites of the same document are byte-identical, the
  region composite equals the full-composite slice at 0 LSB, every channel
  differs from `composite_rgba(doc)` by at most 1 LSB, and the alpha channel is
  unchanged

#### Scenario: The ignored profile test prints the phase timings

- **WHEN** the `#[ignore]` profile test is run with `--ignored --nocapture` on a
  machine with a usable adapter
- **THEN** it prints the per-phase timings and the total for the 4000×4000
  two-pixel-layer composite

#### Scenario: A full composite stays within a small time budget

- **WHEN** a 4000×4000 two-pixel-layer document is composited through the full
  GPU path after the warm-up composite
- **THEN** the composite completes within a small time budget
