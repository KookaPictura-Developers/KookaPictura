## Context

The codec preserves every unmodeled per-layer tagged block in
`Layer.extra_blocks` (`crates/pictura-codec/src/read.rs` `_ => extra_blocks.push`,
re-emitted by `write_extra` in `crates/pictura-codec/src/write.rs`). `vmsk`
(vector mask) and `vscg` (vector fill/stroke content) are both in that set, so
they survive open→save but are never rendered. A shape layer is normally a fill
layer — a `SoCo`/`GdFl`/`PtFl` adjustment, all three already decoded and
composited — plus a `vmsk` that clips it. Rendering the `vmsk` therefore makes
basic shape layers correct.

The compositor applies a layer's raster mask in one place:
`mask_alpha(layer, x, y) -> u8` (`crates/pictura-render/src/composite.rs:978`),
returning `255` for an absent/disabled/data-less mask and otherwise the `-2`
channel sample at the canvas pixel. `blend_into` multiplies the source alpha by
`opacity × fill × mask_alpha`. The GPU path assembles the same per-pixel mask
plane via `assemble_mask`/`assemble_mask_per_pixel` and decides between it and a
constant fill with `mask_has_data`
(`crates/pictura-render/src/gpu/backend.rs:756`). Layer-effect modules also call
`mask_alpha` to gate effect coverage.

`pictura-render` depends on `pictura-codec` (for smart-object source decode)
and `pictura-core`, but not on `pictura-select`, so the even-odd polygon fill in
`pictura_select::Selection::polygon` cannot be reused without a new dependency.

### Grounding

**Block header** (psd-tools `psd/vector.py` `VectorMaskSetting`, ag-psd
`additionalInfo.ts` `vmsk` handler):

- `u32 version` — must be `3`.
- `u32 flags` — bit 0 `invert`, bit 1 `not_link`, bit 2 `disable`.

**Path records** (Adobe's path-record format, restated in the Photoshop
community thread "How to read path data", and implemented identically by
psd-tools `Path.read` and ag-psd `readVectorMask`). Each record begins with a
big-endian `u16` selector and is read until fewer than 26 bytes remain:

| selector | meaning | payload after the selector |
|---|---|---|
| 0 / 3 | closed / open subpath length | `u16` knot count, `i16` operation, `u16` fill-rule field, 18 unused bytes |
| 1 / 2 | closed knot, linked / unlinked | three `(y, x)` 8.24 fixed-point points: preceding, anchor, leaving |
| 4 / 5 | open knot, linked / unlinked | same |
| 6 | path fill rule | 24 unused bytes |
| 7 | clipboard | 5 fixed-point numbers + 4 bytes (24 unused) |
| 8 | initial fill | `u16` value + 22 unused bytes |

psd-tools models the subpath's `operation` (`1` union, `2` not-or, `3`
intersect, `0` xor, `-1` continuation, per its `Subpath` docstring) and the
knots; it names the fill-rule field `_unknown1` and drops it. ag-psd reads that
same field as `flags` and derives `fillRule = flags === 2 ? 'non-zero' :
'even-odd'`, writing `2`/`1`. Adobe's format text says the subpath-length record
tail is "unused, and should be zeroes" and that "Paths use even/odd ruling", and
its `PathFillRule` record is 24 zero bytes.

**Coordinates** (Adobe format, psd-tools `composite/vector.py`, ag-psd
`readBezierKnot`): 8.24 signed fixed point (`/ 0x01000000`), normalized to the
document, `[0,0]` top-left and `0x01000000` the full width/height; ag-psd and
psd-tools both multiply by document width for x and height for y. The mask is
therefore **document-relative**, not layer-relative.

**Fixture round-trip (confirmed empirically in this session):** psd-tools'
typed `VectorMaskSetting`/`Path`/`ClosedPath`/`ClosedKnotLinked` writer emits a
`vmsk` block that psd-tools re-reads (version 3, flags, one closed subpath,
operation 1, four knots, anchors `(0.25,0.25)…(0.75,0.75)` for a `(2,2)-(6,6)`
rectangle in an 8×8 document) and that ag-psd reads (`invert` from bit 0,
`operation: "combine"`, `fillRule: "even-odd"`, pixel knots). The block payload
is `u32` version, `u32` flags, then six 26-byte records = 164 bytes.

## Goals / Non-Goals

**Goals:**

