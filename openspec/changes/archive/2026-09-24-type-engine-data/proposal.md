# Proposal: type-engine-data

## Why

Roadmap P3: the `TySh` framing and `Txt ` string decode (`type-layer-kind`,
`tysh-model-roundtrip`), but the text **style** lives in an opaque EngineData
blob inside the `Txt ` descriptor and is not decoded. Without it the engine
cannot expose the font, size, or fill colour a renderer needs.

## What Changes

- **Decode EngineData** — the `EngineData` value of the `Txt ` descriptor
  (descriptor type `tdta`, already surfaced as `DescValue::Raw`) — into a typed
  tree, with bounded token/depth/size caps and no entity or external access.
- **Typed text style on `TypeTool`**: the font-set names and the first style
  run's effective font, size, fill colour, tracking, and the paragraph
  justification, derived from the EngineData tree.
- **Malformed EngineData degrades**: a parse failure leaves the style unset and
  never fails the document read; the raw `TySh` bytes still round-trip.
- **No rasterization yet.** Glyph rasterization and a Rasterize Type command are
  the follow-up; this change is the deterministic, oracle-checkable decode the
  renderer consumes.
- **BREAKING**: none.

## Capabilities

### New Capabilities

- `type-engine-data`: decode a type layer's EngineData into a typed tree and
  expose the font set and first-run style on `TypeTool`.

## Impact

- `crates/pictura-core`: `TextStyle`, `TypeTool.fonts` / `TypeTool.style`.
- `crates/pictura-codec`: `engine_data.rs` parser + extraction; `type_tool.rs`
  fills the new fields.
- New fixture `tests/fixtures/engine_data.bin` extracted from a reference file
  text layer, with a `psd-tools` differential oracle.
- No new dependency; no app change.
