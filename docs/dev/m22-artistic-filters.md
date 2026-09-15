# M22 — Artistic filters

Goal: implement the 15 CS6 Artistic filters in `pictura-filters` behind the
shared `apply` contract. Adobe's kernels are closed, so these are
**behavioural-parity** models (as M6–M15): effect, parameter sensitivity,
determinism, alpha preservation, and no-panic edges, without an oracle.
OpenSpec change `m22-artistic-filters` (new capability `artistic-filters`).

## Scope

- Shared helpers: posterize/quantize, gradient/edge magnitude, seeded noise,
  oriented daub/smear, procedural paper height map + light emboss.
- 15 filters: Colored Pencil, Cutout, Dry Brush, Film Grain, Fresco, Neon Glow,
  Paint Daubs, Palette Knife, Plastic Wrap, Poster Edges, Rough Pastels, Smudge
  Stick, Sponge, Underpainting, Watercolor — as `Filter` variants with typed
  parameters, validation, and seeds.
- Foreground/background/glow colours and shared `TextureOptions` as explicit
  parameters.
- App `filter_from_kind` mappings and self-test coverage.

## Out of scope (later milestones)

- Filter Gallery dialog, cumulative/reorder/hide/delete stack, thumbnails.
- Smart Filter entries and `Edit > Fade`.
- 16/32-bit and CMYK/Lab capability gating; `Load Texture` file I/O.
- The remaining families: Brush Strokes, Sketch, Texture, Oil Paint.

## Process

Orchestrator: brief, OpenSpec artifacts, dispatch, integration, verification,
archive, commit. Waves: (1) shared helpers + deterministic group (8 filters),
(2) colour/texture group (7 filters), (3) app mapping + self-test.

## Verification

- `cargo test -p pictura-filters` and `cargo test --workspace`
- `cmake --build build`; fixture and no-argument self-tests exit 0 with new
  filter checks (exit codes from 57)
- `cargo fmt/clippy`; `openspec validate --all --strict`; `guard.sh`
