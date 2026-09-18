//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.
#![allow(clippy::too_many_arguments)] // brush parameter lists mirror the C++ API

use crate::history::{History, Snapshot};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QImage;
use pictura_core::{Document, PsdRect};
use pictura_paint::Stroke;
use pictura_select::Selection;

mod helpers;
mod helpers_composite;
mod impl_core;
mod impl_filters;
mod impl_history;
mod impl_layers;
mod impl_paint;
mod impl_selection;
mod impl_transform;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_impl;
use helpers::*;
use helpers_composite::*;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;

        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[namespace = "pictura"]
        type PictureView = super::PictureViewRust;

        /// Emitted whenever the layer stack changes and the image is refreshed.
        #[qsignal]
        fn changed(self: Pin<&mut Self>);

        /// Emitted after a region composite. The receiver blits `region` at
        /// `(x, y)`; the full image was not rebuilt.
        #[qsignal]
        #[cxx_name = "regionBlitted"]
        fn region_blitted(self: Pin<&mut Self>, region: QImage, x: i32, y: i32);

        /// Try to load a PSD through `pictura-codec`. Returns `false` and falls
        /// back to a generated test image when the file is missing or unsupported.
        #[qinvokable]
        fn open(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Create a new `width`×`height` document. `mode` is `"rgb"` or
        /// `"grayscale"`, `depth` must be 8, and `background` is `"white"` or
        /// `"transparent"`. Resets the selection and history and clears the
        /// file path and dirty flag. Returns false and leaves state unchanged
        /// for any invalid parameter.
        #[qinvokable]
        fn new_document(
            self: Pin<&mut Self>,
            width: i32,
            height: i32,
            mode: &QString,
            depth: i32,
            background: &QString,
        ) -> bool;

        /// Serialize the document to `path` as a PSD, writing a sibling
        /// `<path>.tmp` first and renaming it over `path`. Clears the dirty
        /// flag on success. Returns false without a document or on any encode
        /// or IO error.
        #[qinvokable]
        fn save(self: Pin<&mut Self>, path: &QString) -> bool;

        /// Whether the document has unsaved changes (false when none).
        #[qinvokable]
        fn is_dirty(&self) -> bool;

        /// Path the document was last opened from or saved to; empty when
        /// untitled.
        #[qinvokable]
        fn file_path(&self) -> QString;

        /// The image to display. Never null. Returns the cached image when the
        /// display is clean; rebuilds it from the document composite when a
        /// region refresh has marked the display dirty.
        #[qinvokable]
        fn image(self: Pin<&mut Self>) -> QImage;

        /// Whether a document is loaded (false when only the fallback image is
        /// shown). Drives command enablement in the shell.
        #[qinvokable]
        fn has_document(&self) -> bool;

        /// Width of the loaded document in pixels (0 when none). Reads the
        /// document directly without materializing or copying any image.
        #[qinvokable]
        fn document_width(&self) -> i32;

        /// Height of the loaded document in pixels (0 when none). Reads the
        /// document directly without materializing or copying any image.
        #[qinvokable]
        fn document_height(&self) -> i32;

        /// Number of top-level layers in the loaded document (0 when none).
        #[qinvokable]
        fn layer_count(&self) -> i32;

        /// Index of the topmost pixel layer (neither a group nor an adjustment),
        /// or -1 when there is none.
        #[qinvokable]
        fn topmost_pixel_layer_index(&self) -> i32;

        /// Name of layer `i`, or empty when out of range.
        #[qinvokable]
        fn layer_name(&self, i: i32) -> QString;

        /// `"pixel"`, `"group"`, `"adjustment"`, or `"background"` for layer `i`;
        /// empty when out of range.
        #[qinvokable]
        fn layer_kind(&self, i: i32) -> QString;

        /// Visibility flag of layer `i` (false when out of range).
        #[qinvokable]
        fn layer_visible(&self, i: i32) -> bool;

        /// Blend mode of layer `i` as its 4-byte PSD key (e.g. `"mul "`), or
        /// empty when out of range.
        #[qinvokable]
        fn layer_blend(&self, i: i32) -> QString;

        /// Set layer `i`'s blend mode from a 4-byte PSD `key`. Captures history,
        /// marks dirty, recomposites, and emits [`changed`]. Returns false for an
        /// unknown key or when layer `i` is out of range, leaving state unchanged.
        #[qinvokable]
        fn set_layer_blend(self: Pin<&mut Self>, i: i32, key: &QString) -> bool;

        /// Opacity of layer `i` in `0..=255`, or 0 when out of range.
        #[qinvokable]
        fn layer_opacity(&self, i: i32) -> i32;

        /// Set layer `i`'s opacity, clamped to `0..=255`. Refused (false, state
        /// unchanged) for the Background layer or a fully locked layer. Captures
        /// history, marks dirty, recomposites, and emits [`changed`]. Returns
        /// false when layer `i` is out of range.
        #[qinvokable]
        fn set_layer_opacity(self: Pin<&mut Self>, i: i32, value: i32) -> bool;

        /// Fill opacity of layer `i` in `0..=255` (content opacity, distinct
        /// from Opacity), or 0 when out of range.
        #[qinvokable]
        fn layer_fill(&self, i: i32) -> i32;

        /// Set layer `i`'s fill, clamped to `0..=255`. Refused (false, state
        /// unchanged) for a group, the Background layer, or a fully locked layer
        /// (CS6 exposes no group Fill). Captures history, marks dirty,
        /// recomposites, and emits [`changed`].
        #[qinvokable]
        fn set_layer_fill(self: Pin<&mut Self>, i: i32, value: i32) -> bool;

        /// Lock flags of layer `i` as a bitmask (`0x01` transparency, `0x02`
        /// pixels, `0x04` position), or 0 when out of range.
        #[qinvokable]
        fn layer_lock(&self, i: i32) -> i32;

        /// Set one lock flag of layer `i`. `flag` is `"transparency"`, `"pixels"`,
        /// `"position"`, or `"all"`; an unknown flag returns false. Refused for
        /// the Background layer. Captures history, recomposites, and emits
        /// [`changed`].
        #[qinvokable]
        fn set_layer_lock(self: Pin<&mut Self>, i: i32, flag: &QString, on: bool) -> bool;

        /// Color label of layer `i` as a byte (`0` none … `7` gray), or 0.
        #[qinvokable]
        fn layer_color(&self, i: i32) -> i32;

        /// Set layer `i`'s color label. A `value` outside `0..=7` returns false.
        /// Refused for the Background layer. Captures history, recomposites, and
        /// emits [`changed`].
        #[qinvokable]
        fn set_layer_color(self: Pin<&mut Self>, i: i32, value: i32) -> bool;

        /// Rename layer `i`. Captures history, marks dirty, recomposites, and
        /// emits [`changed`]. Returns false when layer `i` is out of range.
        #[qinvokable]
        fn set_layer_name(self: Pin<&mut Self>, i: i32, name: &QString) -> bool;

        /// Swap layer `i` with the neighbour `delta` positions away in the
        /// bottom-first list. Captures history, marks dirty, recomposites, and
        /// emits [`changed`]. Returns false when either position is out of range.
        #[qinvokable]
        fn move_layer(self: Pin<&mut Self>, i: i32, delta: i32) -> bool;

        /// Insert a new empty transparent raster layer above layer `above`
        /// (`above < 0` or out of range means the top of the stack). Captures
        /// history, marks dirty, recomposites, and emits [`changed`]. Returns
        /// the new index, or -1 without a document.
        #[qinvokable]
        fn add_layer(self: Pin<&mut Self>, above: i32) -> i32;

        /// Insert an empty group above layer `above`. Captures history, marks
        /// dirty, recomposites, and emits [`changed`]. Returns the new index, or
        /// -1 without a document.
        #[qinvokable]
        fn add_group(self: Pin<&mut Self>, above: i32) -> i32;

        /// Deep-clone layer `index` (children, mask, adjustment, and all
        /// attributes) directly above it, named `"<name> copy"`. Captures
        /// history, marks dirty, recomposites, and emits [`changed`]. Returns
        /// the new index, or -1 when `index` is out of range.
        #[qinvokable]
        fn duplicate_layer(self: Pin<&mut Self>, index: i32) -> i32;

        /// Wrap layer `index` in a new group at the same stack position.
        /// Captures history, marks dirty, recomposites, and emits [`changed`].
        /// Returns the group's index, or -1 when `index` is out of range.
        #[qinvokable]
        fn group_layer(self: Pin<&mut Self>, index: i32) -> i32;

        /// Splice group `index`'s children into its parent position. Captures
        /// history, marks dirty, recomposites, and emits [`changed`]. Returns
        /// false, leaving state unchanged, when `index` is out of range or not
        /// a group.
        #[qinvokable]
        fn ungroup_layer(self: Pin<&mut Self>, index: i32) -> bool;

        /// The RGBA content of layer `i` scaled to fit `size`×`size`, keeping
        /// the aspect ratio with smooth filtering. Null for group or adjustment
        /// layers and when `i` or `size` is out of range.
        #[qinvokable]
        fn layer_thumbnail(&self, i: i32, size: i32) -> QImage;

        // ------------------------------------------------------------------
        // M39 tree rows, path/batch mutation. See
        // `docs/dev/m39-panel-anatomy.md` §3.2.
        // ------------------------------------------------------------------

        /// Number of nodes in the whole layer tree (all depths; 0 without a
        /// document).
        #[qinvokable]
        fn layer_row_count(&self) -> i32;

        /// Frozen path of flat row `i` (depth-first, topmost-first), or empty.
        #[qinvokable]
        fn layer_row_path(&self, i: i32) -> QString;

        /// Nesting depth of row `i` (top-level rows are 0).
        #[qinvokable]
        fn layer_row_depth(&self, i: i32) -> i32;

        /// Name of row `i`, or empty when out of range.
        #[qinvokable]
        fn layer_row_name(&self, i: i32) -> QString;

        /// Kind of row `i`: `"pixel"`, `"group"`, `"adjustment"`, or
        /// `"background"` (the same strings as [`layer_kind`]), or empty.
        #[qinvokable]
        fn layer_row_kind(&self, i: i32) -> QString;

        /// Visibility flag of row `i`.
        #[qinvokable]
        fn layer_row_visible(&self, i: i32) -> bool;

        /// Blend mode of row `i` as its 4-byte PSD key, or empty.
        #[qinvokable]
        fn layer_row_blend(&self, i: i32) -> QString;

        /// Opacity of row `i` in `0..=255`.
        #[qinvokable]
        fn layer_row_opacity(&self, i: i32) -> i32;

        /// Fill opacity of row `i` in `0..=255`.
        #[qinvokable]
        fn layer_row_fill(&self, i: i32) -> i32;

        /// Lock flags of row `i` as a bitmask `0x01/0x02/0x04`.
        #[qinvokable]
        fn layer_row_lock(&self, i: i32) -> i32;

        /// Color label of row `i` as a byte (`0` none … `7` gray).
        #[qinvokable]
        fn layer_row_color(&self, i: i32) -> i32;

        /// Clipping flag of row `i`.
        #[qinvokable]
        fn layer_row_clipping(&self, i: i32) -> bool;

        /// Whether row `i` carries a layer mask.
        #[qinvokable]
        fn layer_row_has_mask(&self, i: i32) -> bool;

        /// Whether row `i` carries adjustment content.
        #[qinvokable]
        fn layer_row_has_adjustment(&self, i: i32) -> bool;

        /// Whether row `i` is a group with at least one child.
        #[qinvokable]
        fn layer_row_expandable(&self, i: i32) -> bool;

        /// Number of direct children of row `i`.
        #[qinvokable]
        fn layer_row_child_count(&self, i: i32) -> i32;

        /// Thumbnail of row `i` scaled to `size`. `entire_document` places the
        /// layer at its document position in a transparent `size`×`size`
        /// square; otherwise the layer's own bounds fill the square. Null for
        /// a group, an adjustment layer, or an invalid index/size.
        #[qinvokable]
        fn layer_row_thumbnail(&self, i: i32, size: i32, entire_document: bool) -> QImage;

        /// Mask thumbnail of row `i` scaled to `size`, or null without a mask.
        #[qinvokable]
        fn layer_row_mask_thumbnail(&self, i: i32, size: i32) -> QImage;

        /// Rename the node at `path`. Recomposites and records one undo state
        /// on success. Returns false for a path that does not resolve.
        #[qinvokable]
        fn set_layer_name_path(self: Pin<&mut Self>, path: &QString, name: &QString) -> bool;

        /// Move the node at `path` `delta` places within its own container.
        /// Refuses the Background and fully-locked nodes. Recomposites and
        /// records one undo state on success.
        #[qinvokable]
        fn move_layer_path(self: Pin<&mut Self>, path: &QString, delta: i32) -> bool;

        /// Move the node at `path` next to (or into) `target`: mode 0 = above,
        /// 1 = below, 2 = into a group. An empty target means the document top.
        /// Refuses the Background, fully/nesting-locked sources, a self or
        /// descendant drop, and an `Into` target that is not a group.
        #[qinvokable]
        fn move_layer_to(self: Pin<&mut Self>, path: &QString, target: &QString, mode: i32)
            -> bool;

        /// Set visibility on every path (always eligible per node). Returns the
        /// number of nodes changed; recomposites and records one undo state
        /// only when that count is non-zero.
        #[qinvokable]
        fn set_layers_visible(self: Pin<&mut Self>, paths: &QStringList, visible: bool) -> i32;

        /// Set the blend mode from a 4-byte PSD `key` on every path. Returns
        /// the number changed; records one undo state only when non-zero.
        #[qinvokable]
        fn set_layers_blend(self: Pin<&mut Self>, paths: &QStringList, key: &QString) -> i32;

        /// Set opacity (clamped `0..=255`) on every path. Returns the number
        /// changed; records one undo state only when non-zero.
        #[qinvokable]
        fn set_layers_opacity(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;

        /// Set fill opacity (clamped `0..=255`) on every path. Returns the
        /// number changed; records one undo state only when non-zero.
        #[qinvokable]
        fn set_layers_fill(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;

        /// Set one lock flag on every path. `flag` is `"transparency"`,
        /// `"pixels"`, `"position"`, or `"all"`. Returns the number changed;
        /// records one undo state only when non-zero.
        #[qinvokable]
        fn set_layers_lock(
            self: Pin<&mut Self>,
            paths: &QStringList,
            flag: &QString,
            on: bool,
        ) -> i32;

        /// Set the color label on every path. Returns the number changed;
        /// records one undo state only when non-zero.
        #[qinvokable]
        fn set_layers_color(self: Pin<&mut Self>, paths: &QStringList, value: i32) -> i32;

        /// Solo visibility: set exactly `paths` visible and every other node
        /// invisible, recording one undo state under `label`. Returns the
        /// number of nodes changed.
        #[qinvokable]
        fn apply_visibility(self: Pin<&mut Self>, paths: &QStringList, label: &QString) -> i32;

        /// Delete every eligible path. Returns the number deleted; records one
        /// undo state only when non-zero.
        #[qinvokable]
        fn delete_layers(self: Pin<&mut Self>, paths: &QStringList) -> i32;

        /// Deep-copy every path directly above itself, returning the new paths.
        /// Records one undo state only when at least one copy was created.
        #[qinvokable]
        fn duplicate_layers(self: Pin<&mut Self>, paths: &QStringList) -> QStringList;

        /// Wrap the selection in one new group at the topmost selected
        /// position, returning its path or empty on refusal.
        #[qinvokable]
        fn group_layers(self: Pin<&mut Self>, paths: &QStringList) -> QString;

        /// Splice each selected group's children in place. Returns the number
        /// of groups changed; records one undo state only when non-zero.
        #[qinvokable]
        fn ungroup_layers(self: Pin<&mut Self>, paths: &QStringList) -> i32;

        /// Insert a new transparent raster layer inside `selection_path` when it
        /// is a group else above it, returning the new path or empty.
        #[qinvokable]
        fn add_layer_in(self: Pin<&mut Self>, selection_path: &QString) -> QString;

        /// Insert a new empty group by the [`add_layer_in`] rule, returning the
        /// new path or empty.
        #[qinvokable]
        fn add_group_in(self: Pin<&mut Self>, selection_path: &QString) -> QString;

        /// Set layer `i` visibility, recomposite, and emit [`changed`].
        #[qinvokable]
        fn set_layer_visible(self: Pin<&mut Self>, i: i32, visible: bool);

        /// Select the whole document, recomposite, and emit [`changed`].
        #[qinvokable]
        fn select_all(self: Pin<&mut Self>);

        /// Clear the active selection, recomposite, and emit [`changed`].
        #[qinvokable]
        fn deselect(self: Pin<&mut Self>);

        /// Flood-select the region around `(x, y)` within `tolerance` (0-255),
        /// recomposite, and emit [`changed`]. Returns false without a document
        /// or when the point is out of bounds.
        #[qinvokable]
        fn magic_wand(self: Pin<&mut Self>, x: i32, y: i32, tolerance: i32) -> bool;

        /// Whether a selection is currently active.
        #[qinvokable]
        fn has_selection(&self) -> bool;

        /// Number of pixels with non-zero selection coverage (0 when none).
        #[qinvokable]
        fn selection_count(&self) -> i32;

        /// Replace/combine the selection with the `w`×`h` rectangle at `(x, y)`.
        /// `mode` is `"new"`, `"add"`, `"subtract"`, or `"intersect"` (unknown
        /// means `"new"`). Returns false without a document.
        #[qinvokable]
        fn select_rect(
            self: Pin<&mut Self>,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            mode: &QString,
        ) -> bool;

        /// Replace/combine the selection with the ellipse inscribed in the
        /// `w`×`h` rectangle at `(x, y)`. `mode` as [`select_rect`]. Returns
        /// false without a document.
        #[qinvokable]
        fn select_ellipse(
            self: Pin<&mut Self>,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            mode: &QString,
        ) -> bool;

        /// Start a lasso selection with `mode` as [`select_rect`], clearing any
        /// pending points. Returns false without a document.
        #[qinvokable]
        fn begin_lasso(self: Pin<&mut Self>, mode: &QString) -> bool;

        /// Append a point to the pending lasso path.
        #[qinvokable]
        fn lasso_add_point(self: Pin<&mut Self>, x: i32, y: i32);

        /// Fill the pending lasso polygon and combine it with the current
        /// selection. Returns false without a document or fewer than three
        /// points, leaving the pending state untouched.
        #[qinvokable]
        fn end_lasso(self: Pin<&mut Self>) -> bool;

        /// Flood-select around `(x, y)` within `tolerance` (0-255) and combine
        /// it with the current selection using `mode` as [`select_rect`].
        /// Returns false without a document or when the point is out of bounds.
        #[qinvokable]
        fn quick_select(
            self: Pin<&mut Self>,
            x: i32,
            y: i32,
            tolerance: i32,
            mode: &QString,
        ) -> bool;

        /// Crop the document to the `w`×`h` rectangle at `(x, y)`, clearing the
        /// selection, then recomposite and emit [`changed`]. Returns false
        /// without a document or when the rect misses the canvas.
        #[qinvokable]
        fn crop(self: Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) -> bool;

        /// Move the topmost pixel layer by `(dx, dy)`, recomposite, and emit
        /// [`changed`]. Returns false without a pixel layer.
        #[qinvokable]
        fn translate_layer(self: Pin<&mut Self>, dx: i32, dy: i32) -> bool;

        /// Shift the topmost pixel layer by `(dx, dy)` for a live drag preview:
        /// recomposite and emit [`changed`] but DO NOT add history or mark dirty.
        /// Returns false without a raster layer or for a zero delta.
        /// ponytail: slow path, retained for the self-test.
        #[qinvokable]
        fn move_preview(self: Pin<&mut Self>, dx: i32, dy: i32) -> bool;

        /// Enter move-preview mode: cache the document composited with the
        /// topmost raster layer hidden, that layer's own image, its
        /// document-space top-left, and its opacity (0..=255). Returns false
        /// without a document or raster layer. Reuses the cached base when the
        /// document content, topmost layer, and clamped rect are unchanged.
        #[qinvokable]
        fn begin_move_preview(self: Pin<&mut Self>) -> bool;

        /// Warm the move-preview cache without entering preview mode, so the
        /// next `begin_move_preview` is a cache hit. Returns true when the
        /// cache is valid (reused or just rebuilt), false without a document
        /// or raster layer.
        #[qinvokable]
        fn prepare_move_preview(self: Pin<&mut Self>) -> bool;

        /// Whether the last `begin_move_preview` reused the cached base instead
        /// of recomputing it.
        #[qinvokable]
        fn move_preview_cache_hit(&self) -> bool;

        /// The cached base image (layers with the moved layer hidden); null when
        /// not previewing.
        #[qinvokable]
        fn move_preview_base(&self) -> QImage;

        /// The moved layer's own image; null when not previewing or for a
        /// non-raster layer.
        #[qinvokable]
        fn move_preview_layer(&self) -> QImage;

        /// The moved layer's document-space left edge.
        #[qinvokable]
        fn move_preview_x(&self) -> i32;

        /// The moved layer's document-space top edge.
        #[qinvokable]
        fn move_preview_y(&self) -> i32;

        /// The moved layer's opacity in `0..=255`.
        #[qinvokable]
        fn move_preview_opacity(&self) -> i32;

        /// Leave preview mode and drop the cached images (no document change).
        #[qinvokable]
        fn end_move_preview(self: Pin<&mut Self>);

        /// Apply the real move once: shift the topmost raster layer by `(dx, dy)`,
        /// capture one "Move Layer" history state, mark dirty, refresh the image,
        /// and emit [`changed`]. Uses a single composite. Returns false when there
        /// is no document/raster layer.
        #[qinvokable]
        fn commit_move(self: Pin<&mut Self>, dx: i32, dy: i32) -> bool;

        /// No-argument commit kept for the existing C++ tool and self-test, which
        /// mutate the document through [`move_preview`] before recording. Records
        /// the already-previewed state and recomposites. Returns false without a
        /// document.
        #[qinvokable]
        #[cxx_name = "commit_move"]
        fn commit_move_legacy(self: Pin<&mut Self>) -> bool;

        /// The composited pixel at `(x, y)` as `0xAARRGGBB`, or 0 when there is
        /// no document or the point is out of bounds.
        #[qinvokable]
        fn sample_argb(&self, x: i32, y: i32) -> u32;

        /// Self-test probe: the stored document composite's pixel at `(x, y)` as
        /// `0xAARRGGBB`, or 0 when there is no document or out of bounds.
        #[qinvokable]
        fn composite_argb(&self, x: i32, y: i32) -> u32;

        /// The selected pixels' `"x y w h"` bounding box, or an empty string
        /// when nothing is selected.
        #[qinvokable]
        fn selection_bounds(&self) -> QString;

        /// Append an adjustment layer for `kind` (invert, posterize, threshold,
        /// brightness-contrast, hue-saturation), recomposite, and emit
        /// [`changed`]. When a selection is active the layer gets a raster mask
        /// from its coverage, so only selected pixels change. Returns false for
        /// an unknown kind or no document.
        #[qinvokable]
        fn add_adjustment(self: Pin<&mut Self>, kind: &QString) -> bool;

        /// Apply a destructive filter `kind` to the topmost pixel layer,
        /// confined by the active selection, then recomposite and emit
        /// [`changed`]. Returns false without a document, for an unknown kind,
        /// or when there is no pixel layer.
        #[qinvokable]
        fn apply_filter(self: Pin<&mut Self>, kind: &QString) -> bool;

        /// Begin a paint stroke. Colours are 0xAARRGGBB. `mode` is
        /// "normal" | "dissolve" | "behind" | "clear". Returns false without a
        /// document or when there is no raster layer.
        #[qinvokable]
        fn begin_paint(
            self: Pin<&mut Self>,
            foreground: u32,
            background: u32,
            diameter: i32,
            hardness: i32,
            roundness: i32,
            angle: i32,
            opacity: i32,
            flow: i32,
            spacing: i32,
            mode: &QString,
            aliased: bool,
            auto_erase: bool,
        ) -> bool;

        /// Add a pointer sample to the active stroke and refresh the live image.
        /// Returns false when no stroke is active.
        #[qinvokable]
        fn paint_dab(self: Pin<&mut Self>, x: f64, y: f64, pressure: f64) -> bool;

        /// Commit the active stroke as one history state ("Brush" or "Pencil"),
        /// mark dirty, and recomposite. Returns true when the stroke painted
        /// anything and added history; a stroke that painted nothing leaves the
        /// document unchanged and returns false.
        #[qinvokable]
        fn end_paint(self: Pin<&mut Self>) -> bool;

        /// Drop the active stroke without committing; the document is unchanged.
        #[qinvokable]
        fn cancel_paint(self: Pin<&mut Self>);

        /// Whether a paint stroke is currently active.
        #[qinvokable]
        fn is_painting(&self) -> bool;

        /// Scale the document to `width`×`height` with resample `kind`
        /// (nearest, bilinear, bicubic), clear the selection, recomposite, and
        /// emit [`changed`]. Returns false without a document, for an unknown
        /// kind, or when a dimension is below 1.
        #[qinvokable]
        fn resize_image(self: Pin<&mut Self>, kind: &QString, width: i32, height: i32) -> bool;

        /// Place the document on a `width`×`height` canvas at `anchor`
        /// (top-left, top-center, top-right, center-left, center, center-right,
        /// bottom-left, bottom-center, bottom-right), clear the selection,
        /// recomposite, and emit [`changed`]. Returns false without a document,
        /// for an unknown anchor, or when a dimension is below 1.
        #[qinvokable]
        fn resize_canvas(self: Pin<&mut Self>, anchor: &QString, width: i32, height: i32) -> bool;

        /// Rotate the document `quarter_turns` quarter turns clockwise (1-3),
        /// clear the selection, recomposite, and emit [`changed`]. Returns
        /// false without a document or for a value outside 1-3.
        #[qinvokable]
        fn rotate_doc(self: Pin<&mut Self>, quarter_turns: i32) -> bool;

        /// Mirror the document horizontally or vertically, clear the selection,
        /// recomposite, and emit [`changed`]. Returns false without a document.
        #[qinvokable]
        fn flip_doc(self: Pin<&mut Self>, horizontal: bool) -> bool;

        #[qinvokable]
        fn undo(self: Pin<&mut Self>) -> bool;

        #[qinvokable]
        fn redo(self: Pin<&mut Self>) -> bool;

        #[qinvokable]
        fn can_undo(&self) -> bool;

        #[qinvokable]
        fn can_redo(&self) -> bool;

        #[qinvokable]
        fn history_depth(&self) -> i32;

        /// Number of labeled history states, including the current one.
        #[qinvokable]
        fn history_count(&self) -> i32;

        /// Position of the current history state, in `0..history_count()`.
        #[qinvokable]
        fn history_index(&self) -> i32;

        /// Label of history state `i`, or empty when out of range.
        #[qinvokable]
        fn history_label(&self, i: i32) -> QString;

        /// Restore history state `i`, recomposite, and emit [`changed`]. Returns
        /// false when `i` is out of range.
        #[qinvokable]
        fn history_jump(self: Pin<&mut Self>, i: i32) -> bool;

        /// Capture the current state as a named restore point. Emits [`changed`]
        /// so the History panel refreshes. Returns false without a document.
        #[qinvokable]
        fn history_add_snapshot(self: Pin<&mut Self>, label: &QString) -> bool;

        /// Number of named restore points (capped at 10).
        #[qinvokable]
        fn history_snapshot_count(&self) -> i32;

        /// Label of named restore point `i`, or empty when out of range.
        #[qinvokable]
        fn history_snapshot_label(&self, i: i32) -> QString;

        /// Restore named restore point `i`, recomposite, and emit [`changed`].
        /// Returns false when `i` is out of range.
        #[qinvokable]
        fn history_restore_snapshot(self: Pin<&mut Self>, i: i32) -> bool;

        /// Remove layer `i`, recomposite, and emit [`changed`].
        #[qinvokable]
        fn remove_layer(self: Pin<&mut Self>, i: i32);

        /// M0.5 offscreen GPU spike. Renders a gradient on Vulkan and replaces
        /// the image on success. Returns 0 = no GPU (CPU fallback kept),
        /// 1 = rendered non-blank, 2 = rendered blank.
        #[qinvokable]
        fn render_gpu(self: Pin<&mut Self>) -> i32;

        /// Create a Vulkan device for the zero-copy interop probe and keep it
        /// alive. Returns false when no device is available.
        #[qinvokable]
        fn gpu_interop_prepare(self: Pin<&mut Self>) -> bool;

        /// Raw handles from `gpu_interop_prepare`; 0 when unavailable.
        #[qinvokable]
        fn gpu_vk_instance(&self) -> u64;
        #[qinvokable]
        fn gpu_vk_physical_device(&self) -> u64;
        #[qinvokable]
        fn gpu_vk_device(&self) -> u64;
        #[qinvokable]
        fn gpu_vk_queue_family(&self) -> u32;
        /// Raw VkImage of the wgpu offscreen texture; 0 when unavailable.
        #[qinvokable]
        fn gpu_vk_image(&self) -> u64;
        #[qinvokable]
        fn gpu_image_width(&self) -> u32;
        #[qinvokable]
        fn gpu_image_height(&self) -> u32;

        /// Set the GPU-compute preference and recomposite on the new backend.
        #[qinvokable]
        fn set_gpu_compute(self: Pin<&mut Self>, enabled: bool);

        /// The persisted GPU-compute preference (true by default).
        #[qinvokable]
        fn gpu_compute(&self) -> bool;

        /// Whether a usable GPU adapter exists (cached capability probe).
        #[qinvokable]
        fn gpu_available(&self) -> bool;

        /// Active backend label: `"GPU"`, `"CPU"`, or `"CPU (no GPU)"`.
        #[qinvokable]
        fn active_backend(&self) -> QString;
    }
}

/// Backing Rust state for [`qobject::PictureView`].
pub struct PictureViewRust {
    image: QImage,
    doc: Option<Document>,
    selection: Option<Selection>,
    history: History,
    path: Option<String>,
    dirty: bool,
    interop: Option<crate::gpu::InteropState>,
    pending_lasso: Vec<(i32, i32)>,
    pending_lasso_mode: String,
    stroke: Option<Stroke>,
    stroke_label: String,
    move_base: Option<QImage>,
    move_layer: Option<QImage>,
    move_x: i32,
    move_y: i32,
    move_opacity: i32,
    move_prepared_revision: u64,
    move_prepared_layer: i32,
    move_preview_cache_hit: bool,
    content_revision: u64,
    gpu_compute: bool,
    display_dirty: bool,
}

impl Default for PictureViewRust {
    fn default() -> Self {
        Self {
            image: QImage::default(),
            doc: None,
            selection: None,
            history: History::default(),
            path: None,
            dirty: false,
            interop: None,
            pending_lasso: Vec::new(),
            pending_lasso_mode: String::new(),
            stroke: None,
            stroke_label: String::new(),
            move_base: None,
            move_layer: None,
            move_x: 0,
            move_y: 0,
            move_opacity: 0,
            move_prepared_revision: 0,
            move_prepared_layer: -1,
            move_preview_cache_hit: false,
            content_revision: 0,
            gpu_compute: true,
            display_dirty: false,
        }
    }
}

impl qobject::PictureView {
    /// Snapshot the current state, capture it under `label`, and mark dirty.
    ///
    /// Callers must run their `recomposite`/`refresh_region` first, so the
    /// captured document's composite is the current rendered image.
    fn record(mut self: Pin<&mut Self>, label: &str) {
        if let Some(snapshot) = self.snapshot() {
            let mut rust = self.as_mut().rust_mut();
            rust.history.capture(snapshot, label);
            rust.dirty = true;
            rust.content_revision = rust.content_revision.wrapping_add(1);
        }
    }

    /// As [`record`], but without bumping `content_revision`.
    ///
    /// Only for moves of the topmost pixel layer, which leave the composite
    /// below that layer unchanged, so a cached move-preview base stays valid.
    fn record_move(mut self: Pin<&mut Self>, label: &str) {
        if let Some(snapshot) = self.snapshot() {
            let mut rust = self.as_mut().rust_mut();
            rust.history.capture(snapshot, label);
            rust.dirty = true;
        }
    }

    fn snapshot(&self) -> Option<Snapshot> {
        let rust = self.rust();
        let doc = rust.doc.clone()?;
        Some(Snapshot {
            doc,
            selection: rust.selection.clone(),
        })
    }

    /// Composite only `rect`, patch the authoritative `doc.composite`, and emit
    /// [`region_blitted`] with a rectangle-sized image.
    ///
    /// The source is the active stroke's working document while painting, else
    /// the app document. While painting `doc.composite` is the pre-stroke base
    /// and is left alone (refreshed by `end_paint`'s full recomposite). An empty
    /// clamped rect is a no-op; the region path emits no `changed` and never
    /// rebuilds the full image.
    fn refresh_region(mut self: Pin<&mut Self>, rect: PsdRect) {
        let gpu_compute = self.rust().gpu_compute;
        let (region, painting) = {
            let rust = self.rust();
            let painting = rust.stroke.is_some();
            let dims = rust
                .stroke
                .as_ref()
                .map(|stroke| (stroke.document().width, stroke.document().height))
                .or_else(|| rust.doc.as_ref().map(|doc| (doc.width, doc.height)));
            (
                dims.and_then(|(width, height)| clamp_region(rect, width, height)),
                painting,
            )
        };
        let Some((x0, y0, ..)) = region else {
            return;
        };
        let region_image = {
            let mut rust = self.as_mut().rust_mut();
            let rust = &mut *rust;
            let buffer = {
                let source: &Document = if painting {
                    rust.stroke.as_ref().unwrap().document()
                } else if let Some(doc) = rust.doc.as_ref() {
                    doc
                } else {
                    return;
                };
                pictura_render::composite_region_active(source, rect, gpu_compute).0
            };
            if buffer.width == 0 || buffer.height == 0 {
                return;
            }
            if !painting {
                if let Some(doc) = rust.doc.as_mut() {
                    patch_composite_region(doc, &buffer, x0, y0);
                }
            }
            rust.display_dirty = true;
            buffer_to_image(&buffer)
        };
        self.region_blitted(region_image, x0, y0);
    }

    /// Refresh `image` from the current document and emit [`changed`].
    ///
    /// The full-document path for every mutation that does not report a dirty
    /// rectangle; [`refresh_region`] is the incremental extension point.
    fn recomposite(mut self: Pin<&mut Self>) {
        let gpu_compute = self.rust().gpu_compute;
        let rendered = self
            .rust()
            .doc
            .as_ref()
            .map(|doc| current_buffer(doc, gpu_compute));
        let Some(rendered) = rendered else {
            self.changed();
            return;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            if let Some(doc) = rust.doc.as_mut() {
                store_composite(doc, &rendered);
            }
            rust.image = buffer_to_image(&rendered);
            rust.display_dirty = false;
        }
        self.changed();
    }
}
