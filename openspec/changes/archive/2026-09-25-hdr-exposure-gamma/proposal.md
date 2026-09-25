# Proposal: hdr-exposure-gamma

## Why

The roadmap's 32-bit HDR item is open because no tone-map operator was
specified. Research against the CS6 Help corpus (`docs/04-image-ops/32-bit-hdr.md`,
ADJ-025) found that CS6's *Image > Mode > 32→16/8* HDR Conversion dialog offers
four methods, but only **Exposure & Gamma** is fully documented
(`clip(exposure·2^EV)^(1/gamma)`, Exposure 0 / Gamma 1.0 = identity); Local
Adaptation, Equalize Histogram, and Highlight Compression have closed kernels.
This ships the one operator that can be implemented without inventing behavior,
as the documented foundation for the eventual dialog.

## What Changes

- Add `pictura_adjust::hdr_toning` with `ExposureGamma { exposure_ev, gamma }`
  and `exposure_gamma(&[f32], ExposureGamma) -> Result<Vec<f32>, AdjustError>`:
  a pure operator on linear-light `f32` samples, `gain = 2^EV`, output
  `sign(g)·|g|^(1/gamma)` where `g = x·gain`, so values `> 1.0` are not clamped
  and `0`/negative inputs stay defined. `gamma` must be finite and `> 0`, and
  `exposure_ev` finite, else `AdjustError::InvalidParams`.
- No PSD codec change, no UI: the HDR Toning dialog and the other three methods
  stay deferred (`ponytail:` ceiling).

## Capabilities

### New Capabilities

- `hdr-toning`: the Exposure & Gamma tone-map operator.

## Impact

- New `crates/pictura-adjust/src/hdr_toning.rs` and its `lib.rs` re-export.
- Unit tests: identity, the exact formula on a ramp including `>1.0`, `0`, and a
  negative sample, and parameter rejection.
- No new dependency.
