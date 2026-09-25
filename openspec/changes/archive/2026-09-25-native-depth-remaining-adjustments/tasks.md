# Tasks: native-depth-remaining-adjustments

## 1. Native kernels

- [x] 1.1 Add `ColorLookup` to `apply_native` (`native.rs`): parse the 3-D LUT, trilinear-sample each pixel's unit color via `lut::sample`, write back to the store. An unparseable/abstract/device-link payload is a no-op (mirroring the 8-bit path).
- [x] 1.2 Add `Auto` (`Tone`/`Contrast`/`Color`) to `apply_native`: build the histogram at the store's resolution, derive percentile bounds and stretch in native units, and run the neutral-midtone step in unit space; mirror `auto.rs`'s clip fractions (0.001 / 0.005 / 0.005) and the `64..=192` midtone window.
- [x] 1.3 Remove `Auto` and `ColorLookup` from the `Unsupported` arm; the fill kinds remain. Do not edit `auto.rs`, `lut.rs`, `color.rs`, `tonal.rs`, or the 8-bit `apply`.

## 2. Verification

- [x] 2.1 Extend the u8-equality test in `crates/pictura-adjust/tests/native_depth.rs` to `Auto(Tone)`, `Auto(Contrast)`, `Auto(Color)`, and a parsed non-identity `ColorLookup`.
- [x] 2.2 Add a depth-16 precision test: a non-identity `ColorLookup` on a depth-16 store yields a sample that is not the 8-bit widening.
- [x] 2.3 Repoint/keep the `Unsupported` test at a fill kind with no mutation.
- [x] 2.4 `cargo nextest run -p pictura-adjust -p pictura-core`, `cargo test -p pictura-adjust --test oracle`, `cargo test -p pictura-adjust --test native_depth`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate native-depth-remaining-adjustments --strict`.
