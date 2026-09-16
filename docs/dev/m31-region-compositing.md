# M31 — Region compositing

Goal: stop compositing and transferring the whole document for a small change.
Today every canvas update goes through `document_to_image` → `current_buffer` →
`composite_active` → `buffer_to_image`: the whole layer stack is composited, the
whole packed RGBA buffer is read back, and the whole document is converted to a
`QImage`. At 4000² that is a 16.7 M-pixel composite and a ~64 MB readback for a
one-pixel paint dab. M29 made the composite itself fast (2-D GPU dispatch,
active-backend commit); M31 removes the full composite and the full readback from
the interactive path by compositing and reading back only the changed rectangle.

Compositing is per-pixel: each pixel's blend reads only its own source sample and
the same pixel's running canvas, so disjoint regions do not affect each other. A
sub-rectangle composite is therefore **byte-identical** to the corresponding
slice of the full composite — the region path is checked directly against the
existing `composite_active` oracle rather than approved by eye.

OpenSpec change `m31-region-compositing` (MODIFIED `gpu-compositing`,
`document-canvas`).

## Scope

- **Region render API.** `composite_region_active(doc, rect, gpu_enabled)`
  composites only the clamped document rectangle through the active backend and
  returns a rectangle-sized buffer byte-identical to the corresponding slice of
  `composite_active`, for separable, non-separable, adjustment, group and masked
  scenes. The full-document `composite_active` stays and remains the oracle.
- **GPU region dispatch.** The compute dispatch covers only the region's pixels;
  the region origin/size and row stride are threaded through the shader, the
  per-layer source and mask buffers are sized to the region, and only the region
  is read back.
- **CPU region loop.** The same compositing loop restricted to the region
  (document coordinates preserved), so byte-identity is direct.
- **Dirty-region canvas cache.** `PictureView` keeps the full-document canvas
  `QImage` it already holds as the cache. `refresh_region(rect)` composites the
  rect through the active backend, blits it into the cached canvas at the rect
  origin, and emits `changed`. The Move commit invalidates
  `old_layer_rect ∪ new_layer_rect`; a paint dab invalidates the dab's bounding
  box. Other mutations keep the full recomposite for now.
- **Evidence.** A self-test asserts a region-refreshed canvas equals a full
  recomposite after Move and paint; a timing check shows a small dirty rect on a
  large document is materially cheaper than the full composite/readback.

## Out of scope (later milestones)

- Display-resolution proxies / LoD (M32/M33).
- GPU-resident present without readback (M32): the canvas is still the CPU
  `QImage` path, so a completed region composite is still converted and blitted
  into a `QImage`.
- 256² GPU tiles with LRU, seam gutters and mipmaps (M33).
- Off-GUI-thread compute.
- History copy-on-write / tile diffs: `Document::clone()` stays on the Move and
  stroke commit paths.
- Region refresh for mutations that do not report a dirty rectangle; they keep
  the full recomposite.

## Process

Waves: (1) brief and frozen interfaces; (2) region render API, GPU region
dispatch, byte-identity parity; (3) app cached canvas and dirty-rect Move/paint;
(4) self-test and timing evidence; (5) close-out.

## Verification

- `cargo test -p pictura-render` (full-rect region equals `composite_active`; a
  sub-rect equals its slice for separable, non-separable, adjustment, group and
  masked scenes; out-of-bounds/empty clamp without panicking) and
  `cargo test --workspace`
- A small dirty rect on a 4000² document composites and transfers only the rect,
  materially cheaper than the full composite/readback (skip with a printed note
  when no adapter exists)
- `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` passes on the
  default GPU or the CPU fallback
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
