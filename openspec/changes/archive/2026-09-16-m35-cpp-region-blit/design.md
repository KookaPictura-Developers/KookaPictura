## Context

M31 added dirty-rectangle compositing: `refresh_region(rect)`
(`crates/pictura-app/src/cxxqt_object.rs`) composites the clamped rectangle with
`composite_region_active`, patches the authoritative `doc.composite` with
`patch_composite_region` (a plane `copy_from_slice`), and then writes the rectangle
into the full-resolution cached canvas `QImage` with `blit_image_region` — one
`QImage::set_pixel_color` FFI call per pixel.

Measured at 4000² (2 RGB layers, release, RTX 3090):

| region side | `composite_region_active` | `blit_image_region` |
|---:|---:|---:|
| 512² | 1.46 ms | 7.36 ms |
| 1024² | 5.58 ms | 27.3 ms |

The blit is ~5× the composite and scales with area, so at 10000² a large dab costs
~170 ms. The app hides this with `REGION_REFRESH_BUDGET = 1_000_000` px: a dirty
union larger than the budget skips `blit_image_region` and calls `recomposite`
(130 ms at 4000², 878 ms at 10000²). `begin_move_preview` uses the same
`blit_image_region` and the same budget.

Relevant code: the `#[cxx_qt::bridge]` `PictureView` block and `changed` signal
(`cxxqt_object.rs:31`), the backing struct (`:482`), `image()` (`:682`),
`begin_move_preview` (`:1103`), `move_preview_base` (`:1152`), `commit_move`
(`:1176`), `sample_argb` (`:1231`), `paint_dab` (`:1400`), `refresh_region`
(`:1787`), `recomposite` (`:1857`), `REGION_REFRESH_BUDGET` (`:2456`),
`move_preview_region` (`:2479`), `blit_image_region` (`:2503`),
`patch_composite_region` (`:2604`), `buffer_to_image` (`:2783`);
`crates/pictura-app/cpp/frame.cpp` (the `changed` connection at `:280`,
`setImage`/`replaceImage` at `:274`/`:595`); `crates/pictura-app/cpp/image_view.{h,cpp}`
(`setImage`/`replaceImage`/`paintEvent` and the M32 present cache);
`crates/pictura-app/cpp/main.cpp` (the self-test checks that read `view->image()`).

Constraints: the CPU compositor (`composite_rgba`) is the frozen oracle,
`composite_active`/`composite_region_active` are the full/region oracles, the GPU
path is ±1 LSB against the CPU oracle, `write_psd`'s layout is unchanged, no new
dependency, and `docs/` is the long-form contract. **cxx-qt-lib 0.10 exposes no
`QImage` scanline or pixel-buffer access from Rust**, so an efficient overwrite
cannot be written in Rust; it belongs in C++ where `QPainter` is available.

## Goals / Non-Goals

**Goals:**

- Make the region refresh write the rectangle into the canvas in C++ with one
  `QPainter` call instead of one FFI call per pixel.
- Hold the authoritative canvas as the planar `doc.composite` only; make the
  full-resolution `QImage` a lazy, rebuildable cache (`display_dirty`).
- Remove `REGION_REFRESH_BUDGET` and the "too big, recomposite" fallback; a dirty
  region of any size is blitted in C++.
- Keep `image()`, `sample_argb`, and `move_preview_base` correct against the
  authoritative composite.
- Keep the screen, the document composite, and a full recomposite in agreement.
- Ship a self-test that exercises the C++ blit path and keeps the M31/M32/M34
  canvas-equality checks green.

**Non-Goals:**

- Resident GPU layer sources, tiled compositing (256² tiles + LRU + gutters),
  display-time LoD, history copy-on-write / tile diffs, and the GPU-resident
  zero-copy present.
- Any change to the compositor, `composite_active`/`composite_region_active`, the
  ±1 LSB parity contract, or the `write_psd` format.
- Caching the scaled present image ahead of the next paint (the region blit
  invalidates it; a scaled-region patch is a later optimisation).

## Decisions

### 1. `regionBlitted` signal and its cxx-qt declaration (frozen)

