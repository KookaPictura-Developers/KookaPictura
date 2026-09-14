## 1. Crate and representation

- [x] 1.1 Create `crates/pictura-select` with `pictura-core` (for `Channel`/`PixelBuffer`) and the workspace `thiserror`
- [x] 1.2 Define `Selection { width, height, data: Vec<u8> }` as a document-sized 8-bit coverage mask
- [x] 1.3 Define `SelectError::{SizeMismatch, InvalidParams}` and implement `none` / `all`
- [x] 1.4 Register the crate in the workspace and confirm it builds

## 2. Boolean algebra and set operations

- [x] 2.1 Add the `SelectOp` enum (Replace / Add / Subtract / Intersect)
- [x] 2.2 Implement `combine` with Replace = copy, Add = max, Subtract = saturating subtract, Intersect = round(a*b/255)
- [x] 2.3 Return `SizeMismatch` from `combine` when dimensions or coverage lengths differ
- [x] 2.4 Implement `invert` as `255 - v`

## 3. Modify operations

- [x] 3.1 Implement `feather` as a separable Gaussian with `sigma = radius/2`, `ceil(3*sigma)` support, clamped edges
- [x] 3.2 Implement `expand`/`contract` as a separable square min/max with canvas-edge clamping
- [x] 3.3 Implement `border` as a centred band from an outer dilation and inner erosion
- [x] 3.4 Implement `smooth` as a majority/median filter over a `(2r+1)²` window
- [x] 3.5 Clamp out-of-range radii (feather 250, morphology 100, border 200, smooth 100) and reject non-finite feather radii

## 4. Image-derived tools

- [x] 4.1 Add the shared Chebyshev colour-distance metric and `rgb_at` sampler
- [x] 4.2 Implement `magic_wand` as a four-connected flood fill (contiguous) or a global pass, with out-of-bounds/invalid seed errors
- [x] 4.3 Implement `grow` to add adjacent in-tolerance pixels from the selection boundary
- [x] 4.4 Implement `similar` to add in-tolerance pixels image-wide from any selected colour
- [x] 4.5 Implement `color_range` as a soft distance ramp, exact match at fuzziness 0
- [x] 4.6 Return `SizeMismatch` from `grow`/`similar` when the image dimensions differ

## 5. Alpha-channel save/load

- [x] 5.1 Implement `to_channel(id)` as an 8-bit grayscale channel copy of the coverage
- [x] 5.2 Implement `from_channel(channel, width, height)` with length validation and a mismatch error
- [x] 5.3 Add the byte-for-byte channel round-trip test

## 6. Oracle and tests

- [x] 6.1 Write `scripts/select_oracle.py` to run ImageMagick grayscale morphology/blur on a raw 8-bit mask
- [x] 6.2 Add `crates/pictura-select/tests/oracle.rs` with the op-to-ImageMagick mapping and the `Square:r` note
- [x] 6.3 Add active oracle tests (`-negate`, single-pixel `Disk:1` dilation, mapping consistency) and the differential rows for expand/contract/feather
- [x] 6.4 Record the square-versus-disk structuring-element divergence and tolerances in `tests/README.md`
- [x] 6.5 Remove the `#[ignore = "enable once M5-A lands"]` gate now that the core has landed
- [x] 6.6 Add in-crate unit tests: boolean identities, soft bounds, invert involution, feather ramp, expand/contract round trip, border band, smooth remove/fill, wand contiguous/global, grow/similar adjacency, colour-range monotonicity, parameter clamping

## 7. Validation

- [x] 7.1 Run `cargo test -p pictura-select` (oracle differential rows included when `magick` is present)
- [x] 7.2 Run `openspec validate m5-selection --strict` and fix any schema findings
