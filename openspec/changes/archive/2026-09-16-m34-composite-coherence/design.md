## Context

The canvas is rebuilt in one of two ways and both already keep a whole-document
image, but neither keeps `doc.composite` current:

- `recomposite` (`cxxqt_object.rs:1817`) calls `document_to_image` →
  `current_buffer` → `composite_active`, builds a `QImage`, and stores only
  `rust.image`.
- `refresh_region` (`cxxqt_object.rs:1747`) composites the clamped rect with
  `composite_region_active`, blits it into `rust.image`, and calls
  `patch_composite_region(doc, …)` — which early-returns unless
  `doc.composite.channels == 4` (`cxxqt_object.rs:2482`). A freshly opened or
  new RGB document carries a 3-plane composite (`ColorMode::color_channels()` is
  3 for RGB, 1 for Grayscale), so the patch silently does nothing there.

`pictura_codec::write_psd` (`pictura-codec/src/lib.rs:931`) serializes
`doc.composite` and derives the colour-plane count from `doc.composite.channels`
(line 940), so a stale `doc.composite` is what Save writes.

Undo/redo (`cxxqt_object.rs:1511`, `:1523`) restore `snapshot.doc` and
`snapshot.selection`, then call `recomposite`; `Snapshot` (`history.rs:5`)
already carries the full `Document`, whose `composite` is exactly the rendered
image that was current when the snapshot was captured (once the capture ordering
below is fixed). The full composite at 4000² is ~123 ms after M33 and
`buffer_to_image` ~40 ms, so an undo currently pays ~163 ms it can avoid.

Constraints: the CPU compositor (`composite_rgba`) is the frozen oracle,
`composite_active`/`composite_region_active` are the full/region oracles, the GPU
path is ±1 LSB against the CPU oracle, `write_psd`'s layout and channel-count
rule must not change, no new dependency, and `docs/` is the long-form contract.

Relevant code: `recomposite` (`cxxqt_object.rs:1817`), `refresh_region`
(`:1747`), `patch_composite_region` (`:2482`), `blit_image_region` (`:2455`),
`current_buffer` (`:2529`), `document_to_image` (`:2538`), `buffer_to_image`
(`:2661`), `save` (`:638`), `undo`/`redo` (`:1511`/`:1523`), `record` (`:1615`),
`open` (`:522`), `new_document` (`:558`), `write_psd`
(`pictura-codec/src/lib.rs:931`).

## Goals / Non-Goals

**Goals:**

- Keep `doc.composite` equal to the rendered image after every canvas rebuild
  (full `recomposite` and region `refresh_region`): RGBA for an RGB document,
  plane-count-preserving for a non-RGB mode, so `write_psd`'s layout is
  unchanged for grayscale and RGBA for RGB.
- Make Save serialize the current rendered composite.
- Make `undo`/`redo` restore the display from the snapshot's document composite
  without a full composite, byte-identical (≤1 LSB across a backend change).
- Make the captured snapshot carry a current composite by moving `record` after
  the composite step, without changing history order or labels.
- Ship unit + app-self-test evidence.

**Non-Goals:**

- Copy-on-write / tile-diff history snapshots (`Document::clone()` stays).
- Resident GPU layer sources, zero-copy present, 256² tiles / LoD.
- Any change to `write_psd`'s bytes, channel-count rule, or the codec.
- A second cached composite buffer; `doc.composite` is the one store.
- Patching the transient working document during a stroke (see risks).

## Decisions

### 1. `store_composite(doc, &PixelBuffer)` (frozen)

```rust
/// Persist a full-frame rendered composite into `doc.composite`.
fn store_composite(doc: &mut Document, rendered: &PixelBuffer)
```

Rules, in order:

1. If `rendered.width != doc.width || rendered.height != doc.height`, return
   without touching `doc.composite` (the caller is a region patch or a stale
   render; a full `recomposite` always renders at document size).
