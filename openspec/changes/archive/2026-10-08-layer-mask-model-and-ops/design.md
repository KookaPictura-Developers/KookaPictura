# Design

## Context

See proposal.md — Why. The relevant current state:

- `LayerMask` lives in `pictura-core` (`rect`, `default_color`, `disabled`,
  `flags`, `data: Option<Plane<u8>>`, `extra`). `pictura-render`'s compositor
  reads it through `composite::mask_value` / `raster_mask_alpha`; a disabled or
  absent mask contributes 255. The codec read/write round-trips the block and the
  `-2` mask channel, and write derives the disabled bit from `.disabled`.
- Layer edit operations are free functions over `&mut Document` in
  `document_ops/layer_ops/`, re-exported through `document_ops` and the
  `pictura_render` crate root. `resolve_path_mut` targets a layer by panel path.
- The app's `selection_to_mask` already builds a full-frame `LayerMask` whose
  `data` is the selection coverage, but the engine must not depend on app types.
- The smart-object predicate is `layer.smart_object.is_some()`.

## Goals / Non-Goals

**Goals:**

- One engine module of pure `&mut Document` operations covering create, delete,
  apply, enable, link, and the read predicates.
- Reuse the compositor's own mask sampling so apply matches the compositor.
- An app bridge that mirrors `clipping.rs`, with one history state per mutation.

**Non-Goals:**

- No C++/Qt panel or `CMakeLists.txt` work (the follow-up `layer-mask-ui`).
- No mask density/feather/invert authoring; no new PSD block.
- No vector-mask or smart-filter-mask operations.

## Decisions

### D1. Operations live in a new `layer_ops/layer_masks.rs`

Mirrors `clipping.rs`: a module of free functions, exported from
`layer_ops/mod.rs`, `document_ops/mod.rs`, and the crate root. `add_layer_mask`
returned `bool` per the requested API; the selection variants take
`Option<&LayerMask>` so the engine stays app-free. `LayerMaskKind` is a plain
enum in the same module.

Alternative considered: a struct of methods on `LayerMask`. Rejected — the
existing layer operations are free functions over the document, and matching
that keeps call sites uniform.

### D2. Mask geometry and out-of-rect default color

Reveal All / Hide All / From Transparency size the mask to the layer's content
rect (`layer.rect`); the selection variants size it to the supplied selection's
rect. Coverage inside the rect is 255 (reveal) or 0 (hide); `default_color`
carries the same value outside the rect. `mask_value` already falls back to
`default_color` outside the rect and for an empty rect, so a group or an empty
layer still yields a well-defined reveal/hide.

Alternative considered: build a full document-sized mask for every kind. Rejected
— it wastes memory and diverges from PSD, which stores the mask at the content
rectangle.

### D3. Apply reuses `composite::mask_value` and folds rounded alpha

`apply_layer_mask` samples `crate::composite::mask_value` at each pixel of the
layer rect, so a disabled mask is a no-op on pixels (the compositor also ignores
it) and out-of-rect pixels use `default_color`. Existing alpha is multiplied with
round-to-nearest (`(a * m + 127) / 255`) and the mask is cleared. A layer with no
alpha channel gets one seeded at 255 before folding.

Alternative considered: reimplement the mask math. Rejected — a second copy could
drift from the compositor.

### D4. Smart-object refusal

`apply_layer_mask` returns `false` before touching the mask when
`layer.smart_object.is_some()`, matching CS6.

### D5. Link flag is bit 0, other bits preserved

A `MASK_FLAG_LINKED: u8 = 0x01` constant (the PSD "position relative to layer"
bit) is the typed accessor. `set_layer_mask_linked` sets or clears only that bit;
`layer_mask_linked` reads it. `.disabled` stays the model field for bit 1 (write
overlays it), so `flags` keeps round-tripping whatever an imported file held.

### D6. Bridge mirrors `clipping.rs`

A new `cxxqt_object/layer_masks.rs` with its own `#[cxx_qt::bridge] pub mod ffi`
and free functions over `Pin<&mut PictureView>`. It resolves the single active
layer from `PictureViewRust::active_layer`, builds the selection coverage with
`helpers_composite::selection_to_mask`, and routes mutations through the existing
`batch_changed` (one path) so each recomposites and records one state. The module
is added to `cxxqt_object.rs`'s `mod` list and to the cxx-qt bridge file list in
`build.rs` (that is the Rust build script, not `CMakeLists.txt`).

### D7. History labels

"Add Layer Mask", "Delete Layer Mask", "Apply Layer Mask", "Enable Layer Mask" /
"Disable Layer Mask", "Link Layer Mask" / "Unlink Layer Mask", matching the
existing action-label convention.

## Risks / Trade-offs

- **Alpha rounding drift between apply and the float compositor** → folding
  rounds to nearest, and the apply test asserts the before/after composite within
  a one-unit-per-channel tolerance.
- **New masks must serialize** → a created mask has `data`; the codec already
  encodes a `-2` channel from `mask.data`, covered by an existing round-trip
  suite, and the engine tests assert the in-session composite.
- **A group or empty layer has an empty content rect** → covered by D2's
  `default_color` fallback rather than special-casing groups.

## Open Questions

- Mask **density and feather** (and their "user mask parameters" PSD block) are
  not parsed today; the `extra` tail is preserved verbatim but not interpreted.
  Exposing/authoring them needs that block identified and is deferred to the
  follow-up `layer-mask-ui` change (or a codec change before it).
- Whether Reveal Selection / Hide Selection should be *refused* or fall back to
  reveal-all / hide-all when no selection exists. Chosen: refuse (spec), which is
  the Photoshop menu's enabled/disabled behavior; the bridge supplies the
  selection when one exists.
