## Why

A Photoshop **pattern fill layer** is a `PtFl` additional-layer-information block
whose descriptor references a document pattern by id; a real file therefore
carries both the `PtFl` fill descriptor (per layer) and a **Patterns** resource
(`Patt`/`Pat2`/`Pat3` global tagged blocks) holding the tiled pixels. Today
`pictura-codec` does not list `PtFl` in `ADJUSTMENT_KEYS`, so the block is
dropped to `extra_blocks`; `decode_adjustment` has no `PtFl` arm; the pattern
pixels are never parsed; and the renderer has no pattern tiling. A Photoshop
pattern fill therefore renders as a no-op. Read support is the last of the three
fill kinds (after `SoCo` and `GdFl`) and is a P3 roadmap gap.

Ground-truth note: the brief expected the patterns in PSD **image resource
1039**. Resource 1039 is the **ICC profile** (Adobe File Formats Specification;
`psd_tools.psd.image_resources` lists `Resource.ICC_PROFILE = 1039`), and the
spec keeps patterns in the `Patt`/`Pat2`/`Pat3` additional-layer-information
tagged blocks — which this repo already preserves verbatim in
`Document.layer_section_extra`. The change parses those blocks, not
`Document.image_resources`; see design D1.

## What Changes

- `pictura-codec` accepts `PtFl` in `ADJUSTMENT_KEYS` (21→22), so a real
  pattern fill block is recognised as an adjustment and preserved in
  `layer.adjustment` instead of being dropped to `extra_blocks`.
- `pictura-codec` gains a minimal **Patterns** decoder: walk the global
  `8BIM` tagged blocks in `Document.layer_section_extra` (the same 4-byte-padded
  form `smart_object::collect_linked_records` reads), parse each
  `Patt`/`Pat2`/`Pat3` `Patterns` list (version, mode, size, Unicode name,
  Pascal id, `VirtualMemoryArrayList` channels), decompress the written 8-bit
  channel planes, and expose `decode_patterns(&Document) -> Vec<PatternPixels>`
  (`pattern_id`, `width`, `height`, row-major RGBA). Malformed blocks are
  skipped; it never panics.
- `pictura-adjust` gains `PatternFillParams { pattern_id, scale,
  link_with_layer, origin }` and the `Adjustment::PatternFill(PatternFillParams)`
  variant. `apply` SHALL refuse it as `AdjustError::Unsupported`, because a fill
  is composited generatively, not applied to the backdrop.
- `pictura-render` gains `decode_pattern_fill(d) -> Option<Adjustment>` next to
  `decode_gradient_fill`, wired into `decode_adjustment`:
  - the top-level `Ptrn` object (class `Ptrn`) with its `Idnt` text (the pattern
    id, trailing NULs stripped) and optional `Nm  ` name;
  - optional `Scl ` (`doub` or `UntF`, percent, default 100) and `Algn`
    (`bool`, link with layer, default true);
  - optional `phase` (`Pnt ` object `Hrzn`/`Vrtc`) as the origin, default
    `(0, 0)`.
  - A missing/wrong-typed `Ptrn`/`Idnt`, a non-finite scale, or a malformed
    descriptor returns `None`; it never panics.
- The renderer resolves the pattern id against the document's decoded pattern
  set, tiles the pattern over the layer rect with `scale` (nearest-neighbour
  resample when not 100), `link_with_layer` (anchor the tile at the layer's
  top-left when set, at the document origin when clear) and `origin`, and
  composites it through the existing mask/opacity/fill/blend path
  (`composite_pattern_fill` alongside `composite_gradient_fill`). A pattern id
  absent from the decoded set, or one the decoder skipped (non-RGB mode,
  non-8-bit plane, malformed rectangle), falls back to an opaque 50 %-grey
  placeholder; the missing-pattern case never crashes.
- `pictura-render`'s fill-content subset broadens to `PtFl`: `is_fill_content_layer`
  accepts a decoded `Adjustment::PatternFill` and `rasterize_fill_content` bakes
  the tiled pattern over the layer rect using the document's pattern set.
- The app needs **no production change**: `Layer > Rasterize > Fill Content`
  already enables and runs through `pictura_render::is_fill_content_layer` /
  `rasterize_fill_content`, which gain `PtFl` here. No pattern-authoring
  command is added.