- Decode `vmsk` into a typed, sampled view and clip the layer's contribution by
  it, combined with any raster mask.
- Keep the raw block as the serialization source of truth, so round-trips are
  byte-identical and no writer changes are needed.
- Cover the common shape layer: one or more closed subpaths unioned under the
  documented fill rule.
- Prove the decode with psd-tools **and** ag-psd over a committed fixture, and
  prove the clip with a render test.

**Non-Goals:**

- Open subpaths, `vscg`, `vsms`, subtract/intersect/xor geometry, feather,
  density, antialiasing, Reveal/Hide-All, "Vector Mask Hides Effects".
- Editing or authoring a vector mask from the app; it is render-only.
- Transform/crop/resize propagation of a preserved vector mask.
- A GPU-specific rasterizer or Photoshop pixel parity.

## Decisions

### D1. `vmsk` is decoded into a derived `Layer.vector_mask`, not a new field on the wire

`pictura-core` gains `Layer.vector_mask: Option<VectorMask>` as a **read-derived
view**, exactly like `Layer.smart_object` (which is resolved from preserved bytes
and is explicitly "only what the engine can resolve"). The raw `vmsk` block is
left in `extra_blocks`, so `write_extra` re-emits it verbatim and no encoder
exists to drift from the file. `vector_mask` is never serialized; a hand-built
`Layer` defaults to `None`, and the round-trip equality tests hold because both
sides are read.

Alternative rejected: synthesize a raster `LayerMask` from the path at decode
time. That would serialize a synthetic `-2` channel on save (breaking byte
identity) and would put a rasterizer in the pure-format codec. Alternative
rejected: decode from `extra_blocks` inside the renderer. `mask_alpha` is called
per pixel, so decoding there either re-parses the block per pixel or forces a
per-composite cache and a new parameter through every call site (including the
eight layer-effect modules); the model field makes the per-pixel path O(edges)
with no plumbing.

### D2. The view stores flattened, document-pixel polylines, not Bezier knots

The decode parses knots (the typed intermediate), then flattens each cubic
segment to a fixed polyline with De Casteljau subdivision and scales the
coordinates to document pixels. Storing flattened points is what makes per-pixel
sampling cheap. To keep `Layer`'s `PartialEq`/`Eq` derives intact, points are
stored as sub-pixel integers (`i32`, 1/256 px) rather than `f32`; coverage
divides by 256. Flattening is fixed at 16 segments per cubic — `ponytail:` a
fixed subdivision, not adaptive; upgrade to adaptive flatness if curved masks
show visible faceting. (A `Vec<f32>` would force dropping `Eq` from `Layer` and
`Document` and rippling through every deriving type.)

### D3. Fill rule: even-odd default, ag-psd's marker `2` selects non-zero

Adobe's format text states "Paths use even/odd ruling", its `PathFillRule` record
is 24 zero bytes, and it calls the subpath-length tail unused. psd-tools drops
the field. Only ag-psd assigns it meaning (`2` → non-zero, anything else →
even-odd, including the zero Photoshop writes). The decoder therefore defaults
to **even-odd** and promotes to non-zero only when a closed subpath's fill-rule
field is exactly `2`. This matches Photoshop-authored files (even-odd) and
ag-psd-authored files without claiming a Photoshop guarantee for non-zero. The
non-zero reading is marked as ag-psd-derived, not Adobe-documented.

### D4. First slice combines closed subpaths under one fill; union only

The coverage of a canvas pixel is computed by crossing-testing the flattened
polylines of every **closed** subpath: even-odd (odd crossing count) or
non-zero (winding count ≠ 0) per D3. `operation` is recorded but not applied, so
subtract/intersect/xor subpaths are folded in as if union — the documented
ceiling. A path whose hole is a second winding/even-odd subpath renders
correctly; a path whose hole is a `subtract` operation does not. If any subpath
declares non-zero, the whole path is sampled non-zero. Open subpaths contribute
nothing. A path with no closed subpath yields coverage `255` (no clipping) — a
safe no-op that makes an empty/Reveal-All mask inert and avoids accidentally
hiding a layer.

### D5. Coverage is combined with the raster mask in `mask_alpha`

`mask_alpha` becomes "raster sample × vector coverage", returning
`(raster as u16 * vector as u16 + 127) / 255` (255 when either is absent/disabled
and `255 − c` when `invert` is set). This keeps the single hook already used by
content and by layer effects, so a vector mask gates effects too. Defaulting
everything to the same gate is a simplification; the advanced "Vector Mask Hides
Effects" toggle is deferred.

