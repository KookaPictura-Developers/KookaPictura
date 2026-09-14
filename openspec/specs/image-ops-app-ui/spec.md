# image-ops-app-ui Specification

## Purpose
TBD - created by archiving change m13-image-ops-ui. Update Purpose after archive.
## Requirements
### Requirement: App document-resize command

The system SHALL provide a `PictureView::resize_image(kind: QString, width: i32, height: i32) -> bool` command. With `kind` one of `"nearest"`, `"bilinear"`, `"bicubic"` and `width`/`height` both at least 1, it SHALL apply `pictura_render::resize_document` to the loaded document with the matching `pictura_ops::Resample`, recompute the composite, and return `true`. Unknown kinds and non-positive dimensions SHALL be rejected by returning `false` with the document untouched. When no document is loaded it SHALL return `false`.

#### Scenario: Image size resizes the loaded document

- **WHEN** `resize_image("bilinear", w, h)` is called on a loaded document
- **THEN** it returns `true`, the document and displayed image have dimensions `w × h`, and the composite equals a recomposition of the resized layer stack

#### Scenario: Invalid kinds and dimensions are rejected untouched

- **WHEN** `resize_image` is called with an unknown kind, or `width` or `height` below 1
- **THEN** it returns `false` and the document is bit-identical to its prior state

### Requirement: App canvas-resize command

The system SHALL provide a `PictureView::resize_canvas(anchor: QString, width: i32, height: i32) -> bool` command. With `anchor` one of the nine `pictura_ops::Anchor` names (`"top-left"`, `"top-center"`, `"top-right"`, `"center-left"`, `"center"`, `"center-right"`, `"bottom-left"`, `"bottom-center"`, `"bottom-right"`) and positive dimensions, it SHALL apply `pictura_render::resize_canvas_document`, recompute the composite, and return `true`. Unknown anchors and non-positive dimensions SHALL be rejected with `false` and the document untouched.

#### Scenario: Canvas size with a chosen anchor

- **WHEN** `resize_canvas("bottom-right", w, h)` is called on a loaded document
- **THEN** it returns `true`, the document is `w × h`, and existing layer content keeps its position relative to the bottom-right corner

#### Scenario: Unknown anchor is rejected

- **WHEN** `resize_canvas` is called with an anchor name that is not one of the nine
- **THEN** it returns `false` and the document is unchanged

### Requirement: App orientation commands

The system SHALL provide `PictureView::rotate_doc(quarter_turns: i32) -> bool` and `PictureView::flip_doc(horizontal: bool) -> bool`. `rotate_doc` with 1, 2, or 3 SHALL apply `pictura_render::rotate_document` (90° CW, 180°, 90° CCW respectively) and return `true`; any other value SHALL return `false` with the document untouched. `flip_doc` SHALL apply `pictura_render::flip_document` about the vertical axis for `true` and the horizontal axis for `false`, returning `true`. Both SHALL recompute the composite on success.

#### Scenario: Rotate and flip update the displayed image

- **WHEN** `rotate_doc(1)`, `rotate_doc(2)`, `rotate_doc(3)`, `flip_doc(true)`, and `flip_doc(false)` are each called on a loaded document
- **THEN** each returns `true` and the displayed image reflects the corresponding orientation of the recomputed composite

#### Scenario: Invalid quarter turns are rejected

- **WHEN** `rotate_doc` is called with 0, 4, or a negative value
- **THEN** it returns `false` and the document is unchanged

### Requirement: Document operations clear the selection

After any successful `resize_image`, `resize_canvas`, `rotate_doc`, or `flip_doc` command, the app SHALL clear the active selection, and the reported selection pixel count SHALL be 0.

#### Scenario: Selection does not survive a document operation

- **WHEN** a selection is active and `rotate_doc(1)` returns `true`
- **THEN** `has_selection` is `false` and `selection_count` is 0

### Requirement: Image operations dock controls

The Qt shell SHALL expose the document operations in the dock: an Image Size control (width and height spin boxes, a resample combo with Nearest/Bilinear/Bicubic, and an Apply action wired to `resize_image`), a Canvas Size control (width and height spin boxes, a nine-entry anchor combo, and an Apply action wired to `resize_canvas`), and orientation actions (Rotate 90° CW, Rotate 90° CCW, Rotate 180°, Flip Horizontal, Flip Vertical) wired to `rotate_doc` and `flip_doc`. After any successful command the view SHALL refresh to the new dimensions.

#### Scenario: Dock-driven document op refreshes the view

- **WHEN** the rotation action is triggered from the dock on a loaded document
- **THEN** the displayed image and dock state are refreshed for the new orientation

### Requirement: Headless self-test coverage for document operations

The app `--self-test` SHALL exercise at least one document operation end-to-end on the loaded fixture: assert the command returns `true`, the displayed image takes the new dimensions, the composite changed accordingly, and the selection was cleared; and SHALL assert that an invalid command returns `false` without changing the image.

#### Scenario: Self-test proves a document op in the app

- **WHEN** the app runs `--self-test` with the layered fixture
- **THEN** the document-operation assertions pass and the self-test exits successfully

