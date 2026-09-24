# Design: tysh-model-roundtrip

## Context

`type-layer-kind` forces locks and reports kind from `TySh` presence. The block lives in `Layer.extra_blocks` and is re-emitted on save. `vector_mask` and `smart_object` show the pattern: derived view on `Layer`, raw bytes stay authoritative.

Grounding (Adobe File Formats + psd-tools 1.19 `TypeToolObjectSetting`):

```
u16  version        (= 1)
6×f64 transform     (xx, xy, yx, yy, tx, ty)
u16  text_version   (= 50)
…    text descriptor (u32 version 16 + DescriptorBlock)
u16  warp_version   (= 1)
…    warp descriptor (u32 version 16 + DescriptorBlock)
4×i32 left, top, right, bottom   (psd-tools `4i`; Adobe table says "4 * 8" — ceiling)
```

The text descriptor carries `Txt ` (TEXT unicode), enums (`Annt`, `Ornt`, …), and often an `EngineData` blob (Adobe Engine Data markup). Our descriptor DOM already keeps unmodeled ostypes as `DescValue::Raw` and re-emits them.

## Goals / Non-Goals

**Goals:**

- Typed `TypeTool` view: transform, text string, bounds, optional anti-alias/orientation enums if easy from the text descriptor.
- Decode from `TySh` without failing the document on malformed input.
- `encode_type_tool` that rebuilds a well-formed block from the view.
- Unmodified open→save still bit-matches (extra_blocks path).
- Tests with a hand-built or psd-tools-authored synthetic `TySh`.

**Non-Goals:**

- Parsing EngineData style runs / fonts; glyph rasterization; Type tool; Character panel; editing `Txt ` in the UI; document-level `Txt2`.

## Decisions

### D1. Derived view, not a new serialization path

`Layer.type_tool: Option<TypeTool>` is filled in `read` like `vector_mask`. `write_psd` continues to emit `extra_blocks` only. Round-trip of an *unmodified* document never calls the encoder.

### D2. Text string from `Txt ` in the text descriptor

`DescValue::Object` item key `b"Txt "` → `DescValue::Text`. Missing/malformed → empty string (view still holds transform/bounds). Newlines: keep descriptor form (`\r` as stored); do not normalize.

### D3. Encoder layout follows psd-tools

`encode_type_tool`: `version=1`, transform, `text_version=50`, `write_descriptor(text)`, `warp_version=1`, `write_descriptor(warp)`, four `i32` bounds. The text/warp descriptors stored on the view are `DescValue::Object` (parsed once). If a descriptor fails to parse on read, the whole `TypeTool` is `None` (do not half-model).

### D4. Bounds width: `i32` (psd-tools)

Adobe’s “4 * 8” is ambiguous; psd-tools and our fixture writer use `4i`. Mark `ponytail:` if a real CS6 file disagrees. Fixture proves our layout against psd-tools read of our bytes when the oracle runs.

### D5. No EngineData parse

`EngineData` stays inside the text descriptor as `Raw` (ostype + bytes) or whatever the DOM already stores; we only surface `Txt `. Ceiling: style runs and font names are not modeled.

## Risks / Trade-offs

- [Descriptor padding / key order differs from Photoshop] → Unmodified save uses raw `extra_blocks`; encoder is for *edited* views and is proven by decode→encode→decode field equality, not Photoshop bit-identity.
- [Bounds i32 vs f64] → Documented ceiling; switch constant if a real fixture disagrees.
- [No real TySh fixture in-tree] → Synthetic block + psd-tools oracle when present.

## Open Questions

- None blocking. Live text render remains a later change.
