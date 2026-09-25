# Design: hdr-exposure-gamma

## Context

32-bit PSD composite samples are linear-light `f32` (`Samples::F32`); the editing
path narrows them with `clamp(trunc(v·256))`, clipping at 1.0. The HDR Toning
dialog (`docs/04-image-ops/adjustments/hdr-toning.md`, ADJ-025) is the intended
consumer, but it is not built; this change adds the operator only.

## Goals / Non-Goals

**Goals**

- A pure, deterministic Exposure & Gamma operator on linear `f32` samples with
  documented edge behavior.

**Non-Goals**

- Local Adaptation, Equalize Histogram, Highlight Compression (closed kernels),
  the HDR Toning dialog, and the 32→8/16 conversion command.

## Decisions

**Signature.** `exposure_gamma(samples: &[f32], ExposureGamma) -> Result<Vec<f32>,
AdjustError>`, mirroring the other `pictura_adjust` pure functions.

**Math.** `gain = 2^exposure_ev`; for each sample `g = x * gain`, output
`g.signum() * g.abs().powf(1.0 / gamma)`. This is `(x·2^EV)^(1/gamma)` for the
non-negative domain Photoshop documents, keeps `>1.0` values (no clamp), maps
`0 → 0`, and gives negative inputs a defined sign-preserving result
(`ponytail:` inferred — negative light is outside the documented domain).

**Validation.** `exposure_ev` finite, `gamma` finite and `> 0`, else
`AdjustError::InvalidParams`.

## Risks / Trade-offs

- [No consumer yet] → it is a public library seam, tested directly; wire the
  dialog when it ships.
- [Negative-sample rule is inferred] → documented inline.

## Migration Plan

Additive; revert is a revert.
