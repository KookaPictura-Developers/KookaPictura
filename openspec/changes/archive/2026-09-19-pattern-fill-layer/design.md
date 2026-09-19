## Context

A Photoshop pattern fill layer stores a version-16 `DescriptorBlock` under the
per-layer `PtFl` additional-layer-info key, and references a pattern by id. The
pattern pixels are stored separately, in the document's **Patterns** resource:
the global `8BIM` tagged blocks keyed `Patt`, `Pat2`, or `Pat3` in the Layer and
Mask Information section (Adobe File Formats Specification, "Patterns
(Photoshop 6.0 and CS (8.0))", key `Patt`/`Pat2`/`Pat3`; `PtFl` is the
"Pattern fill setting" key, "Adjustment layer" key list). `pictura-codec`
preserves those bytes verbatim in `Document.layer_section_extra`, but there is
no parser; `pictura-render`'s `decode_adjustment` has no `PtFl` arm; and
`ADJUSTMENT_KEYS` does not list `PtFl`, so the block is dropped to
`extra_blocks` rather than `layer.adjustment`. A real pattern fill renders as a
no-op.

### D1 correction: patterns are a tagged block, not image resource 1039

The brief expected the Patterns resource in PSD **image resource 1039** and
asked to parse `Document.image_resources`. That is not where patterns live:

- The Adobe File Formats Specification lists image resource ID 1039 as **ICC
  Profile** (the repo's own `docs/04-image-ops/color-profiles-and-assignment.md`
  and `docs/01-architecture/color-management.md` record the same), and pattern
  data as the `Patt`/`Pat2`/`Pat3` additional-layer-information blocks.
- `psd_tools.psd.image_resources` has no Patterns type and
  `Resource.ICC_PROFILE = 1039`; `psd_tools` reads patterns from
  `TaggedBlocks[Tag.PATTERNS1 | PATTERNS2 | PATTERNS3]`
  (`api/psd_image.py::_get_pattern`), and `TaggedBlocks` is read from the
  Layer and Mask Information section with `padding=4`.

Therefore the decoder reads the global tagged blocks in
`Document.layer_section_extra` — the same bytes the existing
`smart_object::collect_linked_records` already walks — and does **not** add a
parser over `Document.image_resources`. A feature that parsed image resources
would decode nothing from a real Photoshop file.

The pieces that already exist: `pictura-codec`'s public descriptor DOM
(`read_descriptor`/`write_descriptor` and `DescValue::UnitFloat`), its layer
channel RLE/zip decompression, and the `smart_object.rs` precedent for deriving
a typed view from preserved bytes. psd-tools fully models the `Patterns` /
`Pattern` / `VirtualMemoryArrayList` / `VirtualMemoryArray` structures and its
`composite/paint.py::draw_pattern_fill` is a public, testable reference for the
tiling.

## Goals / Non-Goals

**Goals:**

- Recognise `PtFl` as an adjustment key and decode its descriptor into
  `Adjustment::PatternFill(PatternFillParams)`.
- Decode the document's `Patt`/`Pat2`/`Pat3` pattern library into RGBA pixels.
- Tile the referenced pattern over the layer rect so a Photoshop pattern fill
  renders instead of being a no-op, with a missing-pattern placeholder.
- Treat a decodable `PtFl` layer as fill content for the Rasterize subset.
- Add a psd-tools-authored fixture and prove the descriptor and pattern pixels
  survive read, whole-`Document` round-trip, decode, tiling, and rasterize.

**Non-Goals:**

- Authoring a pattern fill layer (menu command, encoder, pattern picker).
- A `.pat` preset library and the pattern presets UI.
- CS6-exact scale resampling, non-RGB pattern modes, 16/32-bit pattern planes,
  and pattern drag/`Snap To Origin` editing.
- A `PatternFill` GPU shader; documents with one keep falling back to CPU.

## Decisions

### D1. Read the Patterns tagged blocks, not image resources

