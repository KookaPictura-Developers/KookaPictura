# Proposal: bit-depth-sample-model

## Why

`docs/01-architecture/document-model.md:198` contracts that pixels are stored
at the document bit depth as `u8`, `u16`, or `f32` per channel, but the engine's
`PixelBuffer` is a hard-wired `Vec<u8>` and the codec's retained native samples
(`SourcePlanes`/`SourceChannels`) are untyped raw bytes. A 16/32-bit document is
therefore edited at 8-bit precision and, on any edit, its native low bits are
lost to widening. This foundation change introduces the sample-typed
representation that later phases need to edit at native depth.

## What Changes

- `PixelBuffer` becomes sample-generic: `PixelBuffer<T = u8>`. Every existing
  `PixelBuffer` use means `PixelBuffer<u8>` and is behavior-unchanged, so the
  ~900 existing 8-bit access sites need no rework.
- The codec's retained native sample store becomes typed instead of raw bytes:
  `SourcePlanes`/`SourceChannels` carry `u16`/`f32` samples (or `u8` for the
  8-bit Lab store) rather than the PSD byte image, decoded on read and encoded on
  write.
- Add the small conversion helpers the phases share: sample widening/narrowing
  between `u8`, `u16`, `f32` per the `psd-bit-depth` rules.
- No pixel-behavior change: an unchanged 16/32-bit document still round-trips
  byte-for-byte; an edited one still widens as today. Native-depth *editing* is
  Phase 2 and is explicitly out of scope here.

## Capabilities

### New Capabilities

- `bit-depth-sample-model`: the sample-typed storage primitives and their
  conversion rules.

### Modified Capabilities

<!-- None. Existing psd-bit-depth requirements (narrowing, byte-exact
     round-trip, widening on edit) keep their behavior; this change only
     changes how the retained samples are represented in memory. -->

## Impact

- `crates/pictura-core/src/lib.rs`: `PixelBuffer<T = u8>`, `Samples` (any-depth
  storage), typed `SourcePlanes`/`SourceChannels`, conversion helpers.
- `crates/pictura-codec/src/{read,write,color_mode}.rs`: decode native samples
  into the typed store, encode back out; narrowing/widening use the helpers.
- `crates/pictura-render/src/document_ops/{resize,orient,transform}.rs`: only
  the places that read/write the retained store.
- Tests: existing `depth_oracle`/`color_mode_oracle`/`depth`/`color_modes` stay
  green; add a unit test for byte-exact typed round-trip and the converters.
- No new dependency, no app UI change, no spec-visible behavior change.
