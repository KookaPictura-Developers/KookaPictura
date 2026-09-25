# Design: image-resize-bicubic-kernel

## Context

`resize.rs` samples each plane with a separable 4×4 Keys cubic and normalizes by
the weight sum (`acc / ws`) so clamped edge duplicates do not bias corners.
`scripts/ops_oracle.py --op resize` emits `-filter <f> -resize WxH!`; the
differential test passes `--filter catrom` for Bicubic.

## Goals / Non-Goals

**Goals**

- Use Photoshop's documented bicubic coefficients (`C = 0.75`).
- Prove it against ImageMagick's identical parameterization.

**Non-Goals**

- Bicubic Smoother / Sharper / Automatic and the Image Size dialog (the
  `Image > Image Size…` command is still an unimplemented stub, so there is no
  UI consumer; add the family when the dialog ships).
- Any change to Nearest, Bilinear, the entry point, or alpha handling.

## Decisions

**Kernel.** Keep the existing Keys two-piece form and set `a = -0.75`:
`|x| <= 1`: `(a+2)x³ - (a+3)x² + 1`; `1 < |x| < 2`: `a x³ - 5a x² + 8a x - 4a`.
This is Mitchell–Netravali `cubic(0, 0.75)`.

**Oracle.** Add `--im-args` passthrough to the `resize` branch of
`build_im_args` (prepend to the operator list) so the test can pass
`-define filter:b=0 -define filter:c=0.75`. Map `Resample::Bicubic` to
`-filter cubic` + those defines. Keep tolerance at the measured value (start 1,
raise only if the measured delta requires it, recording the number).

## Risks / Trade-offs

- [ImageMagick phase/edge handling still differs] → the differential test keeps a
  measured tolerance and a no-equivalent note; the *kernel* is exact, not the
  edge policy.
- [Bicubic output changes] → expected; regenerate any golden using Bicubic and
  state it in the task result.

## Migration Plan

Additive to behavior only; revert is a revert.