`crates/pictura-codec/src/patterns.rs` walks the global tagged blocks in
`Document.layer_section_extra` exactly as `smart_object::collect_linked_records`
does (`8BIM` signature, 4-char key, `u32` length, `u64` for the PSB big keys,
data padded externally to 4), and for each `Patt`/`Pat2`/`Pat3` parses the
`Patterns` list: repeated `[u32 length + padded pattern body]`, each body
`u32 version == 1`, `u32 image_mode`, `2 × i16` point (vertical, horizontal), a
Unicode name, a Pascal `pattern_id` (ASCII, padding 1), an optional indexed
colour table, and a `VirtualMemoryArrayList` (version 3, length block, `4 × u32`
rectangle, `u32 num_channels`, then `num_channels + 2` `VirtualMemoryArray`
entries). Each `VirtualMemoryArray` is `u32 is_written` (stop when 0),
`u32 length` (stop when 0), `u32 depth`, `4 × u32` rectangle, `u16 pixel_depth`,
`u8 compression`, and `length - 23` data bytes, decompressed with the existing
channel RLE/zip decoders.

The public surface is `decode_patterns(&Document) -> Vec<PatternPixels>` where
`PatternPixels { pattern_id: String, width: u32, height: u32, rgba: Vec<u8> }`
is row-major, 4 components per pixel. Only Grayscale and RGB patterns are
decoded (the first written colour channel is replicated for grayscale; the first
three are R/G/B for RGB); a written channel in the alpha region is transparency
and an absent one is 255. A malformed block stops the walk and keeps what parsed
so far — never a panic. This mirrors the smart-object derivation precedent
(D5 there) and reuses `decode_packbits`/inflate instead of re-implementing
decompression in the render crate.

Alternative considered: expose the decompression helpers and parse Patterns in
`pictura-render`. Rejected — it widens the codec API and moves format parsing
out of the crate that owns the format; the smart-object precedent settles it.

### D2. `PatternFillParams { pattern_id, scale, link_with_layer, origin }`

`PatternFillParams` lives in `pictura-adjust::types` next to
`GradientFillParams`, mirroring its shape: `pattern_id: String`,
`scale: f32` (percent, default 100), `link_with_layer: bool` (default true), and
`origin: (i32, i32)` (default `(0, 0)`). The field set follows the docs model
`pictura_core::fill::PatternFill { pattern_id, scale, link_with_layer, origin }`
(`docs/05-layers/fill-layers.md:166`) and the four controls the CS6 Help names
(pattern, Scale, Link With Layer, Snap To Origin). A fill is not a destructive
adjustment, so `apply` returns `AdjustError::Unsupported`, exactly as for
`SolidFill`/`GradientFill`; the variant only carries the descriptor to the
generative composite.

### D3. Descriptor keys, grounded with psd-tools

`decode_pattern_fill` reads a version-16 descriptor object whose `Ptrn` item is
an object of class `Ptrn` with `Nm  ` (name, ignored) and `Idnt` (the pattern
id, a text string; trailing NULs stripped). The top-level `Scl ` is the scale
percent, read as `DescValue::Double` **or** `DescValue::UnitFloat` (psd-tools
and the layer-style pattern-fill descriptor write `Scl ` as a unit float), and
defaults to 100. The top-level `Algn` bool is `link_with_layer` and defaults to
true. The optional `phase` (`Pnt ` object with `Hrzn`/`Vrtc` doubles) is the
origin in pixels, default `(0, 0)`.

Grounding: psd-tools' `api/adjustments.py::PatternFill` exposes `PtFl`'s `Ptrn`
descriptor as the fill's `data`; its `composite/paint.py::draw_pattern_fill`
docstring gives the pattern-fill descriptor shape
`Ptrn { Nm  , Idnt }, Angl, Scl , Algn, phase { Hrzn, Vrtc }`, and resolves the
pattern with `desc[Ptrn][Idnt].rstrip("\0")`. A psd-tools author probe
(`PatternFill` descriptor `tobytes`/`frombytes`) confirmed the round-trip keys
`Ptrn`/`Scl `/`Algn` and `Ptrn`'s `Nm  `/`Idnt`. There is no CS6 capture in the
repo, so `phase`/origin is decoded as written but marked inferred (D9). A
missing/wrong-typed `Ptrn`/`Idnt`, a non-finite scale, or a parse failure
returns `None`; it never panics.

### D4. Tiling geometry and the pattern origin

