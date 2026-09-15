## 1. Brief and frozen interfaces (orchestrator)

- [x] 1.1 Write `docs/dev/m22-artistic-filters.md` milestone brief
- [x] 1.2 Freeze the `Filter` variant field names, ranges, and shared helper signatures in `design.md`
- [x] 1.3 Commit brief + proposal with a `TASK-ALLOWS-DOCS` message

## 2. Wave 1: shared helpers + the deterministic group (sub-agent, Rust)

- [x] 2.1 `artistic/mod.rs` module; `artistic/reduce.rs` (posterize/quantize + edge magnitude), `artistic/noise.rs` (seeded grain field), `artistic/stroke.rs` (oriented daub/smear helper) reusing existing kernels where present
- [x] 2.2 Add `Filter` variants + validation + `apply` dispatch for: Cutout, Film Grain, Neon Glow, Paint Daubs, Palette Knife, Plastic Wrap, Poster Edges, Sponge
- [x] 2.3 Implement each kernel as a behavioural-parity model; seed all stochastic ones
- [x] 2.4 Tests: effect is non-empty, alpha preserved, parameter validation (boundary accept / out-of-range reject), determinism, and the per-filter scenarios in the spec (levels, glow extent, brush types, highlight strength, posterized flat areas, size monotonicity)
- [x] 2.5 `cargo test -p pictura-filters`, clippy clean

## 3. Wave 2: colour- and texture-driven group (sub-agent, Rust)

- [x] 3.1 `artistic/texture.rs`: procedural Brick/Burlap/Canvas/Sandstone height map + light-direction emboss, with `TextureOptions`
- [x] 3.2 Add `Filter` variants + validation + `apply` dispatch for: Colored Pencil, Dry Brush, Fresco, Rough Pastels, Smudge Stick, Underpainting, Watercolor
- [x] 3.3 Foreground/background and glow colours as explicit variant fields; no panic on equal colours
- [x] 3.4 Tests: effect non-empty, texture options change the result, background colour shows through, equal fg/bg safe, alpha preserved, determinism
- [x] 3.5 `cargo test -p pictura-filters`, clippy clean

## 4. Wave 3: app mapping and self-test (sub-agent)

- [x] 4.1 `filter_from_kind` maps the 15 new kinds to defaults
- [x] 4.2 Extend `--self-test` (exit codes from 57): apply several Artistic filters through the bridge and assert the image changes, alpha is preserved, and a redo/undo round-trips
- [x] 4.3 Build green; M16–M21 self-tests still exit 0

## 5. Close-out

- [x] 5.1 `cargo fmt/clippy/test`; `openspec validate --all --strict`; `guard.sh`
- [x] 5.2 Update `docs/dev/STATE.md` with a `TASK-ALLOWS-DOCS` message
- [x] 5.3 Archive the change and commit
