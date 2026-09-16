# M35 — region blit in C++ (canvas image owned by the view)

- **Status:** proposed OpenSpec change (`openspec/changes/m35-cpp-region-blit`).
- **Capabilities:** MODIFIED `document-canvas`. No new capability.
- **Depends on:** M31 (region compositing), M32 (interactive canvas / present cache),
  M34 (composite coherence).
- **Non-goals:** resident GPU layer sources, tiled compositing, display-time LoD,
  history copy-on-write / tile diffs, the GPU-resident zero-copy present, and any
  change to the compositor or the ±1 LSB parity contract.

## Why

M31 made a small dirty rectangle cheap to composite, but the app then writes that
rectangle into the full-resolution canvas `QImage` with one
`QImage::set_pixel_color` FFI call per pixel (`blit_image_region`,
`crates/pictura-app/src/cxxqt_object.rs`). Measured at 4000² (2 RGB layers,
release, RTX 3090):

| region side | `composite_region_active` | `blit_image_region` | ratio |
|---:|---:|---:|---:|
| 512² | 1.46 ms | 7.36 ms | ~5× |
| 1024² | 5.58 ms | 27.3 ms | ~5× |

The blit scales with area: at 10000² a large dab is ~170 ms. The composite is no
longer the cost — the per-pixel FFI blit is. `REGION_REFRESH_BUDGET = 1_000_000`
px exists only to hide it: a dirty region larger than the budget drops the region
path and recomposites the whole document (130 ms at 4000², 878 ms at 10000²).
`begin_move_preview` uses the same blit and the same budget.

`cxx-qt-lib` 0.10 exposes no `QImage` scanline or pixel-buffer access from Rust, so
an efficient overwrite cannot be written in Rust. The canvas `QImage` is owned by
the C++ `ImageView`, which has `QPainter`, so the region belongs there.

## Design in one paragraph

The planar `doc.composite` (already kept current by M34) becomes the authoritative
canvas. `refresh_region(rect)` composites the clamped rectangle
(`composite_region_active`), writes it into `doc.composite` with a plane
`copy_from_slice` (no FFI), converts only the rectangle to a `QImage`, marks the
display dirty (`display_dirty`), and emits a new `regionBlitted(QImage, x, y)`
signal instead of `changed`. `frame.cpp` connects `regionBlitted` to
`ImageView::blitRegion(region, x, y)`, which overwrites the canvas with `QPainter`
under `CompositionMode_Source`, keeps the checkerboard/clip/pan/zoom behaviour,
invalidates the zoom present cache, and repaints. `recomposite()` still builds the
full `QImage`, clears `display_dirty`, and emits `changed`. `image()` returns the
cached image when clean and rebuilds it from `doc.composite` when dirty;
`sample_argb` reads `doc.composite` directly; `begin_move_preview` builds its base
from `doc.composite`, not a stale cache. The budget and `blit_image_region` are
deleted, so a dirty region of any size is blitted in C++.

## Frozen interfaces

```rust
// cxxqt_object.rs — next to `changed`
#[qsignal]
#[cxx_name = "regionBlitted"]
fn region_blitted(self: Pin<&mut Self>, region: QImage, x: i32, y: i32);
```

```cpp
// image_view.h
void blitRegion(const QImage& region, int x, int y); // QPainter, CompositionMode_Source
```

- `refresh_region(rect)` instead of `changed` emits
  `regionBlitted(buffer_to_image(region), x0, y0)`; empty rect = no-op.
- `display_dirty` is set by `refresh_region` and cleared by `recomposite`,
  `open`, `new_document`, `undo`, `redo`.
- `REGION_REFRESH_BUDGET` and the over-budget fallback are removed;
  `move_preview_region` keeps only the `cached_ok` guard.
- Region blits do not emit `changed`; the frame handler blits, restarts the 120 ms
  panel timer, and does the cheap tab/title/registry sync — it never calls
  `image()`/`replaceImage`, so panels cannot refresh faster than today.

## Evidence / acceptance

- A region-refreshed canvas equals a full recomposite (Rust `image()` rebuilt from
  `doc.composite`, and the C++ `canvas->image()`).
- A 1024² dab on a 4000² document takes the region-blit path (no size fallback).
- The C++ blit path is exercised by the self-test; the present cache is invalidated
  and rebuilt on the next paint.
- The M31/M32/M34 canvas-equality checks still hold.
- No per-pixel FFI blit remains.

## Honest limits

- The C++ blit invalidates the present zoom cache rather than patching it, so a
  region refresh still costs one full-document present rescale on the next paint —
  the same cost today's `replaceImage` path already pays. Patching the scaled cache
  is a later optimisation.
- `begin_move_preview` rebuilds its base from a planar clone of `doc.composite`
  with one full conversion per drag start (a small Rust cost, no FFI); a
  C++-callable blit is the upgrade if that start latency matters.
- Mid-stroke, `sample_argb` reads the pre-stroke document composite; the
  eyedropper cannot run while painting, so this is unreachable in practice.
- A non-constant present cache across a backend change can differ by ≤1 LSB, which
  the existing GPU parity contract allows.

## Verification

- `cargo test --workspace`; `cargo fmt`/`clippy` clean.
- `cmake --build build`; `xvfb-run -a ./build/pictura --self-test` on the GPU
  default and the CPU fallback.
- `openspec validate m35-cpp-region-blit --strict`;
  `openspec validate --all --strict`.
