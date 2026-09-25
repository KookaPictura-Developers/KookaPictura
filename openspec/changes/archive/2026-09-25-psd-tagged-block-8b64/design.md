# Design: psd-tagged-block-8b64

## Context

`read.rs` per-layer additional-layer-info loop:
```
let mut tag_sig = [0u8; 4];
tag_sig.copy_from_slice(er.take(4)?);
if &tag_sig != b"8BIM" {
    return Err(PsdError::Invalid("bad tagged block signature".into()));
}
```
`psd-tools.psd.tagged_blocks`:
- `TaggedBlock._SIGNATURES = (b"8BIM", b"8B64")` and
  `signature: bytes = field(default=b"8BIM", validator=in_(_SIGNATURES))`.
So `8B64` is a legitimate signature; psd-tools preserves and re-emits it.

The writer's `write_tag`/`write_tag_document` hardcode `8BIM`; the document-level
`reframe_document_extra` already *reads* both.

## Goals / Non-Goals

**Goals:**

- A valid `8B64` per-layer block no longer fails the read.
- `8BIM` behavior is byte-identical.

**Non-Goals:**

- Preserving the `8B64` signature on write (needs a model field; ceiling).
- Changing the blend-signature rule (still `8BIM`-only, per psd-tools
  `LayerRecord.signature` validator `in_((b"8BIM",))`).

## Decisions

### D1. Accept both signatures

Change the condition to:
```
if &tag_sig != b"8BIM" && &tag_sig != b"8B64" {
    return Err(PsdError::Invalid("bad tagged block signature".into()));
}
```
The loop already reads the key and length and stores unmodeled blocks in
`extra_blocks`; no other change.

### D2. Writer unchanged (normalization ceiling)

`write_tag` keeps emitting `8BIM` for stored blocks. The key and payload are
preserved; only the signature normalizes, as the document-level path already
does. Marked `ponytail:`.

### D3. Tests

- A synthetic layer record whose tagged block uses the `8B64` signature reads
  successfully (no error) and the block appears in `extra_blocks` with its key
  and payload.
- A signature that is neither `8BIM` nor `8B64` (e.g. `zzzz`) still errors.
- Existing `8BIM` round-trip tests are unchanged.

## Risks / Trade-offs

- [Normalization] → documented ceiling; the lossless re-emit needs a
  `LayerBlock.signature` field (45 construction sites) and is out of scope.
