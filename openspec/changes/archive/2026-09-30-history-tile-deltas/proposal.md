# Proposal

## Why

Every undo state keeps a copy-on-write clone of the whole document. A clone is a
refcount bump for planes the edit did not touch, but a paint commit forks the
target layer's plane, so each state pins a full plane — 61 MiB on a 4000²
document. Twenty states of a 4000² document retain roughly 1.2 GB, and undo
history grows with the document size rather than with what was painted.

## What Changes

- **A state is stored as its changed tiles, not a whole document.** The oldest
  state stays materialized as the base; each later state stores the 64×64 tiles
  whose bytes differ from the previous state — for the layer channel planes, the
  composite, and the layer masks — plus its selection and non-pixel metadata.
- **Retention scales with the edited area.** Twenty small paint commits retain
  their changed tiles and one materialized base, not twenty full planes.
- **Undo/redo/jump materialize byte-identical states.** Navigation applies the
  stored before/after tiles incrementally, or rebuilds from the nearest full
  anchor when the chain is broken.
- **A geometry or structure change stores that state in full.** Resize, crop,
  flatten, and any edit that changes the tracked-plane set or strides start a new
  full anchor, so tile deltas only ever span a stable geometry.
- The public `History` API and the 20-state / 10-snapshot bounds are unchanged.

## Capabilities

### New Capabilities

- `document/history-retention`: history stores states as region deltas and
  materializes them byte-identically.

### Modified Capabilities

(none)

## Impact

- `crates/pictura-app/src/history.rs` — the delta model, tile diff/apply, the
  geometry-anchor fallback, and unit tests. No new dependencies; no core pixel
  API change (`Plane` stays a contiguous `Arc<[T]>`).
- `openspec/specs/document/edit-history/spec.md` behavior is preserved: labels,
  cursor movement, depth bound, redo truncation, and restored bytes are unchanged.
