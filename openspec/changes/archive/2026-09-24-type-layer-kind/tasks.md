# Tasks: type-layer-kind

## 1. Detection and locks

- [x] 1.1 Add `Layer::is_type(&self) -> bool` (or equivalent) true when `extra_block(b"TySh")` is present.
- [x] 1.2 In `crates/pictura-codec/src/read.rs`, after tagged-block parse, if `TySh` is in `extra_blocks`, force `lock` TRANSPARENCY|PIXELS (leave POSITION/NESTING).
- [x] 1.3 Unit test: synthetic write/read or constructed layer — TySh forces locks; without TySh locks unchanged; TySh bytes round-trip.

## 2. Kind string and UI

- [x] 2.1 `layer_kind_str`: after background, if `layer.is_type()` return `"type"`, else `"pixel"`.
- [x] 2.2 `layers_filter_bar`: add `{"type", "Type"}` to `kKinds`; grow `kindButtons_` to 5; map asset `layers.kindType`.
- [x] 2.3 Add `assets/icons/layers.kindType.svg` (T-glyph, stroke style matching other kind icons) and register in `assets/pictura.qrc`.
- [x] 2.4 Confirm filter proxy already matches any KindRole string (it does via `filter.kinds.contains`).

## 3. Tests and gates

- [x] 3.1 Rust unit test: layer with TySh → kind projection is `type` (app or codec-level as appropriate).
- [x] 3.2 Codec test: open→save preserves TySh; locks forced on read.
- [ ] 3.3 Optional self-test code 199: load/open a synthetic type layer or document built in-process if cheap; otherwise skip C++ self-test (Rust coverage is enough for this slice).
- [x] 3.4 `openspec validate type-layer-kind --strict`
- [x] 3.5 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p pictura-codec -p pictura-app` (and C++ self-test if 3.3 added).
