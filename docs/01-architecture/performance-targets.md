# Performance Targets

- **Spec ID:** `ARCH-013`
- **Status:** `Draft`
- **Parity tier:** `Core`
- **New in CS6:** `Changed` — CS6 adds the Mercury Graphics Engine (GPU-accelerated pan/zoom/rotate and filters) and documents a ≥10% 64-bit speedup over CS5.
- **Depends on:** `ARCH-001`, `ARCH-002`.

This document sets the latency, throughput, memory, cache, and startup budgets
Kooka Pictura must meet to be usable as a CS6 replacement. Numbers are labeled
**Sourced** (taken from an Adobe/benchmark source), **Derived** (computed from a
sourced number), or **Target** (a to-be-validated engineering goal). A **Target**
is a hypothesis to test, not a fact about CS6.

## CS6 behavior

Adobe documents the following performance-relevant behavior:

- 64-bit builds complete "day-to-day imaging tasks at least 10% faster" than
  32-bit. 32-bit builds are limited to ~3 GB of addressable RAM; 64-bit builds
  can address as much as the machine has.
- The Mercury Graphics Engine accelerates specific interactive operations:
  Liquify, Warp/Puppet Warp, Lighting Effects, Oil Paint, Adaptive Wide Angle,
  the Blur Gallery, and 3D. Pan/zoom/rotate and flick-panning are GPU-assisted in
  CS6 via OpenGL; the Blur Gallery requires OpenCL 1.1.
- The status bar **Efficiency** metric is the percentage of time spent computing
  rather than paging to scratch. Adobe treats sustained values below ~95% as a
  signal that more RAM or faster scratch is needed.
- Photoshop splits images into 128 KB tiles and maintains a multi-level cache
  (default 4 levels) for screen redraws.
- Puget Systems measured peak RAM for a single document at default settings:
  109 MB image → ~4.8 GB peak; 250 MB → ~7.8 GB; 500 MB → ~12.3 GB; 750 MB →
  ~17.6 GB; 1024 MB → ~22.5 GB, with a default 60% RAM allocation.
- Puget's GPU benchmark applied default-option effects to a 38 MP (12 MB
  compressed) image and reported total suite times per GPU; the 13.0.1 update
  improved GPU acceleration "across the board." The article does not publish a
  per-filter table in text.

What CS6 does **not** document: frame-rate targets, brush input-to-pixel latency,
cold-start time, or per-filter apply times. Those are marked **Target** below.

## UI surface

Performance is surfaced through diagnostics rather than a dedicated panel.

| Location | Type | Shortcut | Notes |
|---|---|---|---|
| Status bar | Readout | — | SCRATCH SIZES (left = bytes used, right = RAM available) and EFFICIENCY (%). |
| `Edit > Preferences > Performance` | Preferences page | — | RAM %, scratch, history states, cache level, cache tile size. |
| `Help > System Info` | Dialog | — | GPU/VRAM detection. |
| Progress bar / busy cursor | Status | — | Long filters must show progress and remain cancellable. |

## Parameters & ranges

Budgets are expressed per workload class. All values must be validated on a
reference machine (8-core CPU, mid-range discrete GPU, 16 GB RAM, NVMe scratch)
unless stated otherwise.

| Workload | Metric | Budget | Basis |
|---|---|---|---|
| Pan (drag) | Frame rate | 60 FPS sustained on a 24 MP doc | Target |
| Zoom (scrubby/continuous) | Frame rate | 60 FPS sustained on a 24 MP doc | Target |
| Rotate view | Frame rate | 60 FPS on a 24 MP doc | Target |
| Brush stroke | Input-to-first-pixel | ≤ 16 ms (one 60 Hz frame) | Target |
| Brush stroke | Sustained updates | 120 Hz sampling tolerated, ≥60 FPS redraw | Target |
| Single-tile filter, 1000×1000 px, 8-bit | Apply time | ≤ 50 ms | Target |
| Gaussian Blur, radius 50 px, 24 MP | Apply time | vs. CS6 within 1.5× | Target |
| Liquify, GPU available | Interactive | ≥ 30 FPS warp preview | Target |
| Open document, 100 MB PSD | Time | ≤ 2 s | Target |
| Cold start (no document) | Time to interactive | ≤ 3 s | Target |
| Cold start memory | RSS | ≤ 250 MB | Target (community figure: CS6 ~115 MB at idle, unverified) |
| 1024 MB document peak RAM | RSS | ≤ CS6 peak + 10% (~24.7 GB on a 32 GB machine) | Derived (Puget 22.48 GB peak) |
| Paging efficiency | Efficiency % | ≥ 95% during normal edits | Sourced (Adobe threshold) |
| Filter cancellation | Latency after cancel | ≤ 100 ms | Target |

