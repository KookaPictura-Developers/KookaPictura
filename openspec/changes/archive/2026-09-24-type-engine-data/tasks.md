# Tasks: type-engine-data

## 1. Model

- [x] 1.1 Add `pictura_core::TextStyle { font: Option<String>, font_size: f64, fill_color: [f64;4], tracking: f64, justification: u8 }` (Default) and export it.
- [x] 1.2 Add `TypeTool.fonts: Vec<String>` and `TypeTool.style: Option<TextStyle>`; extend the manual `PartialEq`/`Eq`.

## 2. Parser

- [x] 2.1 `crates/pictura-codec/src/engine_data.rs`: `EngineValue`, `parse_engine_data`, tokenizer (dict/list/number/bool/string/property; UTF-16BE strings with escapes; MacRoman names), and caps (depth 64, 100k tokens, 16 MiB).
- [x] 2.2 Extraction: `fonts` from the first `FontSet`; `style` from the run/first-default/paragraph paths (design D3).
- [x] 2.3 `decode_type_tool`: read `EngineData` (`DescValue::Raw`) from the text descriptor, parse, fill `fonts`/`style`; parse failure → empty/unset, never a document error.
- [x] 2.4 Export `parse_engine_data`/`EngineValue` from `pictura-codec` `lib.rs`.

## 3. Fixture and oracle

- [x] 3.1 The fixture `tests/fixtures/engine_data.bin` (extracted from the a reference build `the synthetic source` text layer, re-serialized by psd-tools) already exists; record provenance in `tests/fixtures/README.md`.
- [x] 3.2 Oracle test (`tests/oracle/type_engine_data.rs`): parse the fixture with `parse_engine_data` and assert `fonts` = `[AdobeInvisFont, MyriadPro-Regular]`, style font `MyriadPro-Regular`, size `150.0`, fill colour `[1,1,1,1]`, justification `0`; a python `psd-tools` cross-check of the same fixture asserts the same values.

## 4. Tests and gates

- [x] 4.1 Unit tests: token types, UTF-16BE + escapes, nested dict/list, malformed/truncated/over-depth errors; a synthetic `TySh` with a raw `EngineData` fills `TypeTool.style`; no `EngineData` → `style` None.
- [x] 4.2 `cargo nextest run -p pictura-codec -p pictura-core`, `cargo fmt --all --check`, `cargo clippy -p pictura-codec -p pictura-core --all-targets -- -D warnings`, `openspec validate type-engine-data --strict`.
