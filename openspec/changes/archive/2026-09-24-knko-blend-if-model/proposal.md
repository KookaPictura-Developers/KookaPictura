# Proposal: knko-blend-if-model

## Why

Roadmap G6: blend-if and knockout remain unmodeled. The `knko` (knockout), `clbl` (blend clipped elements), and `infx` (blend interior elements) tagged blocks fall into `extra_blocks` opaque preserve; `blending_ranges` is kept as raw bytes only. The engine cannot read or edit advanced-blending options, and there is no typed Blend If view for a future compositor.

## What Changes

- Model **knockout** as a first-class `Layer.knockout: Knockout` (`None` | `Shallow` | `Deep`, values 0/1/2 per psd-tools). Consume `knko` on read (not stored twice in `extra_blocks`); write it back when non-default.
- Model **advanced-blending booleans** `blend_clipping` (`clbl`) and `blend_interior` (`infx`) as `bool` on `Layer` with CS6 defaults `true` (Photoshop omits the block when the option is at default). Consume on read; write when non-default.
- Model **Blend If** as a typed view of the layer-record `blending_ranges` field: composite gray source/destination ranges plus per-channel source/destination ranges (big-endian `u16` pairs). Keep raw bytes as the write source of truth when unchanged; expose `encode_blend_if` so an edited view can re-serialize.
- **No compositor change** in this slice: knockout punch-through and Blend If filtering stay out of scope (exact knockout math is still an open question in the roadmap).
- **BREAKING**: none for files without these tags; open→save of an unmodified document still re-emits original `blending_ranges` bytes and omits default advanced-blending tags the same way Photoshop does.

## Capabilities

### New Capabilities

- `psd-advanced-blending`: typed knockout mode, blend-clipping/interior flags, and Blend If ranges with read/write round-trip.

### Modified Capabilities

<!-- None: psd-opaque-preservation continues to cover other unknown keys; this change only lifts knko/clbl/infx and interprets blending_ranges. -->

## Impact

- `crates/pictura-core`: `Knockout`, `BlendIf` types; `Layer` fields + defaults.
- `crates/pictura-codec`: `read_layer_record` consume `knko`/`clbl`/`infx`; `write_extra` emit from model; `blending_ranges` parse + `encode_blend_if`.
- Tests: hand-built tagged PSD for knko/clbl/infx round-trip; blending-ranges parse/encode round-trip; default omission matches Photoshop.
- No new dependency; no app UI change required for this slice.
