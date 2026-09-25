# Proposal: native-depth-save

## Why

The depth epic now composites, adjusts, reads layer content, and gates masks at
native precision, and `composite_native` emits the source-depth composite — but
the application never consumes it. On save, an edited 16/32-bit document still
writes the widened 8-bit `doc.composite` for its composite color planes, so all
that native precision is discarded at the last step. This wires the native
composite into the app's save path.

## What Changes

- On saving a **dirty** 16/32-bit RGB/Grayscale document that has layers, the app
  SHALL recompute the composite with `pictura_render::composite_native`, store it
  as the working composite (mirroring `store_composite`'s plane-count logic for
  the displayed/composite buffer), and replace the retained composite **color**
  planes in `Document.source_planes` with the native ones, so `write_psd`
  re-emits native samples instead of widening.
- The refresh SHALL NOT run for: a document with no layers (the merged composite
  is authoritative), a converted color mode (`source_mode` is `Some`, whose
  retained planes are source-mode, not working RGB), a non-16/32-bit document,
  or a clean (non-dirty) document. Those paths SHALL be byte-identical to today.
- Alpha and non-composite extra channels SHALL be left untouched.

## Capabilities

### New Capabilities

- `native-depth-save`: consuming the native composite at save time.

### Modified Capabilities

<!-- None. psd-bit-depth's writer rules are unchanged; this changes what the app
     hands the writer. -->

## Impact

- `crates/pictura-render/src/composite_native.rs` (or a small helper):
  `refresh_native_composite(doc) -> bool` that updates `doc.source_planes` color
  planes + `doc.composite` from `composite_native`, gated as above.
- `crates/pictura-app/src/cxxqt_object/impl_core.rs`: `save` calls the refresh
  when the view is dirty, before `write_psd`.
- Tests: a render/codec test that an edited stacked 16-bit doc saves at 16 with
  native (non-widened) composite samples; an app test that a dirty save writes
  depth 16 and a clean open→save is byte-identical.
- No new dependency.
