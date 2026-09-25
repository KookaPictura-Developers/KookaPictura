# Proposal: psd-tagged-block-8b64

## Why

The layer reader rejects any per-layer additional-layer-info block whose 4-byte
signature is not `8BIM` (`read.rs:792`, `PsdError::Invalid("bad tagged block
signature")`). But `8B64` is a valid tagged-block signature: `psd-tools`'
`TaggedBlock._SIGNATURES = (b"8BIM", b"8B64")`, and this codec already accepts
`8B64` for the document-level blocks (`reframe_document_extra`). A file carrying
an `8B64` per-layer block therefore fails to open though it is valid — a
roadmap-G1-class interop bug ("open any PSD").

## What Changes

- **Accept `8B64` as a per-layer tagged-block signature**, alongside `8BIM`.
  The block is read as usual and preserved; a bad signature is still an error.
- **The writer still normalizes an unmodeled block's signature to `8BIM`** —
  the existing opaque-preservation behavior; the block's key and payload are
  unchanged. (Lossless signature re-emission would need a model field and is
  left a marked ceiling.)
- **No behavior change for `8BIM` files.**
- **BREAKING**: none.

## Capabilities

### New Capabilities

<!-- None. -->

### Modified Capabilities

- `psd-layer-io`: a per-layer tagged block is accepted when its signature is
  `8BIM` **or** `8B64`, matching psd-tools.

## Impact

- `crates/pictura-codec/src/read.rs`: one signature condition + tests.
- No model change, no app UI, no byte-layout change for `8BIM` files.
- Ceiling: the `8B64` signature is not preserved on write (normalized to `8BIM`).