```rust
/// Emitted after a region composite. The receiver blits `region` at `(x, y)`.
#[qsignal]
#[cxx_name = "regionBlitted"]
fn region_blitted(self: Pin<&mut Self>, region: QImage, x: i32, y: i32);
```

- Declared in the `extern "RustQt"` block next to `changed`. cxx-qt maps it to the
  C++ signal `void regionBlitted(QImage region, int x, int y)`; `QImage` is a
  cxx-qt-lib shared type (`ExternType`, `kind = Trivial`, implicit sharing), so the
  pass-by-value is a refcount, not a pixel copy. The default cxx-qt naming would
  keep `region_blitted`, hence the explicit `#[cxx_name]`.
- Emitted from Rust as `self.region_blitted(region, x0, y0)`.
- *Why a signal and not a qinvokable return:* a region refresh is asynchronous to
  the frame's own callbacks (paint dabs come from `paint_dab`, a signal fires under
  the Qt event loop) and the view already owns the canvas the region lands in.
- *Alternatives:* pass the region origin as a `QRect` (rejected — the two ints are
  what `QPainter::drawImage` needs); a `PictureView` qinvokable that mutates the
  canvas (rejected — the canvas is an `ImageView`, not the `PictureView`).

### 2. `ImageView::blitRegion` (frozen)

```cpp
// image_view.h
void blitRegion(const QImage& region, int x, int y);
```

- Implemented with `QPainter` in `CompositionMode_Source` (an overwrite) onto
  `image_`, clipped to the document rect, then `presentCache_.valid = false;
  update();`. No transform, no scaling: source and destination are the same
  document resolution, so the region lands at document coordinates.
- The detach that a `QPainter` write triggers is exactly the copy `replaceImage`
  already pays per full frame; after the first region blit `image_` is unshared
  and subsequent blits touch only the region (`plus QPainter` setup).
- The checkerboard, clip, and move-preview branches of `paintEvent` are untouched.
- *Why invalidate rather than patch the scaled present cache:* the present cache is
  keyed on the source `cacheKey()` + zoom; an in-place `QPainter` write does not
  reliably bump `cacheKey()`, so an explicit `valid = false` is the correct edge
  case. The next paint rebuilds through the same painter transform, so the
  presented pixels stay identical to the transform draw (the M32 contract).
- *Alternatives:* patch `presentCache_.scaled` at scaled coordinates (rejected for
  this change — needs the zoom/offset mapping and a filter-phase argument; it is a
  later optimisation); trust `cacheKey()` (rejected — unreliable for in-place
  writes).

### 3. `display_dirty` lifecycle (frozen)

`PictureViewRust` gains `display_dirty: bool` (default `false`). `doc.composite`
is authoritative and is updated only by `store_composite` (`recomposite`,
`open`, `new_document`) and `patch_composite_region` (`refresh_region`);
`rust.image` is a cache of it.

| site | action |
|---|---|
| `refresh_region` (region composite) | patch `doc.composite`; **set `display_dirty = true`**; emit `regionBlitted`; no full-image touch |
| `recomposite` | `rust.image = buffer_to_image(&rendered)`; **clear `display_dirty`** |
| `open` / `new_document` | build `rust.image` from the render; **clear `display_dirty`** |
| `undo` / `redo` | `rust.image = buffer_to_image(&snapshot.doc.composite)`; **clear `display_dirty`** |
| `history_jump` / `history_restore_snapshot` | go through `recomposite`; cleared there |
| dimension ops (`resize_*`, `crop`, `rotate`, `flip`) | `document_ops::recompute` + `recomposite`; cleared there |

Mid-stroke, `refresh_region` composites from the stroke's working document and
does **not** patch the app document's `composite` (M34); it still sets
`display_dirty`. `end_paint`/`cancel_paint` call `recomposite`, which clears it. If
`image()` is called while a stroke is active, it rebuilds from the stroke's working
document (the same source `refresh_region` composites from), so the full image
stays consistent with the live region blits.

### 4. `refresh_region` (frozen)

1. Clamp the rectangle to the source document; an empty rectangle is a no-op (no
   composite, no signal).
