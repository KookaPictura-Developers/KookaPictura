## 1. M10-A1 — `pictura-ops` crate and image resize

- [ ] 1.1 Add `crates/pictura-ops` to the workspace with `Cargo.toml` (depends on `pictura-core` only), `src/lib.rs` re-exporting the public functions, and the `Resample::{Nearest, Bilinear, Bicubic}` and `OpsError::InvalidParams` types
- [ ] 1.2 Implement `resize` as per-channel resampling that returns a new `PixelBuffer`, leaves the input bit-identical, and validates `width`/`height` >= 1 and `data.len() == width * height * channels` with `OpsError::InvalidParams`
- [ ] 1.3 Implement the `Nearest` sampler (nearest source sample), the `Bilinear` 2×2 tent blend, and the `Bicubic` 4×4 Keys/Catmull-Rom convolution, all with clamp-to-edge source coordinates
- [ ] 1.4 Unit-test `resize`: the output dimensions and channel count match the request, the input is unchanged, and `width`/`height` 0 and a malformed buffer return `InvalidParams` without panicking
- [ ] 1.5 Unit-test the kernels: `Nearest` on a two-color image emits only the two source colors, `Bilinear`/`Bicubic` produce monotone intermediates on a gradient, and `Bicubic` does not panic or read out of bounds on 1-pixel and 1×1 inputs
- [ ] 1.6 Unit-test alpha: a 4-channel buffer resamples channel 4 with the same kernel and keeps the 4-channel planar layout

## 2. M10-A2 — Canvas operations

- [ ] 2.1 Implement `resize_canvas` returning a new `PixelBuffer`, validating `width`/`height` >= 1 and the input buffer, with the same new-buffer and untouched-input contract as `resize`
- [ ] 2.2 Add the nine-variant `Anchor` enum (`TopLeft` … `BottomRight`) and one placement helper shared by grow and shrink, so the anchor offsets the source in the destination and crops to the anchor-aligned rectangle identically
- [ ] 2.3 Implement grow: allocate the destination filled with `background` (alpha from `background[3]` for 4-channel buffers, ignored for 3-channel), then copy the source rect bit-exactly; no interpolation
- [ ] 2.4 Implement shrink: copy only the in-bounds anchor-aligned rectangle and discard the rest, with no resampling
- [ ] 2.5 Unit-test grow and shrink: center grow adds a symmetric border, top-left grow adds only right/bottom, shrink keeps only the anchor-aligned rectangle, and the added pixels equal the background including alpha
- [ ] 2.6 Unit-test the new-buffer contract, `InvalidParams` for zero dimensions and malformed buffers, and no panic on 1×1 and 1-pixel inputs

## 3. M10-A3 — Image orientation

- [ ] 3.1 Implement the exact index remaps `rotate90_cw`, `rotate90_ccw`, `rotate180`, `flip_horizontal`, and `flip_vertical` returning new buffers, preserving every channel bit-for-bit and never resampling
- [ ] 3.2 Implement `rotate_arbitrary(buf, angle_deg, background)` with the expanded bounding box `W' = W·|cos θ| + H·|sin θ|`, `H' = W·|sin θ| + H·|cos θ|`, a bilinear inverse map, clamp-to-edge sampling, and background-filled corners (alpha from `background[3]`)
- [ ] 3.3 Validate `angle_deg` as finite and within `-359.99..=359.99` before any work, returning `OpsError::InvalidParams` otherwise, and make `angle_deg` 0.0 a bit-exact no-op
- [ ] 3.4 Unit-test the exact remaps: 90° CW swaps dimensions and remaps every sample, 90° CW then CCW restores the original, 180° twice and the two flips restore the original, and odd/1×1 buffers do not panic or shift by one
- [ ] 3.5 Unit-test `rotate_arbitrary`: angle 0.0 is bit-identical, a non-90 angle grows the canvas to the bounding box, corners equal the background, and out-of-range/non-finite angles are rejected
- [ ] 3.6 Unit-test that the exact remaps preserve alpha and that `rotate_arbitrary` leaves the input untouched

## 4. M10-B — ImageMagick oracle and measured classification

- [ ] 4.1 Add `scripts/ops_oracle.py` with a resize operator (point / triangle / Catmull-Rom), an `-extent <W>x<H> -gravity <g>` operator, and an `-rotate <angle> -background <color>` operator over a raw 8-bit image
- [ ] 4.2 Add `crates/pictura-ops/tests/oracle.rs` with one mapping row per `Resample` method and diff each within its measured tolerance, recording the maximum and mean per-sample delta
- [ ] 4.3 Add one canvas mapping row per `Anchor` variant, map each to its ImageMagick gravity, and diff centered growth and gravity-aligned crop against `-extent`
- [ ] 4.4 Validate the right-angle rotations and flips bit-for-bit against hand-computed index remaps at zero tolerance, and diff `rotate_arbitrary` against `-rotate`, recording the measured delta or classifying the case no-equivalent with the observed delta
- [ ] 4.5 Skip the differential tests with a message when `magick` is absent and add no `#[ignore]`
- [ ] 4.6 Document the mapping, exact flags, the measured resize/canvas/rotation tolerances, any no-equivalent deltas, and the verified ImageMagick version in `crates/pictura-ops/tests/README.md`

## 5. M10-C — OpenSpec change, reconcile, and verify

- [ ] 5.1 Validate `openspec validate m10-image-ops --strict` and `openspec validate --all --strict` green
- [ ] 5.2 Run `cargo test --workspace` green with the per-primitive unit tests and the recorded oracle tolerances
- [ ] 5.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 5.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [ ] 5.5 Reconcile `docs/dev/m10-image-ops.md` against the shipped `pictura-ops` signatures and record any divergence
