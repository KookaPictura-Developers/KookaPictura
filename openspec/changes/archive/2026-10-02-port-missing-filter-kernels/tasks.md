# Tasks: port-missing-filter-kernels

## 1. Engine types and dispatch

- [x] 1.1 Add the six `Filter` variants and the parameter enums
      (`ExtrudeType`, `TileFill`, `ContourEdge`, `WindMethod`,
      `SharpenRemove`) in `crates/pictura-filters/src/lib.rs` and
      `crates/pictura-filters/src/filter/types.rs`; re-export the public enums at
      the crate root
- [x] 1.2 Add one `apply` arm per variant in
      `crates/pictura-filters/src/filter/apply.rs`

## 2. Noise kernel

- [x] 2.1 Implement `noise::dust_and_scratches(buf, radius, threshold)` in
      `crates/pictura-filters/src/noise.rs` (threshold-gated windowed median,
      clamp-to-edge, validate `radius` 1..=16 and `threshold` 0..=255 before
      mutation)
- [x] 2.2 Unit tests: scratch removal, threshold gate (0 replaces, 255 keeps),
      uniform no-op, out-of-range rejection, alpha preservation

## 3. Stylize kernels

- [x] 3.1 Implement `stylize::extrude(...)` in
      `crates/pictura-filters/src/stylize/extrude.rs` (Blocks/Pyramids, size
      2..=255, depth 1.0..=255.0, level-based or coordinate-hash protrusion,
      Solid Front, Mask Incomplete, back-to-front towers)
- [x] 3.2 Implement `stylize::tiles(...)` in
      `crates/pictura-filters/src/stylize/tiles.rs` (count 1..=99, offset
      1..=99 %, Background/Foreground/Inverse/Unaltered fill, coordinate-hash
      offsets, source snapshot for the Inverse/Unaltered fills)
- [x] 3.3 Implement `stylize::trace_contour(...)` in
      `crates/pictura-filters/src/stylize/trace_contour.rs` (per-channel
      Lower/Upper crossing at Level, flat image maps to 255)
- [x] 3.4 Implement `stylize::wind(...)` in
      `crates/pictura-filters/src/stylize/wind.rs` (Wind/Blast/Stagger, From the
      Right/From the Left, fixed luma gate)
- [x] 3.5 Unit tests: level ordering + solid front + coordinate-hash
      determinism (Extrude); fill rules + determinism (Tiles); flat no-op +
      crossing side + level shift (Trace Contour); flat ground untouched +
      mirror + determinism (Wind); out-of-range rejection and alpha
      preservation for each

## 4. Sharpen kernel

- [x] 4.1 Implement `sharpen::smart_sharpen(...)` in
      `crates/pictura-filters/src/sharpen.rs`, reusing the shared Gaussian
      kernel, adding the disc (Lens) summed-area blur and the `blur::motion`
      path for Motion Blur
- [x] 4.2 Unit tests: amount 0 rejection, GaussianBlur matches Unsharp Mask,
      LensBlur/MotionBlur differ from Gaussian, reduce-noise holds back low
      contrast, out-of-range rejection, alpha preservation and determinism
- [x] 4.3 Implement Smart Sharpen's `more_accurate` higher-fidelity blur path
      and the `shadow`/`highlight` `TonalFade { amount, width, radius }` fields
      (amount/width 0..=100, radius 1..=100, rejected before mutation)
- [x] 4.4 Unit tests: More Accurate differs from the default path for each
      remove and is deterministic; zero `TonalFade` amount is a no-op; a high
      shadow/highlight fade damps its tonal band; out-of-range tonal rejection

## 5. App mappings

- [x] 5.1 Add the six `filter_from_kind` arms in
      `crates/pictura-app/src/cxxqt_object/helpers.rs` with the `filter-app-ui`
      defaults
- [x] 5.2 Extend the mapping guard test to cover the six new kinds

## 6. Verification

- [x] 6.1 `openspec validate port-missing-filter-kernels --strict` and
      `openspec validate --all --strict` green
- [x] 6.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
      -- -D warnings`, `cargo test --workspace` green
- [x] 6.3 `bash scripts/verify-fast.sh` green
- [x] 6.4 Add a `Source:` provenance header to each of the six ported kernels
- [x] 6.5 Add one no-equivalent row per new kernel (tolerance 0, non-empty note)
      to `crates/pictura-filters/tests/oracle/mapping.rs` (`NO_EQUIVALENT`),
      covered by property tests
- [x] 6.6 Correct `blur::motion` to Photoshop's counter-clockwise screen-angle
      convention (negate the y component) and pin it with a 45° test, fixing the
      Motion Blur filter and Smart Sharpen's Motion Blur removal
- [x] 6.7 Record the CS6 Smart Sharpen More Accurate / Shadow-Highlight port and
      the `docs/06-filters/stylize-filters.md` Extrude Depth/Solid-Front-Faces
      correction (`TASK-ALLOWS-DOCS` at commit)