2. Normalise the render to a 4-plane RGBA frame (`rgba_frame`), expanding a
   lower-plane render (a layerless document's embedded composite) with opaque
   alpha and grey replication exactly as `buffer_to_image` does.
3. If `doc.mode == ColorMode::Rgb`, or `doc.composite`'s dimensions do not match
   the document (a dimension change), **replace** `doc.composite` with the
   4-plane frame. An RGB document is therefore RGBA after any rebuild.
4. Otherwise (a non-RGB mode at document size) **preserve**
   `doc.composite.channels`: allocate that many planes and copy
   `min(4, doc.composite.channels)` planes of the RGBA frame in order. A
   1-plane grayscale composite stays one plane (the R plane, which is the grey
   value), so `write_psd` keeps writing the same channel layout. No resize, no
   per-pixel arithmetic — a planar `copy_from_slice` per plane.

- *Why:* one helper is the single funnel for "rendered pixels reach the
  document", so `recomposite`, `refresh_region`, `open`, and `new_document` all
  agree. RGB stores the compositor's RGBA (the cheap-undo path needs the alpha
  plane to be byte-identical), while a non-RGB mode keeps the PSD channel
  layout it had.
- *Alternatives:* keep the pre-revision plane-count preservation for RGB
  (rejected — a 3-plane RGB composite drops the rendered alpha, so the cheap
  undo path is not byte-identical and would fall back to a full composite for
  almost every document); always store four planes (rejected — emits an unusual
  4-plane grayscale PSD).

### 2. Channel-consistent `patch_composite_region` (frozen)

`patch_composite_region(doc, region, x0, y0)` keeps its signature, its
dimension guard, and its bounds guards. It requires the region to be 4-plane
(the region oracle always is) and writes `min(4, doc.composite.channels)`
planes at the composite's own plane count: plane `c` of the region maps to
plane `c` of the composite, one `copy_from_slice` per row per plane. Pixels
outside `[x0, x0+rw) × [y0, y0+rh)` are untouched. For RGB the composite is
4-plane (decision 1), so all four planes are written; for grayscale only the
grey plane is.

- *Why:* the region path must keep the same channel layout as the full path;
  the old `channels == 4` early-return let `doc.composite` go stale for a
  grayscale document (and, pre-revision, for the common 3-plane RGB one).
- *Alternatives:* widen the guard to `region.channels >= dst.channels`
  (insufficient — the plane mapping still has to be explicit for grayscale).

### 3. `recomposite` renders once and stores (frozen)

`recomposite` becomes: take the document and `gpu_compute`; compute
`let rendered = current_buffer(doc, gpu_compute);` (the existing oracle-routed
path); `store_composite(doc, &rendered)`; build `rust.image =
buffer_to_image(&rendered)`; `changed()`. For a layerless document
`current_buffer` returns `doc.composite.clone()`, so `store_composite`
normalises the embedded composite (RGBA for RGB); the display is
`buffer_to_image` of that same render and is byte-identical to today.

- *Why:* the stored composite and the displayed `QImage` are derived from the
  same `PixelBuffer`, so they cannot diverge.
- *Alternatives:* keep `document_to_image` and re-render for the store
  (rejected — two composites per rebuild; the whole point is coherence at no
  extra cost).

### 4. `refresh_region` stores the region on the document (frozen)

On the region path, after the display blit, call
`patch_composite_region(doc, &buffer, x0, y0)` against the app document exactly
as today — now channel-consistent, so an RGB 4-plane and a grayscale 1-plane
composite are both patched. `refresh_region`'s over-budget fallback keeps
calling `recomposite` (which now stores). During an active stroke
(`painting == true`) the region is composited from the stroke's working
document; `rust.doc` is the pre-stroke base, so the app document's composite is
**not** patched mid-stroke. `end_paint` assigns the finished document and calls
`recomposite`, which stores the finished composite, and it does so before
`record` (decision 7).

- *Why:* a committed region refresh must leave `doc.composite` current; the
  stroke workspace is transient and reconciled at commit.
- *Alternatives:* patch the working document too (needs a mutable handle the
  `Stroke` does not expose, and it is discarded on cancel — not worth it).

