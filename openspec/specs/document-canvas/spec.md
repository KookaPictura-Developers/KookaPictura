# document-canvas Specification

## Purpose
TBD - created by archiving change m12-document-ops. Update Purpose after archive.
## Requirements
### Requirement: Document canvas resize entry point and error contract

The system SHALL provide `pictura_render::resize_canvas_document(doc: &mut Document, width: u32, height: u32, anchor: Anchor) -> Result<(), OpsError>`. On success it SHALL set `doc.width` to `width` and `doc.height` to `height` and return `Ok(())`, performing all validation before mutating `doc`. A `width` or `height` below 1 SHALL be rejected with `OpsError::InvalidParams`, and a malformed pixel-layer channel or decoded mask SHALL likewise be rejected before mutation. On any error `doc` MUST be left bit-identical to its state before the call. The function MUST NOT panic for any input, including 1×1 documents and empty layer stacks.

#### Scenario: A successful canvas resize sets the document dimensions

- **WHEN** `resize_canvas_document` is called with `width` and `height` of 1 or greater
- **THEN** it returns `Ok(())` and `doc.width` and `doc.height` equal the requested values

#### Scenario: Invalid dimensions are rejected and the document is untouched

- **WHEN** `resize_canvas_document` is called with `width` 0 or `height` 0
- **THEN** it returns `OpsError::InvalidParams` and every field of `doc` equals its pre-call value

#### Scenario: A malformed layer channel errors instead of panicking

- **WHEN** a pixel layer's channel or mask `data.len()` does not match the pixel count implied by its `rect`
- **THEN** `resize_canvas_document` returns `OpsError::InvalidParams`, leaves `doc` unchanged, and does not panic

### Requirement: Nine-anchor translation of layer and mask bounds

`resize_canvas_document` SHALL compute the horizontal and vertical offsets `dx`
and `dy` from the differences `width - old_width` and `height - old_height` and
`anchor` using the same nine-anchor math as `pictura_ops::resize_canvas` (the
left/middle/right family yields `0`, half the delta, or the whole delta, and
likewise for the top/middle/bottom family). It SHALL translate every layer
`rect` and every decoded `LayerMask` `rect` by `(dx, dy)` and SHALL recurse
through group `children`. Layer and mask channel `data` MUST NOT be resampled or
reordered; only bounds move, and the compositor clips content to the new canvas.
Every layer's `name`, `blend`, `opacity`, `clipping`, `visible`, and
`adjustment` payload MUST be preserved unchanged.

#### Scenario: Center anchor translates a full-canvas layer symmetrically

- **WHEN** a 1000×1000 document with a full-canvas layer is grown to 1200×1200 with `Anchor::Center`
- **THEN** the layer `rect` becomes `left`/`top` 100 and `right`/`bottom` 1100

#### Scenario: Top-left anchor adds space on the right and bottom

- **WHEN** a document is grown with `Anchor::TopLeft`
- **THEN** every layer `rect` keeps its `left` and `top` and only `right` and `bottom` increase by the deltas

#### Scenario: Bottom-right anchor adds space on the left and top

- **WHEN** a document is grown with `Anchor::BottomRight`
- **THEN** every layer `rect` keeps its `right` and `bottom` and `left` and `top` decrease by the deltas

#### Scenario: Nested layers and masks are translated

- **WHEN** a group contains a nested masked pixel layer and the canvas is resized
- **THEN** the nested layer `rect` and its mask `rect` are both translated by the same `(dx, dy)`

### Requirement: Document channel re-extension and transparent added canvas

`resize_canvas_document` SHALL re-extend or crop every entry of `doc.channels` to
the new document size with an all-zero fill, preserving each channel's `id`.
Because layer content only translates and no layer data covers the grown region,
the added canvas area MUST be transparent (alpha 0) in the recomputed composite:
the added document-channel samples are extended with 0 and no added pixel carries
any fill color.

#### Scenario: Document channels are re-extended and cropped