- `scripts/generate-fixtures.py` gains a `pattern_fill()` builder authoring a
  new `pattern_fill.psd` (a `Base` pixel layer plus a channel-stripped,
  document-sized layer whose tagged-block key is `PtFl`, referencing a 2×2 RGB
  pattern stored as a `Patt` tagged block — matching psd-tools' authoring
  recipe), registered in `FIXTURES`; existing fixtures are unchanged. A codec
  oracle proves the `PtFl` block and the `Patt` pattern pixels survive read and
  whole-`Document` round-trip, and a render test proves decode, tiling, and
  rasterize.
- **BREAKING**: none. Authoring (`Layer > New Fill Layer > Pattern…`) is out of
  scope (it needs a pattern preset/library) and is recorded as a deferred
  follow-up.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `adjustment-layer-rendering`: the committed-decode requirement adds the `PtFl`
  payload; a new requirement covers decoding the document's `Patt` pattern
  library, another covers tiling a pattern fill over the layer rect (with the
  missing-pattern fallback), and another covers fill-content rasterization
  through the shared decoder.
- `image-adjustments`: a new `PatternFill` model requirement (with the
  `Unsupported` refusal); the alpha-preservation and parameter-validation
  requirements account for the variant, and the ImageMagick oracle table gains a
  `PatternFill` no-equivalent row.
- `layer-management`: the `Rasterize subset` requirement broadens the
  fill-content predicate to pattern fills and defines rasterizing them.

## Impact

- `crates/pictura-codec/src/common.rs`: `ADJUSTMENT_KEYS` gains `PtFl` (21→22).
- `crates/pictura-codec/src/patterns.rs` (new) and `src/lib.rs`: the Patterns
  block walker, channel decompression, `PatternPixels`, and `decode_patterns`.
- `crates/pictura-adjust/src/types.rs`, `src/apply.rs`, `src/lib.rs`: the
  `PatternFillParams` model, the `Unsupported` refusal, and the exports.
- `crates/pictura-adjust/src/tests.rs` and `tests/oracle.rs`: the alpha case and
  one no-equivalent oracle row (`MAPPING.len()` 18, `NO_EQUIVALENT` 15).
- `crates/pictura-render/src/fill.rs`: `decode_pattern_fill`, the tiling
  helpers, the placeholder, and `composite_pattern_fill`.
- `crates/pictura-render/src/composite.rs`: the `PtFl` match arm,
  `composite_adjustment` threading the document, and the doc comment.
- `crates/pictura-render/src/lib.rs`: re-exports (`PatternFillParams`).
- `crates/pictura-render/src/document_ops/layer_ops/rasterize.rs`: the
  generalized fill-content predicate and pattern rasterization.
- `crates/pictura-render/src/tests/fill.rs` (or `.../tests/adjustment.rs`) and
  `.../tests/rasterize.rs`: decode, tiling, fallback, and rasterize cases.
- `scripts/generate-fixtures.py`,
  `crates/pictura-codec/tests/fixtures/pattern_fill.psd`, `.../tests/oracle.rs`,
  and `.../tests/fixtures/README.md`: the `pattern_fill()` builder, the new
  golden fixture, and its oracle.
- No new dependency.
- GPU path unchanged: `gpu/mod.rs::adjustment_params` has no `PatternFill`
  shader, so a document containing one keeps falling back to the CPU composite.

## Out of scope (deferred)

- **Pattern fill authoring** (`Layer > New Fill Layer > Pattern…`, the Layers
  panel fill entry, and an encoder for the `PtFl` descriptor). It needs a
  pattern preset/library and a pattern picker; recorded as a follow-up.
- **Pattern presets and the `.pat` library** (load/save/define/reset, pattern
  picker UI). Out of scope for read support.
- **Pattern scale resampling fidelity.** Scale 100 is exact; other scales use a
  nearest-neighbour approximation, not CS6's interpolation preference.
- **`Snap To Origin` as a separate toggle and pattern drag editing.** Only the
  stored `Algn`/`phase` values drive anchor and offset; there is no editing UI.
- **Non-RGB pattern modes** (indexed/CMYK/Lab/duotone) and indexed colour
  tables: decoded only for Grayscale/RGB patterns; others fall back to the
  placeholder.
- **16/32-bit pattern planes** and pattern transparency edge cases beyond a
  single trailing alpha channel.
- **A `PatternFill` GPU shader.** Documents with one keep falling back to CPU.
