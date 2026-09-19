## 1. Render

- [x] 1.1 Add `pictura-codec = { path = "../pictura-codec" }` as a runtime dependency in `crates/pictura-render/Cargo.toml`; remove the now-redundant dev-dependency entry.
- [x] 1.2 In `composite.rs`, add a smart-object branch reached only when the layer has an `Embedded` smart object with a non-empty payload and no channel `id` 0. A layer with channel `id` 0 keeps the existing raster path unchanged.
- [x] 1.3 Decode the payload with `pictura_codec::read_psd`; use the decoded document's stored merged composite when it is usable (RGB or grayscale, non-zero size), otherwise composite the decoded document's layers.
- [x] 1.4 Sample the source nearest-neighbour into the layer's rect, clamped to the canvas, and feed each sample through `blend_into` so opacity, fill, mask, and blend apply as for a raster layer.
- [x] 1.5 Treat `External`/`Alias`/`Unresolved`, an empty payload, and a decode failure as a no-op; no error, no panic. Mark the `Trnf`/warp and bilinear ceiling with a `ponytail:` comment.

## 2. Tests

- [x] 2.1 Author a document in memory with one layer whose `SmartObject` payload is a written solid-colour document and whose layer has no color channel; assert `composite_rgba` yields that colour inside the layer rect and leaves the rest unchanged.
- [x] 2.2 Assert a layer with a color channel renders from the channel and never from a payload that would decode to a different colour.
- [x] 2.3 Assert `External`, `Alias`, and `Unresolved` kinds each leave the backdrop unchanged without error.
- [x] 2.4 Assert an empty payload and a garbage payload each leave the backdrop unchanged without error.
- [x] 2.5 Assert a source smaller and larger than the layer rect is scaled into the rect by nearest-neighbour (spot-check corner samples against the expected source pixel).
- [x] 2.6 Assert a decoded document with no stored merged composite falls back to compositing its layers.

## 3. Gates

- [x] 3.1 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run --workspace`.
- [x] 3.2 `bash scripts/verify-full.sh` and a headless self-test; record counts.
- [x] 3.3 `openspec validate smart-object-source-render --strict` and `openspec validate --all --strict`.
- [x] 3.4 Update `docs/dev/STATE.md` and `docs/dev/psd-support-roadmap.md` (smart-object rendering moves out of the render phase's open work); commit with `TASK-ALLOWS-DOCS`.
