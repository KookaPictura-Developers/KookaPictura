# Design

## Context

See proposal.md — Why. `crates/pictura-app/src/history.rs` holds a bounded stack
of `Snapshot { doc: Document, selection }`. `Document` and `Plane` are
copy-on-write (`Arc<[T]>`), so an untouched plane is shared between states, but
a paint commit forks the edited plane and pins a whole one per state. The core
pixel API must not change: `Plane` stays a contiguous `Arc<[T]>` deref'd to
`[T]`, and `PixelBuffer`/`Layer`/`Channel` stay contiguous.

## Goals / Non-Goals

**Goals:**

- Retained undo memory scales with the changed area, not the document area.
- `undo`/`redo`/`jump` return a `Snapshot` byte-identical to the state captured
  under the previous implementation.
- The public `History` API shape and the 20-state / 10-snapshot bounds are
  unchanged.

**Non-Goals:**

- Tiling the core `Plane` type, or changing `PixelBuffer`/`Layer`/`Channel`.
- Compressing the per-state tile lists, or bounding the named-snapshot store
  (it stays a full snapshot, capped at 10).
- A GPU or interop history path.

## Decisions

**D1 — States after the oldest are region deltas.** The oldest state is a full
materialized base. Each later state stores a `Delta`: the 64×64 tiles that differ
from the previous state for the tracked planes (composite, document channels,
layer channel planes, layer masks), its `Selection`, and its non-pixel metadata.
A tile is `(plane identity, tile rect, before bytes, after bytes)`.

**D2 — Tracked planes are enumerated by a fixed traversal.** A document is walked
in a fixed order (composite, document channels, then layers depth-first: channels,
mask, children). A plane's identity is its index in that walk, with its row
stride and height. The traversal depends on the document structure and vector
lengths, not on plane data, so a cleared or refilled plane enumerates to the same
index. Plane rows are addressed with the stored stride, so a plane that does not
fill its rect still tiles correctly.

**D3 — A state materializes by adopting metadata and writing tiles.** Metadata is
carried as the state's `Document` with every tracked plane cleared. Materializing
copies the running state's tracked planes into that metadata shell and writes the
stored after-tiles (or before-tiles when stepping backward). Because the delta
stores every changed byte, the result equals the captured state exactly.

**D4 — A geometry or structure change is a full anchor.** If the tracked-plane
count, stride, or height differs from the previous state (resize, crop, flatten,
add/remove/reorder a layer, add a mask), `capture` stores a full `Snapshot` and
that state becomes an anchor. Materialization starts from the nearest anchor at
or before the target, and `drop_oldest` promotes state 1 to an anchor after
materializing it, so the chain stays valid as the depth bound evicts states.

**D5 — Navigation is incremental with an anchor fallback.** `redo`/forward
applies after-tiles to the materialized current state; `undo`/backward applies
before-tiles and the previous state's metadata. Crossing a full anchor backward
rebuilds from that anchor. The named-snapshot store is untouched.

## Risks / Trade-offs

- [A sprite sheet of before-bytes doubles the tile payload] → Still O(changed
  area); before-bytes are what makes backward navigation in-place rather than a
  full rebuild. Drop them if only the after direction is ever needed.
- [Planes the task does not track (retained source-depth samples, undecoded raw
  channels) are carried whole in each state's metadata] → They are not the paint
  path and stay correct; only the tracked planes are deltaed.
- [An edit that changes few bytes still byte-compares the whole plane] → At
  capture only, not per dab, and only for planes whose `Arc` is no longer shared;
  a cheap pointer check skips untouched planes.
- [A geometry change retains one full snapshot] → Geometry changes are rare
  user commands, not paint commits; the retained set stays one anchor per change.

## Open Questions

None outstanding. Compressing tiles and bounding named snapshots remain later
work if measurements call for them.
