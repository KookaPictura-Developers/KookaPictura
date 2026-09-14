# M4 — Adjustments

Goal: the destructive adjustment math as pure, tested functions, then wire them
into adjustment layers.

Spec: `docs/04-image-ops/adjustments/*.md` (one file per adjustment).

## Scope

In (M4-A): planar 8-bit `PixelBuffer` (3 or 4 channels; alpha untouched):
- Tonal: `Levels`, `Curves`, `BrightnessContrast`, `Exposure`, `Invert`,
  `Posterize`, `Threshold`, `Desaturate`, `Auto(Tone|Contrast|Color)`.
- Color: `HueSaturation`, `BlackWhite`, `PhotoFilter`, `ChannelMixer`,
  `Vibrance`, `ColorBalance`.

Out (later):
- `GradientMap`, `SelectiveColor`, `ShadowHighlight`, `HDRToning`, `MatchColor`,
  `ReplaceColor`, 3D LUT / Color Lookup.
- Per-range Hue/Saturation channels and per-channel Levels/Curves (composite only for M4).
- Adjustment **layers** (model + PSD keys + compositor) — task M4-B, after M4-A.
- 16/32-bit adjustment math.

## Contract (M4-A authoritative)

`crates/pictura-adjust` (stub already in tree): an `Adjustment` enum with the
params above and

```rust
pub fn apply(adjustment: &Adjustment, buf: &mut PixelBuffer) -> Result<(), AdjustError>;
```

- Operates in place on planar channel data; alpha (channel 4) is never modified.
- Deterministic; no panics on out-of-range params (return `AdjustError`).
- Parameters follow the CS6 specs (ranges/defaults documented per adjustment).

## Task DAG

| ID | Task | Owner | Owns | Acceptance |
|---|---|---|---|---|
| M4-A | Adjustment math + unit tests | agent | `crates/pictura-adjust` | All 15 variants implemented; unit tests per adjustment |
| M4-C | ImageMagick differential oracle | agent | `scripts/**`, `crates/pictura-adjust/tests/**` | IM equivalents where they exist match within tolerance; mapping documented |
| M4-B | Adjustment layers (model + codec + render) | agent | later wave | Adjustment layer applies to the backdrop, gated by mask/opacity |

## Oracle

ImageMagick equivalents where they exist: `-level`, `-gamma`, `-brightness-contrast`,
`-negate`, `-posterize`, `-threshold`, `-colorspace Gray`, `-modulate`
(hue/sat), `-color-matrix` (channel mixer). PhotoFilter/BlackWhite/Vibrance/
ColorBalance have no faithful IM operator — cover those with known-value and
property tests instead and say so.

## Exit gate

- `cargo test --workspace` green; per-adjustment tests + IM differential pass
  within tolerance (or are documented as no-IM-equivalent).
- `scripts/guard.sh` green.