`composite_pattern_fill` reuses `blend_into`, like `composite_gradient_fill`:
each in-rect pixel becomes source content at the pattern sample's alpha, so the
layer mask, opacity, fill, and blend still apply, and pixels outside the rect
are untouched. For a layer rect and a pattern of `tw × th` pixels:

- `tw = max(1, round(pattern.width × scale / 100))`, likewise `th`; at
  `scale == 100` the tile is the pattern pixels unchanged.
- The tile anchor is the layer's top-left when `link_with_layer` is set (the
  pattern moves with the layer) and the document origin `(0, 0)` when clear
  (`Snap To Origin`), following `docs/05-layers/fill-layers.md` parity criterion
  3. At `scale == 100` and `link_with_layer` true this is byte-identical to
  psd-tools' `draw_pattern_fill`, which tiles from the viewport top-left.
- The origin offsets the sample: `sx = ((x - anchor_x - origin.0) mod tw)`,
  `sy = ((y - anchor_y - origin.1) mod th)`.
- The tile is sampled nearest-neighbour when `scale != 100` (a stated ceiling,
  D9) and the pattern colour is blended with the pattern's alpha.

### D5. Missing or undecodable pattern falls back to a placeholder

`composite_pattern_fill` looks the `pattern_id` up in the document's decoded
pattern set. When no pattern matches — a missing pattern (the docs'
"Pattern not installed" edge case, `docs/05-layers/fill-layers.md:231`), or one
the codec skipped as non-RGB, non-8-bit, or malformed — it composites an opaque
**50 %-grey placeholder** over the rect instead of a no-op, so the layer is
visible and the file still renders. The docs describe a placeholder-or-last-known
with a warning; this change uses the grey placeholder and the render crate has
no warning surface, so the "and warn" half is not implemented. Both are recorded
as a `ponytail:` ceiling, not an intended parity claim.

### D6. One fill-content predicate

`is_fill_content_layer` becomes `decode_adjustment(..)` matching
`Some(Adjustment::SolidFill(_) | Adjustment::GradientFill(_) | Adjustment::PatternFill(_))`,
and `rasterize_fill_content` bakes the decoded content with the same decoder.
Routing both through `decode_adjustment` means a pattern fill is recognised in
one place and composite and rasterize cannot disagree. `is_fill_content_layer`
takes only the layer (no pattern lookup needed: the descriptor is enough);
`rasterize_fill_content(doc, path)` decodes the document's pattern set once
before taking the mutable layer borrow, then bakes the tiled pattern (or the
placeholder) into the layer's `0/1/2/-1` channels and clears the adjustment. The
behavior requirement lives in `layer-management` (which owns `Rasterize
subset`).

### D7. The document is threaded to the composite

`composite_layer` already receives `&Document`; `composite_adjustment` gains the
document parameter and routes `Adjustment::PatternFill` to
`composite_pattern_fill(canvas, layer, doc, params)`, which decodes the pattern
library once per pattern-fill layer. This is the smallest signature change that
reaches the pixels; a shared `&PatternSet` threaded through the whole composite
is the upgrade path if re-parsing ever shows up.

### D8. The app already routes the Rasterize subset

The Layers-panel and `Layer > Rasterize` actions call
`pictura_render::is_fill_content_layer` and `pictura_render::rasterize_fill_content`
(`crates/pictura-app/cpp/frame_menus.cpp:565`, `.../src/cxxqt_object/impl_layers_rasterize.rs`).
Extending those two functions enables `Layer > Rasterize > Fill Content` for a
pattern-fill layer with **no C++ change**. No authoring command is added (that
would need a pattern preset/library, D10).

### D9. Deferred fidelity, marked as ceilings

