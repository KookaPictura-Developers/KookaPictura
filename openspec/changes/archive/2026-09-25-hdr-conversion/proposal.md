# Proposal: hdr-conversion

## Why

The roadmap's P4/G4 item "32-bit HDR tone map" is open because the 32-bit HDR
Conversion path was never wired: `pictura_adjust::hdr_toning::exposure_gamma`
(the documented CS6 *Image > Mode > 32 -> 16/8* "Exposure & Gamma" operator)
ships but has no caller, and `Image > Mode > 16 Bits/Channel` and
`8 Bits/Channel` are inert `leaf()` menu stubs.

## What Changes

- Add `pictura_render::document_ops::convert_depth_exposure_gamma(doc, out,
  params)`: a real 32 -> 16/8 conversion for a native Grayscale/RGB document
  that runs the Exposure & Gamma operator on the retained `f32` color planes,
  quantizes to the output depth, rebuilds the retained source store, and
  updates `doc.depth`/`source_depth`/`source_planes`/`composite` so a save
  re-emits the converted depth.
- Add `Samples::to_u16` (the missing `f32` -> `u16` quantization, mirroring
  `narrow_to_u8`).
- Add the app command `Image > Mode > 16 Bits/Channel` / `8 Bits/Channel`
  (real ids, handlers, enabled only for a 32-bit document), a minimal
  `HdrConversionDialog` with only Exposure (EV) and Gamma, and a
  `convert_depth` bridge call.
- The other three CS6 HDR methods (Local Adaptation, Equalize Histogram,
  Highlight Compression) and their Method combo stay deferred (closed
  kernels); only Exposure & Gamma is grounded in the spec.

## Capabilities

### New Capabilities

- `hdr-conversion`: the 32 -> 16/8 Exposure & Gamma conversion, its dialog,
  and the `Image > Mode` bit-depth commands that invoke it.

### Modified Capabilities

<!-- None: `hdr-toning` (the operator itself) is unchanged. -->

## Impact

- New `crates/pictura-render/src/document_ops/depth.rs` and its re-exports.
- `crates/pictura-core/src/samples.rs`: `Samples::to_u16`.
- App: `commands.h`, `command_tree.cpp`, `frame_menus.cpp`, a new
  `hdr_conversion_dialog.{h,cpp}` + `CMakeLists.txt`, and a `convert_depth`
  bridge in `impl_transform/dispatch.rs` + `cxxqt_object.rs`.
- Tests: render unit tests, a codec round-trip test, and a C++ self-test check.
- No new external dependency.