- **WHEN** a document carrying a document-level channel is grown and then shrunk
- **THEN** the channel `data.len()` tracks the new document size, its `id` is unchanged, and the added samples are 0

#### Scenario: The added canvas is transparent

- **WHEN** the canvas is grown
- **THEN** the recomputed composite has alpha 0 in every added pixel

#### Scenario: Grown and cropped content matches the anchor offset

- **WHEN** the canvas is grown and then a document-level channel is inspected
- **THEN** the original samples appear at the anchor offset and the newly exposed samples are 0

### Requirement: Composite recomputation after document canvas resize

After a successful `resize_canvas_document`, `doc.composite` SHALL be replaced by
`pictura_render::composite_rgba(doc)` at the new document size. The composite
MUST be recomputed from the translated layer tree so that `doc.composite` always
equals `composite_rgba(doc)` immediately after the call.

#### Scenario: The cached composite equals a fresh composite after canvas resize

- **WHEN** `resize_canvas_document` succeeds
- **THEN** `doc.composite` is bit-identical to `composite_rgba(&doc)` and has the new dimensions

### Requirement: Document canvas resize oracle

The system SHALL ship an oracle for `resize_canvas_document` in
`crates/pictura-render/tests/document_oracle.rs` that (a) writes the resized
document with `pictura_codec::write_psd`, re-reads it with
`pictura_codec::read_psd`, and confirms the structural round-trip, (b) opens the
written file with the independent `psd-tools` library and confirms the reported
document dimensions and layer count match the
anchor-translated document, and (c) asserts `doc.composite ==
composite_rgba(&doc)`. The oracle SHALL skip with a message when `psd-tools` is
not importable and MUST NOT be marked `#[ignore]`.

#### Scenario: The canvas-resized document round-trips structurally

- **WHEN** a layered document is canvas-resized and written, then re-read by `pictura_codec::read_psd`
- **THEN** the re-read document has the new dimensions and every layer's `rect` matches the translated rect

#### Scenario: psd-tools sees the canvas-resized document

- **WHEN** the written PSD is opened with `psd-tools`
- **THEN** psd-tools reports the new document dimensions and layer count

#### Scenario: Missing psd-tools skips cleanly

- **WHEN** `psd-tools` is not importable
- **THEN** the oracle prints a skip message and the suite still passes

### Requirement: Layer translation uses the active backend

Translating a layer SHALL recompute the document composite through the active
backend — the GPU compositor when GPU compute is enabled and a usable adapter
exists, otherwise the CPU compositor — and SHALL remain within ±1 LSB of the CPU
oracle. The CPU `translate_layer` and `recompute` remain the oracle and MUST NOT
change.

#### Scenario: A Move commit on a large document composites on the GPU

- **WHEN** a layer move is committed on a 4000×4000 document with GPU compute
  enabled and a usable adapter present
- **THEN** the document composite is recomputed on the GPU and every channel
  differs from the CPU oracle by at most 1 LSB

### Requirement: Dirty-region canvas refresh

The canvas SHALL cache the composited document as a full-document image and,
when a mutation reports a dirty rectangle, SHALL recomposite only that rectangle
through the active backend and blit it into the cached image at the rectangle's
origin, then emit `changed`. A Move commit SHALL invalidate
`old_layer_rect ∪ new_layer_rect`; a paint dab SHALL invalidate the dab's
bounding box. A canvas updated only through such region refreshes SHALL be
byte-identical to a full recomposite after any sequence of those mutations. A
mutation that does not report a dirty rectangle SHALL keep the full recomposite.

#### Scenario: After several Move commits the cached canvas equals a full recomposite

- **WHEN** several Move commits are applied to a document, with GPU compute
  enabled or falling back to the CPU oracle
- **THEN** the cached canvas is byte-identical to a full recomposite of the final
  document

#### Scenario: A region update does not disturb pixels outside the region

- **WHEN** a dirty rectangle is refreshed through the active backend
- **THEN** every pixel outside the rectangle keeps its previous value and every
  pixel inside the rectangle equals the corresponding pixel of a full recomposite

