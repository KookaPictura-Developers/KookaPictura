# Proposal: tysh-model-roundtrip

## Why

`type-layer-kind` detects `TySh` and reports kind `type`, but the block is still opaque: the text string, affine transform, and bounds are not available to the engine, so Type cannot round-trip an edited string and the Layers/canvas path has no typed view. This is the next P3 text step after kind detection.

## What Changes

- Parse a layer's preserved `TySh` into a derived `TypeTool` view on `Layer` (same pattern as `vector_mask` / `smart_object`): version-1 framing, `6×f64` transform, text descriptor (including `Txt ` string), warp descriptor, and `left/top/right/bottom`.
- The raw `TySh` bytes stay in `Layer.extra_blocks` and remain the write source of truth for an unmodified open→save.
- Expose `encode_type_tool(&TypeTool) -> Vec<u8>` so an edited view can replace the preserved block (text/transform/bounds changes); re-encoding an unchanged view SHALL match the framed structure psd-tools writes (descriptor padding ceilings as needed).
- `Layer::type_tool(): Option<&TypeTool>` derived on read when the block parses; malformed `TySh` leaves the view `None` without failing the document.
- No font stack, no glyph rasterization, no Type tool UI, no EngineData markup parse (EngineData stays a `DescValue::Raw` / opaque blob inside the text descriptor).
- **BREAKING**: none for files without `TySh`; open→save of an unmodified type layer still re-emits the original bytes.

## Capabilities

### New Capabilities

- `psd-type-tool`: decode and encode the `TySh` type-tool object setting into a typed view; unmodified round-trip via preserved bytes.

### Modified Capabilities

<!-- None: type-layer-kind detection stays presence-only. -->

## Impact

- `crates/pictura-core`: `TypeTool` struct + `Layer.type_tool`.
- `crates/pictura-codec`: `type_tool.rs` decode/encode; wire resolve after `vector_mask` in `read.rs`.
- Tests: synthetic TySh (hand-built or psd-tools `TypeToolObjectSetting`) — text string, transform, bounds, open→save bytes, encode round-trip.
- No new dependency; no app UI change required for this slice.
