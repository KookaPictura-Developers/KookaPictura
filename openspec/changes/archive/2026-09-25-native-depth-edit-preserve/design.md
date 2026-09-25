# Design: native-depth-edit-preserve

## Context

At read, a 16/32-bit Grayscale/RGB document retains `Document.source_planes`
(the composite color channels followed by the document extras) and each layer's
`SourceChannels` (color, `-1` transparency, `-2` mask, and unmodeled planes,
keyed by channel id, with the layer rect they were decoded for). The writer
re-emits a retained plane only when `retained.narrow_to_u8() == current`, and
`layer_retained` additionally requires `store.rect == layer.rect` and a depth
match; `composite_retained` requires the store dimensions to equal the document
dimensions. The live compositor reads a layer's native store under the same
`store.rect == layer.rect` rule.

The destructive ops currently break one of those invariants: the translate paths
change `layer.rect` but not `store.rect`; the transform family drops the store;
the canvas/crop ops change the document size but not `source_planes`.

## Goals / Non-Goals

**Goals:**

- A moved/translated layer, a resized/rotated/flipped/projectively mapped/warped
  layer, an image-size resize, and a canvas-resize/crop all leave the retained
  stores valid, so a later save re-emits native samples.
- The `u8` path stays byte-identical to today for an 8-bit document (no store to
  resample), and the existing 16/32-bit behavior for an untouched open→save is
  unchanged.

**Non-Goals:**

- A true working-depth model: `write_container` still requires `doc.depth ==
  Eight`, so the store is a retention overlay, not the edit representation.
- Unmodeled raw on-disk channel streams (`Layer.raw_channels`) surviving a
  resample: they carry no position and no per-plane dims, so they still drop on
  a scale/rotate/warp (as today).
- Merge/flatten, via-copy, rasterize, Preserve Transparency, content
  generation, and GPU paths building or carrying a native store.
- Converting an 8-bit Lab/CMYK store (a source-mode store, not working RGB)
  through a geometry edit: it is resampled like any store, but the 8-bit working
  channel is not derived from it.

## Decisions

**One sample-typed kernel layer, dispatched on `Samples`.** A new
`document_ops/native_store.rs` holds `extend_samples` (offset blit),
`resize_samples` (the `pictura_ops::resize` kernels at native precision via
`Sample::to_unit`/`from_unit`), and `remap_samples` (the exact orientation
index maps). A new `layer_ops/transform_native.rs` holds `resample_native`
(inverse-map bilinear), `resample_store`, `warp_store`, and `bilinear_sample`.
The `u8` arm of each delegates to the existing 8-bit kernel
(`resample_plane`/`bilinear`/`warp_plane`) so 8-bit output is byte-identical and
only one new arithmetic path per sample type is introduced.

**Derive the 8-bit channel from the native narrowing.** After resampling the
store, the op overwrites the matching 8-bit channel (and the mask `data` for
`-2`) with `resampled.narrow_to_u8()`. That is exactly the writer's
`native_plane` equality test, so the store is used on save. This runs only when
`doc.source_mode.is_none()` (working Grayscale/RGB); a source-mode store (Lab/
CMYK) holds non-working planes and must not overwrite the working channel.

**The store rect is the layer rect; the `-2` plane follows the mask rect.** A
store keeps one `rect` (the layer bounds) plus planes whose `-2` plane may be
mask-sized. Move/translate/crop/canvas offset the store rect by the same delta
as `layer.rect` (the plane data is layer-local). Resample/warp size the `-2`
plane by the mask map and its destination, and all other planes by the layer
map; orient/resize pick the mask dimensions for `-2`.

**Canvas/crop rebase `source_planes`.** Because `composite_retained` checks the
store dimensions, a canvas-resize/crop offset-blits every composite/extra plane
and updates `store.width`/`height`. Image-size resamples them and orientation
remaps them (swapping the recorded dimensions for 90°/270°).

**File-size split.** `native_store.rs` and `transform_native.rs` are new
translation units so `transform.rs`/`warp.rs`/`resize.rs` stay under the cap;
`write.rs` is not modified.

## Risks / Trade-offs

- [Deriving the 8-bit channel changes the rendered 8-bit result for a high-depth
  edit, since the native resample's narrowing replaces `pictura_ops::resize`'s
  output] → the two agree in the untouched case and the native result is the
  intended one; 8-bit documents have no store and are unchanged.
- [An 8-bit Lab/CMYK source-mode store is resampled but not derived] → the
  working channel keeps its existing resample and the store's write-back path is
  unchanged.
- [A malformed store plane whose length is not a multiple of its plane size] →
  `chunks_exact` drops the partial plane, the same failure mode as before.
- [A store with no `-2` plane while the layer has a mask] → the `-2` native is
  absent, so the writer widens the mask as before.