Frame budgets assume a full-HD viewport. The 60 FPS target equals a 16.6 ms frame;
of that, no more than 10 ms may be spent in the Rust core and GPU submit path.

## Algorithms & pipeline

### Budget decomposition (pan/zoom)

A 16.6 ms frame is split as: input handling (≤1 ms), core tile gather (≤3 ms),
GPU composite + present (≤8 ms), Qt scene-graph overhead (≤2 ms), slack (≤2 ms).
The TileCache must serve the visible tile set without touching scratch on the
common path; only tiles outside the cache window may require spill-in.

### Budget decomposition (filter apply)

- Separable convolutions (Gaussian and most blur kernels) run two 1-D passes.
  Cost ≈ O(2 · N · r) where N = tile pixels and r = radius; the design must tile
  the operation so the working set fits L2/L3 rather than streaming whole rows.
- CPU kernels parallelize with `rayon` across tiles; the GUI thread must never be
  part of the filter work.
- GPU paths use `wgpu` compute or fragment passes and must read back only when the
  result must be visible or persisted.
- A filter must publish progress and check a cancellation token at tile
  boundaries, not only at completion.

### Tile cache sizing

Cache capacity is the primary lever for pan/zoom latency and for staying above
the 95% Efficiency threshold.

| Machine RAM | Proposed cache budget | Approx. 128 KB tiles |
|---|---|---|
| 8 GB | 25% of RAM ≈ 2 GB | ~16,000 |
| 16 GB | 35% of RAM ≈ 5.6 GB | ~45,000 |
| 32 GB | 40% of RAM ≈ 12.8 GB | ~100,000 |

These are **Targets**, not CS6-derived. The CS6 signal to mirror is the Efficiency
metric: if paging drives Efficiency below 95%, the cache is undersized or the RAM
allocation is too high.

### Memory ceiling

CS6 on 64-bit has no fixed ceiling beyond available RAM; the practical ceiling is
the point where scratch paging dominates. Kooka Pictura should enforce a soft
ceiling derived from the configurable RAM allocation (default 60% of system RAM,
matching CS6) and report it. A hard ceiling is optional but must never be silent:
crossing it raises a typed error, not an OOM kill.

## Rust module mapping

| Module (proposed) | Budget responsibility |
|---|---|
| `pictura-core::cache` | TileCache hit rate and spill-over decisions. |
| `pictura-core::scratch` | Scratch I/O; efficiency accounting. |
| `pictura-render::frame` | Frame budget instrumentation (CPU and GPU timestamps). |
| `pictura-render::scheduler` | Tile job batching and cancellation. |
| `pictura-filters::cancel` | Cancellation tokens checked per tile. |
| `pictura-app::metrics` | Optional opt-in timing log for validation. |

Instrumentation is required to validate these targets: `QElapsedTimer` on the Qt
side and `QRhiCommandBuffer::lastCompletedGpuTime` (with `EnableTimestamps`) on
the GPU side are documented mechanisms; the Rust side uses its own monotonic
timers.

## Qt6 component mapping

| Component | Budget responsibility |
|---|---|
| `CanvasView` | Present within the frame budget; no per-frame heap allocation. |
| Progress indicator | Report and cancel long filters. |
| Status-bar readout | Surface scratch/efficiency-equivalent metrics. |
| `QThreadPool` | Not used for image work; must not contend with `rayon`. |

## Data-model impact

- Every long-running operation carries a `Progress` handle and a `CancelToken`
  in its command record.
- Timing data, if persisted, is opt-in and stored outside document files.
- The cache/scratch split is runtime state, not serialized document state.

## Edge cases

- **GPU slower than CPU** for a given op (small images): the scheduler must pick
  the cheaper path; do not blindly offload.
