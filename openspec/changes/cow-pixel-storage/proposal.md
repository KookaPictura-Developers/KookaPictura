# Proposal

## Why

Starting a brush stroke costs two deep `Document` clones and committing one
costs a third; on a 4000×4000 document the measured release-build figures are
81 ms with one layer and 360 ms with eight, and they scale with the layer count,
not with what the stroke touches (`stroke_split_profile_4000`, recorded in
`docs/dev/canvas-view-spec.md` § 3.4). At 16000² a single clone is a gigabyte.
The history stack keeps up to 21 of those documents, so a session grows toward
several gigabytes of pixel data that mostly still agrees with itself. Every one
of those copies exists only because `Channel::data`, `PixelBuffer::data` and
`LayerMask::data` are `Vec`s.

## What Changes

- **Pixel planes become refcounted and copy-on-write.** A new `pictura_core::Plane<T>`
  wraps `Arc<[T]>`: cloning a plane is a refcount bump, and a write forks only
  the plane being written (`Arc::make_mut`), never the document.
- **The three plane-bearing fields convert:** `Channel::data`,
  `PixelBuffer::data` (the composite and every image buffer) and
  `LayerMask::data`. `Document::clone` — and therefore `Stroke::begin_at`,
  `PictureView::snapshot`, and every `History` undo/redo/jump — becomes a
  refcount bump instead of a copy.
- **Construction and comparison sites follow.** Struct literals take
  `Plane::from(vec)` and the existing `assert_eq!(plane, vec![…])` comparisons
  keep compiling through `Plane`'s `PartialEq` impls, so test intent is
  unchanged.
- **Out of scope:** tiling (64×64/128×128) and region-delta history — they need
  a content-versioning design this change does not introduce. `RawChannel`,
  `AdjustmentData`, `LayerBlock` and the retained `SourcePlanes`/`SourceChannels`
  keep their `Vec`s: they are verbatim re-emit stores, absent from an 8-bit
  document, and out of the stroke path.
- **BREAKING**: none observable. Sharing is invisible to callers: `Arc::make_mut`
  preserves in-place mutation when a plane is unique, and a write through one
  `Document` never reaches another. Every existing golden, oracle and self-test
  must still pass byte-identically.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none — see `.openspec.yaml`, which sets `skip_specs: true`).

No requirement changes. `document/document-model` constrains layer ordering,
identities and serialization; `document/edit-history` requires snapshots that
restore bit-identically and a depth of 20. Copy-on-write planes satisfy both
unchanged — the pixels a restore produces are the same bytes either way. The
diff is a storage and latency change, and OpenSpec asks for a delta only when
observable behavior moves, so inventing a requirement here would be padding.

## Impact

- `crates/pictura-core/src/plane.rs` (new) — the `Plane<T>` type and its tests.
- `crates/pictura-core/src/lib.rs` — `PixelBuffer::data`, `Channel::data`,
  `LayerMask::data`.
- Every construction site for those three fields across `pictura-codec`,
  `pictura-render`, `pictura-paint`, `pictura-select`, `pictura-adjust`,
  `pictura-ops`, `pictura-app`.
- `crates/pictura-app/src/history.rs` — the `ponytail:` note about
  full-document clones is retired.
- No new dependencies (`std::sync::Arc` only). PSD read/write, the codec oracles
  and the GPU parity suite are the sharpest edges: they index planes directly.
