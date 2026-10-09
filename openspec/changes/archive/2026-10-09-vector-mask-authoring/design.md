# Design

## Context

See proposal.md — Why. The relevant current state:

- `VectorMask` lives in `pictura-core` (`subpaths`, `invert`, `disabled`, and a
  `rows` sampling cache). The raw `vmsk` block stays in `Layer::extra_blocks`
  and is re-emitted verbatim; the view is rebuilt from the block on read
  (`pictura_codec::resolve_vector_masks`).
- `pictura_codec::encode_vector_mask(subpaths, width, height)` authors a
  version-3 block with a zero flag word; `decode_vector_mask` reads bit 0 as
  invert and bit 2 as disabled.
- The compositor's `vector_mask::coverage` samples the document-relative path
  (even-odd / non-zero) and is permissive (255) for an absent, disabled, or
  all-open mask.
- `shape_layer::set_layer_shape_paths` already authors a `vmsk` block, but from
  editable shape subpaths — it re-encodes, which is lossy for an imported mask's
  cubic handles.
- The Adobe PSD spec gives the vector-mask flag word as bit 1 invert, bit 2
  not-link, bit 3 disabled (one-indexed).

## Goals / Non-Goals

**Goals:**

- One engine module of pure `&mut Document` operations covering add, delete,
  enable, link, rasterize, and the read predicates.
- Preserve an existing vector mask's geometry byte-for-byte across
  enable/disable and link/unlink.
- An app bridge mirroring `layer_masks.rs`, with one history state per mutation.
- Vector-mask row indicators and a Properties section.

**Non-Goals:**

- No vector-mask density/feather/invert authoring (the model stores only invert
  and disabled; the Help scopes Invert to layer masks).
- No editing of the vector-mask path itself (the Pen/Direct Selection tools own
  that elsewhere).
- No new PSD block or dependency.

## Decisions

### D1. Operations live in `layer_ops/vector_masks.rs`

Mirrors `layer_masks.rs`: free functions over `&mut Document`, exported from
`layer_ops/mod.rs`, `document_ops/mod.rs`, and the crate root. `VectorMaskKind`
is a plain enum. `add_vector_mask` returns `bool` like `add_layer_mask`.

### D2. Flag word bits and in-place patching

The PSD vector-mask flags word is version-relative at bytes `4..8` of the block.
Constants:

- `VECTOR_MASK_FLAG_INVERT = 0x01`
- `VECTOR_MASK_FLAG_NOT_LINKED = 0x02`
- `VECTOR_MASK_FLAG_DISABLED = 0x04`

Enable/disable and link/unlink **do not re-encode**: they rewrite only the
relevant bit in the existing block's flag bytes and re-decode the view from it,
so an imported mask's cubic handles and precision survive. A newly authored mask
sets its flags by patching the zero word `encode_vector_mask` writes.

Alternative considered: extend `encode_vector_mask` to take flags. Rejected for
enable/link — re-encoding from the *decoded, flattened* view would corrupt an
imported mask. The patch path is the one that keeps PSD round-trip exact.

### D3. Reveal All / Hide All geometry

Reveal All authors a closed rectangle subpath over the layer's content rect
(`layer.rect`) with no flags. Hide All authors the same rectangle with the invert
flag, so the compositor shows nothing inside it. This mirrors `add_layer_mask`'s
choice of `layer.rect` and needs no document-sized path.

### D4. Current Path from the work path

Current Path clones `doc.work_path.subpaths` (already document pixels, `f64`)
and encodes them. It is refused, returning `false`, when the work path is empty
or the target already has a vector mask. Saved paths are not yet selectable as
the "current path"; the work path is the document's current path.

### D5. Rasterize folds coverage into a layer mask

`rasterize_vector_mask` samples the effective vector coverage (via
`crate::vector_mask::coverage`, so a disabled mask contributes 255) and the
effective existing layer-mask coverage (via `crate::composite::mask_value`, also
255 when disabled) over `layer.rect`, multiplies them rounded to nearest, stores
the product as the layer's `LayerMask` (linked, enabled), and drops the `vmsk`
block. This matches LAY-005's "both masks multiply" and is exact for the
composite.

### D6. Bridge mirrors `layer_masks.rs`

A new `impl_layers/vector_masks.rs` with its own `#[cxx_qt::bridge] pub mod ffi`
and free functions over `Pin<&mut PictureView>`, resolving the active layer from
`PictureViewRust::active_layer` and routing mutations through `batch_changed`
(one path) so each recomposites and records one state. Added to the `mod` list
in `impl_layers.rs` and to the cxx-qt bridge file list in `build.rs`.

### D7. History labels

"Add Vector Mask", "Delete Vector Mask", "Enable/Disable Vector Mask",
"Link/Unlink Vector Mask", "Rasterize Vector Mask".

### D8. Row indicators and Properties section

The delegate draws the vector-mask thumbnail to the left of the raster-mask
thumbnail (at the right edge when no raster mask exists), each with its own link
glyph and disabled red-X. New model roles `HasVectorMaskRole`,
`VectorMaskThumbnailRole`, `VectorMaskLinkedRole`, `VectorMaskDisabledRole`
carry per-row state from bridge reads. `Shift`-clicking the vector thumbnail
toggles enabled; clicking its link glyph toggles link. The Properties panel adds
a Vector Mask section shown when the active layer has a vector mask.

### D9. Vector-mask thumbnail is an approximation

`vector_mask_thumbnail_image` rasterizes the mask's coverage (via
`VectorMask::inside` plus `invert`) at the thumbnail resolution, rendering the
same white-shows / black-hides convention as the raster-mask thumbnail.
`ponytail:` CS6 strokes the path outline rather than filling coverage; upgrade to
an outline render if the filled form proves confusing.

## Risks / Trade-offs

- **A shape layer already carries a `vmsk`**, so it now shows a vector-mask
  thumbnail and its `Layer ▸ Vector Mask` add/current-path commands are refused
  (a vector mask already exists). This matches CS6's single-vector-mask-per-layer
  rule for this codebase's storage.
- **Rasterize drops resolution independence**, matching CS6; the spec says so.
- **Flag patching relies on the `4..8` layout** — the codec owns that layout and
  the engine asserts a block of at least eight bytes before patching.

## Open Questions

- Vector-mask **density and feather** (the "user mask parameters" PSD block) are
  not parsed; the Properties section shows them as disabled placeholders, as the
  layer-mask section does.
- Whether the vector mask's **path** should be editable from the Paths panel is
  deferred; this change does not touch the Paths panel.
