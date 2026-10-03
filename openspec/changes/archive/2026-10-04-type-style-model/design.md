# Design

## Context

See `proposal.md` for motivation. The relevant current state:

- `pictura-core/src/type_tool.rs` holds `TypeSpec` (what a commit produces) and
  `TextStyle` (the read view). Both are narrow; only `TypeTool`'s raw
  `text_desc`/`warp_desc` bytes are lossless, and only on an unmodified save.
- `pictura-codec/src/type_write.rs` builds `EngineData` from scratch with a
  `Dump` helper; every attribute the model lacks is a literal in `style_sheet`
  (line 295) / `paragraph_sheet` (line 239). `/AntiAlias` is written at
  EngineDict level (line 202) and the method is also written to the text
  descriptor's `AntA` enum (lines 102-111).
- `pictura-codec/src/engine_data.rs` parses `EngineData` into an `EngineValue`
  tree (`parse_engine_data`, line 31) and nothing more. There is no
  `EngineValue` encoder — the parser is read-only, and `String` carries no
  encoding tag, so a re-encoder cannot distinguish UTF-16BE from MacRoman and
  there is no MacRoman *encode* table.
- `pictura-render/src/type_layer.rs` is where a type layer is actually edited:
  `replace_type_layer` (line 450) → `type_content` (line 323) →
  `author_type_tool(spec, bounds)` rebuilds everything; `type_layer_spec`
  (line 413) is the only read→`TypeSpec` path and currently keeps only
  font/size/colour/justification and hardcodes anti-aliasing on. The old
  descriptor is discarded.
- `pictura-app` crosses the model through the flat cxx-qt `TypeSetting` struct
  (`cxxqt_object/type_tools.rs:34`), which also carries placement
  (`x, y, xx, xy, yx, yy`), and the type options bar / `tool_type.cpp` session.

## Goals / Non-Goals

**Goals:**

- One character + paragraph attribute model shared by the codec author, the codec
  reader, `pictura-render`, and the app bridge.
- A re-authored type block that no longer destroys the EngineData keys it does
  not model, for the single-run text the model represents.
- Named character/paragraph styles with the CS6 resolution order, stored on the
  document and undoable.

**Non-Goals:**

- Faux Bold/Italic, tsume, type language, vertical underline variants, and
  paragraph direction. Their PSD keys or a preference surface do not exist; they
  are deferred to a preferences/type-tools follow-up.
