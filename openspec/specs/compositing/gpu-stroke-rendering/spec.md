# compositing/gpu-stroke-rendering Specification

## Purpose
A GPU-resident paint-stroke path: the target layer stays resident for the
stroke, each dab composites on the GPU over its bounding box, and only the
presented region is read back. The CPU stroke engine remains the oracle, the
golden verifier and the fallback.

## Requirements

### Requirement: GPU-resident stroke rendering with a CPU oracle

When a Vulkan adapter is available and the layer stack is supported, the system
SHALL render the in-progress stroke on the GPU: the target layer's source SHALL
stay resident for the stroke's duration, each dab SHALL be computed on the GPU
over its bounding box, and only the presented region SHALL be read back to the
host. The CPU stroke engine SHALL remain the oracle, the CI/golden verifier and
the fallback, and the GPU path's committed pixels SHALL equal the GPU stroke and
lie within ±1 LSB of the CPU oracle.

#### Scenario: A supported stroke renders on the GPU [gsr_gpu_selected]

- **WHEN** a stroke begins on a supported stack with an available adapter
- **THEN** its presented regions are produced on the GPU, and the target layer's
  source is uploaded once for the stroke rather than once per presented region

#### Scenario: An unsupported stack falls back to the CPU [gsr_cpu_fallback]

- **WHEN** the stack uses a blend mode, adjustment or layer feature with no GPU
  implementation, or no adapter is available
- **THEN** the stroke uses the CPU engine exactly as before, with no GPU buffers
  allocated for it

#### Scenario: Only the presented region is read back [gsr_region_readback]

- **WHEN** a GPU stroke presents a region
- **THEN** only that region is mapped to the host, so the GPU-to-CPU stall is
  bounded by the display region and not by the brush's bounding box

#### Scenario: Committed pixels are the GPU stroke's [gsr_commit_parity]

- **WHEN** a GPU stroke commits
- **THEN** the committed document is the GPU-authored stroke, produced by the
  same dabs on the same target layer, and lies within ±1 LSB of the same stroke
  run on the CPU, so the CPU engine stays the oracle, the golden verifier and
  the fallback

### Requirement: Bounded per-dab write-back and stroke-start cost

The system SHALL keep the host cost of a GPU dab proportional to its changed
region: the changed region SHALL be de-interleaved into channel planes on the
GPU and patched into the working layer by row copies, so the host performs no
per-pixel gather or scatter and holds no intermediate interleaved buffer. The
stroke SHALL build its GPU seed with a straight interleave of the target layer's
channel planes where the layer spans the document, and upload it without a
per-pixel document pass.

#### Scenario: Overlapping dabs patch planar rows [gsr_planar_patch]

- **WHEN** two GPU dabs overlap on the same region
- **THEN** each dab's changed rectangle is de-interleaved on the GPU into four
  channel planes and copied row-wise into the working layer, and only that
  rectangle is read back

#### Scenario: A document-spanning layer seeds without a per-pixel pass [gsr_seed_build]

- **WHEN** a GPU stroke begins on a layer whose rect spans the document
- **THEN** its seed interleaves the layer's four channel planes directly, with
  no per-pixel channel lookup, and is written into the resident buffer without a
  staging copy
