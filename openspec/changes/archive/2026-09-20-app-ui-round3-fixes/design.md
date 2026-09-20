## Context

Round-3 verification (see `proposal.md`). Only the four confirmed gaps are in
scope; the rest of the report is already implemented and self-test-covered.

## Decisions

### D1 — Layer reparent index math

`move_path_to_dest` (`properties.rs:402-447`) computes `dest_parent` in
**pre-removal** coordinates but validates it with `container_exists_after_removal`
(`:377-396`), whose `parent[depth] + 1` mapping assumes **post-removal**
coordinates; the mutation then resolves `dest_parent` against the post-removal
document. Dropping layer 0 into group 1 of `[A, G]` is therefore refused
(`+1` overshoots) and dropping it into group 1 of `[A, G, B]` resolves the wrong
node and mis-nests `A` (it disappears from the tree but the move reports
success).

Fix: keep `dest_parent` pre-removal, validate it with the pre-removal
`container_of`, and convert it once to post-removal coordinates before
`container_of_mut`:

```
fn to_post_removal(parent, src_parent, src_index) -> Vec<usize>
// for each depth: if parent[..depth] == src_parent && parent[depth] > src_index
//                 then parent[depth] - 1 else parent[depth]
```

`dest_index` keeps its existing same-parent `-1` adjustment. A destination inside
the source's own subtree is already refused by the `target.starts_with(path + "/")`
guard. Delete `container_exists_after_removal`.

Test matrix: `[A,G]` → A into G; `[A,G,B]` → A into G (A must be G's child, B
stays at root); a group child moved out to root and above/below a root sibling;
same-parent reorder unchanged; existing refusal cases unchanged.

### D2 — Active-layer cursor and refusal

`topmostPixelLocked` (`tools_marquee.cpp:23-30`) and the cursor branch
(`:72-75`) plus the brush refusal message (`tool_brush.cpp:43-49`) key on
`topmost_pixel_layer_index`, while the paint/filter target is the **active**
layer (`helpers.rs:612-616`). Resolve the active layer's lock and visibility
instead (a bridge query mirroring `active_pixel_layer`, or reuse
`active_layer_path` + `layer_lock`/`active_layer_visible`). The blank-cursor
affordance and the Alt-eyedropper precedence stay.

### D3 — Panel width/mode persistence

The mode and width already persist per column (schema v8). Harden the
iconic→normal flip: when persisting while `normalWidthBeforeIconic_` is known and
the column is mid-flip, write the remembered normal width rather than the
transient `width()`. Add a regression that flips the **primary** column
iconic→normal, saves, reconstructs, and asserts normal mode at the remembered
width. The v7→v8 seed from the primary's legacy mode stays (documented).

### D4 — Paint responsiveness (Krita-style)

Krita paints strokes on the CPU into tile data, touching only dirty regions, and
does not GPU-sync per dab; its documented hot spots are thumbnails, histogram,
and the brush outline. Apply the same levers:

1. **Keep the faster in-stroke backend.** A 4000² profile
   (`paint_dab_profile_4000`) measured the CPU region oracle at ~11 ms/dab vs
   ~1.4 ms/dab for the GPU region composite, so painting keeps the preferred
   (GPU) backend. Forcing the CPU in-stroke was budgeted first but is a measured
   regression and is **not** done; the per-dab GPU cost is not the bottleneck.
2. **No per-dab GUI fan-out.** On a paint `regionBlitted` (`frame.cpp:375-395`),
   update the canvas and the coalesced panel timer, but skip
   `registry_->refresh()`, `updateTabTitle`, `updateWindowTitle`, and the cursor
   refresh while a stroke is active; run them once on commit. (Krita's
   "unnecessary objects per event" hot spot.)
3. **Patch, don't rebuild, and don't detach.** Blit into the full-resolution
   canvas with direct row writes rather than constructing a `QPainter` on the
   possibly-shared `QImage` (which detaches the whole 64 MB buffer), and keep the
   present-cache patch path.
4. **Benchmark.** Add a `#[ignore]`d 4000² live-dab latency profile that prints
   per-dab CPU-composite + blit timings, so a regression is measurable. No
   wall-clock pass/fail budget (CI flakiness); assert the shape (CPU path taken,
   no registry refresh per dab).

### D5 — Lock coverage

Add a C++ self-test that sets a transparency lock through the bridge, paints, and
asserts RGB changes while alpha is preserved (including an `A=180` pixel), plus a
panel-driven multi-selection check that edits are refused. Content move keeps its
existing transparency refusal as a documented divergence.

## Deferred

- **Artboard/frame nesting lock.** The engine has no artboards/frames and the
  Move tool never reparents, so the Photoshop "prevent auto-nesting" behaviour
  has nothing to gate. Recorded as a ceiling.
- Any broader stroke-engine rewrite (tiles, threading, vectorization) or
  GPU-resident stroke buffers.

## Risks

- D4 changes the in-stroke backend; the CPU region composite is already the
  byte-exact oracle, so output is unchanged, but a GPU-vs-CPU painting parity
  check is added.
- D4.2 must not skip a needed repaint when the stroke ends; the commit path
  already runs the full refresh/record.
- D1 must keep every existing refusal (Background, fully/nesting locked, own
  descendant, non-group Into) intact.
