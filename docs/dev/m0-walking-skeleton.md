# M0 — Walking Skeleton

Goal: prove the whole stack end-to-end **before** any feature spend:
Rust ⇄ cxx-qt ⇄ Qt 6 ⇄ GPU ⇄ PSD codec ⇄ test harness.

No CS6 behavior is implemented in M0. The only document available is a
single-composite 8-bit PSD.

## Task DAG

| ID | Task | Owner | Deliverable | Acceptance |
|---|---|---|---|---|
| M0-A | cxx-qt ↔ Qt 6.11 bridge + window | agent | `crates/pictura-app`, `CMakeLists.txt` | App builds via CMake and opens a window headless (Xvfb) |
| M0-B | Minimal PSD read/write | agent | `crates/pictura-codec` | Round-trip: `read(write(doc)) == doc`; malformed input returns `PsdError`, never panics |
| M0-C | Verification harness | agent | `crates/pictura-testkit`, CI, guard | `compare`/`hash` unit tests; CI runs fmt/clippy/test; guard fails on non-goal paths |
| M0-D | Assemble + integrate | orchestrator | passing workspace | `cargo test --workspace` green; app loads a codec-produced PSD |
| M0-E | Determinism + perf smoke | orchestrator | report | two runs hash-equal; 4k open+render+save within a generous budget |

## Exit gate (human review)

1. The stack builds and runs in CI/headless.
2. A deliberately introduced bug turns a harness check red (prove the harness
   works), and reverting it goes green.
3. Each of the four risk areas is **proven or named as blocked**:
   - cxx-qt ↔ Qt 6.11 compatibility
   - GPU path (wgpu/QRhi) — may be deferred to M0.5 if it blocks
   - PSD read/write fidelity
   - golden/determinism harness

If the cxx-qt ↔ Qt 6.11 bridge fails, that is the deliverable: document the
failure and choose the fallback (Qt 6.8 LTS via aqtinstall, or `qmetaobject-rs`).

## Out of scope for M0

Layers, masks, channels, filters, color management, real UI panels, GPU
compositing, PSD write beyond the composite, any format other than PSD/PSB.
