# Design: pictura-raw-ui

## Context

`pictura-raw-core` already provides `pictura_render::convert_to_smart_object`,
`pictura_render::apply_pictura_raw`, and `pictura_codec::decode_pictura_raw_settings` /
`CAMERA_RAW_FILTER_ID`. The app already exposes the smart-object conversion command and
uses modal dialogs for one-shot numeric settings (`HdrConversionDialog`). This
change reuses those seams rather than adding a new engine path.

## Goals / Non-Goals

**Goals**

- One command that turns a raster layer into a smart object carrying the
  camera-raw filter, in exactly one undo state.
- A small modal dialog exposing only the 11 PV2012 Basic controls.
- Dialog prefill from an existing camera-raw filter so a re-edit starts from the
  stored values.

**Non-Goals**

- ACR tabs (Tone Curve, Detail, HSL, …), local corrections, filmstrip, histogram,
  or live preview.
- Document-level `FXid`/`FEid`/`FMsk` render caches.
- Group/nested or legacy `plLd` placed-layer targets (refused, no history).
- A full ACR workflow (`crs:` XMP, raw decode).

## Decisions

**Eleven values cross the bridge as scalars.** `apply_pictura_raw_filter` takes
the active layer path plus 11 `f64`s; `layer_pictura_raw_settings` returns the 11 stored
values as a space-separated `QString`, matching the shipped `layer_rect`
encoding. An absent camera-raw filter returns an empty string and the dialog
prefills zeros.

**Convert-then-apply in one bridge call.** When the target is a plain raster
pixel layer the bridge calls `convert_to_smart_object` first, then `apply_pictura_raw`;
when it is already an embedded smart object the conversion is skipped. One
`record("Pictura Raw")` follows a successful apply, so convert + filter is
a single undo step. Any engine refusal returns false and records nothing.

**Gating reuses existing predicates.** The command is enabled when the current
layer is either convertible to a smart object (`layer_can_convert_to_smart_object`)
or an embedded object to re-filter (`layer_can_replace_smart_object_contents`),
which is exactly `apply_pictura_raw`'s eligible set.

**Dialog ranges.** Temperature and Tint use the approximate JPEG ±100 scale,
Exposure ±5, and the other eight controls ±100, per
`docs/06-filters/camera-raw-filter.md`. Ranges not stated in the PDF are marked
with a `ponytail:` comment.

## Risks / Trade-offs

- [Sliders are approximated spin boxes] → each labelled control is a
  `QDoubleSpinBox`; a slider pair is deferred with the rest of the ACR chrome.
- [Integer-typed controls round on encode] → inherited from `pictura-raw-core`; the
  dialog uses integer steps for those controls.
- [A group or legacy placed layer is refused] → the bridge returns false without
  recording, matching `apply_pictura_raw`.

## Migration Plan

Additive; revert is a revert. No data migration: a filtered document is an
ordinary smart-object PSD.
