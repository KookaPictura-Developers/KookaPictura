# Proposal: type-layer-kind

## Why

Roadmap P3 remaining kind: text (`TySh`). Type layers currently open as `"pixel"` because `layer_kind` never inspects the preserved `TySh` block, so the Layers panel cannot show or filter Type, and CS6's default type locks are not enforced when the file omits them. This is the smallest step that makes type layers first-class without a font engine.

## What Changes

- A layer whose `extra_blocks` contain key `TySh` is a **type** layer: `layer_kind` / `layer_row_kind` / `layer_kind_str` return `"type"` (after group/adjustment/background, before pixel).
- On codec read, a layer with `TySh` forces Lock Transparency and Lock Image (`LockFlags::TRANSPARENCY | PIXELS`) to match CS6 defaults when those bits are clear.
- Layers panel Kind filter gains a Type toggle (`"type"` / `"Type"`); the filter predicate accepts `type` alongside pixel/adjustment/group/background.
- `TySh` bytes stay in `extra_blocks` and re-emit on open→save (already true; covered by a round-trip test). No text decoding, no font stack, no live re-render from TySh.
- **BREAKING**: none for documents without `TySh`; kind strings for type layers change from `"pixel"` to `"type"` (consumers that only expected the four old kinds must tolerate `"type"`).

## Capabilities

### New Capabilities

- `type-layer-kind`: detection of type layers from `TySh`, the `"type"` kind string, forced type locks, and opaque round-trip of the block.

### Modified Capabilities

- `layers-filtering-search`: Kind filter selectable kinds gain `type`.
- `layers-panel`: row kind vocabulary includes `type` where it lists pixel/group/adjustment/background.

## Impact

- `crates/pictura-app/.../helpers.rs`: `layer_kind_str` branch.
- `crates/pictura-codec/src/read.rs`: force type locks when `TySh` is present.
- `crates/pictura-app/cpp/panels/layers_filter_bar.cpp`: Type kind option (+ icon if a Type icon exists).
- `crates/pictura-app/src/.../helpers.rs` `filter_from_kind`: map `"type"`.
- Unit tests (synthetic layer with `TySh`); optional self-test code 199.
- No new dependency; no `pictura-render` composite change (render stays on raster channels).
