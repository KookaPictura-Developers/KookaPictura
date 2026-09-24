# Design: type-layer-kind

## Context

`TySh` is already captured by the catch-all in `read.rs` into `Layer.extra_blocks` and re-emitted by `write_psd`. Nothing inspects it. `layer_kind_str` is group → adjustment → background → pixel. Photoshop type layers also carry raster channels, so canvas display already works; the gap is identity, panel filter, and default locks.

## Goals / Non-Goals

**Goals:**

- Kind `"type"` whenever `TySh` is present (presence-only; no payload parse).
- CS6 default locks (transparency + image) forced when `TySh` is present and those bits are clear.
- Kind filter includes Type; open→save still byte-preserves `TySh`.
- Unit tests without a font or a Photoshop-authored fixture.

**Non-Goals:**

- Parsing or rendering TySh text; Type tool; Character panel; forced locks on layers without TySh; changing `NodeKind` beyond the app-facing kind string.

## Decisions

### D1. Presence-only detection

`layer.extra_block(b"TySh").is_some()` is the type predicate. Payload shape is ungrounded and unused for this slice. Same pattern as other derived views (`smart_object`, `vector_mask`) but without a new `Layer` field — no model change.

### D2. Kind order

group → adjustment → background → **type** → pixel. A type layer is never a group or adjustment; background is checked first so a mis-named Background with TySh stays background only if it is actually flagged background (extremely rare; prefer type if both — **decision: type wins over background** only if we put type before background? Photoshop never marks a type layer Background. Keep: group, adjustment, type, background, pixel — **final: group, adjustment, background, type, pixel** so existing background tests stay green; type before pixel.)

### D3. Forced locks at read

When the layer’s tagged blocks include `TySh`, set `lock = lock.with(TRANSPARENCY, true).with(PIXELS, true)` after the lock parse (same place as the legacy transparency flag). Does not clear POSITION/NESTING. Ceiling: we do not clear bits Photoshop left clear on purpose for non-type reasons; we only force the two CS6 defaults.

### D4. Filter and panel

- `kKinds` gains `{"type", "Type"}`.
- `filter_from_kind` accepts `"type"` → family Kind bit (same mapping family as pixel).
- Icon: reuse the existing Type/`T` toolbox icon if `icons.h` exposes it; otherwise a text-free placeholder is out of scope — use the pixel-style kind icon only if no Type icon exists, with a `ponytail:` note in the task (prefer real Type icon if trivial).

### D5. Tests without a Photoshop TySh file

Synthetic `Layer { extra_blocks: [TySh …] }` unit tests for kind + locks + round-trip through `write_psd`/`read_psd` on a minimal document. No golden churn.

## Risks / Trade-offs

- [Consumers switch on kind strings] → Unknown `"type"` should be treated like `"pixel"` for pixel ops (locks still gate edits). Mitigation: `filter_from_kind` and docs updated; pixel-edit paths use `layer_pixel_locked`.
- [No fixture proves real TySh] → Presence-only; layout irrelevant until a later model change.

## Open Questions

- None blocking. Live text render remains a follow-up OpenSpec change.
