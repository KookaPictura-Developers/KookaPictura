# Tasks: blend-if-render

## 1. Model

- [x] 1.1 Add `BlendIf::is_default()` in `crates/pictura-core/src/advanced_blending.rs`: true when the composite ranges are full (`(0, 65535)`) and every channel group is full.

## 2. CPU compositor

- [x] 2.1 Add `blend_if_factor(view: Option<&BlendIf>, cs: [f32; 3], backdrop: [f32; 3]) -> f32` in `crates/pictura-render/src/composite.rs` (composite + per-channel gates; product; inactive range = 1).
- [x] 2.2 Apply it in `blend_into`: read the backdrop channels from `canvas.px[i]`, multiply the effective alpha; early-return 1.0 for absent/default so the hot path is unchanged.

## 3. GPU decline

- [x] 3.1 Add `GpuError::UnsupportedAdvancedBlending` with a `Display` arm in `crates/pictura-render/src/gpu/mod.rs`.
- [x] 3.2 In `walk`, return it for any layer (recursively) whose `blend_if` is present and not default.

## 4. Tests and gates

- [x] 4.1 Unit tests in `crates/pictura-render/src/tests/composite.rs`: absent/default → 1; source gate hides below black and keeps above; dest gate; per-channel gate; extra channel group ignored.
- [x] 4.2 A composite-level test proving a non-default source gate changes the output and a default one does not.
- [x] 4.3 `cargo nextest run -p pictura-render -p pictura-core`, `cargo fmt --all --check`, `cargo clippy -p pictura-render -p pictura-core --all-targets -- -D warnings`, `openspec validate blend-if-render --strict`.
- [x] 4.4 Confirm the GPU parity suites still pass (no non-default range in their fixtures).
