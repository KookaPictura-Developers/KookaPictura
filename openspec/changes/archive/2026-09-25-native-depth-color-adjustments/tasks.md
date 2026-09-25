# Tasks: native-depth-color-adjustments

## 1. Native color kernels

- [x] 1.1 In `crates/pictura-adjust/src/native.rs`, add unit-domain kernels for `HueSaturation`, `Vibrance`, `ColorBalance`, `BlackWhite`, `PhotoFilter`, `ChannelMixer`, `SelectiveColor`, mirroring `color.rs` (reuse `rgb_to_hsl`/`hsl_to_rgb`/`luma`/`skin_bump`). `ColorBalance` works in the 0–255 domain: scale by 255 in and /255 out.
- [x] 1.2 Extend the `apply_native` match with these variants; leave every other `Adjustment` on the `Unsupported` arm with no mutation.
- [x] 1.3 Do not edit `color.rs` or its kernels.

## 2. Verification

- [x] 2.1 Extend `crates/pictura-adjust/tests/native_depth.rs`'s u8-equality test to cover each new variant (and useful parameter axes: `preserve_luminosity` on/off, selective-color methods, a non-identity mixer).
- [x] 2.2 Add a depth-16 precision test for at least one color adjustment (e.g. `Vibrance` or `HueSaturation`) asserting a low byte != high byte.
- [x] 2.3 Change the unsupported test to use `Auto` (or `ColorLookup`) and assert no mutation.
- [x] 2.4 `cargo nextest run -p pictura-adjust -p pictura-core`, `cargo test -p pictura-adjust --test oracle`, `cargo test -p pictura-adjust --test native_depth`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate native-depth-color-adjustments --strict`.
