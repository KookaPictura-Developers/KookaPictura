# Tasks: image-resize-bicubic-kernel

## 1. Kernel

- [x] 1.1 In `crates/pictura-ops/src/resize.rs`, set the Keys cubic coefficient to `a = -0.75` (Mitchell–Netravali `cubic(0, 0.75)`) and update the `cubic`/module doc comments to name the Photoshop Bicubic publicly documenting, not Catmull-Rom.

## 2. Oracle

- [x] 2.1 In `scripts/ops_oracle.py`, allow the `resize` op to prepend `--im-args` (shlex-split) before `-filter`, so ImageMagick defines can be passed.
- [x] 2.2 In `crates/pictura-ops/tests/oracle.rs`, change the Bicubic mapping row and `resize_bicubic_matches_imagemagick` to `-filter cubic` with `--im-args "-define filter:b=0 -define filter:c=0.75"`, recording the measured max delta and setting the tolerance accordingly.

## 3. Verification

- [x] 3.1 `cargo test -p pictura-ops --test oracle` green (or self-skips without `magick`).
- [x] 3.2 `cargo nextest run --workspace` green; confirm no PSD oracle (`depth_oracle`, `color_mode_oracle`, `document_oracle`) regressed; if a Bicubic golden changed, regenerate it and state so.
- [x] 3.3 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `bash scripts/check-file-size.sh`, `openspec validate image-resize-bicubic-kernel --strict`.
