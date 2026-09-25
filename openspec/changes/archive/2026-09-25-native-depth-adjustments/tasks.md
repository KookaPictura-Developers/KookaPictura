# Tasks: native-depth-adjustments

## 1. Sample abstraction

- [x] 1.1 Add a `Sample` trait to `crates/pictura-core/src/samples.rs` with `to_unit(self) -> f64` and `from_unit(v: f64) -> Self` impls for `u8`, `u16`, `f32` (f32 clamps to `[0,1]`, NaN to 0); unit-test the u16/f32 conversions.
- [x] 1.2 Add a generic planar color-plane iterator/map over `Samples` (3 or 4 channels, alpha untouched) that the native kernels use.

## 2. Native tonal application

- [x] 2.1 `crates/pictura-adjust/src/native.rs` (new): `apply_native(adjustment, samples, width, height, channels) -> Result<(), AdjustError>` covering `Invert`, `Desaturate`, `Levels`, `Curves`, `BrightnessContrast`, `Exposure`, `Posterize`, `Threshold`, `GradientMap`, mirroring the 8-bit formulas in unit domain; everything else `Unsupported` with no mutation.
- [x] 2.2 `lib.rs`: export `apply_native`.
- [x] 2.3 Keep `apply` and all existing kernels byte-identical (no edits to their math).

## 3. Verification

- [x] 3.1 `pictura-adjust` test: for each covered adjustment, native-u8 `apply_native` equals the 8-bit `apply` byte-for-byte on a fixed image.
- [x] 3.2 `pictura-adjust` test: a depth-16 store edited through `apply_native` yields `low != high` for at least one sample (not widen-on-edit); a depth-32 store keeps values above `1.0`.
- [x] 3.3 `pictura-adjust` test: `Unsupported` for `HueSaturation` on a native store leaves the samples unchanged.
- [x] 3.4 `pictura-render` codec e2e test: read `rgb16.psd`, apply a native tonal adjustment to `document.source_planes`, set `document.composite` to the narrowing, `write_psd`, re-read, and assert the retained samples equal the adjusted ones and are not `high * 257`.
- [x] 3.5 `cargo nextest run -p pictura-adjust -p pictura-render -p pictura-codec -p pictura-core`, `cargo test -p pictura-adjust --test oracle`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/check-file-size.sh`, `openspec validate native-depth-adjustments --strict`.