### 5. Save path (frozen)

`save` (`cxxqt_object.rs:638`) is unchanged in mechanism: it still calls
`write_psd(doc)` and writes the bytes atomically. It is now correct because
requirement 1 keeps `doc.composite` current for every committed mutation. The
spec requirement (document-lifecycle) is what makes this a contract rather than
an accident. The only state in which `doc.composite` is not the current render is
mid-stroke, which is transient, unreachable for a committed document, and is
resolved by `end_paint`'s `recomposite`.

- *Why:* no code change, no new format, no extra render per save.
- *Alternatives:* recomposite inside `save` (rejected — pays a full composite on
  every save and cannot see the uncommitted stroke anyway).

### 6. `undo`/`redo` restore from the snapshot composite (frozen)

`undo`/`redo` replace `rust.doc` and `rust.selection` from the snapshot exactly as
today. Then, instead of `recomposite`:

```rust
rust.image = buffer_to_image(&rust.doc.as_ref().unwrap().composite);
rust.changed();
```

The restore is unconditional. For an RGB document the snapshot's composite is
the 4-plane RGBA render (decision 1), so `buffer_to_image` is exactly
`document_to_image(restored)` and the restore is byte-identical to a full
recomposite. There is no `channels == 4 || layers.is_empty()` fallback: the
revision makes the composite a complete rendered image for the common case, so
the cheap path is always taken. `history_jump` and `history_restore_snapshot`
restore snapshots too and keep their `recomposite`; they are not on the frozen
undo/redo contract and remain byte-correct.

- *Why:* the snapshot already contains the rendered composite after decision 4
  and decision 7; restoring it directly removes the ~123 ms composite +
  ~40 ms `buffer_to_image` from undo/redo.
- *Alternatives:* keep the low-plane fallback (rejected — the revision removes
  the condition that made it necessary, and the fallback re-composited almost
  every RGB document).

Honest limit: a **non-RGB** composite keeps its plane count (decision 1), so a
grayscale snapshot's composite drops the rendered alpha. `buffer_to_image`
forces alpha 255 for a 1-plane composite, so a layered grayscale document with
transparent coverage would restore opaque. The app's only non-RGB path is
Grayscale (`new_document` accepts `"rgb"` and `"grayscale"`), this is not on the
common RGB path, and the contract's byte-identity claim is stated for the
rendered composite that the store carries. It is called out in the brief rather
than hidden behind a fallback.

### 7. Capture ordering and Open/New (frozen)

Every mutation that composites moves its `record(...)` to **after** the
`recomposite`/`refresh_region` (18 sites; the table is in
`docs/dev/m34-composite-coherence.md`). The three selection handlers
(`select_all`, `deselect`, `magic_wand`) call `changed()` directly and never
composite; they are unchanged because a selection does not alter composite
pixels. `record`'s body and `History` are unchanged.

`open` and `new_document` build the initial state with
`current_buffer` → `buffer_to_image` and store that render before
`history.capture`, so the `"Open"`/`"New"` snapshot is current. For a layerless
PSD `current_buffer` is the file composite: an RGB document normalises it to
4-plane RGBA (decision 1), a grayscale document keeps its one plane. The
displayed image is built from the same render, so it is unchanged.

- *Why:* requirement 3 needs every captured state to carry a rendered composite,
  and the captured state is the post-mutation state (the M14 model). Moving the
  call changes only the composite bytes inside the state, never which state is
  captured. Labels, stack order, cursor movement, the depth bound, and redo
  truncation are untouched.
- *Alternatives:* capture before the mutation (rejected — a different history
  model; out of scope).

### 8. Process (frozen)

Waves: (1) brief and frozen interfaces; (2) `store_composite` +
channel-consistent region patch + `recomposite`; (3) save coverage; (4)
`undo`/`redo`; (5) `record` ordering + Open/New; (6) unit tests + app self-test
+ evidence; (7) close-out.

## Risks / Trade-offs

