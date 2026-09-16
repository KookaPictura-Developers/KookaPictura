## MODIFIED Requirements

### Requirement: Undo and redo restore the captured state

The system SHALL provide `undo() -> bool` and `redo() -> bool`. `undo()` SHALL
restore the most recent captured snapshot (document and selection), decrement the
history position, and return `true`; with no state to undo it SHALL return
`false` and leave the document untouched. `redo()` SHALL re-apply the state that
was undone and return `true`; with nothing to redo it SHALL return `false`. Both
SHALL leave the document bit-identical to the snapshot being restored. Both SHALL
restore the displayed image from the restored document's `composite` buffer
without running a full composite, unconditionally — there is no colour-plane or
layer-count fallback. The restored display MUST be byte-identical to a full
recomposite of the restored document; across a GPU/CPU backend change it MAY
differ by at most 1 LSB per channel, consistent with the existing GPU parity
contract.

#### Scenario: Undo reverts a filter bit-exactly

- **WHEN** a filter is applied to a document and `undo()` is called
- **THEN** it returns `true` and every document field, including the composite and the selection, equals its pre-filter value

#### Scenario: Redo re-applies the undone op bit-exactly

- **WHEN** `undo()` succeeded and `redo()` is called
- **THEN** the document equals its post-filter value

#### Scenario: Undo and redo at the stack ends are no-ops

- **WHEN** `undo()` is called with an empty history, or `redo()` is called with no undone state
- **THEN** each returns `false` and the document is unchanged

#### Scenario: Undo restores the display without a full composite

- **WHEN** any document is edited and `undo()` is called
- **THEN** the displayed image is restored from the snapshot's composite buffer, no full composite runs, and the display equals a full recomposite of the restored document

## ADDED Requirements

### Requirement: Composite kept current on the canvas rebuild

The system SHALL persist the rendered result into the document's `composite`
buffer whenever the canvas is rebuilt — a full recomposite or a region refresh.
An RGB document's stored composite SHALL be the rendered 4-plane RGBA frame, so
its merged composite is RGBA after any rebuild. A non-RGB mode
(Grayscale/Bitmap/Duotone/CMYK/Lab) SHALL keep its existing composite
colour-plane count, copying `min(rendered.channels, composite.channels)` planes,
so a 1-plane grayscale composite stays 1-plane. When the document's composite
dimensions no longer match the document — after a resize, crop, rotate, flip, or
canvas-size change — the system SHALL replace the composite with the rendered
result rather than patching it. A region refresh MUST leave every pixel outside
its region with its previous value. The stored composite MUST be derived from the
same rendered buffer that is converted for display, so the document composite and
the displayed image cannot diverge.

#### Scenario: A full recomposite stores the rendered result

- **WHEN** a filter, adjustment, blend, opacity, visibility, reorder, or delete mutation rebuilds the canvas in full
- **THEN** `doc.composite` equals the rendered composite that was displayed

#### Scenario: An RGB composite is stored as RGBA

- **WHEN** an RGB document's canvas is rebuilt
- **THEN** its composite has four colour planes and equals the rendered RGBA frame

#### Scenario: A region refresh patches only its region

- **WHEN** a region refresh composites a clamped rectangle
- **THEN** the pixels inside the rectangle are written from the rendered region, every pixel outside it keeps its previous value, and the composite's colour-plane count is unchanged

#### Scenario: A dimension change replaces the composite

- **WHEN** a resize, crop, rotate, flip, or canvas-size change rebuilds the canvas and the composite's dimensions no longer match the document
- **THEN** the composite is replaced with the rendered result at the new document dimensions

#### Scenario: A grayscale composite stays one plane

- **WHEN** a grayscale document is edited and its canvas rebuilt
- **THEN** its composite remains a single colour plane and the stored values are the rendered colour plane

### Requirement: Current composite at history capture

The history snapshot captured for a mutating command SHALL carry a document whose
`composite` buffer is current — equal to the rendered result of that document.
A mutating operation SHALL apply its region refresh or full recomposite before it
records the snapshot. The history order, labels, cursor movement, depth bound, and
redo truncation MUST otherwise be unchanged, and the snapshot SHALL continue to
carry the post-command document state.

#### Scenario: A captured snapshot carries a current composite

- **WHEN** any mutating command completes and its state is captured
- **THEN** the captured document's composite equals a full recomposite of the captured document

#### Scenario: Capture ordering does not change history order

- **WHEN** a mutating command runs after one or more undos
- **THEN** the captured state is still the post-command state, redo is discarded as before, and the labels and depth bound are unchanged
