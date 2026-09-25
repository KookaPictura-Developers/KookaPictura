# Design: native-depth-layer-content

## Context

`composite_pixels` reads `channel(layer, id)` (8-bit planes) and normalizes by
`/255.0`. `Layer.source_channels` (a `SourceChannels { depth, rect, planes:
Vec<(i16, Samples)> }`) holds the retained native samples for a 16/32-bit read.
For RGB/Grayscale those planes are the working-space color planes (ids
`0..color_channels`) and `-1` transparency; for a converted Lab/CMYK read they
are the source-mode planes, so they must not be treated as RGB.

## Goals / Non-Goals

**Goals**

- Compose a high-depth RGB/Grayscale pixel layer from its native samples.
- Keep the 8-bit path and every existing result byte-identical.

**Non-Goals**

- Native masks (the `-2` plane is native but `mask_alpha` stays 8-bit).
- Group/clipping semantics changes, app wiring, HDR, GPU.

## Decisions

**Gate on `source_depth.is_some() && source_mode.is_none()`.** `source_mode` is
`Some` exactly for a mode the read normalized (Lab/CMYK/Duotone/Multichannel), so
this admits only high-depth Grayscale/RGB, where `source_channels` planes are the
working-space color. The store's `depth` must also equal the document source
depth and its `rect` the layer rect.

**Per-channel lookup with fallback.** A helper returns a channel's unit-domain
`f32` value by id when the plane exists and is long enough, else falls back to
the 8-bit `sample(ch, li) / 255.0` for that channel. Missing alpha keeps the
existing `255` default.

**File cap.** `composite.rs` is at its allowlist ceiling (1221, after
`native-depth-composite` shrank it from 1275) and only shrinks. If the branch
would exceed it, move `composite_pixels` (and the native helper) into
`composite_native.rs` and publish the few items it needs (`channel`, `sample`,
`blend_into`, `Canvas`) as `pub(crate)`. Lower the allowlist entry if
`composite.rs` shrinks.

## Risks / Trade-offs

- [A converted-mode document could composite source-mode planes as RGB] → gated
  on `source_mode.is_none()`; tested.
- [Native and 8-bit content differing at the seam] → 8-bit docs never take the
  branch; the existing suite is the gate.
- [Per-pixel native lookup cost] → indexed planar slices, comparable to the
  existing `sample` calls; no allocation.

## Migration Plan

Additive; revert is a revert. Native masks and app wiring are later.
