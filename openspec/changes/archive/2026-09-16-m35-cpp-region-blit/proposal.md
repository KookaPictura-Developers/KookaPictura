## Why

M31 made a small dirty rectangle cheap to composite, but the app then writes that
rectangle into its full-resolution canvas `QImage` with one
`QImage::set_pixel_color` FFI call per pixel (`blit_image_region`,
`crates/pictura-app/src/cxxqt_object.rs`). Measured at 4000² (release, RTX 3090):

| region side | region composite | `blit_image_region` | ratio |
|---:|---:|---:|---:|
| 512² | 1.46 ms | 7.36 ms | ~5× |
| 1024² | 5.58 ms | 27.3 ms | ~5× |

At 10000² a single large dab's blit scales to ~170 ms. The composite is no longer
the cost; the per-pixel FFI blit is. `REGION_REFRESH_BUDGET = 1_000_000` px exists
only to hide it: a dirty region larger than the budget abandons the region path and
recomposites the whole document (130 ms at 4000², 878 ms at 10000²). The budget is
a workaround for the blit, not a real limit.

The canvas `QImage` is already owned by the C++ `ImageView` (which has `QPainter`),
so the region can be painted by Qt instead of by a Rust pixel loop. Compositing is
per-pixel and M34 keeps `doc.composite` byte-identical to a full composite after a
region patch, so the Rust side can keep the planar buffer as the authoritative
canvas and hand the small region to C++ for the blit.

## What Changes

- **The planar composite is the authoritative canvas.** `PictureView` keeps
  `doc.composite` (already kept current by M34) as the canvas and stops
  maintaining a full-resolution `QImage` as a per-pixel region-patch target. A
  `display_dirty` marker replaces the in-place patch; `image()` rebuilds from
  `doc.composite` when the display is dirty.
- **`refresh_region` signals the region.** It composites the clamped rectangle,
  writes it into `doc.composite` with a plane `copy_from_slice` (no FFI), converts
  only the rectangle-sized buffer to a `QImage`, marks the display dirty, and emits
  a new `regionBlitted(QImage, i32, i32)` signal. It does NOT emit `changed` and
  does not touch a full-frame image.
- **C++ blits the region.** `ImageView::blitRegion(const QImage& region, int x,
  int y)` overwrites the canvas with `QPainter` under
  `CompositionMode_Source`, keeping the checkerboard, document clip, and pan/zoom
  behaviour unchanged, and invalidates the zoom present cache so the next paint
  rebuilds it. `frame.cpp` connects `regionBlitted` to it alongside the existing
  `changed` → `replaceImage`.
- **The full recomposite still replaces the image.** `recomposite()` builds the
  full `QImage`, clears `display_dirty`, and emits `changed` as today.
- **`image()` / `sample_argb` / `move_preview_base` stay correct.** `image()`
  returns the cached image when clean and rebuilds from `doc.composite` when
  dirty; `sample_argb` reads `doc.composite` directly; `move_preview_base` is
  derived from the current composite, never a stale cached image.
- **The budget is removed.** `REGION_REFRESH_BUDGET` and the "too big for the
  per-pixel blit" full-recomposite fallback are deleted; a dirty region of any
  size is blitted in C++.
- **Panel refresh stays throttled.** The `regionBlitted` path restarts the same
  120 ms single-shot panel timer `changed` uses and never calls `image()` /
  `replaceImage`; it cannot refresh panels more often than today.

## Capabilities

### New Capabilities

None. M35 extends an existing capability.

### Modified Capabilities

- `document-canvas`: the canvas SHALL treat the document's planar composite as
  authoritative and refresh a dirty rectangle by signalling the region to the view
  (`regionBlitted`) rather than patching a full-resolution image per pixel; there
  SHALL be no area budget and a dirty region of any size SHALL take the region-blit
  path; a C++ `ImageView::blitRegion` SHALL overwrite the region under
  `CompositionMode_Source` and invalidate the present cache; the rebuilt
  `image()`/`sample_argb`/`move_preview_base` SHALL remain byte-identical to a full
  recomposite.

## Impact

- `crates/pictura-app/src/cxxqt_object.rs` — the `regionBlitted` qsignal;
  `refresh_region` (planar patch + region `QImage` + signal, budget and fallback
  removed); the `display_dirty` field and lifecycle; `image()` rebuilt-if-dirty;
  `sample_argb` reading `doc.composite`; `begin_move_preview`/`move_preview_base`
  from the current composite; `blit_image_region` and `REGION_REFRESH_BUDGET`
  deleted; `move_preview_region` loses the budget check.
- `crates/pictura-app/cpp/image_view.{h,cpp}` — `ImageView::blitRegion` and the
  present-cache invalidation.
- `crates/pictura-app/cpp/frame.cpp` — the `regionBlitted` connection and the
  light, panel-free UI sync for the region path.
- `crates/pictura-app/cpp/main.cpp` — the self-test walks the C++ blit path and
  re-points the M31 large-union check at it; the M31/M32/M34 equality checks read
  `view->image()`, which now rebuilds-if-dirty.
- No new dependency. The CPU compositor, `composite_active`/`composite_region_active`,
  and the ±1 LSB parity contract are unchanged. Resident GPU layer sources, tiled
  compositing, display-time LoD, history copy-on-write / tile diffs, and the
  GPU-resident zero-copy present are deferred.
