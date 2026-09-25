# Design: bit-depth-sample-model

## Context

`PixelBuffer` is the engine's planar image buffer (`width`, `height`,
`channels`, `data: Vec<u8>`). It is used in ~81 files with ~930 direct `data`
index sites, all 8-bit. The codec retains native-depth source samples for a
lossless 16/32-bit save in `SourcePlanes` (composite + document extra channels)
and `SourceChannels` (per layer), both as raw `Vec<u8>` in PSD byte order.

The contract (`docs/01-architecture/document-model.md`) calls for samples stored
at the document depth as `u8`/`u16`/`f32`, and makes a tiled store an "inferred"
preference, not a requirement. This change is the representation step; a tiled
store is not attempted.

## Goals / Non-Goals

**Goals:**

- A sample-typed buffer that existing 8-bit code adopts with no behavior change.
- Typed native-sample retention, decoded on read and encoded on write, so later
  phases can edit at depth.
- Byte-exact round-trip for unchanged 8/16/32-bit documents (unchanged from
  today).

**Non-Goals:**

- No native-depth *editing* (ops stay 8-bit; Phase 2).
- No `f32`/HDR tone map, no tiled store, no GPU, no threading (Phase 3+).
- No UI change; no spec-visible behavior change.

## Decisions

**`PixelBuffer<T = u8>` (generic with a default), not a runtime enum on
`data`.** A default type parameter keeps every `PixelBuffer` in type position
meaning `PixelBuffer<u8>` and every `buf.data[i]` an 8-bit access, so existing
code compiles unchanged. A runtime enum `{ U8, U16, F32 }` on `data` would break
all ~930 access sites and force conversion at each — rejected. A generic lets a
later phase introduce `PixelBuffer<u16>` for the working composite incrementally,
operation by operation.

**Typed native store, not raw bytes.** `SourcePlanes`/`SourceChannels` hold
`Samples` (an enum `{ U8(Vec<u8>), U16(Vec<u16>), F32(Vec<f32>) }`) plus the
existing `depth`/dims/rect. Read decodes the PSD byte image into samples; write
re-encodes with the depth-specific rule. This removes the implicit "the bytes are
in PSD order" invariant from the store and makes the native values the source of
truth. Byte-exactness holds because encode(decode(bytes)) is the identity for
big-endian `u16` and IEEE-754 big-endian `f32`.

**Conversions live in one place.** `narrow` (`u16 -> u8` = `>>8`; `f32 -> u8` =
`clamp(trunc(v*256),0,255)`) and `widen` (`u8 -> u16` = `v*257`; `u8 -> f32` =
`v/255`) are public helpers on `Samples`, replacing the copies scattered in
read/write, so Phase 2's op ports share them.

## Risks / Trade-offs

- [A generic default can still require a turbofish where inference is weak
  (e.g. `let b = PixelBuffer::new(..)` with no 8-bit use)] → add `::<u8>` at
  those few sites; the compiler names them.
- [Converting raw byte stores to typed values could break the byte-exact depth
  oracle] → the oracle (`depth_oracle`, `color_mode_oracle`) is the regression
  gate and runs unchanged.
- [Foundation churn with no user-visible change] → it is the prerequisite the
  user chose; Phase 2 delivers the visible 16-bit editing gain.

## Migration Plan

Land Phase 1 as a behavior-preserving refactor. Rollback is a `git revert`.
Later phases are separate changes: Phase 2 ports the adjustment/ops pipeline to
`PixelBuffer<u16>` for 16-bit documents; Phase 3 adds `f32`/HDR and the GPU
path.
