# M34 — composite coherence and cheap undo/redo

- **Status:** implemented OpenSpec change
  (`openspec/changes/m34-composite-coherence`); pending archive.
- **Capabilities:** MODIFIED `edit-history`, MODIFIED `document-lifecycle`.
  No new capability.
- **Depends on:** M14 (undo history), M17 (document lifecycle / Save), M31
  (region compositing), M33 (full-composite throughput).
- **Non-goals:** copy-on-write / tile-diff history snapshots, resident GPU layer
  sources, zero-copy present, 256² tiles, any change to `write_psd`'s format.

## Why

Two defects share one root: the document's `composite` buffer and the canvas
`QImage` drift apart, and the app treats the canvas as the only source of truth.

1. **Save writes a stale merged composite.** The displayed image is produced by
   `recomposite` → `document_to_image` → `current_buffer` → `composite_active`,
   but `recomposite` never writes the result back to `doc.composite`; it only
   replaces `rust.image`. `pictura_codec::write_psd` serializes
   `doc.composite`. So after any filter, adjustment, resize, undo, or blend
   change, Save emits a PSD whose merged image is the **pre-edit** one. M31
   added `patch_composite_region` for the region path, but it early-returns
   unless `doc.composite.channels == 4`, so it is a silent no-op for the
   3-plane RGB composite a freshly opened or new document carries — and it is
   the only place that tried.
2. **Undo/redo always recomposites the whole document.** `undo`/`redo` replace
   the document with the historical snapshot and then call `recomposite` — a
   full composite (~123 ms at 4000², M33-measured) plus a `buffer_to_image`
   (~40 ms). The snapshot already carries the full document, including its
   composite, so the recomposite recomputes bytes that were already captured.

## Frozen contract (the four requirements)

1. **The document composite is kept current.** Whenever the canvas is rebuilt —
   a full `recomposite` or a `refresh_region` — the rendered result SHALL be
   persisted into the document's `composite` buffer. An RGB document SHALL store
   the rendered **4-plane RGBA** frame, so its merged composite is RGBA after any
   rebuild. A non-RGB mode (Grayscale/Bitmap/Duotone/CMYK/Lab) SHALL keep its
   existing composite **colour-plane count** by copying
   `min(rendered.channels, composite.channels)` planes, so a 1-plane grayscale
   composite stays 1-plane and `write_psd` keeps its channel layout. When the
   document's composite dimensions no longer match the document (e.g. after a
   resize/crop/rotate), it SHALL be replaced with the rendered result rather
   than patched. Pixels outside the region of a `refresh_region` MUST keep
   their previous values.
2. **Save writes the current composite.** `save` SHALL serialize a document
   whose `composite` is the current rendered image, so a save after any edit
   round-trips to the edited pixels (not the pre-edit merged image). The
   existing `write_psd` byte layout and its use of `doc.composite.channels` MUST
   NOT change; an RGB document's merged composite is RGBA after a canvas
   rebuild, and a grayscale document keeps its plane count.
3. **Undo/redo restore without a full composite (unconditional).** `undo` and
   `redo` SHALL restore the displayed image from the historical snapshot's
   document composite instead of running a full composite, with no
   colour-plane or layer-count fallback, and the restored display MUST be
   byte-identical to a full recomposite of the restored document. (Across a
   GPU/CPU backend change it may differ by at most 1 LSB, consistent with the
   existing GPU parity contract.)
4. **History capture sees a current composite.** The history snapshot taken for
   a mutation SHALL carry a document whose composite is current, so
   requirement 3 holds for every recorded state. A mutating operation SHALL
   apply its region refresh / recomposite **before** recording the snapshot;
   the documented behaviour of `record` and the history order MUST otherwise be
   unchanged.

## Design in one paragraph

A single helper, `store_composite(doc, &PixelBuffer)`, becomes the one way a
rendered composite reaches the document. An RGB document takes the rendered
4-plane RGBA frame directly, so its merged composite is RGBA after any rebuild
and the cheap undo path is byte-identical to a full recomposite. A non-RGB mode
keeps its existing composite colour-plane count and copies
`min(rendered, composite)` planes, so a 1-plane grayscale composite stays
1-plane and `write_psd`'s layout is unchanged for it. `patch_composite_region`
becomes the region-limited form of the same plane rule instead of a
`channels == 4` early-return, and `recomposite` renders into a buffer, stores
it, then builds the `QImage` from that same buffer, so `doc.composite` and
`rust.image` are always the same pixels. `save` needs no code change: it
serializes `doc.composite`, which the rule keeps current — an in-progress stroke
is the one transient exception and is resolved by `end_paint`'s recomposite
before the stroke is recorded. `undo`/`redo` replace the document and restore
the display from `snapshot.doc.composite` directly (no composite), with no
fallback. The `record` call site in every mutation moves to **after** its
`recomposite`/`refresh_region`, so the captured snapshot carries the rendered
composite; history order, labels, and depth are unchanged.