- **RGB stores four planes.** An RGB document's merged composite becomes RGBA
  after any rebuild, so `write_psd` writes four colour planes for an RGB
  document that previously carried three. This is the deliberate revision: the
  cheap undo path needs the rendered alpha to be byte-identical, and the merged
  image is a rendered frame, not the original file's channel list. The
  `write_psd` byte layout is untouched; only the header channel count changes.
- **Plane-count preservation for a non-RGB mode.** A 1-plane grayscale
  composite cannot carry the rendered alpha, so `buffer_to_image` forces alpha
  255 for it (the same as the pre-M34 display of a layerless grayscale
  document). `undo`/`redo` are unconditional; a **layered grayscale** snapshot
  with transparent coverage can therefore restore opaque. This is the honest
  limit of keeping the plane count; the alternative (a 2- or 4-plane grayscale
  composite) emits an unusual PSD channel layout.
- **Dimension changes.** A separate document op (`resize`, `crop`, `rotate`,
  `flip`, `resize_canvas`) calls `document_ops::recompute`, which already
  replaces `doc.composite` with a 4-plane `composite_rgba` at the new size
  before `recomposite` runs. `store_composite` sees a matching 4-plane composite
  and stores it; `write_psd` validates the size, so the op keeps saving. A
  grayscale document that goes through a dimension op therefore becomes
  4-plane, as it already does today.
- **Backend change.** `set_gpu_compute` calls `recomposite`; a snapshot taken
  before the change holds a CPU (or GPU) composite and an undo after it may
  differ by ≤1 LSB per channel. The existing GPU parity contract permits this.
- **History semantics.** The captured state is unchanged except for its
  composite bytes; the `record` doc comment gains a sentence that it expects a
  current composite. A stale composite can no longer be captured because the
  ordering fix removes the only window in which one existed.
- **Existing M14/M17 self-tests.** They compare `view->image()` before/after
  undo/redo. The unconditional fast path produces
  `buffer_to_image(snapshot.composite)`, which equals the captured display
  because the snapshot was captured from that same render, so the comparisons
  still hold. The M17 save round-trip saves a freshly created white 4×3 RGB
  scratch document: `new_document` now stores the white render as 4-plane RGBA,
  and the reopened display is the same white pixels. Any assertion that relied
  on Save preserving a stale merged image would be corrected, not weakened, and
  the task result names it if found.
- **Region patch plane mapping.** Unit tests assert the exact plane count and
  that bytes outside the region are untouched; `right`/`bottom` exclusive is
  asserted.
- **A mutation that composites but does not call `record`** (e.g. `move_preview`
  during a drag) updates the cache without a snapshot; it is a preview, and
  `commit_move` records the committed state. `begin_move_preview` deliberately
  does not modify `doc.composite` (M32), so it does not go through
  `store_composite`.
- **Mid-stroke Save.** Not a committed state; the app's stroke is committed on
  release and `end_paint` stores before recording. Documented, not handled.

## Migration Plan

Additive and internal. `store_composite` is new; `patch_composite_region`
widens; `recomposite`, `refresh_region`, `undo`, `redo`, `open`,
`new_document`, and the 18 `record` sites change in place. `save`, `write_psd`,
`Snapshot`, `History`, the CPU compositor, `composite_active`, and the ±1 LSB
contract are unchanged. Rollback: stop calling `store_composite` from
`recomposite`/`refresh_region`, restore the `channels == 4` guard and the
`undo`/`redo` `recomposite`, and put `record` back before the composite step;
the pre-M34 behaviour returns.

## Open Questions

- Whether `history_jump` / `history_restore_snapshot` should adopt the same fast
  restore — not required by the frozen contract; they stay on `recomposite` and
  remain correct.
- Whether `Open` should store the recomposited colour planes over the file's
  merged composite or keep the file bytes when the document is unmodified — the
  contract says the captured composite is current, so store; revisit if a
  round-trip fidelity test on a stale-merged PSD disagrees.
- Whether the `record`-ordering change should be one mechanical pass per wave
  or split by file region — either is fine; keep the diff reviewable.