### D6. GPU parity by inclusion in `mask_has_data`

`mask_has_data` returns true when the layer has a live vector mask, so
`assemble_mask` takes the per-pixel branch and uploads a plane built from the
same `mask_alpha`. No GPU rasterizer is added; the GPU keeps its existing ±1 LSB
CPU parity and the CPU remains the oracle (rule 6).

### D7. Fixture authored with psd-tools' typed `vmsk` writer; two independent oracles

psd-tools' typed `VectorMaskSetting` writer round-trips (D-grounding), so the
fixture is authored with it (not hand-built bytes). `scripts/generate-fixtures.py`
gains `vector_mask()` and registers `vector_mask.psd`. The fixture is 8×8 RGB:
a `Base` pixel layer, a `Shape Inverted` solid-fill layer (`SoCo`, blue) with a
closed `(0,0)-(6,6)` rectangle `vmsk` whose flags bit 0 is set, and a `Shape`
solid-fill layer (`SoCo`, red) with a closed `(1,1)-(3,3)` rectangle (flags 0).
Composited bottom-first (Base, Shape Inverted, Shape) the test can assert red
inside the smaller rect, base inside the larger inverted rect but outside the
smaller one, and blue outside the larger rect.

`crates/pictura-codec/tests/oracle.rs` is 1398 lines against the 1400-line test
cap, so the psd-tools check goes in a new `vector_mask_oracle.rs`; the ag-psd
check is appended to `agpsd_oracle.rs` (260 lines). Both assert the version,
`invert`, and the path geometry (`ag-psd` fully; psd-tools fully for the
format).

### D8. No app change and no self-test code

The app renders through `composite_rgba`/`composite_active`, so the clip appears
with no bridge, CPP, or panel change; there is no vector-mask UI in this slice.
The runnable check is the render test (rule 2), so no `ST_FAIL` code is consumed.

## Risks / Trade-offs

- **The derived view can desync from the raw bytes.** `vector_mask` is computed
  once on read and is not recomputed by document ops. Mitigation: it is only a
  render input; the raw block remains authoritative and round-trips untouched,
  and a document resize/crop leaving a stale mask is a listed ceiling.
- **A hand-built `vmsk` extra block will not populate the view.** A test that
  constructs `Layer.extra_blocks` with a `vmsk` directly sees `vector_mask:
  None`, unlike a read document. Mitigation: document the read-derived contract;
  tests decode through `read_psd` (or set the view explicitly).
- **Adding a field to `Layer` touches every exhaustive literal** (~32 sites).
  Mitigation: mechanical `vector_mask: None`; literals using
  `..Default::default()` are unaffected.
- **Fixed polyline subdivision and hard 0/255 coverage** approximate curved
  masks and produce aliased edges. Mitigation: `ponytail:` ceiling; the fixture
  is a polygon where the result is exact.
- **Folding coverage into `mask_alpha` also gates effects.** Mitigation:
  documented; matches the observable default more often than not, and the
  advanced toggle is out of scope.
- **ag-psd's non-zero reading is undocumented by Adobe.** Mitigation: it is
  opt-in on the value `2`, which Photoshop does not write; the default is the
  documented even-odd.
- **No Photoshop pixel parity claim.** The oracle proves structure, not Adobe's
  rasterization; the existing per-feature parity caveat is unchanged.

## Migration Plan

None for documents: an existing file is read as before, and a `vmsk` that
previously no-op'd now clips its layer. No golden fixture changes (the new
fixture is additive) and no rollback beyond reverting the commit.

## Open Questions

- **Hide All / InitialFillRule.** Whether an empty-path `vmsk` with
  `InitialFillRule` 0 encodes "Hide All" is unconfirmed; the slice treats an
  empty path as a no-op (Reveal All), which is the safe default. Resolves with a
  CS6 Hide-All byte inspection.
- **`operation` semantics in real Photoshop files.** Whether real shape layers
  use `-1` continuation and how holes are encoded (winding vs operation) is not
  settled; the slice folds all closed subpaths into one fill and defers the
  operation. Resolves with CS6 shape-layer fixtures.
- **`vscg` scope.** Whether a basic shape layer ever needs `vscg` (rather than
  its fill adjustment) is unconfirmed; deferred.
