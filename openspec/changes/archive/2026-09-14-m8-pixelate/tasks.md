## 1. M8-A — Pixelate family and tests

- [x] 1.1 Implement `pixelate::mosaic` as the `cell_size × cell_size` block mean with `cell_size` 2..=200, edge blocks averaging their available pixels, and `FilterError::InvalidParams` outside the range
- [x] 1.2 Implement `pixelate::crystallize` as a seeded-jittered Voronoi tessellation with per-cell mean fill, `cell_size` 3..=300, and a `ChaCha8Rng::seed_from_u64(seed)` for the jitter
- [x] 1.3 Implement `pixelate::facet` as the fixed-pass similar-neighbor local average that flattens gradients while preserving strong edges
- [x] 1.4 Implement `pixelate::fragment` as the average of four small fixed-offset copies, a bit-exact no-op on a flat region
- [x] 1.5 Implement `pixelate::mezzotint` as the per-`kind` seeded procedural dot / line / stroke pattern, luma pattern on grayscale and saturated color on color, with a `ChaCha8Rng` from the seed
- [x] 1.6 Implement `pixelate::pointillize` as seeded dots of radius proportional to `cell_size`, filled with the local source color over the `background` parameter, `cell_size` 3..=300
- [x] 1.7 Implement `pixelate::color_halftone` as per-channel rotated screening at `angles[channel]`, grayscale using `angles[0]`, dot radius proportional to cell brightness, and `max_radius` 4..=127 with every angle finite
- [x] 1.8 Unit-test Mosaic: every pixel in a block equals the block mean, larger cells yield larger blocks, edge blocks average the available pixels, and out-of-range `cell_size` is rejected
- [x] 1.9 Unit-test Crystallize: the same seed is bit-identical, a different seed may differ, larger cells yield fewer/larger regions, and `cell_size` outside 3..=300 is rejected
- [x] 1.10 Unit-test Facet: a gradient is flattened and a strong step edge is preserved
- [x] 1.11 Unit-test Fragment: the output equals the four-copy average, a flat region is a bit-exact no-op, and repeated runs match
- [x] 1.12 Unit-test Mezzotint: each `kind` produces a distinct pattern, the same seed is bit-identical, a different seed differs, and an out-of-range `kind` is rejected
- [x] 1.13 Unit-test Pointillize: the background shows between dots, dot size scales with `cell_size`, the same seed is bit-identical, and `cell_size` outside 3..=300 is rejected
- [x] 1.14 Unit-test Color Halftone: increasing `max_radius` enlarges the dots, grayscale uses `angles[0]`, a 3-channel buffer uses the first three angles, and out-of-range radius / non-finite angles are rejected
- [x] 1.15 Unit-test alpha preservation, clamp-to-edge, and 1×1 / 1-px / cell-larger-than-image safety for every Pixelate variant

## 2. M8-B — ImageMagick oracle and no-equivalent table

- [x] 2.1 Extend `scripts/filter_oracle.py` with a Mosaic operator: box downsample then point upsample by the cell size (block average)
- [x] 2.2 Diff Mosaic within its stated tolerance against the oracle image
- [x] 2.3 Add one mapping-table row per Pixelate `Filter` variant and assert Crystallize, Facet, Fragment, Mezzotint, Pointillize, and Color Halftone use tolerance 0 with a property or known-value test, a non-empty no-equivalent note, and the observed delta recorded
- [x] 2.4 Skip the differential tests with a message when `magick` is absent and add no `#[ignore]`
- [x] 2.5 Document the mapping, exact flags, the measured Mosaic tolerance, the no-equivalent deltas, and the verified ImageMagick version in `crates/pictura-filters/tests/README.md`

## 3. M8-C — App filter kinds and unit test

- [x] 3.1 Register the Pixelate filter kinds (Color Halftone, Crystallize, Facet, Fragment, Mezzotint, Mosaic, Pointillize) in the app filter menu with their parameter descriptors and `MezzotintType` options
- [x] 3.2 Add an app unit test asserting each new kind maps to the matching `pictura-filters` variant and parameter set, including a seed-carrying variant

## 4. M8-D — OpenSpec change, reconcile, and verify

- [x] 4.1 Validate `openspec validate m8-pixelate --strict` and `openspec validate --all --strict` green
- [x] 4.2 Run `cargo test --workspace` green with the per-filter unit tests and oracle differentials within tolerance or documented no-equivalent
- [x] 4.3 Run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] 4.4 Run `scripts/guard.sh` green (no `.8bf`, no artboards, no unmarked `docs/` edits)
- [x] 4.5 Reconcile `docs/dev/m8-pixelate.md` against the shipped `Filter` variants and record any divergence
