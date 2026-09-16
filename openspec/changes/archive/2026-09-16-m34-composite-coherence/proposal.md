## Why

Two defects share one root: the canvas `QImage` and the document's `composite`
buffer drift apart, and the app treats the canvas as the only source of truth.

- **Save writes a stale merged composite.** The displayed image comes from
  `recomposite` → `document_to_image` → `current_buffer` → `composite_active`,
  but `recomposite` only replaces `rust.image` and never writes the result back
  to `doc.composite`. `pictura_codec::write_psd` serializes `doc.composite`. So
  after any filter, adjustment, resize, undo, or blend change, Save emits a PSD
  whose merged/composite image is the **pre-edit** one. M31's
  `patch_composite_region` is the only place that tries to keep the document
  composite current, and it early-returns unless `doc.composite.channels == 4`,
  so it silently no-ops for the 3-plane RGB composite a freshly opened or new
  document carries.
- **Undo/redo always recomposites the whole document.** `undo`/`redo` replace
  the document with the historical snapshot and then call `recomposite` — a full
  composite (~123 ms at 4000², M33-measured) plus a `buffer_to_image`
  (~40 ms) — even though the snapshot already carries the full document,
  including its composite.

## What Changes

- **Composite coherence on the canvas rebuild.** A single helper,
  `store_composite(doc, &PixelBuffer)`, persists a rendered composite into
  `doc.composite`. An RGB document takes the rendered **4-plane RGBA** frame
  directly (`doc.composite = rendered`), so its merged composite is RGBA after
  any rebuild. A non-RGB mode (Grayscale/Bitmap/Duotone/CMYK/Lab) keeps its
  existing composite colour-plane count and copies `min(rendered, composite)`
  planes, so a 1-plane grayscale composite stays 1-plane and `write_psd` keeps
  its channel layout. A dimension change (resize, crop, rotate, canvas size)
  replaces the composite with the rendered result. `recomposite` renders once
  into a buffer, stores it, and builds the `QImage` from that same buffer, so
  `doc.composite` and `rust.image` are always the same pixels.
- **Channel-consistent region patch.** `patch_composite_region` writes the
  composite's own plane count from the rendered 4-plane region instead of
  early-returning unless the composite is 4-plane, so a `refresh_region` keeps
  `doc.composite` current for an RGB (4-plane) or grayscale (1-plane) document;
  pixels outside the region keep their previous values.
- **Save writes the current composite.** `save` serializes the current
  rendered composite; the `write_psd` byte layout and channel-count rules are
  unchanged. `Open` and `New` render and store the initial composite before
  capturing the first history state.
- **Undo/redo restore without a full composite.** `undo` and `redo`
  unconditionally restore the displayed image from the snapshot's
  `doc.composite` instead of recompositing, byte-identical to a full recomposite
  of the restored document (within the existing ±1 LSB GPU parity across a
  backend change). There is no channel-count or layer-count fallback.
- **Capture ordering.** In every mutating handler, `record(...)` moves to
  **after** its `recomposite`/`refresh_region`, so the captured snapshot carries
  the rendered composite. History order, labels, depth, and redo truncation are
  otherwise unchanged.

## Capabilities

### New Capabilities

None. M34 extends existing capabilities.

### Modified Capabilities

- `edit-history`: `undo`/`redo` unconditionally restore the displayed image
  from the captured document's composite instead of running a full composite
  (byte-identical to a full recomposite of the restored document, no fallback);
  the canvas rebuild persists the rendered result into the document's composite
  — RGBA for RGB, plane-count-preserving for a non-RGB mode; and the snapshot
  captured for a mutation carries that current composite.
- `document-lifecycle`: Save serializes the active document's current rendered
  composite, so a save after any edit round-trips to the edited pixels; the
  `write_psd` byte layout and channel-count rules are unchanged, and an RGB
  document's merged composite is RGBA after a canvas rebuild while a grayscale
  document keeps its plane count.

## Impact

- `crates/pictura-app/src/cxxqt_object.rs` — `store_composite` (new),
  `patch_composite_region` (channel-consistent), `recomposite` (render → store
  → `QImage`), `refresh_region` (store on the region path), `save` (covered by
  the coherent composite; no format change), `undo`/`redo` (restore from the
  snapshot composite, no fallback), `open`/`new_document` (render-and-store
  before capture), and the 18 `record(...)` call sites that move after their
  composite step.
- `crates/pictura-app/src/history.rs` — unchanged: `Snapshot` already carries
  the full `Document`.
- `crates/pictura-codec/src/lib.rs` — unchanged: `write_psd` keeps its layout
  and channel-count rule.
- `crates/pictura-app/cpp/main.cpp` — extend the M14/M17 self-test with a
  save-after-edit round-trip and an undo fast-path check.
- No new dependency. The CPU compositor, `composite_active`, and the ±1 LSB
  parity contract are unchanged. Copy-on-write / tile-diff history, resident
  GPU layer sources, zero-copy present, and 256² tiles are deferred.
