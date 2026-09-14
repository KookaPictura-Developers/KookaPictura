# Tasks: m15-render-filters

## 1. M15-A — `pictura-filters` render module

- [ ] 1.1 Add `crates/pictura-filters/src/render.rs` with a module-private seeded lattice value-noise helper (ChaCha8Rng, smoothstep bilinear, 5 octaves, lacunarity 2, gain 0.5, normalized [0,1]) and a `ponytail:` note naming the divergence points (Adobe noise closed; grid artifacts → gradient noise upgrade path)
- [ ] 1.2 Implement `clouds(buf, color_a, color_b, starker, seed)` — replaces RGB, alpha untouched, `starker` = smoothstep contrast curve on the normalized field
- [ ] 1.3 Implement `difference_clouds(buf, color_a, color_b, starker, seed)` — same field, per-channel `|existing − field|` on RGB, alpha untouched
- [ ] 1.4 Implement `fibers(buf, variance, strength, color_a, color_b, seed)` — x-elongated noise; validate `variance` 0..=100 and `strength` 1..=100 with `InvalidParams` before mutation
- [ ] 1.5 Add `LensType { Zoom, Prime35, Prime105, MoviePrime }` and implement `lens_flare(buf, brightness, center, lens)` — additive core/ghosts/rays clamped to 255, unit center clamped to 0..=1, validate brightness 10..=300, deterministic (no seed)
- [ ] 1.6 Extend `Filter` enum with `Clouds`, `DifferenceClouds`, `Fibers`, `LensFlare` variants and dispatch them in `apply`
- [ ] 1.7 Unit tests per filter pinning the spec'd behaviors: seeded bit-determinism, RGB range within color endpoints, alpha untouched, starker contrast, Difference formula on a known buffer, repeated Difference Clouds changes, fibers row-variation vs variance, lens flare brightness monotonicity + per-lens geometry difference + center move + clamp, and both validation rejections leaving the buffer untouched

## 2. M15-B — App kinds and shell

- [ ] 2.1 Map kinds `clouds`, `difference-clouds`, `fibers`, `lens-flare` in `filter_from_kind` with the spec'd defaults (black/white, starker false, seed 1, variance 16, strength 4, brightness 100, center (0.5, 0.5), Zoom)
- [ ] 2.2 Add the four entries to the filter combo in `crates/pictura-app/cpp/main.cpp` (Clouds, Difference Clouds, Fibers, Lens Flare)
- [ ] 2.3 Self-test block (exit code 24, after the M14 checks and before zoom/pan): the M14 block reopens the fixture (8×8, red top-left quadrant, blue bottom-right quadrant); capture the composite, `apply_filter("clouds")`, assert true, composite changed only inside the blue layer's rect (bottom-right quadrant, pixels (4..8, 4..8)), alpha outside unchanged; apply again and assert bit-identical to the first result; also `apply_filter("lens-flare")` returns true and changes the composite

## 3. M15-C — Verify and reconcile

- [ ] 3.1 `openspec validate --all --strict` green
- [ ] 3.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` green
- [ ] 3.3 `cmake --build build` green; `xvfb-run -a ./build/pictura --self-test crates/pictura-codec/tests/fixtures/two_layers.psd` exits 0 with the new assertions logged
- [ ] 3.4 `bash scripts/guard.sh` green (docs/dev notes carry the commit marker)
- [ ] 3.5 Record the milestone in `docs/dev/m15-render-filters.md` (incl. the no-equivalent classification rationale), update `docs/dev/STATE.md`, archive the change (commit carries `TASK-ALLOWS-DOCS`)
