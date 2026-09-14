## Context

The document model, codec, and compositor existed (`m1`–`m3`) but had no notion
of an adjustment layer: `Layer` had no adjustment field, the codec skipped the
adjustment additional-layer-info block, and the compositor only handled pixel
and group layers. `pictura-adjust` already provided the destructive adjustment
kernels and typed `Adjustment` enum from `m4-image-adjustments`.

Two crate-layering facts constrain the design. `pictura-core` sits at the bottom
of the dependency graph and must stay free of `pictura-adjust`; `pictura-codec`
depends only on `pictura-core`; `pictura-render` is the first crate allowed to
depend on `pictura-adjust`. The PSD format stores adjustment parameters as raw
additional-layer-info blocks keyed by four bytes (`brit`, `levl`, `nvrt`, …),
and Photoshop writers include fields a partial reader may not understand.

## Goals / Non-Goals

**Goals:**

- Carry an adjustment through the model as opaque raw bytes so `pictura-core`
  needs no adjustment dependency.
- Round-trip every recognised adjustment key and its payload byte-for-byte
  through `read_psd`/`write_psd`.
- Decode a practical subset into destructive adjustments in `pictura-render` and
  apply them to the accumulated backdrop, gated by mask/opacity/blend.
- Treat unknown or undecodable adjustments as a preserved no-op, never an error.

**Non-Goals:**

- Full descriptor coverage for every adjustment type.
- Adjustment-layer clipping subtleties and `Pass Through` group semantics.
- Decoding Gradient Map (`grdm`), Selective Color (`selc`), Color Lookup
  (`clrL`), Curves (`curv`), Exposure (`expA`), Vibrance (`vibA`), Black & White
  (`blwh`), Photo Filter (`phfl`), or Channel Mixer (`mixr`).
- Claiming pixel parity with Photoshop's closed algorithms beyond the tolerance
  the oracle tests actually prove.

## Decisions

**Opaque bytes in the model, decode in the renderer.** `Layer.adjustment` is
`Option<AdjustmentData>` where `data: Vec<u8>` is the raw PSD payload. Alternative
considered: a typed `Adjustment` in `pictura-core`. Rejected because it forces a
`pictura-core → pictura-adjust` dependency and leaks adjustment semantics into
the model; the opaque form also preserves unknown fields by construction.

**Codec stores the block verbatim with a key whitelist.** `is_adjustment_key`
recognises the 17 keys Photoshop writes (including the `hue ` legacy alias and
`invr`) and copies the tagged block's key and bytes into `AdjustmentData`; the
writer emits the same key and bytes in the same tagged-block slot. Alternative
considered: decode payloads in the codec. Rejected — the codec has no
`pictura-adjust` dependency and any decode would be lossy on save.

**Renderer decodes a subset; failures are `None`, not `Err`.** `decode_adjustment`
maps `nvrt`/`invr`, `post`, `thrs`, `brit`, `hue2`/`hue ` (and `levl`) to typed
adjustments, validating ranges; anything else returns `None`. Keeping the decode
total and fallible-to-`None` lets unsupported files open and re-save cleanly,
which a `Result` would break.

**Apply-to-backdrop compositing.** `composite_adjustment` transforms the running
backdrop into a fresh buffer via `pictura_adjust::apply`, then blends the adjusted
colors back into the canvas through the existing `blend_into` path using the
layer's mask, opacity, and blend mode. This mirrors Photoshop (adjust the
backdrop, blend the adjusted result back) and reuses the compositor's existing
gate logic instead of adding a parallel one. Source coverage is the backdrop's own
alpha, so transparent backdrop pixels gain no content.

## Risks / Trade-offs

- **Approximation, not parity.** Adjustment math is a reimplementation; only the
  Invert-over-pixel case is asserted against a flattened oracle (±1). Other keys
  are covered structurally, not pixel-exactly. Mitigation: keep parity claims
  scoped to tested cases and mark unsourced algorithms.
- **Silent no-op for unsupported adjustments.** An undecodable layer looks like
  nothing happened in the viewport. Mitigation: bytes survive save, and the model
  keeps the key so a later decoder can pick it up; document the ceiling.
- **Whitelist drift.** A key Photoshop writes but this list omits is treated as an
  unknown block and dropped on read. Mitigation: the list mirrors the PSD spec and
  the psd-tools fixtures; extend it with a test when a new key appears.
- **Byte-format divergence from psd-tools.** Encoders hand-build payloads; a
  layout error would only surface against real files. Mitigation: the
  `adjustment.psd` oracle fixture authored by psd-tools checks key and payload
  bytes.
- **Levels is decoded but not required.** `levl` is included for usefulness and
  will no-op if its version or ranges are unexpected. Mitigation: range checks
  return `None` rather than guessing.
