# M15 — Render Filters: Design

## Context

`pictura-filters` is a pure-function crate: planar 8-bit `PixelBuffer`
(channels 3 or 4), `Filter` enum + `apply` dispatch, `ChaCha8Rng::seed_from_u64`
for seeded filters (Add Noise, Crystallize, Mezzotint, Pointillize, Wave,
Ocean Ripple), alpha never modified, `FilterError` for bad input. The
`render-filters` doc (`FILT-060`) marks Adobe's noise and flare models as
closed ("behavioral parity only, algorithm TBD"), so like M8/M9 these are
no-equivalent approximations verified by property tests.

## Goals / Non-Goals

**Goals:**
- The four pipeline-compatible Render filters with spec'd parameter contracts.
- Property/known-value tests per filter (behaviors the CS6 Help states).
- App kinds with fixed defaults + self-test coverage.

**Non-Goals:**
- Lighting Effects (GPU workspace, 17 presets, bump maps — its own milestone).
- Scripted Patterns (Fill-dialog feature, shares the future fill/paint path).
- Verified parity with Adobe's closed noise/flare models (impossible by
  construction; the spec marks them algorithm-TBD).
- Smart Filter support (no smart objects in the engine yet).

## Decisions

- **One `render.rs` module, four functions.** Same shape as `pixelate.rs`/`distort.rs`
  (public fns + module-level tests). A shared seeded value-noise helper lives
  inside the module — only Clouds/Difference Clouds/Fibers use it; a public
  `FractalNoise` abstraction waits for a second consumer (Lighting Effects).
- **Noise generator: seeded lattice value noise, fBm.** Grid lattice with
  per-node ChaCha8 values, smoothstep bilinear interpolation, 5 octaves,
  lacunarity 2, gain 0.5, normalized to [0,1] and mapped between `color_a`
  and `color_b`. `starker` applies a smoothstep contrast curve to the
  normalized field (the documented "starker pattern" Alt variant). Internals
  are free to change; the tests pin behaviors, not pixels.
- **Difference Clouds = Clouds field + Difference blend.** Per pixel:
  `|existing − cloud_color|` on RGB (the blend-modes Difference formula),
  which makes repeated runs accumulate marble patterning — the documented
  behavior the tests assert.
- **Fibers: x-elongated value noise.** Cross-axis (y) frequency scales with
  `strength`, per-run color jitter with `variance`. Ranges: `variance`
  0..=100, `strength` 1..=100 accepted (Help states no limits; 16/4 defaults
  are the spec's inferred values — validated bounds are conservative and
  documented as such).
- **Lens Flare: deterministic additive pass.** Bright core (radial falloff)
  at `center`, ghost reflections mirrored through the center per `LensType`
  (positions/sizes differ per type), starburst rays, all additive and
  clamped to 255. `brightness` 10..=300 (%) per the AppleScript reference;
  `center` is unit coordinates (0..1) clamped into range — unit coords let
  the app default to the image center without knowing its size. No seed: the
  model is geometry-driven, matching the CS6 dialog (no Randomize).
- **Alpha untouched for all four** (crate-wide convention). Divergence from
  CS6: Photoshop's Clouds/Difference Clouds/Fibers also replace alpha on
  transparent layers; our layer-filter path preserves it. Marked with a
  `ponytail:` comment in the module docs; revisit when fill layers exist.
- **Color params, not global fg/bg.** The engine has no foreground/background
  color state; the filters take explicit `color_a: [u8; 3]` /
  `color_b: [u8; 3]` (interpolation endpoints). The app passes black/white
  defaults, matching CS6's default swatches.

## Risks / Trade-offs

- [Noise look diverges from Adobe] → Accepted and documented (closed model);
  behavior contract is "fg/bg-colored fractal pattern", not pixel parity.
- [Value noise shows grid artifacts at high octaves] → Smoothstep +
  normalized fBm keeps it subtle; upgrade path is gradient (Perlin) noise,
  noted in the module docs.
- [Lens flare ghosts arbitrary] → Property tests pin the spec'd invariants
  (center-dominant brightness, monotone in Brightness, per-type geometry
  differs, clamp, alpha), not the exact look.

## Open Questions

None — the CS6 Help + AppleScript reference pinned everything this milestone
needs; Lighting Effects open questions stay with that filter's milestone.