- **GPU unavailable or device lost:** CPU path must still meet a relaxed budget
  (proposed: ≤2× the GPU target) and never hang.
- **Very large radius blurs:** fall back to a multi-scale / IIR approximation
  rather than a linear-time kernel that misses the budget.
- **Small documents:** per-frame overhead dominates; the floor should be well
  under the frame budget so 1-px documents are instant.
- **Scratch on slow media:** Efficiency floor of 95% may be unattainable; report
  the measured value rather than failing.
- **High-DPI / 4K viewport:** frame budget must scale with pixel count; the FPS
  target applies at full HD, with degraded-resolution previews allowed above it.
- **Many open documents:** the sum of per-document caches must respect the global
  budget; one huge document must not starve the others.

## Parity acceptance criteria

1. Given a 24 MP 8-bit RGB document on the reference machine, pan, zoom, and
   rotate each sustain ≥60 FPS over a 5-second gesture (measured p95 frame time
   ≤ 20 ms).
2. Given a tablet-class input stream at 120 Hz, the first brush pixel appears
   within one 60 Hz frame of the first input event.
3. Given Gaussian Blur radius 50 on a 24 MP image, apply time is within 1.5× of
   CS6 13.0.1 on identical hardware, or an explicit waiver is recorded.
4. Given a 1024 MB document, peak RSS does not exceed the CS6-measured peak by
   more than 10%.
5. Given a document exceeding the RAM allocation, the app keeps Efficiency ≥95%
   on normal edits or reports the shortfall; it never aborts without a typed error.
6. Given a long filter, cancelling returns control within 100 ms and leaves the
   document unchanged.
7. Given a cold launch with no document, time-to-interactive is ≤3 s and idle RSS
   is ≤250 MB.

## Sources

- `https://web.archive.org/web/20140204041700/http://blogs.adobe.com/crawlspace/2012/10/how-to-tune-photoshop-cs6-for-peak-performance.html` — 64-bit ≥10% claim, ~3 GB 32-bit ceiling, Efficiency threshold ~95%, History States default 20, Cache Levels default 4, 128 KB tiles, 256/512 MB VRAM, PSD 2 GB / PSB 4 EB.
- `https://www.pugetsystems.com/labs/articles/Adobe-Photoshop-CS6-Memory-Optimization-182` — peak RAM vs. image size table; default 60% RAM allocation.
- `https://www.pugetsystems.com/labs/articles/Adobe-Photoshop-CS6-GPU-Acceleration-161` — GPU benchmark methodology (default effects on a 38 MP image), 13.0.1 speedup, no per-filter numbers published.
- `https://www.barefeats.com/pscs6.html` — Liquify/Iris Blur GPU vs. CPU comparisons; blur gallery requires OpenCL 1.1.
- `https://help.adobe.com/archive/en/photoshop/cs6/photoshop_reference.pdf` — Efficiency/Scratch Sizes readouts, 64-bit performance note.
- `https://web.archive.org/web/20140401152634/http://helpx.adobe.com/photoshop/kb/photoshop-cs6-gpu-faq.html` — Cache Levels default 4, GPU mode implications.
- `https://doc.qt.io/qt-6/qrhi.html` — `lastCompletedGpuTime` / `EnableTimestamps`, frame capture tooling.
- `https://docs.rs/rayon` — work-stealing pool used for CPU filter parallelism.
- `https://docs.rs/wgpu` — GPU compute/render path used by the compositor.

## Open questions

- **CS6 absolute filter times** are not published in machine-readable form. *Resolves with:* running CS6 13.0.1 and Kooka Pictura on identical hardware with the same action set and publishing the paired numbers.
- **CS6 cold-start time and idle RSS** are unverified (a community figure of ~115 MB exists). *Resolves with:* a controlled measurement on reference hardware.
- **CS6 pan/zoom FPS** is not documented. *Resolves with:* screen-capture frame analysis of CS6 during gestures.
- **Tile pixel geometry** affects every budget; see `ARCH-002`. *Resolves with:* the tile-size benchmark.
- **Whether 60 FPS is the right target** for a Linux-native app (vs. display refresh rate, e.g. 120/144 Hz). *Resolves with:* a product decision on frame pacing.
- **GPU timestamp availability** varies by driver; the validation plan must degrade gracefully. *Resolves with:* a device/feature matrix test.
