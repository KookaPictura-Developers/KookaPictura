## Why

Opening a real PSD and saving it silently discards everything the engine does
not model: the image-resource and color-mode-data sections are skipped on read
and written back as zero, unknown per-layer tagged blocks (layer effects, smart
objects, text, vector masks, gradient/pattern fills, blend-if, knockout) are
dropped, the original blend key of an unknown mode is lost, layer blending
ranges are re-emitted empty, layer-mask parameter bytes are dropped, and
unmodeled layer channels (e.g. the `-3` real user mask) are discarded. This is a
data-loss hazard worse than refusing to open: a profile, metadata, effect, or
smart object is gone after the first save. Roadmap phase P2
(`docs/dev/psd-support-roadmap.md`, gaps G3/G5/G6/G7).

## What Changes

- The document model gains opaque preservation storage: `Document` keeps the raw
  color-mode-data and image-resource sections, the global layer mask block, and
  the trailing global additional-layer information; `Layer` keeps its original
  blend key, raw blending ranges, unknown tagged blocks, and unmodeled channels;
  `LayerMask` keeps its trailing parameter bytes.
- `read_psd` captures those bytes instead of skipping or dropping them; it no
  longer discards the `-3` (real user mask) channel or unknown tagged blocks.
- `write_psd` re-emits them verbatim, so an open→save round-trip preserves every
  block the engine does not interpret. A known blend key is not stored (the mode
  already round-trips), so a constructed document can still equal a read one.
- Engine-created documents have empty preservation storage, so `write_psd`
  output for them is unchanged (the byte-layout golden still holds).
- **BREAKING**: none at the API-consumer level; the model structs gain fields
  (constructors/literals use `Default`).

## Capabilities

### New Capabilities

- `psd-opaque-preservation`: the guarantee that unmodeled PSD blocks survive an
  open→save round-trip, and the enumeration of what is preserved.

### Modified Capabilities

- `psd-codec`: the composite-read requirement no longer skips the color-mode and
  image-resource sections but captures and preserves them.
- `psd-layer-io`: unknown tagged blocks, the original blend key, blending
  ranges, mask parameter bytes, and unmodeled layer channels are preserved
  through read/write.

## Impact

- `crates/pictura-core/src/lib.rs`: new `Document`/`Layer`/`LayerMask` fields,
  `LayerBlock`/`RawChannel` types, `Default` impls; every struct literal in the
  workspace gains `..Default::default()`.
- `crates/pictura-codec/src/{read,write,common}.rs`: capture and re-emit the
  preserved blocks.
- `crates/pictura-codec/src/tests.rs`, `tests/oracle.rs`: round-trip tests,
  including a psd-tools-authored fixture with resources/effects re-emitted
  through write and re-read.
- No render or app UI change; the new fields are inert there.
