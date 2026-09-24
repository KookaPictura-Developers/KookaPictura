# Design: phfl-v3-xyz-decode

## Context

`decode_photo_filter` (`crates/pictura-render/src/composite.rs`) only accepts version 2. Version 3 was deferred in archived `2026-09-19-photo-filter-adjustment-decode` (D3) because the repo lacked a confident XYZ→sRGB path. The same transform class already ships for Lab documents: `pictura-codec`'s `lab_to_rgb` applies a fixed D50 XYZ → linear sRGB matrix then `srgb_encode`.

psd-tools 1.19 (`psd_tools.psd.adjustments.PhotoFilter`) is the structural reference (confirmed live):

```
H   version          (2 or 3)
-- v3: 3I xyz        (big-endian u32 × 3)
-- v2: H color_space, 4H color_components
I   density          (u32)
B   luminosity       (u8)
    pad to 4-byte boundary
```

Adobe’s public File Formats table only says “4 bytes each for XYZ color (Only in Version 3)” — no numeric scale. ag-psd reads v3 as three `int32/100` and marks `// TODO: test this, this is probably wrong`; it never writes v3.

## Goals / Non-Goals

**Goals:**

- Decode a well-formed version-3 `phfl` into `Adjustment::PhotoFilter` so the layer renders like v2.
- Keep version-2 decode/encode byte-compatible with today.
- Prove layout + scale assumptions with unit tests (and a psd-tools write/read of a v3 block if easy).
- Never panic; reject malformed density/truncation as `None`.

**Non-Goals:**

- Authoring version 3 from the app (`encode_photo_filter` stays v2).
- Verified Adobe pixel parity (no CS6-authored v3 fixture in-tree).
- Colour-managed XYZ under an arbitrary document profile (profile-free, same class as Lab read).
- GPU Photo Filter shader (still CPU fallback).

## Decisions

### D1. Version-3 byte layout (grounded on psd-tools)

| Offset | Size | Field |
|---|---|---|
| 0 | 2 | version (`u16` = 3) |
| 2 | 4 | X (`u32` BE) |
| 6 | 4 | Y (`u32` BE) |
| 10 | 4 | Z (`u32` BE) |
| 14 | 4 | density (`u32`, percent `0..=100`) |
| 18 | 1 | luminosity (`u8`) |
| 19 | 1–3 | pad (ignored) |

Reading past the end → `None`. Density `> 100` → `None` (same as v2).

### D2. XYZ scale: s15Fixed16-style `÷ 65536`, D50 white

**Assumption (ponytail ceiling).** Interpret each `u32` as a 16.16 fixed-point value (`x / 65536.0`), matching ICC `XYZNumber` / Adobe’s usual 4-byte XYZ packing. Treat the triple as **CIE XYZ relative to D50** (same white as `lab_to_rgb`: Xn=0.96422, Zn=0.82521).

Rationale:

- Adobe’s table gives no scale; ag-psd’s Lab/100 guess is explicitly untrusted.
- 16.16 is the standard Adobe/ICC packing for XYZ stored in four bytes.
- Relative D50 matches the Lab path already in this repo and lcms2’s unoptimized transform class.

Ceiling comment in code: no CS6 v3 fixture proves the scale or white point; if a real file disagrees, only the scale constant / white adaptation changes, not the field layout.

### D3. XYZ → sRGB: reuse the Lab path’s matrix

Apply the same constant matrix `M` as `lab_to_rgb` (D50 XYZ → linear sRGB via Bradford D50→D65), then `srgb_encode`, then clamp to `0..=255` for `PhotoFilterParams.color`.

Preferred implementation: extract/expose a small `xyz_d50_to_srgb_u8([f64; 3]) -> [u8; 3]` next to `lab_to_rgb` in `pictura-codec` (or a private copy in `pictura-render` if a public export is not worth the API surface). **No new dependency.**

Negative / out-of-gamut linear components clamp to 0 (same as Lab). Density and luminosity map exactly as v2.

### D4. Encoder stays version 2

`encode_photo_filter` is unchanged. Open→save of a v3 layer therefore re-emits a v2 block with the decoded colour — consistent with how every other adjustment re-encodes from typed params. P2 opaque preservation still applies only when the payload is not re-encoded through the typed path.

### D5. Tests

1. `phfl_decodes_version_three`: hand-built v3 payload (known XYZ for a warm colour, density 25, luminosity 1) → `Some(PhotoFilter { … })`.
2. Malformed: truncated before luminosity, density 101, version 4 → `None`.
3. Remove v3 `None` asserts from `deferred_keys_still_none` / `phfl_decodes_version_two` (function may collapse if v3 was its only case).
4. Optional: psd-tools `PhotoFilter(version=3, xyz=(…))` written bytes match our field offsets (oracle self-skips if psd-tools missing — actually psd-tools is already used in-repo; a pure unit test on the struct layout is enough if oracle plumbing is heavy).
5. Composite: existing Photo Filter layer test continues to pass (path shared with v2 after decode).

## Risks / Trade-offs

- [Wrong XYZ scale or white point] → Marked `ponytail:`; layout still matches psd-tools; colour is best-effort until a CS6 v3 fixture exists. Mitigation: isolate scale in one constant.
- [v3→v2 save changes container version of that block] → Same as all other adjustment re-encodes; content (colour/density/flag) preserved in the typed model.
- [Out-of-gamut filter colour clips] → Same ceiling as Lab; document in ponytail comment.

## Open Questions

- None blocking. A real Photoshop-authored v3 `phfl` fixture remains a follow-up (roadmap G16-class manual baseline).
