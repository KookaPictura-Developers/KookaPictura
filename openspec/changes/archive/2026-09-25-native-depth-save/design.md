# Design: native-depth-save

## Context

At read, `doc.composite` holds `color_channels` planes (1 Grayscale / 3 RGB) and
`doc.channels` holds the extras; `doc.source_planes.samples` holds **all** header
planes (color then extras), typed. `write_psd` re-emits a retained plane when
`retained.narrow_to_u8()` equals the current plane, else widens the current
plane. The app's `store_composite` re-expands an RGB composite to 4-plane RGBA
and does not touch `doc.channels`.

## Goals / Non-Goals

**Goals**

- A dirty 16/32-bit RGB/Grayscale layered document saves its composite color
  planes at source depth from `composite_native`, not widened 8-bit.
- Clean and out-of-scope saves are byte-identical to today.

**Non-Goals**

- Native layer-channel save (a moved/edited layer still widens; that is
  `depth-preserve`'s behavior).
- Converted-mode composite depth, HDR tone map, GPU.

## Decisions

**`refresh_native_composite(doc) -> bool` in `pictura-render`.** Gates:
`source_depth` is 16/32, `source_mode` is `None` (working RGB/Gray; a converted
Lab/CMYK store holds source-mode planes and must not be overwritten), and
`doc.layers` is non-empty (a no-layer document's merged composite is already
authoritative, and `composite_native` would composite an empty stack to
transparent). Returns `false` without mutation otherwise.

**Composite buffer.** Set `doc.composite` from `native.narrow_to_u8()` mirroring
`store_composite`: 4-plane RGBA for an RGB mode, 1-plane for Grayscale. This is
the same shape the app's refresh produces, so the writer's
`retained.narrow_to_u8() == current` check holds.

**Retained color planes.** Splice the native color planes into the first
`color_channels` planes of `source_planes.samples`, keeping planes
`color_channels..` (alpha/extras) byte-for-byte. Use `Samples::to_bytes` /
`from_bytes` at the plane offset (plane-major, `width*height` samples each), not a
new typed-vector API. Alpha/extras are intentionally not refreshed.

**App call site.** `save` runs the refresh only when `view.dirty`, then writes.
A clean save of an unedited file is untouched, preserving the byte-exact
round-trip the depth oracles assert.

## Risks / Trade-offs

- [Plane-offset splice on `f32`/`u16`] → offsets are `plane * sample_size`
  bytes; `from_bytes` re-decodes big-endian; test round-trips.
- [A dirty document that is semantically back to its original] → it still
  refreshes; only clean saves are guaranteed byte-identical, which matches the
  app's existing contract.
- [Composite section grows no new planes] → extras/alpha are preserved, so the
  channel count is unchanged.

## Migration Plan

Additive; revert is a revert.
