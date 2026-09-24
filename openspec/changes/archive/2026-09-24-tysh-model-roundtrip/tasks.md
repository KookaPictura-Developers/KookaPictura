# Tasks: tysh-model-roundtrip

## 1. Model

- [x] 1.1 Add `pictura_core::TypeTool { transform: [f64; 6], text: String, bounds: [i32; 4], text_desc: Vec<u8>, warp_desc: Vec<u8> }` where `*_desc` are the full version-16 descriptor byte slices from the input (framing-only re-encode; EngineData stays opaque inside those bytes).
- [x] 1.2 Add `Layer.type_tool: Option<TypeTool>` default `None`.

## 2. Codec decode/encode

- [x] 2.1 `crates/pictura-codec/src/type_tool.rs`: `decode_type_tool` / `encode_type_tool` (version 1, 6×f64, text_version 50, descriptors, warp_version 1, warp descriptor, 4×i32 bounds; extract `Txt ` from the text descriptor).
- [x] 2.2 Wire `resolve_type_tools` in `read.rs` after `resolve_vector_masks`; malformed → view `None`.
- [x] 2.3 Module registration in `pictura-codec` `lib.rs`.

## 3. Tests and gates

- [x] 3.1 Synthetic TySh decodes transform, `Txt ` text, bounds.
- [x] 3.2 Open→save preserves `TySh` bytes (extra_blocks path).
- [x] 3.3 encode_type_tool → decode_type_tool preserves transform/text/bounds.
- [x] 3.4 Malformed TySh → `type_tool()` is `None`, document still reads.
- [x] 3.5 `cargo nextest run -p pictura-codec`, `cargo fmt --all --check`, `cargo clippy -p pictura-core -p pictura-codec --all-targets -- -D warnings`, `openspec validate tysh-model-roundtrip --strict`.
