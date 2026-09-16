## ADDED Requirements

### Requirement: Layer translation uses the active backend

Translating a layer SHALL recompute the document composite through the active
backend — the GPU compositor when GPU compute is enabled and a usable adapter
exists, otherwise the CPU compositor — and SHALL remain within ±1 LSB of the CPU
oracle. The CPU `translate_layer` and `recompute` remain the oracle and MUST NOT
change.

#### Scenario: A Move commit on a large document composites on the GPU

- **WHEN** a layer move is committed on a 4000×4000 document with GPU compute
  enabled and a usable adapter present
- **THEN** the document composite is recomputed on the GPU and every channel
  differs from the CPU oracle by at most 1 LSB
