# Tasks: psd-tagged-block-8b64

## 1. Reader

- [x] 1.1 In `crates/pictura-codec/src/read.rs`, accept `tag_sig == b"8BIM" || tag_sig == b"8B64"` for per-layer additional-layer-info blocks; a signature that is neither still returns `PsdError::Invalid`.
- [x] 1.2 Add a `ponytail:` note that the signature is normalized to `8BIM` on write (lossless re-emit needs a model field).

## 2. Tests

- [x] 2.1 A synthetic layer record with an `8B64` tagged block reads successfully and its key/payload are in `extra_blocks`.
- [x] 2.2 A tagged-block signature of neither `8BIM` nor `8B64` still errors.
- [x] 2.3 Existing `8BIM` layer round-trip tests are unchanged.

## 3. Gates

- [x] 3.1 `cargo nextest run -p pictura-codec`, `cargo fmt --all --check`, `cargo clippy -p pictura-codec --all-targets -- -D warnings`, `openspec validate psd-tagged-block-8b64 --strict`.
