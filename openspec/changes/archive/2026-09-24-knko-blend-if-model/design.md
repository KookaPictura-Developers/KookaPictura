# Design: knko-blend-if-model

## Context

`read_layer_record` matches a fixed set of tagged keys (`luni`, `lspf`, `lclr`, `iOpa`, `lsct`, adjustment) and pushes everything else—including `knko`, `clbl`, `infx`—into `extra_blocks`. `blending_ranges` is already a `Vec<u8>` field re-emitted verbatim. `vector_mask` / `type_tool` show the derived-view pattern; `fill` (`iOpa`) shows the consume-and-write pattern for small scalars.

Grounding (Adobe File Formats + psd-tools 1.19):

```
knko  1 byte value + 3 pad   Knockout: 0=None, 1=Shallow, 2=Deep (psd-tools Knockout)
clbl  1 byte boolean + 3 pad Blend clipped elements (default true when absent)
infx  1 byte boolean + 3 pad Blend interior elements (default true when absent)

blending_ranges (layer record field, length-prefixed body already stripped into Vec<u8>):
  composite source:  4 bytes  (black u16 + white u16)
  composite dest:    4 bytes  (black u16 + white u16)
  then 8-byte channel groups until end of payload:
    channel source:  4 bytes
    channel dest:    4 bytes
  empty payload → no typed view (raw field still round-trips)
```

Adobe’s table labels `knko` a boolean; psd-tools and Photoshop 6+ use 0/1/2. We accept any byte and map 0/1/2, other bytes → `None` with the raw block preserved only if we do not consume it—see D1.

## Goals / Non-Goals

**Goals:**

- First-class `knockout` / `blend_clipping` / `blend_interior` on `Layer`.
- Typed `BlendIf` view from `blending_ranges`; raw bytes remain write source when unchanged.
- `encode_blend_if` for re-emitting an edited view.
- Round-trip tests; default documents still serialize without the new tags.

**Non-Goals:**

- Compositor knockout or Blend If filtering.
- UI.
- Changing how other unknown keys are preserved.

## Decisions

### D1. Consume `knko`/`clbl`/`infx` (fill pattern), not derived-view

Unlike `TySh` (large opaque payload), these are 1-byte scalars. Consume into fields on read (remove from the `_ => extra_blocks` push), write from the fields on save. That matches `iOpa` and makes a future UI edit a field assignment with no block surgery.

Malformed payloads (empty `data`): leave the field at its default and still consume the key (do not fail the document; do not re-emit a corrupt block).

### D2. Defaults and write gating

| Field | Default (absent block) | Write when |
|-------|------------------------|------------|
| `knockout` | `Knockout::None` | `!= None` |
| `blend_clipping` | `true` | `== false` |
| `blend_interior` | `true` | `== false` |

Payload on write: `[value, 0, 0, 0]` (same 4-byte framing as `iOpa`).

### D3. Blend If: derived view + raw write source

`Layer.blending_ranges` stays the serialization source. On read (after the field is filled), parse into `Layer.blend_if: Option<BlendIf>`; empty payload → `None`; a body that is not a multiple of 8 bytes (or is truncated mid-group) → `None` while raw bytes still round-trip. Non-empty well-formed bodies decode composite source/destination pairs then zero or more channel groups (big-endian `u16`).

`encode_blend_if` rebuilds the body (composite + N channel groups). Unmodified open→save never calls it. An edited view that is written back replaces `blending_ranges` with `encode_blend_if` output—optional in this slice; the function must exist and be tested.

### D4. No compositor work

Knockout punch-through and Blend If source/destination filtering are deferred. Roadmap G6’s “blend-if and knockout remain” closes for *modeling*; render remains a later change with its own oracle story.

### D5. Hand-built fixtures

Use `tagged_layer_psd` for `knko`/`clbl`/`infx`. For `blending_ranges`, extend the helper or build a one-layer PSD that writes a non-empty ranges body (the current helper hard-codes zero length).

## Risks / Trade-offs

- [Adobe “boolean” vs psd-tools 0/1/2] → Accept 0/1/2; map unknown → `None` (ceiling: future modes).
- [Default `clbl`/`infx` = true] → Matches CS6 Help; files that omit the block keep that default. Write only non-defaults so existing fixtures stay byte-identical.
- [Blend If body truncation] → View `None`, raw preserved; no document failure.
