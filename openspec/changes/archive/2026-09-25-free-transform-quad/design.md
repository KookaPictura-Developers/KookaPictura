# Design: free-transform-quad

## Context

`transform_layer` builds a `Map` from `LayerTransform` (similarity) and resamples
each plane by its inverse. The app keeps a `TransformSession` of five scalars
plus the original rect; `transform_quad_points` derives the four display corners
from those scalars, so the overlay and hit-test already work in quad space. Commit
maps the scalars back to `LayerTransform`.

`Resample`/`Interpolation` is out of scope: the shipped op is bilinear only, and
this change keeps bilinear.

## Goals / Non-Goals

**Goals**

- A projective (homography) transform op that reuses the existing refusal,
  materialization, bounding-box, and bilinear machinery.
- CS6 Skew / Distort / Perspective modes that edit a live quad and commit it.
- The three menu commands become real.

**Non-Goals**

- Interpolation choice (Nearest/Bicubic/etc.), Warp (Bezier patch), Puppet Warp,
  Content-Aware Scale. `TOOL-001` marks the warp algorithms TBD/inferred.
- Smart-object `Trnf` application and text warp.

## Decisions

**`transform_layer_quad(doc, path, quad: [(f64, f64); 4]) -> bool`.** `quad[i]`
is the document-space target of source corner `i` in TL, TR, BR, BL order (the
existing `source_corners` order). Solve the 3×3 homography `H` with `H·(x,y,1)`
proportional to the target and 8 degrees of freedom by the standard 8×8 linear
system with Gaussian elimination (deterministic, `f64`). The inverse `H⁻¹` maps a
destination pixel to its source point; out-of-source → `0`, taps edge-clamped.
The destination rect is the integer bounding box of `quad` (`floor(min)`,
`ceil(max)`), matching the existing rounding.

**Refusal and sharing.** `transform_layer_quad` refuses exactly like
`transform_layer` (missing path, group, adjustment, Background, position-locked,
zero-area source, no materializable channel-less payload, empty result) and ALSO
refuses a degenerate quad: a singular/near-singular `H`, a non-finite corner, or
a zero-area destination. Both functions route through the same internal helper
that resolves the target, obtains source channels (materializing a channel-less
smart object), and writes the result, so the smart-object and raw-channel rules
cannot drift. The similarity path stays bit-identical: it keeps its own `Map`.

**Raw/source channels.** A projective transform is not an integer translation, so
`raw_channels` is cleared and `source_channels` dropped, as the existing
similarity op already does for scale/rotate.

**App session.** Add `mode: TransformMode` (`Free | Skew | Distort | Perspective`)
and `quad: Option<[(f64, f64); 4]>`. In `Free`, `quad` is `None` and the existing
scalars drive everything. In a projective mode, `quad` is the live target and the
scalars are unused. `transform_quad_points` returns the live `quad` when present,
so the overlay and hit-test need no fork.

**Gestures (inferred; no Photoshop oracle).**
- Distort: dragging corner `h` (0..=3) sets `quad[h] = pointer`.
- Perspective: dragging corner `h` sets `quad[h] = pointer` and moves `quad[opp]`
  by the negated delta, keeping the quad's centre fixed.
- Skew: dragging edge `h` (4..=7) translates that edge's two endpoints by the
  same delta, keeping the opposite edge fixed; Shift constrains the delta to the
  edge's own axis.
Each gesture clamps the resulting quad to a non-degenerate area; a gesture that
would collapse it is refused.

**Hit-testing.** In a projective mode only the eight handles are active: an edge
handle for Skew, a corner handle for Distort/Perspective; a press away from a
handle returns `-1` (no move/rotate). `Free` is unchanged.

**Commit.** `commit_transform` calls `transform_layer` in `Free` mode and
`transform_layer_quad` otherwise; identity (all four corners equal the source
corners, within an epsilon) records nothing.

**Commands.** New ids `edit.transform.skew|distort|perspective`; specs marked
`implemented = true`; handlers call `beginTransformMode`; enablement reuses the
Free Transform predicate (`layer_can_free_transform`). `begin_transform_mode`
shares `begin_free_transform`'s target resolution, refusing identically.

## Risks / Trade-offs

- [Projective division near a singular `H`] → refuse when `|det|` is below a small
  epsilon or any corner is non-finite; the op must not panic or allocate a
  pathological rect (the existing `MAX_RESULT_PIXELS` guard is reused).
- [Gesture semantics are inferred] → documented in the spec as inferred; the
  engine op is exact and is what the tests pin.
- [Session refactor regresses the tested similarity path] → the similarity `Map`
  and its gestures are untouched; new state is additive.

## Migration Plan

Additive; revert is a revert.
