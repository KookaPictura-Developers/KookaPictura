# Patent / freedom-to-operate notes

- **Status:** engineering input for counsel. **Not legal advice.**
- **Purpose:** give patent counsel a concrete map of every algorithmic area in
  the project, where it lives, how it was derived, and what is public, so a
  freedom-to-operate (FTO) review can be scoped.

> Independent creation and provenance separation are a defence against
> **copyright**, not patents. Public availability of the PSD format or of an
> algorithm's description grants no patent rights. This inventory exists because
> the project approximates several closed, likely-patented image-processing
> pipelines.

## How claims are marked

- **documented** — implemented from a public specification/standard or standard
  mathematics.
- **inferred / approximated** — Adobe's exact algorithm is closed; the
  implementation is a behavioural approximation from public observation, marked
  `*(inferred)*` / *behavioral parity only* in the specs.
- **deferred** — specced but not implemented.

## Inventory

| Family | Implementation | Status | Public basis | FTO review |
|---|---|---|---|---|
| Resample kernels (nearest / bilinear / Keys bicubic `a=-0.75`) | `crates/pictura-ops/src/resize.rs` | inferred | Mitchell–Netravali; public analysis (J. Summers) | **High** — resampling/generic interpolation patents |
| Blend modes (27 separable + non-separable, Porter-Duff compositing) | `crates/pictura-core/src/lib.rs` (`BlendMode`), `crates/pictura-render` | documented | PDF/PSD blend tables; Porter–Duff (expired) | Medium |
| Tonal adjustments (levels, curves, brightness/contrast, exposure, vibrance, HDR toning) | `crates/pictura-adjust/src/{tonal,color,hdr_toning}.rs` | documented/inferred | Standard color math; Auto Tone partly public analysis | Medium |
| Channel mixer / selective color / color balance / photo filter / black&white | `crates/pictura-adjust/src/{color,native}.rs` | documented | Standard matrix math; PSD descriptors | Low–Medium |
| Color lookup (3D LUT) | `crates/pictura-adjust/src/lut.rs` | documented | `.CUBE`/LUT standard | Low |
| Blur family (Gaussian, box, median, motion, radial, surface/bilateral, high-pass) | `crates/pictura-filters/src/blur.rs` | mixed | Gaussian/box/median standard; **Surface Blur bilateral closed** | Medium (bilateral) |
| Sharpen family (unsharp, sharpen/sharpen-more, sharpen-edges) | `crates/pictura-filters/src/sharpen.rs` | documented | Classic convolution | Low |
| Noise (add noise, despeckle, dust&scratches, median) | `crates/pictura-filters/src/noise.rs` | documented | Standard | Low |
| Distort (twirl, pinch, spherize, ripple, wave, zigzag, polar, shear) | `crates/pictura-filters/src/distort/` | inferred | No-equivalent classification in `tests/README.md` | **High** — warp/distortion patents |
| Stylize / pixelate (halftone, mezzotint, crystallize, pointillize, mosaic, facet, fragment, emboss, find-edges, solarize, oil paint) | `crates/pictura-filters/src/{stylize,oil_paint,other}.rs`, `pixelate/` | mixed (mostly inferred) | Standard/adhoc; Adobe closed | Medium |
| Artistic / sketch / texture / brush-strokes | `crates/pictura-filters/src/{artistic,sketch,texture,brush_strokes}.rs` | inferred | Painterly technique from public art references | Medium |
| Render (clouds, fibers, lens flare, lighting effects) | `crates/pictura-filters/src/render.rs` | inferred | Adobe noise/flare models closed | **High** — lighting-effects / lens-flare patents |
| Warp (4×4 cubic Bézier mesh, presets) | `crates/pictura-render/src/document_ops/layer_ops/warp.rs`, `warp_styles.rs` | inferred | `SethRobinson/Patchy` public mesh analysis | Medium–High |
| Layer styles (drop shadow, glows, bevel/emboss, satin, stroke, overlays) | `crates/pictura-render` (blend/layer-effects paths) | documented/inferred | PSD `lfx2`/`lrFX` descriptor semantics | Low–Medium |
| Healing / clone / patch / content-aware fill & scale | specced (`docs/03-tools/healing-brushes.md`, `content-aware-move-and-patch.md`, `move-and-transform.md`) | **deferred** | Seam carving (Avidan–Shamir), patch-match | **High** — content-aware and texture-synthesis patents |
| Refine Edge / matting | specced (`docs/08-selection/`) | **deferred** | Closed-form matting, shared matting (public papers) | **High** — matting patents |
| Shake Reduction (blind deconvolution) | specced (`docs/06-filters/sharpening-tools.md`) | **deferred** | Blind-deconvolution literature | **High** |
| Pictura Raw / Camera Raw pipeline | `crates/pictura-adjust/src/pictura_raw.rs`, `crates/pictura-codec/src/{pictura_raw,smart_filter,crs_xmp}.rs` | documented/inferred | Public raw-converter stages | Medium (raw-pipeline patents) |
| Type / EngineData layout | `crates/pictura-codec/src/{engine_data,type_tool}.rs` | inferred | Adobe closed; format facts only | Low–Medium |
| Selection / morphology / blur-mask | `crates/pictura-select/src/lib.rs` | documented | Standard morphology | Low |
| Colour management (ICC / Little CMS) | `crates/pictura-color/src/lib.rs` (lcms2, MIT) | documented | ICC spec; lcms2 MIT | Low (ICC patents expired) |

Per-filter classification and measured ImageMagick divergences live in
[`crates/pictura-filters/tests/README.md`](../../crates/pictura-filters/tests/README.md).

## Recommended counsel scope

1. **FTO search** on the high-risk families: interpolation/resampling,
   distortion/warp, lighting-effects/lens-flare, content-aware fill/scale,
   matting, blind deconvolution, raw pipeline.
2. **In-force status** — many classic image-processing patents (e.g. Porter–Duff,
   early ICC) have expired; confirm per claim and jurisdiction.
3. **Defensive posture** — document the independent derivation (this repo's
   sources) for each high-risk family; do not rely on it against patents.

## Open questions for counsel

- Which of the identified families have in-force patents in our target markets?
- Does shipping the *deferred* features later (content-aware, matting, shake
  reduction) require a separate FTO pass before implementation?
- Any patent-licensing commitments implied by the dependencies (lcms2, wgpu,
  image codecs)?