2. `composite_region_active(source, rect, gpu_compute)`.
3. Non-painting only: `patch_composite_region(doc, &buffer, x0, y0)` (unchanged
   plane `copy_from_slice`, no FFI).
4. `let region = buffer_to_image(&buffer);` — a rectangle-sized `QImage`.
5. Set `display_dirty = true`; emit `region_blitted(region, x0, y0)`.
6. The budget check and both fallback branches (the `document_to_image` stroke
   branch and the `recomposite` branch) are deleted. `move_preview_region` loses
   its budget comparison and keeps only the `cached_ok` guard.

### 5. `image()`, `sample_argb`, `move_preview_base` (frozen)

- **`image()`** becomes `pub fn image(mut self: Pin<&mut Self>) -> QImage` (the
  bridge declaration `fn image(self: Pin<&mut Self>) -> QImage`). It returns the
  cached `rust.image` when `!display_dirty`; otherwise it rebuilds from
  `doc.composite` (`buffer_to_image`) — or from the stroke's working document while
  painting — clears `display_dirty`, and returns it. Callers in C++ are non-const
  `PictureView*`, so `view->image()` is source-compatible.
- **`sample_argb(x, y)`** reads `doc.composite` directly with a small planar
  sampler (1-plane → grey/opaque; 2-plane → grey + alpha; 3-plane → RGB opaque;
  4-plane → RGBA), mirroring `buffer_to_image`'s rules, and bounds-checks against
  the composite dimensions. It never builds a full image. (`composite_argb` already
  reads `doc.composite` and is unchanged.)
- **`move_preview_base()`** keeps its `&self` signature; `begin_move_preview`
  builds `move_base` from the authoritative `doc.composite` — never the stale
  `rust.image`: clone the planar composite, overwrite the hidden-layer region with
  the same plane `copy_from_slice` used by `patch_composite_region`, and convert
  once with `buffer_to_image`. This removes the per-pixel FFI from the preview path.
  The cost (one planar clone + one full conversion per drag start) replaces the M32
  region blit; `ponytail:` names the C++-callable blit as the upgrade if that start
  latency ever matters.

### 6. `frame.cpp` connection and panel throttling (frozen)

```cpp
connect(view, &PictureView::regionBlitted, this,
        [this, canvas](const QImage& region, int x, int y) {
            if (canvas) canvas->blitRegion(region, x, y);
            // Same debounce as the `changed` path; no image()/replaceImage.
            panelRefreshTimer_->start();
            updateTabTitle(activeDocumentIndex());
            updateWindowTitle();
            if (registry_) registry_->refresh();
        });
```

- The lambda captures the document's own `canvas`, not `imageView()`, so a
  background tab's region blit lands on the right view.
- It performs the cheap, non-image parts of `refresh()` (modified marker, window
  title, command enablement, the debounced panel timer) but **never** calls
  `view->image()` or `canvas->replaceImage()` and never calls `refreshPanels()`
  synchronously. The debounced panel refresh fires at most once per region blit,
  exactly as today's `changed`-per-dab did, and because region blits no longer
  emit `changed`, the region path cannot refresh panels faster than today.
- `refresh()` itself is unchanged and still drives the full `changed` path.

### 7. Oracles, parity, and correctness checks (frozen)

`composite_rgba`, `composite_active`, `composite_region_active`,
`translate_layer*`, and the ±1 LSB contract are unchanged. One new C++ self-test
step exercises the blit path: after a region refresh, `canvas->image()` is
compared pixel-for-pixel with `view->image()` after a forced full recomposite, and
the present cache is observed to be invalidated and rebuilt on the next paint. The
existing M31/M32/M34 checks that read `view->image()` still hold because
`doc.composite` is byte-identical to a full composite after a region patch (M31)
and `image()` rebuilds from it.

### 8. Process (frozen)

Waves: (1) brief and frozen interfaces; (2) Rust composite owner + `display_dirty` +
`image()`/`sample_argb`; (3) Rust `regionBlitted` + `refresh_region` + budget
removal; (4) C++ `ImageView::blitRegion` + `frame` connection; (5) tests + self-test
+ evidence; (6) close-out.

## Risks / Trade-offs

