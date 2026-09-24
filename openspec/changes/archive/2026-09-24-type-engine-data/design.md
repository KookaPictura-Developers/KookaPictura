# Design: type-engine-data

## Context

`decode_type_tool` reads the `TySh` framing and the `Txt ` descriptor. The
descriptor's `EngineData` item is descriptor type `tdta` (`OSType.RAW_DATA`),
which `descriptor.rs` already maps to `DescValue::Raw(Vec<u8>)` — the raw
EngineData byte stream. psd-tools 1.19 parses the same bytes with
`psd_tools.psd.engine_data`.

EngineData grammar (from psd-tools `engine_data.py`):

- `<<` … `>>` a dict; `/Name value` a property (name MacRoman, ASCII in
  practice).
- `[` … `]` a list.
- values: integers `-?\d+`, decimals `-?\d*\.\d+`, `true`/`false`, and
  parenthesised strings `(…)` whose payload is UTF-16BE when it begins with the
  BOM `\xfe\xff`, with `\(`, `\)`, `\\` escapes; unknown parenthesised tags are
  treated as strings.
- whitespace (space/newline/tab) separates tokens.

A real text layer's structure (grounded on a reference build
`the synthetic source`, extracted to the fixture):

```
EngineDict/Editor/Text              -> "hello world\r"
EngineDict/StyleRun/RunArray[0]/StyleSheet/StyleSheetData
                                    -> {FontSize 150.0, FillColor.Values [1,1,1,1], …}
ResourceDict/StyleSheetSet[0]/StyleSheetData -> {Font 1, FontSize 12.0, …}  (defaults)
ResourceDict/FontSet                -> [AdobeInvisFont, MyriadPro-Regular]
EngineDict/ParagraphRun/RunArray[0]/ParagraphSheet/Properties/Justification -> 0
```

An omitted run field falls back to the `StyleSheetSet` default; the font index
resolves into the `FontSet`.

## Goals / Non-Goals

**Goals:**

- A bounded EngineData parser producing a typed tree.
- `TypeTool.fonts` (font-set names) and `TypeTool.style` (first run's effective
  values).
- A `psd-tools` differential oracle on the real fixture.
- Unmodified documents stay byte-identical (no serialization change).

**Non-Goals:**

- Glyph rasterization, layout, per-run styles beyond the first run, or text
  editing.
- Adobe font parity (the font is a name; no font file is resolved).
- EngineData2 (`TEXT_ENGINE_DATA` tagged block) — decode only the `Txt `
  descriptor blob.

## Decisions

### D1. Parser shape

`engine_data.rs`: `parse_engine_data(&[u8]) -> Result<EngineValue, PsdError>`
with `EngineValue::{ Dict(Vec<(String, EngineValue)>), List(Vec<EngineValue>),
Int(i64), Double(f64), Bool(bool), String(String) }`. A recursive-descent parser
over a token slice, capped at 64 depth, 100 000 tokens, and 16 MiB. Duplicate
keys are kept in order (a dict is a list of pairs); lookup takes the first match.

### D2. Caps and strictness

A malformed or truncated blob returns `Err(PsdError::Invalid)`; the caller
(`decode_type_tool`) maps that to `style: None` / `fonts: vec![]` and still
returns the `TypeTool` view. No panics.

### D3. Extraction rules

- `fonts`: walk the first `FontSet` list found under `ResourceDict` (else
  `DocumentResources`), taking each entry's `Name` string.
- `style.font`: the run's `StyleSheetData.Font` index when present, else the
  `StyleSheetSet[0].StyleSheetData.Font` default, resolved through `fonts`
  (`None` if the index is out of range).
- `style.font_size`: the run's `FontSize`, else the default's.
- `style.fill_color`: the run's `FillColor.Values` (four `f64`), else the
  default's, else `[0,0,0,1]`.
- `style.tracking`: the run's `Tracking`, else the default's, else `0`.
- `style.justification`: `ParagraphRun/RunArray[0]/ParagraphSheet/Properties/
  Justification`, else `0`.
- Lookup is a small helper that walks a `Dict` by key path; a missing key or a
  wrong type yields the fallback.

### D4. No raw-byte change

`decode_type_tool` already preserves `text_desc`/`warp_desc` verbatim; the new
fields are derived. `encode_type_tool` is unchanged, so an open→save round-trip
is byte-identical.

## Risks / Trade-offs

- [Field-fallback ambiguity] → the fixture's run omits `Font`, exercising the
  default fallback; the oracle checks the resolved font is `MyriadPro-Regular`.
- [Grammar edge cases] → unknown parenthesised tags decode as strings; a blob
  the parser rejects leaves the style unset rather than failing the document.
- [CC-only fixture] → the only real EngineData available is a reference build;
  EngineData is stable across CS6→CC, recorded in the fixture provenance.
