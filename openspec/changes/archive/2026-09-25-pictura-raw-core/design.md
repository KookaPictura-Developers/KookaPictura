# Design: pictura-raw-core

## Context

`CrsSettings` already carries the 11 PV2012 Basic controls from `crs:` XMP.
`SmartFilter` exposes one filter's `Fltr` descriptor bytes, and
`set_camera_raw_option` can replace one key in a preserved `SoLd`. On-disk
Photoshop compatibility requires the Camera Raw Filter's real id/name and the
real `Fltr` short keys, grounded on `assets/test_with_smart_object02.psd`.

## Goals / Non-Goals

**Goals**

- A typed, grounded read/write of the camera-raw `Fltr` Basic keys.
- A deterministic CPU pipeline for the 11 controls with a no-op default.
- An engine op that bakes the filtered source into the layer proxy and records
  the settings so they round-trip on save.

**Non-Goals**

- The UI/menu/dialog and C++ wiring (Batch 2).
- Document-level `FXid`/`FEid`/`FMsk` render caches and their channel data.
- Camera matrix/raw demosaic, Tone Curve/Detail/HSL tabs, local corrections.
- A compositor smart-filter pipeline (the apply model is bake-into-proxy).

## Decisions

**Thin settings alias.** `pub type PicturaRawSettings = CrsSettings` in `pictura-core`,
re-exported. Pictura Raw is an internal/API name; the struct stays single-sourced.

**Key map grounded on the CC fixture.** `Ex12` is a `Double`; `Temp`, `Tint`,
`Cr12`, `Hi12`, `Sh12`, `Wh12`, `Bk12`, `Cl12`, `Vibr`, `Strt` are `Long`.
Decode accepts either numeric type and ignores non-finite/unmodeled values.
Encode rounds integer-typed controls (Adobe's sliders are integers), so
`decode(encode(s)) == s` holds for integral slider values.

**Attach has three paths.** (1) an existing camera-raw filter is edited key by
key through the shipped `set_camera_raw_option`, preserving unmodeled keys such
as `Dhze`/`Crv `; (2) a preserved `SoLd`/`SoLE` without a camera-raw entry gets
one appended into its `filterFXList` (or a new `filterFX`), all other descriptor
keys preserved; (3) a converted embedded object with no preserved block gets
`SmartObject.smart_filters` set and the writer authors `filterFX` in the `SoLd`
it builds. `None` fields are not written, so they leave any existing key as-is.

**Renderer order and no-op.** White balance → exposure (linear-light EV gain) →
contrast (monotone S-curve) → highlights/shadows/whites/blacks (weighted tonal
lifts) → clarity (luminance detail against a separable box blur) → vibrance →
saturation (HSL). All-`None`/zero settings return the input unchanged, so the
default is byte-identical. Non-finite fields are treated as zero rather than
erroring.

**Engine bake.** `apply_pictura_raw` renders the embedded source into the layer rect via
the shipped `render_smart_source`, filters it, attaches the settings, then
rewrites the layer color channels and preserves an existing alpha channel. This
matches how Photoshop stores a smart-filtered object and needs no compositor
smart-filter pipeline.

## Risks / Trade-offs

- [Photoshop may expect `FEid`/`FMsk` for a live smart filter] → the baked proxy
  carries the visible pixels; the settings are preserved for re-edit. Marked a
  `ponytail:` ceiling; authoring the render caches is a follow-up.
- [Adobe's PV2012 curves are closed] → every stage is an approximation, marked
  inline; there is no pixel-parity claim.
- [Non-integral integer-typed control values round on encode] → documented; the
  UI slider keys are integers.

## Migration Plan

Additive; revert is a revert. No data migration: a filtered document is an
ordinary smart-object PSD.