- Panel widgets (`character-panel` #73, `paragraph-panel` #74,
  `glyphs-panel` #75, `paragraph-styles-panel` #76) — separate changes.
- Serializing the style definitions to PSD. Where the style block lives is
  unresolved; an existing block is preserved verbatim.
- Multi-run / multi-paragraph editing. The model is first-run only; a multi-run
  block collapses on re-author (documented ceiling).
- A new `pictura-text` crate (the panel spec's proposal). Not adopted.

## Decisions

### Model lives in `pictura-core`, serialization in `pictura-codec`

Add `CharacterAttrs`, `ParagraphAttrs`, `TextStyleSheet`, and the enums
(`AntiAlias`, `KerningMode`, `Leading`, `Justify`, `Composer`, `Direction`) in
`pictura-core/src/type_tool.rs`, deriving `Default` from the documented CS6
defaults. `TypeSpec` gains `character` and `paragraph` fields plus a
`TypeSpec::new` constructor; the six existing struct literals
(`cxxqt_object/type_tools.rs:130`, `type_layer.rs:256/326/416`,
`type_caret.rs:78`, `type_write.rs:442`) migrate to it. `TextStyle` is
re-expressed over the same structs so read and write agree.

*Alternative — a new `pictura-text` crate:* rejected. AGENTS.md rule 4 is no new
dependencies; a new in-repo crate is also unjustified for what fits the existing
split, and the crate boundary is its own architectural change.

### Anti-aliasing: the descriptor `AntA` is canonical

`TypeSpec.antialias: bool` becomes `AntiAlias { None, Sharp, Crisp, Strong,
Smooth }`. Author to **both** the `TySh` text descriptor's `AntA` enum and the
EngineData `/AntiAlias` flag; read `AntA` first, fall back to the flag. The
current code emits only `antiAliasNone`/`antiAliasSharp`; the other three
spellings and the `/AntiAlias` integer width are **inferred** and get a dedicated
validation task against a CS6-authored PSD (rule 5). `render_type`
(`type_layer.rs:242`) must switch on the enum; Crisp/Strong/Smooth are
approximated as Sharp for rasterization with a `ponytail:` ceiling.

### EngineData encoder, and preservation scoped to the existing block

Add `pictura-codec::encode_engine_data(&EngineValue) -> Vec<u8>`. It emits every
string as UTF-16BE with a BOM: psd-tools and Photoshop only accept the BOM form,
and the round-trip oracle rejected the earlier bare-MacRoman choice. It needs
only *semantic* equality, not byte equality: open→save byte preservation still
uses `extra_blocks` verbatim, so lossless open→save is untouched. Known lossy
edges (unpaired surrogates via `from_utf16_lossy`, key-order and number-format
drift) are acceptable because only re-set blocks are re-encoded; list them in the
encoder's doc comment.

`author_type_tool` gains an optional existing-descriptor parameter, and
`type_content`/`replace_type_layer` pass the layer's current `text_desc` through.
The author parses that EngineData and overwrites only the modelled keys of the
single style run and paragraph sheet; every other key of the block survives.
With no existing block it emits the from-scratch skeleton. A block with more than
one style run collapses to the single modelled run — a `ponytail:` ceiling named
in the code and the spec.

### The app bridge carries placement separately

Keep `TypeSetting` for placement and orientation (`x, y, xx, xy, yx, yy`,
`vertical`), and add `CharacterSetting` + `ParagraphSetting` (enums as `i32`,
toggles as `bool`). Affected bridge functions: `type_preview_rect`,
`type_caret_stops`, `type_preview_rgba`, `type_commit_layer`, `type_commit_mask`,
`type_layer_setting`, `type_commit_edit`, `type_update_layer`; affected C++:
`tool_type.cpp`, `options_bar_type.cpp` (its anti-aliasing combo grows from
None/Sharp to five), `image_view_type.cpp`, and `tool_context.h` (`TypeOptions`
anti-aliasing bool → enum). cxx-qt codegen is regenerated by the build.

### Styles: storage, identity, undo

`TextStyleSheet` lives on `Document` (defaulted to a `Basic Paragraph` entry). A
style is identified by its name and holds **sparse** per-field overrides
(`CharacterOverrides`/`ParagraphOverrides`), not complete attribute sets, so a
character style only shadows the paragraph style's character defaults for the
fields it actually sets. Resolution is per-attribute:
`manual run override > applied character style > applied paragraph style > Basic Paragraph > default`.

A layer stores its applied character/paragraph style names and its manual
`overrides` **in memory on `Layer`** (not serialized to PSD), and `TypeSpec`
carries them across `replace_type_layer`. The bridge edit path records a manual
override for every field a real edit changes (diffed against the layer's current
values, not against defaults), so `apply_type_style` genuinely preserves a user's
manual edits. Editing a named style re-resolves every layer that applies it
(`history::re_resolve_layers`), which is how "editing a style updates all text
using it" holds. Creating, editing, deleting, and applying a style each record
one history state through the existing `Snapshot { doc, selection }` model.

### Rendering coverage

`render_type` now applies the character-level attributes: auto/fixed leading,
tracking plus manual kerning, horizontal and vertical scale, baseline shift, and
alignment. Paragraph layout (indents, space before/after, hanging punctuation,
hyphenation, the word/letter/glyph spacing triplets, composer, and the
justify-last/all variants) is authored and round-tripped but **not laid out**;
this is a `ponytail:` ceiling in `text_render.rs::layout_type`, to be lifted when
the type engine gains a paragraph composer.

## Key mapping

Grounded against `type_write.rs` unless marked *(inferred)* or *(no key)*.

| Attribute | EngineData key(s) |
|---|---|
| font family/style | `/FontSet` entry + `/Font` index |
| size | `/FontSize` |
| leading | `/Leading` + `/AutoLeading` (note: also a paragraph float) |
| tracking | `/Tracking` |
| kerning | `/Kerning` + `/AutoKerning` (manual only; Optical removed — no groundable key, deferred) |
| H/V scale | `/HorizontalScale`, `/VerticalScale` |
| baseline shift | `/BaselineShift` |
| anti-aliasing | descriptor `AntA` + EngineDict `/AntiAlias` *(integers inferred)* |
| colour | `/FillColor /Values` |
| caps / super-sub / underline / strike | `/FontCaps`, `/FontBaseline`, `/Underline`, `/Strikethrough` |
| faux bold/italic | *(no key — deferred)* |
| fractional widths | `/UseFractionalGlyphWidths` (EngineDict level) |
| alignment / justification | `/Justification` + paragraph `/Properties` |
| indents | `/FirstLineIndent`, `/StartIndent`, `/EndIndent` |
| paragraph spacing | `/SpaceBefore`, `/SpaceAfter` |
| word/letter/glyph spacing | `/WordSpacing`, `/LetterSpacing`, `/GlyphSpacing` (min/desired/max) |
| hyphenation | `/AutoHyphenate` (+ `/HyphenatedWordSize`, `/PreHyphen`, `/PostHyphen`, `/ConsecutiveHyphens`, `/Zone`) |
| hanging punctuation | `/Hanging` |
| leading type / composer | `/LeadingType` (preserved, not modelled), `/EveryLineComposer` |

Already written but unmodelled and therefore left untouched by the merge:
`/Ligatures`, `/StyleRunAlignment`, `/NoBreak`, `/FillFlag`, `/StrokeFlag`,
`/Burasagari`, `/KinsokuOrder`, `/GridInfo`, `/Adjustments`.

## Risks / Trade-offs

- [`AntA` spellings and `/AntiAlias` integers are inferred] → a self-skipping
  validation test is in place, but **no CS6-authored fixture exists, so parity is
  not proven**; the mapping is isolated in the codec so correcting it is local.
- [The EngineData encoder is lossy for exotic strings] → semantic-only equality
  is the contract; open→save byte preservation is unaffected.
- [Multi-run and multi-paragraph text collapse on re-set] → named in the spec and
  code as ceilings.
- [Paragraph layout attributes are authored but not rendered] → explicit
  `ponytail:` ceiling in `text_render.rs`; only character attributes affect pixels.
- [`pictura-render` is a new touch point in this change] → its `type_layer_spec`
  and `type_content` changes are called out as their own tasks so the read→write
  round trip is real, not assumed.
- [File size] → `engine_data.rs`, `type_write.rs`, and `history.rs` are near their
  caps; tests were split into submodules where needed.

## Open Questions

- The CS6 default anti-aliasing preset and default auto-leading percentage are
  unstated in the fetched Help text (also open in
  `docs/02-ui-ux/panels/character-and-paragraph.md`). Both are defaults, not
  behavior; the model uses Sharp and 120 percent and they are corrected when a
  CS6 capture surfaces. These do not change the specs or tasks.