`ponytail:` ceilings record: `scale != 100` uses nearest-neighbour rather than
CS6's interpolation; non-RGB pattern modes, indexed colour tables, and 16/32-bit
pattern planes are skipped by the decoder, so the renderer draws the grey
placeholder for them; a pattern-level rectangle that disagrees with its written
channel rectangles (or exceeds the decoder's pixel cap) is skipped the same way;
`phase`/origin is inferred from the layer-style convention and unverified for
`PtFl`; the missing-pattern path has no warning; and the pattern library is
re-decoded per fill layer.

### D10. Out of scope: authoring

`Layer > New Fill Layer > Pattern…`, the Layers-panel fill entry, and a
`PtFl` encoder are deferred. Unlike solid and gradient fills, authoring needs a
pattern preset/library to choose from and cannot default to a single fixed
pattern without inventing one. Recorded as a follow-up change.

### D11. The fixture and its oracle

`scripts/generate-fixtures.py` gains `pattern_fill()`: a document-sized `Base`
pixel layer plus a channel-stripped document-sized `PtFl` layer, and a 2×2
Grayscale/RGB pattern stored in the global `Patt` tagged block
(`Tag.PATTERNS1`). The pattern's channels are the Photoshop slot layout — at
least three written colour slots plus the two mask slots psd-tools'
`get_pattern_color_channels` expects — so both a Rust decoder and psd-tools
read the same RGBA. The descriptor is
`DescriptorBlock(Descriptor({ Ptrn: Descriptor({Nm  , Idnt}, classID=b"Ptrn"),
Scl : UnitFloat(100.0, #Prc), Algn: Bool(True) }, classID=b"PtFl"))`, attached
under `Tag.PATTERN_FILL_SETTING`. Keeping the fill layer's rect document-sized
(not collapsed) lets psd-tools composite the tile.

The codec oracle asserts the `PtFl` block and the `Patt` pattern pixels survive
read and whole-`Document` round-trip; a render test asserts `decode_adjustment`
yields the expected params and that the composite is the exact tiling (verified
against psd-tools' `draw_pattern_fill` output at scale 100). `pictura-codec`
cannot call `decode_adjustment` (it would invert the crate dependency), so the
decode assertion lives in `pictura-render`.

## Risks / Trade-offs

- **The brief named the wrong location (image resource 1039).** Implementing the
  brief literally would decode nothing. Mitigation: D1 grounds the actual
  location in the Adobe spec and psd-tools; the fixture is a real `Patt` tagged
  block, so a wrong location fails the oracle loudly.
- **`PtFl` was never in `ADJUSTMENT_KEYS`.** Without the codec change the block
  never reaches `layer.adjustment`. Mitigation: D1/D3 and the codec task call it
  out; the fixture oracle fails if it is missed.
- **The `phase`/origin key is inferred.** No CS6 capture confirms it for `PtFl`.
  Mitigation: it defaults to `(0, 0)` and only offsets tiling; the fixture omits
  it, and the ceiling is recorded.
- **`Scl ` unit is not fixed.** Photoshop may write `#Prc` or another unit; the
  decoder reads the numeric value regardless of unit, matching psd-tools'
  `desc.get(Key.Scale) / 100`.
- **Scale resampling is not CS6-exact.** A scaled pattern renders
  nearest-neighbour rather than with the document interpolation preference.
  Stated ceiling; the fixture uses scale 100.
- **A wrong-but-parseable descriptor renders a wrong pattern.** Type checks and
  the id lookup bound the damage; the fixture pins the known values.
- **Pattern channel slot layout is quirky.** Photoshop writes fixed colour slots,
  not a packed channel list. Mitigation: D1 follows psd-tools'
  `get_pattern_color_channels` boundary rule; the fixture writes the slot layout
  and both readers verify it.
- **The committed golden fixture is new, not modified.** No existing fixture
  changes, so regeneration cannot churn the existing oracle.
- **A GPU document with a pattern fill decodes further before rejection.** It now
  decodes to `PatternFill` and is still rejected by `adjustment_params` (no
  shader), so the CPU fallback outcome is unchanged.

## Open Questions

- Whether CS6 ever writes a `PtFl` without a `Ptrn`/`Idnt` (an "unset" pattern),
  and what it should render. The decoder treats it as malformed (`None`); a CS6
  capture would settle the fallback.
- Whether CS6's `PtFl` stores `Snap To Origin` as `Algn`, as a separate key, or
  only as the `phase` offset. psd-tools grounds only `Algn`; the decoder reads
  both and defaults them.
- Whether `phase`'s `Hrzn`/`Vrtc` are pixels or normalized units. The decoder
  rounds them to integer pixels, matching the docs' pixel-origin model.
- The resample filter CS6 uses for a non-100 pattern `Scale`, and the indexed/
  CMYK pattern colour conversion. Both await a control capture.