- **`QPainter`-Source blit vs `from_raw_bytes` pixel equality.** The C++ blit and
  the Rust `buffer_to_image` must produce identical pixels for a region refresh.
  Both write `Format_RGBA8888` at integer offsets with overwrite semantics; the new
  self-test compares `canvas->image()` to `view->image()` pixel-for-pixel and the
  M31/M32 tests compare Rust-side images. If a byte differs, the blit is wrong, not
  the oracle.
- **Present-cache invalidation.** Relying on `image_.cacheKey()` alone is
  unreliable for in-place `QPainter` writes; `blitRegion` must set
  `presentCache_.valid = false` explicitly, or a stale scaled image is presented. A
  test asserts `presentCacheRebuiltOnLastPaint()` after a region blit + repaint.
- **Self-tests that read `view->image()`.** `image()` now returns a freshly built
  `QImage` when the display is dirty rather than the previously patched cache. The
  M31 (`m31_region`, `m31_region_large`), M32 (`m32_region`), M34 (`m34_coherent`,
  undo/redo) and present-cache checks must be re-run; they are expected to hold. The
  `m31_region_large` check changes meaning: with the budget gone, a 1024² move on a
  1024² canvas now takes the region-blit path instead of the full-recomposite
  fallback, so that check is re-pointed at the C++ blit equality.
- **Move-preview base staleness.** `begin_move_preview` must build from
  `doc.composite` (authoritative), not `rust.image` (stale after a region blit);
  otherwise the drag base shows the pre-blit pixels. The cost of a full planar
  clone + conversion per drag start is the trade-off; the C++-callable blit is the
  named upgrade.
- **`regionBlitted` while a pan/zoom present cache is valid.** A paint dab or
  visibility toggle can arrive at any zoom. `blitRegion` invalidates the present
  cache, the next paint rebuilds at the current zoom, and the presented pixels
  match the transform draw — no correctness issue. The per-dab present rescale is
  the same cost today's `replaceImage` path already pays; patching the scaled cache
  is deferred.
- **Mid-stroke `sample_argb`.** The eyedropper now reads `doc.composite`, the
  pre-stroke base during an active stroke, rather than the live patched image. The
  eyedropper cannot run while painting, so this is a documented, unreachable
  difference.
- **Documents with no layers / embedded composite.** `current_buffer` returns
  `doc.composite`; `buffer_to_image` and `sample_argb` handle 1/2/3/4 planes;
  `patch_composite_region` writes `min(4, composite.channels)` planes, so a
  1-plane grayscale canvas stays 1-plane (M34 behaviour).
- **`image()` becomes `&mut`.** Any Rust caller holding only `&self` breaks; there
  are none (only field access internally, and non-const C++ pointers call
  `view->image()`).
- **Signal type across the bridge.** A `QImage` signal parameter is new for this
  bridge; the cxx-qt codegen and the direct connection must compile. If the
  by-value form does not, fall back to `&QImage` (the C++ signal becomes
  `const QImage&`).

## Migration Plan

Additive and internal. `regionBlitted`, `display_dirty`, and `blitRegion` are new;
`refresh_region`, `image()`, `sample_argb`, `begin_move_preview`, and `frame`'s
connections change in place; `blit_image_region` and `REGION_REFRESH_BUDGET` are
deleted. The compositor, the oracles, the ±1 LSB contract, and `write_psd` are
unchanged. Rollback: restore the per-pixel `blit_image_region`, the budget and its
fallback, make `image()` return `rust.image` unconditionally, and drop the
`regionBlitted` connection; the M31/M32 behaviour returns.

## Open Questions

- Whether the C++ blit should also patch `presentCache_.scaled` (needs the
  zoom/offset mapping and a filter-phase argument) — deferred; invalidation is
  correct and matches today's cost.
- Whether `begin_move_preview` should instead call a bridge-callable C++
  `blitRegionInto` to avoid the full planar clone + conversion — deferred; the
  Rust-planar build is smaller and correct, and the drag-start latency is not the
  measured hot path.
- Whether the `regionBlitted` handler should restart `panelRefreshTimer_` (as
  written) or let panels wait for the next full `changed` — restarting is the
  closer match to today's behaviour.