## record-ordering change

Every site where `record(...)` is immediately followed by a
`recomposite()`/`refresh_region(...)` moves to after it (18 sites):

| line | handler | label | follows |
|---|---|---|---|
| 730 | `set_layer_visible` | `Layer Visibility` | `refresh_region` / `recomposite` |
| 764 | `set_layer_blend` | `Blend Mode` | `recomposite` |
| 788 | `set_layer_opacity` | `Opacity` | `recomposite` |
| 808 | `set_layer_name` | `Rename Layer` | `recomposite` |
| 828 | `move_layer` | `Reorder Layer` | `recomposite` |
| 919 | `apply_selection` | `Selection` | `recomposite` |
| 1045 | `crop` | `Crop` | `recomposite` |
| 1060 | `translate_layer` | `Move Layer` | `recomposite` |
| 1198 | `commit_move` | `Move Layer` | `refresh_region` |
| 1208 | `commit_move_legacy` | `Move Layer` | `recomposite` |
| 1284 | `add_adjustment` | `Adjustment` | `recomposite` |
| 1315 | `apply_filter` | `Filter` | `recomposite` |
| 1412 | `end_paint` | stroke label | `recomposite` |
| 1445 | `resize_image` | `Image Size` | `recomposite` |
| 1472 | `resize_canvas` | `Canvas Size` | `recomposite` |
| 1491 | `rotate_doc` | `Rotate` | `recomposite` |
| 1506 | `flip_doc` | `Flip` | `recomposite` |
| 1645 | `remove_layer` | `Delete Layer` | `recomposite` |

The three that do **not** move — `select_all` (851), `deselect` (858),
`magic_wand` (885) — call `changed()` directly and never composite; selection
does not alter composite pixels, so their snapshots already carry a current
composite. `Open` (546) and `New` (626) render/capture the initial state: they
gain the same render-and-store step before `history.capture`, so the first
snapshot is current too.

## History-semantics argument

`record` captures the **post-mutation** state (the M14 model: the initial state
at Open/New, then one captured state per command). Moving the call after the
composite step does not change *which* document state is captured — the layer
tree, selection, dimensions, and pixels are identical — it only changes the
composite bytes inside that state from stale to current. The stack order,
labels, cursor movement, depth bound, and redo truncation are untouched. The
sole observable difference is that an undone state now restores exactly what
the canvas showed when it was captured, which is the point.

## Equivalence and honest limits

- **RGB (4-plane composite).** `store_composite` replaces the composite with the
  rendered RGBA frame, so restoring from the snapshot composite is byte-identical
  to a full recomposite. Undo/redo take the no-composite path unconditionally.
  After a canvas rebuild an RGB document's composite is RGBA; `write_psd` writes
  four colour planes for it.
- **Non-RGB modes (Grayscale/Bitmap/Duotone/CMYK/Lab).** The stored composite
  keeps its plane count and copies `min(rendered, composite)` planes, so alpha
  (and, for a claimed-grey composite, any R≠G≠B content) is dropped.
  `buffer_to_image` forces alpha 255 for a 1-plane composite, matching the
  pre-M34 display of a layerless grayscale document. A **layered grayscale**
  document with transparent coverage is the one case where the snapshot
  composite does not reproduce the rendered alpha; requirement 3 is still
  unconditional, and this is the honest limit of keeping the plane count rather
  than emitting an unusual 4-plane grayscale PSD. The RGB path — the common
  document — has no such limit.
- **Dimension changes.** `document_ops::recompute` already replaces the
  composite with a 4-plane `composite_rgba` at the new size before `recomposite`
  runs, so a grayscale resize keeps the pre-M34 4-plane outcome; `write_psd`
  validates the size and the op keeps saving.
- **Backend change.** `set_gpu_compute` calls `recomposite`, which stores the
  active backend's result; undo across a backend change may differ by ≤1 LSB,
  which the contract allows.

## Verification

- Unit: `store_composite` stores four planes for RGB and preserves the plane
  count for Grayscale; a region patch preserves the count and leaves outside
  pixels untouched; the undo/redo display path (`buffer_to_image` of the
  snapshot composite) equals a full recomposite after a sequence of ops,
  including a `refresh_region`-based one.
- App: after an edit, `doc.composite` equals `composite_active`; Save then
  `read_psd` round-trips the edited pixels (not the pre-edit merged image); undo
  and redo restore the snapshot composite without a full composite and equal a
  recomposite.
- Self-test (C++, `main.cpp`): the composite-coherence probe, the M14 undo/redo
  round-trip, and a save-after-edit pixel round-trip, printed as
  `pictura self-test: m34_coherent composite=… undo=… save=…` with exit codes
  79/80/81.
- `openspec validate m34-composite-coherence --strict` and
  `openspec validate --all --strict`.
