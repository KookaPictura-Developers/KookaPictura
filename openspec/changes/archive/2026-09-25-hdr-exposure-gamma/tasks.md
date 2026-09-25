# Tasks: hdr-exposure-gamma

## 1. Operator

- [x] 1.1 Add `crates/pictura-adjust/src/hdr_toning.rs` with `pub struct ExposureGamma { pub exposure_ev: f64, pub gamma: f64 }` (and a `Default` of `0.0`/`1.0`) and `pub fn exposure_gamma(samples: &[f32], params: ExposureGamma) -> Result<Vec<f32>, AdjustError>` implementing `sign(x·gain)·|x·gain|^(1/gamma)` with `gain = 2^exposure_ev`; validate finite `exposure_ev` and finite, positive `gamma`.
- [x] 1.2 Re-export `exposure_gamma` and `ExposureGamma` from `crates/pictura-adjust/src/lib.rs`.

## 2. Tests

- [x] 2.1 Identity at `0.0`/`1.0`; the documented formula on a ramp including `2.0` (no pre-clamp); `0.0 → 0.0`; a negative input stays negative and non-NaN.
- [x] 2.2 Rejection: `gamma` of `0`, negative, NaN; `exposure_ev` NaN/infinite → `InvalidParams`, no panic.

## 3. Verification

- [x] 3.1 `cargo nextest run -p pictura-adjust`; `cargo nextest run --workspace`.
- [x] 3.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate hdr-exposure-gamma --strict`.
