## 1. Crate and types

- [x] 1.1 Create `crates/pictura-adjust` with `pictura-core` and `thiserror` dependencies and `pictura-testkit` as a dev-dependency
- [x] 1.2 Define `AdjustError { Unsupported, InvalidParams }` and the `Adjustment` enum with all 15 variants
- [x] 1.3 Define the parameter structs (`LevelsParams`, `CurvesParams`, `BrightnessContrastParams`, `ExposureParams`, `HueSaturationParams`, `BlackWhiteParams`, `PhotoFilterParams`, `ChannelMixerParams`, `VibranceParams`, `ColorBalanceParams`) and `AutoKind`
- [x] 1.4 Implement `apply` dispatch plus `validate` for channel count, non-empty pixels, and data length
- [x] 1.5 Add planar plane splitting (`planes_mut`), LUT mapping (`map_lut`), and direct mapping (`map_float`) helpers

## 2. Tonal adjustments

- [x] 2.1 Implement Levels with input/output remap and gamma, rejecting gamma <= 0 and input black >= input white
- [x] 2.2 Implement Curves with Fritsch-Carlson monotone Hermite interpolation and 2..=14 point validation
- [x] 2.3 Implement Brightness/Contrast in legacy and modern modes with range validation
- [x] 2.4 Implement Exposure in sRGB-decoded linear light with EV gain, offset, and gamma
- [x] 2.5 Implement Invert as `255 - v` per color channel
- [x] 2.6 Implement Posterize with uniform quantization, the 255-level identity, and idempotence
- [x] 2.7 Implement Threshold with Rec.601 luminance binarization and equality falling to black
- [x] 2.8 Implement Desaturate as `(min + max) / 2` written to all three channels

## 3. Color adjustments

- [x] 3.1 Implement HSL conversion helpers used by the vector adjustments
- [x] 3.2 Implement Hue/Saturation in HSL space (composite Master path only)
- [x] 3.3 Implement Black & White with hue-sector decomposition, default weights, and tint
- [x] 3.4 Implement Photo Filter as a density-weighted lerp with optional luminosity preservation
- [x] 3.5 Implement Channel Mixer as a percent-weight matrix with constant offset and monochrome collapse
- [x] 3.6 Implement Vibrance with a `1 - S` falloff and skin-hue damping
- [x] 3.7 Implement Color Balance with overlapping tonal-band windows and optional luminosity preservation

## 4. Auto corrections

- [x] 4.1 Implement histogram percentile bounds and the linear stretch LUT
- [x] 4.2 Implement Auto Tone as a per-channel stretch at 0.1 percent clipping
- [x] 4.3 Implement Auto Contrast as a joint three-plane stretch at 0.5 percent clipping
- [x] 4.4 Implement Auto Color as the per-channel stretch plus midtone neutralization

## 5. Contract and invariant tests

- [x] 5.1 Unit-test each adjustment's identity, known values, and validation path
- [x] 5.2 Test that all 15 variants leave channel 4 bit-identical on an RGBA buffer
- [x] 5.3 Test that empty, wrong-channel, and short buffers return errors without panicking
- [x] 5.4 Test deterministic output across repeated runs for Hue/Saturation and Auto Color
- [x] 5.5 Test Desaturate against Hue/Saturation saturation -100 within 1 LSB

## 6. ImageMagick oracle script

- [x] 6.1 Add `scripts/adjust_oracle.py` with `version` and `apply` subcommands and interleaved/planar raw I/O
- [x] 6.2 Expose named operators (levels, gamma, brightness-contrast, negate, posterize, threshold, gray, desaturate, modulate, color-matrix, evaluate) and verbatim `--im-args`
- [x] 6.3 Pass percentages to `-level`, `+level`, `-threshold`, and additive `-evaluate` so Q16 builds read 8-bit values correctly

## 7. Differential and property tests

- [x] 7.1 Add `crates/pictura-adjust/tests/oracle.rs` with the 8x8 RGB test image and planar round-trip
- [x] 7.2 Diff Levels against `-level ... +level ...` within tolerance 1
- [x] 7.3 Diff Invert against `-negate` within tolerance 0
- [x] 7.4 Diff Desaturate against `-modulate 100,0,100` within tolerance 1
- [x] 7.5 Add the 15-row mapping table and assert the twelve no-equivalent adjustments use tolerance 0
- [x] 7.6 Add property tests for Brightness/Contrast, Hue/Saturation, Channel Mixer, Posterize, and Threshold
- [x] 7.7 Skip differential tests with a message when `magick` is absent; add no `#[ignore]`
- [x] 7.8 Document the mapping, exact flags, divergences, and the verified ImageMagick version in `tests/README.md`

## 8. Verification

- [x] 8.1 `cargo test -p pictura-adjust` is green with 34 unit tests and 12 oracle tests
- [x] 8.2 `cargo clippy --all-targets -- -D warnings` is clean for the crate
- [x] 8.3 `scripts/guard.sh` is green
