# Design

## Context

See proposal.md — Why.

Two measurements bound the problem. `stroke_split_profile_4000` (release,
4000², recorded in `docs/dev/canvas-view-spec.md` § 3.4) puts `Stroke::begin_at`
at 81 ms with one layer and 360 ms with eight, against 9.9 ms of rasterization
and 1.6 ms of present for a 500 px dab — the clone is now the largest single
cost in a stroke. And `History::capture` stores `Document`s directly, so the
`ponytail:` note at `crates/pictura-app/src/history.rs:15` ("full-document
clones; COW or tile diffs if PSB-size docs hit RAM") is the ceiling this change
is named for.

The stroke path reads only what it writes. `composite_region_active` composites
`doc.layers`, never `doc.composite`, and `composite_pixel` writes exactly one
layer's four planes — so while a stroke runs, every plane but the painted
layer's is provably unchanged, and every one of those copies is pure overhead.

## Goals / Non-Goals

**Goals:**

- `Document::clone` costs a refcount bump for the common 8-bit document.
- A write forks only the plane it writes; the pre-write snapshot survives
  untouched, so undo stays bit-identical.
- No change to any byte the engine produces, on any code path.

**Non-Goals:**

- Tiling, sparse allocation, or region-delta history (the deferred
  `tiled-layer-history`): those need content versioning and a tile identity this
  change does not introduce.
- Converting `RawChannel`, `AdjustmentData`, `LayerBlock`, `SourcePlanes` or
  `SourceChannels`. They are verbatim re-emit stores, absent from an 8-bit
  document, and never on the stroke path.
- Replacing the `Vec` behind a plane with a `SmallVec`/bump arena: `Arc` is std
  and is the whole point.

## Decisions

**D1 — A `Plane<T>` newtype over `Arc<[T]>`, not a bare `Arc`.** A bare
`Arc<[u8]>` breaks `assert_eq!(plane, vec![…])` (std implements no
`PartialEq<Vec<_>>` for `Arc<[_]>`), of which the workspace has ~50, and a bare
`Arc<Vec<u8>>` additionally changes `Option::as_deref` from `&[u8]` to `&Vec<u8>`,
which 22 mask sites rely on. `Plane` costs ~40 lines in its own module and buys
`Deref<Target = [T]>`, `DerefMut`, `From<Vec<T>>`, `AsRef<[T]>` and six
`PartialEq` impls, so construction is one `.into()` and every existing
assertion reads unchanged. *Alternatives considered:* bare `Arc<[u8]>` plus
edits at each comparison — more diff for no gain; `Arc<Vec<u8>>` — keeps `push`
and `truncate` but silently changes `as_deref` semantics.

**D2 — `DerefMut` goes through `Arc::make_mut`.** A plane that is uniquely held
keeps its allocation and every in-place write behaves exactly as it does today;
a shared plane forks once on first write and the sharer keeps the old bytes.
This is what makes the change behavior-preserving rather than merely
probably-correct: the aliasing question is answered by `Arc`'s own contract
instead of by an audit.

**D3 — Three fields, not every `Vec<u8>`.** `Channel::data`,
`PixelBuffer::data` and `LayerMask::data` carry the pixels; at 4000² they are
16 MB and 64 MB apiece. The verbatim stores (D's non-goals) are one to two
orders of magnitude smaller on the documents that matter and are re-emitted
unaltered, so sharing them buys nothing and costs conversions in the codec's
inner loops.

**D4 — Convert with the compiler, not with a regex.** Every site is found by
`cargo check --workspace --all-targets` after the three field types change:
the `E0308` list *is* the work list. No textual rewrite of `.data` across the
workspace, because `data` is also the name of a dozen unrelated fields.

## Risks / Trade-offs

- [`Arc::make_mut` on a shared plane copies where the old code wrote in place]
  → Only a plane that is still shared writes that way, and sharing exists only
  between a document and a snapshot that must not see the write. When a plane is
  unique — every steady-state edit — `make_mut` returns the buffer directly.
- [A holder of a raw `as_ptr` into a plane observes the fork] → Audit the `as_ptr`
  / `copy_from_slice` FFI sites for a pointer that outlives a clone-plus-write;
  if one exists it must re-read the pointer after the write, as it already must
  across a `Vec` reallocation.
- [A hot loop calls a slice method that now goes through `DerefMut`] →
  `make_mut` is a refcount load and a branch once the plane is unique; hoist it
  to a `&mut [u8]` binding where a loop touches the same plane repeatedly.
- [The diff is wide (~500 construction sites across eight crates)] → Mitigated
  by D4: the compiler enumerates every site, `cargo nextest run --workspace`
  (1805 tests) plus the 511-check self-test and the codec/GPU oracles are the
  acceptance gate, and no golden baseline may move.
- [16/32-bit documents still deep-copy `SourcePlanes`] → Accepted and recorded;
  add it to `Plane` only if a profile of a 16-bit stroke shows it.

## Open Questions

None. Tiling and region-delta history are explicitly out of scope and stay in
their own change; nothing here changes a spec, the approach, or the task
breakdown.
