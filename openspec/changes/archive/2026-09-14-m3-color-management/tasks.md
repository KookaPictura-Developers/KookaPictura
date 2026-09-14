## 1. Crate scaffold

- [x] 1.1 Create `crates/pictura-color` with `thiserror` and `lcms2 = "6"` dependencies
- [x] 1.2 Add `pictura-testkit` as a dev-dependency for byte-diff comparison
- [x] 1.3 Add the crate to the workspace members

## 2. Profiles

- [x] 2.1 Define `Intent` and its mapping to `lcms2::Intent`
- [x] 2.2 Define `ColorError` (`InvalidProfile`, `Unsupported`) with `thiserror`
- [x] 2.3 Implement `Profile::srgb` via `lcms2::Profile::new_srgb`
- [x] 2.4 Implement `Profile::adobe_rgb` from D65 white, Adobe primaries, gamma 563/256
- [x] 2.5 Implement the ROMM tone curve (linear toe below 1/32, gamma 1.8) as a tabulated `ToneCurve`
- [x] 2.6 Implement `Profile::pro_photo` from D50 white, ROMM primaries, ROMM curve
- [x] 2.7 Implement `Profile::from_icc` (malformed → `InvalidProfile`)
- [x] 2.8 Implement `Profile::to_icc` without panicking

## 3. Convert and assign

- [x] 3.1 Map `(channels, bits)` to lcms2 pixel formats for 1/3/4-channel at 8/16-bit
- [x] 3.2 Validate input length with checked arithmetic and reject mismatches as `Unsupported`
- [x] 3.3 Build the transform with intent + optional black point compensation + copy-alpha flags
- [x] 3.4 Run `Transform<u8, u8>` over the interleaved buffer and return the output
- [x] 3.5 Implement `assign` as a retag returning pixels plus the new profile

## 4. Unit tests

- [x] 4.1 Identity conversion for 8-bit RGB within ±1 LSB
- [x] 4.2 Identity conversion for 16-bit RGB within ±1 LSB
- [x] 4.3 sRGB → Adobe RGB → sRGB round trip within tolerance
- [x] 4.4 Every rendering intent produces non-degenerate output
- [x] 4.5 Gray identity through a generated gray profile
- [x] 4.6 RGBA alpha channel is preserved through a conversion
- [x] 4.7 Malformed ICC bytes return errors and never panic
- [x] 4.8 ICC serialize/reload round trip for all built-ins
- [x] 4.9 `assign` returns byte-identical pixels and an RGB profile

## 5. ImageMagick oracle

- [x] 5.1 Write `scripts/color_oracle.py` with `version` and `convert` subcommands
- [x] 5.2 Compare `convert` against `magick … -profile src -profile dst` at 3 LSB tolerance
- [x] 5.3 Add the engine-independent sRGB → Adobe RGB primaries known-value test
- [x] 5.4 Skip oracle tests cleanly when `magick` or system ICC profiles are absent
- [x] 5.5 Document flags, tolerances, and the measured divergence in `tests/README.md`
- [x] 5.6 Record the matrix/TRC intent/BPC limitation in `tests/README.md`

## 6. Validation

- [x] 6.1 `cargo fmt --all` clean
- [x] 6.2 `cargo clippy --all-targets -- -D warnings` clean
- [x] 6.3 `cargo test -p pictura-color` green (oracle tests skip without ImageMagick)
- [x] 6.4 `openspec validate m3-color-management --strict` passes
