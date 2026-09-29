# Spec Delta

## ADDED Requirements

### Requirement: Row-parallel CPU fallback composite

The CPU compositor SHALL evaluate independent output rows in parallel. Its output
MUST be byte-identical to the single-threaded composite for every layer stack, and
the CPU oracle's output MUST NOT change: `composite_rgba` returns exactly the bytes
it returned before this change. Only its execution is parallelised. Layer order,
group and knockout topology, and neighborhood effect kernels (blur-, offset-, and
shape-based layer effects) SHALL remain sequential; only per-pixel content loops
whose rows are independent MAY run in parallel. The GPU compositor remains the
preferred backend; this parallelism accelerates the CPU fallback when no adapter is
usable, the stack is unsupported on the GPU, or the document exceeds the device
buffer limits.

#### Scenario: Parallel output equals the sequential oracle

- **WHEN** a document is composited through the parallel CPU path and through the
  single-threaded oracle
- **THEN** the two buffers are byte-identical

#### Scenario: The CPU oracle's output is unchanged

- **WHEN** `composite_rgba` is run before and after the parallelism change on the
  same document
- **THEN** it returns byte-identical output; only its execution differs

#### Scenario: A stack with an effect or knockout is unaffected

- **WHEN** a document containing a layer effect or a knockout layer is composited
- **THEN** the result is byte-identical to the single-threaded composite of the
  same document
