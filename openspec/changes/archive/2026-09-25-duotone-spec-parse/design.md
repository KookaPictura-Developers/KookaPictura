# Design: duotone-spec-parse

## Context

`read.rs` reads the color-mode-data length and stores the bytes in
`Document.color_mode_data` (cleared for Indexed, retained for Duotone). The
layout (Adobe "Duotone Options", confirmed by `psdparse` `duotone.c`) is, after
the 4-byte section length already stripped on read:

```
u16 version
u16 plates                    (1..=4)
4 x 10  ink ColorStruct       space u16 + 4 x u16 components
4 x 64  ink name              Pascal string (length byte + chars)
4 x 28  transfer             13 x i16 points (value/10 = percent, -1 = unset) + i16 override
u16     dot gain
11 x 10 overprint ColorStruct (plates 1/2/3/4 -> 0/1/4/11 entries)
```

Total 524 bytes.

## Goals / Non-Goals

**Goals**

- Decode the block into a typed view with grounded offsets.

**Non-Goals**

- Rendering (single-ink monotone or multi-ink overprint), storing the spec on
  `Document`, or changing the grayscale open path. A real Duotone fixture is
  also out of scope (psd-tools cannot author one).

## Decisions

**`parse_duotone(&[u8]) -> Option<DuotoneSpec>`.** Read big-endian; require
`len >= 524`; reject `plates` outside 1–4. Names are Pascal strings (read `p[0]`
bytes from `p+1`). Curves are 13 `i16` (keep `-1` sentinels verbatim) plus the
override `i16`. `plates` 1/2/3/4 map to overprint counts 0/1/4/11; ignore any
extra bytes past 524.

**Tests.** Build the 524-byte buffer in-test from the offsets above (grounded in
psdparse), parse it, and assert every field; assert `None` for a short buffer
and for `plates` 0 or 5.

## Risks / Trade-offs

- [No executable oracle for the byte layout] → offsets are grounded in the Adobe
  spec and two independent parsers; the test pins them so a future fixture or
  Photoshop comparison is a one-line change.
- [Unused until rendering lands] → it is public API, the documented decode of a
  preserved structure.

## Migration Plan

Additive; revert is a revert.
