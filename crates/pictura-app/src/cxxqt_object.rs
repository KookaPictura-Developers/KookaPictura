//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.

use core::pin::Pin;

use crate::history::{History, Snapshot};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{AspectRatioMode, QImage, QImageFormat, QString, TransformationMode};
use pictura_core::{
    AdjustmentData, BitDepth, BlendMode, ColorMode, Document, Layer, LayerMask, PixelBuffer,
    PsdRect,
};
use pictura_select::{CombineMode, Selection};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
    }

    extern "RustQt" {
        #[qobject]
        #[namespace = "pictura"]
        type PictureView = super::PictureViewRust;

        /// Emitted whenever the layer stack changes and the image is refreshed.
        #[qsignal]
        fn changed(self: Pin<&mut Self>);

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

        /// The image to display. Never null.
        #[qinvokable]
        fn image(&self) -> QImage;

        /// Whether a document is loaded (false when only the fallback image is
        /// shown). Drives command enablement in the shell.
        #[qinvokable]
        fn has_document(&self) -> bool;

        /// Number of top-level layers in the loaded document (0 when none).
        #[qinvokable]
        fn layer_count(&self) -> i32;

        /// Name of layer `i`, or empty when out of range.
        #[qinvokable]
        fn layer_name(&self, i: i32) -> QString;

        /// `"pixel"`, `"group"`, or `"adjustment"` for layer `i`; empty when out
        /// of range.
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

        /// Set layer `i`'s opacity, clamped to `0..=255`. Captures history, marks
        /// dirty, recomposites, and emits [`changed`]. Returns false when layer
        /// `i` is out of range, leaving state unchanged.
        #[qinvokable]
        fn set_layer_opacity(self: Pin<&mut Self>, i: i32, value: i32) -> bool;

        /// Rename layer `i`. Captures history, marks dirty, recomposites, and
        /// emits [`changed`]. Returns false when layer `i` is out of range.
        #[qinvokable]
        fn set_layer_name(self: Pin<&mut Self>, i: i32, name: &QString) -> bool;

        /// Swap layer `i` with the neighbour `delta` positions away in the
        /// bottom-first list. Captures history, marks dirty, recomposites, and
        /// emits [`changed`]. Returns false when either position is out of range.
        #[qinvokable]
        fn move_layer(self: Pin<&mut Self>, i: i32, delta: i32) -> bool;

        /// The RGBA content of layer `i` scaled to fit `size`×`size`, keeping
        /// the aspect ratio with smooth filtering. Null for group or adjustment
        /// layers and when `i` or `size` is out of range.
        #[qinvokable]
        fn layer_thumbnail(&self, i: i32, size: i32) -> QImage;

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

        /// The composited pixel at `(x, y)` as `0xAARRGGBB`, or 0 when there is
        /// no document or the point is out of bounds.
        #[qinvokable]
        fn sample_argb(&self, x: i32, y: i32) -> u32;

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
    }
}

/// Backing Rust state for [`qobject::PictureView`].
#[derive(Default)]
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
}

impl qobject::PictureView {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let loaded = std::fs::read(&path)
            .ok()
            .and_then(|bytes| pictura_codec::read_psd(&bytes).ok());

        let ok = loaded.is_some();
        let image = loaded
            .as_ref()
            .map(document_to_image)
            .unwrap_or_else(test_image);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = loaded;
        view.selection = None;
        view.history = History::default();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "Open");
        }
        view.path = if ok { Some(path) } else { None };
        view.dirty = false;
        ok
    }

    pub fn new_document(
        self: Pin<&mut Self>,
        width: i32,
        height: i32,
        mode: &QString,
        depth: i32,
        background: &QString,
    ) -> bool {
        if width < 1 || height < 1 || depth != 8 {
            return false;
        }
        let mode = match mode.to_string().as_str() {
            "rgb" => ColorMode::Rgb,
            "grayscale" => ColorMode::Grayscale,
            _ => return false,
        };
        let white = match background.to_string().as_str() {
            "white" => true,
            "transparent" => false,
            _ => return false,
        };
        let mut doc = Document::new(width as u32, height as u32, mode, BitDepth::Eight);
        if white {
            doc.composite.data.fill(255);
        }
        let image = document_to_image(&doc);
        let mut view = self.rust_mut();
        view.image = image;
        view.doc = Some(doc);
        view.selection = None;
        view.history = History::default();
        let initial = view.doc.as_ref().map(|doc| Snapshot {
            doc: doc.clone(),
            selection: None,
        });
        if let Some(snapshot) = initial {
            view.history.capture(snapshot, "New");
        }
        view.path = None;
        view.dirty = false;
        true
    }

    pub fn save(self: Pin<&mut Self>, path: &QString) -> bool {
        let Some(bytes) = self
            .rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_codec::write_psd(doc).ok())
        else {
            return false;
        };
        let path = path.to_string();
        let tmp = format!("{path}.tmp");
        if std::fs::write(&tmp, &bytes).is_err() {
            return false;
        }
        if std::fs::rename(&tmp, &path).is_err() {
            return false;
        }
        let mut view = self.rust_mut();
        view.path = Some(path);
        view.dirty = false;
        true
    }

    pub fn is_dirty(&self) -> bool {
        self.rust().dirty
    }

    pub fn file_path(&self) -> QString {
        self.rust()
            .path
            .as_deref()
            .map(QString::from)
            .unwrap_or_default()
    }

    pub fn image(&self) -> QImage {
        self.rust().image.clone()
    }

    pub fn has_document(&self) -> bool {
        self.rust().doc.is_some()
    }

    pub fn layer_count(&self) -> i32 {
        self.rust()
            .doc
            .as_ref()
            .map_or(0, |d| d.layers.len() as i32)
    }

    pub fn layer_name(&self, i: i32) -> QString {
        self.layer(i)
            .map(|l| QString::from(l.name.as_str()))
            .unwrap_or_default()
    }

    pub fn layer_kind(&self, i: i32) -> QString {
        match self.layer(i) {
            Some(l) if l.is_group => QString::from("group"),
            Some(l) if l.adjustment.is_some() => QString::from("adjustment"),
            Some(_) => QString::from("pixel"),
            None => QString::default(),
        }
    }

    pub fn layer_visible(&self, i: i32) -> bool {
        self.layer(i).is_some_and(|l| l.visible)
    }

    pub fn set_layer_visible(mut self: Pin<&mut Self>, i: i32, visible: bool) {
        let changed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            match doc.layers.get_mut(i as usize) {
                Some(layer) => {
                    layer.visible = visible;
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if changed {
            self.as_mut().record("Layer Visibility");
            self.recomposite();
        }
    }

    pub fn layer_blend(&self, i: i32) -> QString {
        self.layer(i)
            .map(|l| QString::from(blend_key(l.blend).as_str()))
            .unwrap_or_default()
    }

    pub fn set_layer_blend(mut self: Pin<&mut Self>, i: i32, key: &QString) -> bool {
        let key = key.to_string();
        let bytes = key.as_bytes();
        if bytes.len() != 4 {
            return false;
        }
        let Some(mode) = BlendMode::from_psd_key([bytes[0], bytes[1], bytes[2], bytes[3]]) else {
            return false;
        };
        let changed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            match doc.layers.get_mut(i as usize) {
                Some(layer) => {
                    layer.blend = mode;
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if changed {
            self.as_mut().record("Blend Mode");
            self.recomposite();
        }
        changed
    }

    pub fn layer_opacity(&self, i: i32) -> i32 {
        self.layer(i).map_or(0, |l| l.opacity as i32)
    }

    pub fn set_layer_opacity(mut self: Pin<&mut Self>, i: i32, value: i32) -> bool {
        let value = value.clamp(0, 255) as u8;
        let changed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            match doc.layers.get_mut(i as usize) {
                Some(layer) => {
                    layer.opacity = value;
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if changed {
            self.as_mut().record("Opacity");
            self.recomposite();
        }
        changed
    }

    pub fn set_layer_name(mut self: Pin<&mut Self>, i: i32, name: &QString) -> bool {
        let name = name.to_string();
        let changed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            match doc.layers.get_mut(i as usize) {
                Some(layer) => {
                    layer.name = name;
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if changed {
            self.as_mut().record("Rename Layer");
            self.recomposite();
        }
        changed
    }

    pub fn move_layer(mut self: Pin<&mut Self>, i: i32, delta: i32) -> bool {
        let changed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            let len = doc.layers.len() as i32;
            let target = i + delta;
            if i < 0 || i >= len || delta == 0 || target < 0 || target >= len {
                false
            } else {
                doc.layers.swap(i as usize, target as usize);
                true
            }
        } else {
            false
        };
        if changed {
            self.as_mut().record("Reorder Layer");
            self.recomposite();
        }
        changed
    }

    pub fn layer_thumbnail(&self, i: i32, size: i32) -> QImage {
        if size <= 0 {
            return QImage::default();
        }
        let Some(layer) = self.layer(i) else {
            return QImage::default();
        };
        if layer.is_group || layer.adjustment.is_some() {
            return QImage::default();
        }
        let Some(image) = layer_image(layer) else {
            return QImage::default();
        };
        image.scaled(
            size,
            size,
            AspectRatioMode::KeepAspectRatio,
            TransformationMode::SmoothTransformation,
        )
    }

    pub fn select_all(mut self: Pin<&mut Self>) {
        let dims = self.rust().doc.as_ref().map(|d| (d.width, d.height));
        if let Some((w, h)) = dims {
            self.as_mut().rust_mut().selection = Some(Selection::all(w, h));
            self.as_mut().record("Select All");
            self.changed();
        }
    }

    pub fn deselect(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().selection = None;
        self.as_mut().record("Deselect");
        self.changed();
    }

    pub fn magic_wand(mut self: Pin<&mut Self>, x: i32, y: i32, tolerance: i32) -> bool {
        let picked = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if x < 0 || y < 0 {
                None
            } else {
                let tolerance = tolerance.clamp(0, 255) as u8;
                pictura_select::magic_wand(
                    &current_buffer(doc),
                    x as u32,
                    y as u32,
                    tolerance,
                    true,
                )
                .ok()
            }
        };
        match picked {
            Some(selection) => {
                self.as_mut().rust_mut().selection = Some(selection);
                self.as_mut().record("Magic Wand");
                self.changed();
                true
            }
            None => false,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.rust().selection.is_some()
    }

    pub fn selection_count(&self) -> i32 {
        self.rust()
            .selection
            .as_ref()
            .map_or(0, |s| s.data.iter().filter(|&&v| v > 0).count() as i32)
    }

    /// Merge a document-sized `shape` into the current selection with `mode`,
    /// capturing history and emitting [`changed`]. Returns false without a doc.
    fn apply_selection(mut self: Pin<&mut Self>, shape: Selection, mode: CombineMode) -> bool {
        {
            let mut rust = self.as_mut().rust_mut();
            if rust.doc.is_none() {
                return false;
            }
            let mut base = rust
                .selection
                .take()
                .unwrap_or_else(|| Selection::none(shape.width, shape.height));
            base.combine_with(&shape, mode);
            rust.selection = Some(base);
        }
        self.as_mut().record("Selection");
        self.recomposite();
        true
    }

    pub fn select_rect(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            Selection::rect(doc.width, doc.height, x, y, w, h)
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn select_ellipse(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            Selection::ellipse(doc.width, doc.height, x, y, w, h)
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn begin_lasso(mut self: Pin<&mut Self>, mode: &QString) -> bool {
        if self.rust().doc.is_none() {
            return false;
        }
        let mut rust = self.as_mut().rust_mut();
        rust.pending_lasso_mode = mode.to_string();
        rust.pending_lasso.clear();
        true
    }

    pub fn lasso_add_point(mut self: Pin<&mut Self>, x: i32, y: i32) {
        self.as_mut().rust_mut().pending_lasso.push((x, y));
    }

    pub fn end_lasso(mut self: Pin<&mut Self>) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if rust.pending_lasso.len() < 3 {
                return false;
            }
            Selection::polygon(doc.width, doc.height, &rust.pending_lasso)
        };
        let mode = combine_mode_from(&self.rust().pending_lasso_mode);
        if !self.as_mut().apply_selection(shape, mode) {
            return false;
        }
        let mut rust = self.as_mut().rust_mut();
        rust.pending_lasso.clear();
        rust.pending_lasso_mode.clear();
        true
    }

    pub fn quick_select(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        tolerance: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if x < 0 || y < 0 || x as u32 >= doc.width || y as u32 >= doc.height {
                return false;
            }
            let tolerance = tolerance.clamp(0, 255) as u8;
            match pictura_select::magic_wand(
                &current_buffer(doc),
                x as u32,
                y as u32,
                tolerance,
                true,
            ) {
                Ok(selection) => selection,
                Err(_) => return false,
            }
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn crop(mut self: Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) -> bool {
        if w < 1 || h < 1 {
            return false;
        }
        let cropped = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if !pictura_render::crop_document(doc, x, y, w as u32, h as u32) {
                return false;
            }
            // Canvas dimensions changed, so the old selection no longer maps.
            rust.selection = None;
            true
        };
        if cropped {
            self.as_mut().record("Crop");
            self.recomposite();
        }
        cropped
    }

    pub fn translate_layer(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer(doc, dx, dy)
        };
        if moved {
            self.as_mut().record("Move Layer");
            self.recomposite();
        }
        moved
    }

    pub fn sample_argb(&self, x: i32, y: i32) -> u32 {
        let rust = self.rust();
        let Some(doc) = rust.doc.as_ref() else {
            return 0;
        };
        if x < 0 || y < 0 || x as u32 >= doc.width || y as u32 >= doc.height {
            return 0;
        }
        argb_at(&current_buffer(doc), x as u32, y as u32)
    }

    pub fn selection_bounds(&self) -> QString {
        let rust = self.rust();
        let Some(sel) = rust.selection.as_ref() else {
            return QString::default();
        };
        let mut bounds: Option<(u32, u32, u32, u32)> = None;
        for (i, &v) in sel.data.iter().enumerate() {
            if v == 0 {
                continue;
            }
            let x = i as u32 % sel.width;
            let y = i as u32 / sel.width;
            bounds = Some(match bounds {
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                None => (x, y, x, y),
            });
        }
        let Some((x0, y0, x1, y1)) = bounds else {
            return QString::default();
        };
        QString::from(format!("{x0} {y0} {} {}", x1 - x0 + 1, y1 - y0 + 1))
    }

    pub fn add_adjustment(mut self: Pin<&mut Self>, kind: &QString) -> bool {
        let mask = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            rust.selection
                .as_ref()
                .map(|selection| selection_to_mask(selection, doc))
        };
        let Some(layer) = adjustment_layer(&kind.to_string(), mask) else {
            return false;
        };
        let pushed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                doc.layers.push(layer);
                true
            }
            None => false,
        };
        if pushed {
            self.as_mut().record("Adjustment");
            self.recomposite();
        }
        pushed
    }

    pub fn apply_filter(mut self: Pin<&mut Self>, kind: &QString) -> bool {
        let Some(filter) = filter_from_kind(&kind.to_string()) else {
            return false;
        };
        let mask = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            rust.selection
                .as_ref()
                .map(|selection| selection_to_mask(selection, doc))
        };
        let applied = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            let Some(layer) = topmost_pixel_layer(doc) else {
                return false;
            };
            pictura_render::apply_filter(layer, &filter, mask.as_ref()).is_ok()
        };
        if applied {
            self.as_mut().record("Filter");
            self.recomposite();
        }
        applied
    }

    pub fn resize_image(mut self: Pin<&mut Self>, kind: &QString, width: i32, height: i32) -> bool {
        let Some(resample) = parse_resample(&kind.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_document(doc, width as u32, height as u32, resample).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().record("Image Size");
            self.recomposite();
        }
        resized
    }

    pub fn resize_canvas(
        mut self: Pin<&mut Self>,
        anchor: &QString,
        width: i32,
        height: i32,
    ) -> bool {
        let Some(anchor) = parse_anchor(&anchor.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_canvas_document(doc, width as u32, height as u32, anchor).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().record("Canvas Size");
            self.recomposite();
        }
        resized
    }

    pub fn rotate_doc(mut self: Pin<&mut Self>, quarter_turns: i32) -> bool {
        if !(1..=3).contains(&quarter_turns) {
            return false;
        }
        let rotated = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::rotate_document(doc, quarter_turns as u8).is_ok()
        };
        if rotated {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().record("Rotate");
            self.recomposite();
        }
        rotated
    }

    pub fn flip_doc(mut self: Pin<&mut Self>, horizontal: bool) -> bool {
        {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::flip_document(doc, horizontal);
            rust.selection = None;
        }
        self.as_mut().record("Flip");
        self.recomposite();
        true
    }

    pub fn undo(mut self: Pin<&mut Self>) -> bool {
        let restored = self.as_mut().rust_mut().history.undo();
        let Some(snapshot) = restored else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        self.recomposite();
        true
    }

    pub fn redo(mut self: Pin<&mut Self>) -> bool {
        let restored = self.as_mut().rust_mut().history.redo();
        let Some(snapshot) = restored else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        self.recomposite();
        true
    }

    pub fn can_undo(&self) -> bool {
        self.rust().history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.rust().history.can_redo()
    }

    pub fn history_depth(&self) -> i32 {
        self.rust().history.depth() as i32
    }

    pub fn history_count(&self) -> i32 {
        self.rust().history.count() as i32
    }

    pub fn history_index(&self) -> i32 {
        self.rust().history.index() as i32
    }

    pub fn history_label(&self, i: i32) -> QString {
        if i < 0 {
            return QString::default();
        }
        QString::from(self.rust().history.label(i as usize))
    }

    pub fn history_jump(mut self: Pin<&mut Self>, i: i32) -> bool {
        if i < 0 {
            return false;
        }
        let restored = self.as_mut().rust_mut().history.jump(i as usize);
        let Some(snapshot) = restored else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        self.recomposite();
        true
    }

    pub fn history_add_snapshot(mut self: Pin<&mut Self>, label: &QString) -> bool {
        let Some(snapshot) = self.snapshot() else {
            return false;
        };
        self.as_mut()
            .rust_mut()
            .history
            .add_snapshot(&label.to_string(), snapshot);
        self.changed();
        true
    }

    pub fn history_snapshot_count(&self) -> i32 {
        self.rust().history.snapshot_count() as i32
    }

    pub fn history_snapshot_label(&self, i: i32) -> QString {
        if i < 0 {
            return QString::default();
        }
        QString::from(self.rust().history.snapshot_label(i as usize))
    }

    pub fn history_restore_snapshot(mut self: Pin<&mut Self>, i: i32) -> bool {
        if i < 0 {
            return false;
        }
        let Some(snapshot) = self.rust().history.snapshot(i as usize) else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        self.recomposite();
        true
    }

    /// Snapshot the current state, capture it under `label`, and mark dirty.
    fn record(mut self: Pin<&mut Self>, label: &str) {
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

    pub fn remove_layer(mut self: Pin<&mut Self>, i: i32) {
        let removed = if let Some(doc) = self.as_mut().rust_mut().doc.as_mut() {
            let idx = i as usize;
            if idx < doc.layers.len() {
                doc.layers.remove(idx);
                true
            } else {
                false
            }
        } else {
            false
        };
        if removed {
            self.as_mut().record("Delete Layer");
            self.recomposite();
        }
    }

    pub fn render_gpu(self: Pin<&mut Self>) -> i32 {
        let (width, height) = (512u32, 512u32);
        match crate::gpu::render_gradient(width, height) {
            crate::gpu::GpuRender::Unavailable => 0,
            crate::gpu::GpuRender::Rendered {
                width: w,
                height: h,
                rgba,
                distinct,
            } => {
                self.rust_mut().image = rgba_image(rgba, w as i32, h as i32);
                if distinct >= 2 {
                    1
                } else {
                    2
                }
            }
        }
    }

    pub fn gpu_interop_prepare(self: Pin<&mut Self>) -> bool {
        let state = crate::gpu::create_interop_state();
        let ok = state.is_some();
        self.rust_mut().interop = state;
        ok
    }

    pub fn gpu_vk_instance(&self) -> u64 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.instance)
    }

    pub fn gpu_vk_physical_device(&self) -> u64 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.physical_device)
    }

    pub fn gpu_vk_device(&self) -> u64 {
        self.rust().interop.as_ref().map_or(0, |s| s.handles.device)
    }

    pub fn gpu_vk_queue_family(&self) -> u32 {
        self.rust()
            .interop
            .as_ref()
            .map_or(0, |s| s.handles.queue_family)
    }

    pub fn gpu_vk_image(&self) -> u64 {
        self.rust().interop.as_ref().map_or(0, |s| s.image)
    }

    pub fn gpu_image_width(&self) -> u32 {
        self.rust().interop.as_ref().map_or(0, |s| s.width)
    }

    pub fn gpu_image_height(&self) -> u32 {
        self.rust().interop.as_ref().map_or(0, |s| s.height)
    }

    fn layer(&self, i: i32) -> Option<&Layer> {
        self.rust().doc.as_ref()?.layers.get(i as usize)
    }

    /// Refresh `image` from the current document and emit [`changed`].
    fn recomposite(mut self: Pin<&mut Self>) {
        let image = self.rust().doc.as_ref().map(document_to_image);
        if let Some(image) = image {
            self.as_mut().rust_mut().image = image;
        }
        self.changed();
    }
}

/// Build an adjustment layer for `kind`, or `None` for an unknown kind.
///
/// Defaults are chosen so a freshly added layer visibly changes the composite;
/// editing parameters is out of scope for M4-C. `mask` confines the effect to a
/// selection when one is active.
fn adjustment_layer(kind: &str, mask: Option<LayerMask>) -> Option<Layer> {
    use pictura_render::{
        encode_brightness_contrast, encode_hue_saturation, encode_invert, encode_posterize,
        encode_threshold,
    };

    let (name, data): (&str, AdjustmentData) = match kind {
        "invert" => ("Invert", encode_invert()),
        "posterize" => ("Posterize", encode_posterize(4)),
        "threshold" => ("Threshold", encode_threshold(128)),
        "brightness-contrast" => ("Brightness/Contrast", encode_brightness_contrast(20, 0)),
        "hue-saturation" => ("Hue/Saturation", encode_hue_saturation(30, 0, 0)),
        _ => return None,
    };

    Some(Layer {
        name: name.into(),
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        },
        blend: BlendMode::Normal,
        opacity: 255,
        clipping: false,
        visible: true,
        mask,
        adjustment: Some(data),
        channels: Vec::new(),
        children: Vec::new(),
        is_group: false,
    })
}

/// Map a filter `kind` to its [`pictura_filters::Filter`], or `None` unknown.
///
/// Defaults are chosen so a fresh apply visibly changes a non-trivial image;
/// filter dialogs are out of scope for M6-C.
fn filter_from_kind(kind: &str) -> Option<pictura_filters::Filter> {
    use pictura_filters::{
        Filter, LensType, MezzotintType, NoiseDistribution, PolarKind, RippleSize, ShearFill,
        SpherizeMode, WaveType, ZigZagStyle,
    };

    Some(match kind {
        "gaussian-blur" => Filter::GaussianBlur { radius: 5.0 },
        "box-blur" => Filter::BoxBlur { radius: 3 },
        "motion-blur" => Filter::MotionBlur {
            angle: 0.0,
            distance: 15,
        },
        "median" => Filter::Median { radius: 2 },
        "despeckle" => Filter::Despeckle,
        "sharpen" => Filter::Sharpen,
        "sharpen-more" => Filter::SharpenMore,
        "unsharp-mask" => Filter::UnsharpMask {
            amount: 150.0,
            radius: 1.0,
            threshold: 0,
        },
        "add-noise" => Filter::AddNoise {
            amount: 25.0,
            distribution: NoiseDistribution::Uniform,
            monochromatic: false,
            seed: 1,
        },
        "maximum" => Filter::Maximum { radius: 2 },
        "minimum" => Filter::Minimum { radius: 2 },
        "offset" => Filter::Offset {
            horizontal: 4,
            vertical: 4,
            wrap: true,
            background: [0, 0, 0],
        },
        "high-pass" => Filter::HighPass { radius: 4.0 },
        "emboss" => Filter::Emboss {
            angle: 135.0,
            height: 2.0,
            amount: 100.0,
        },
        "find-edges" => Filter::FindEdges,
        "solarize" => Filter::Solarize,
        "mosaic" => Filter::Mosaic { cell_size: 10 },
        "crystallize" => Filter::Crystallize {
            cell_size: 10,
            seed: 1,
        },
        "facet" => Filter::Facet,
        "fragment" => Filter::Fragment,
        "mezzotint" => Filter::Mezzotint {
            kind: MezzotintType::FineDots,
            seed: 1,
        },
        "pointillize" => Filter::Pointillize {
            cell_size: 5,
            background: [0, 0, 0],
            seed: 1,
        },
        "color-halftone" => Filter::ColorHalftone {
            max_radius: 5,
            angles: [108.0, 162.0, 90.0, 45.0],
        },
        "twirl" => Filter::Twirl { angle: 90.0 },
        "pinch" => Filter::Pinch { amount: 50.0 },
        "spherize" => Filter::Spherize {
            amount: 100.0,
            mode: SpherizeMode::Normal,
        },
        "ripple" => Filter::Ripple {
            amount: 100.0,
            size: RippleSize::Medium,
        },
        "wave" => Filter::Wave {
            generators: 5,
            wavelength: (10.0, 120.0),
            amplitude: (5.0, 35.0),
            kind: WaveType::Sine,
            scale: (100.0, 100.0),
            seed: 1,
            repeat_edge: true,
        },
        "polar-coordinates" => Filter::PolarCoordinates {
            kind: PolarKind::RectangularToPolar,
        },
        "shear" => Filter::Shear {
            curve: vec![(-1.0, -0.5), (0.0, 0.0), (1.0, 0.5)],
            fill: ShearFill::RepeatEdgePixels,
        },
        "zigzag" => Filter::ZigZag {
            amount: 50.0,
            ridges: 5,
            style: ZigZagStyle::AroundCenter,
        },
        "ocean-ripple" => Filter::OceanRipple {
            size: 9,
            magnitude: 5,
            seed: 1,
        },
        "clouds" => Filter::Clouds {
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            starker: false,
            seed: 1,
        },
        "difference-clouds" => Filter::DifferenceClouds {
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            starker: false,
            seed: 1,
        },
        "fibers" => Filter::Fibers {
            variance: 16.0,
            strength: 4.0,
            color_a: [0, 0, 0],
            color_b: [255, 255, 255],
            seed: 1,
        },
        "lens-flare" => Filter::LensFlare {
            brightness: 100.0,
            center: (0.5, 0.5),
            lens: LensType::Zoom,
        },
        // `Custom` requires a caller-supplied 5x5 kernel, so no meaningful
        // default exists; it stays out of the dock and is left unmapped.
        _ => return None,
    })
}

/// Map an image-size resample `kind` to [`pictura_render::Resample`], or
/// `None` unknown.
fn parse_resample(kind: &str) -> Option<pictura_render::Resample> {
    use pictura_render::Resample;

    Some(match kind {
        "nearest" => Resample::Nearest,
        "bilinear" => Resample::Bilinear,
        "bicubic" => Resample::Bicubic,
        _ => return None,
    })
}

/// Map a canvas-size `anchor` to [`pictura_render::Anchor`], or `None` unknown.
fn parse_anchor(anchor: &str) -> Option<pictura_render::Anchor> {
    use pictura_render::Anchor;

    Some(match anchor {
        "top-left" => Anchor::TopLeft,
        "top-center" => Anchor::TopCenter,
        "top-right" => Anchor::TopRight,
        "center-left" => Anchor::MiddleLeft,
        "center" => Anchor::Center,
        "center-right" => Anchor::MiddleRight,
        "bottom-left" => Anchor::BottomLeft,
        "bottom-center" => Anchor::BottomCenter,
        "bottom-right" => Anchor::BottomRight,
        _ => return None,
    })
}

/// Map a selection `mode` string to [`CombineMode`]; unknown means `New`.
fn combine_mode_from(mode: &str) -> CombineMode {
    match mode {
        "add" => CombineMode::Add,
        "subtract" => CombineMode::Subtract,
        "intersect" => CombineMode::Intersect,
        _ => CombineMode::New,
    }
}

/// Pack one pixel of a planar 8-bit buffer as `0xAARRGGBB`.
fn argb_at(buffer: &PixelBuffer, x: u32, y: u32) -> u32 {
    let plane = buffer.width as usize * buffer.height as usize;
    let i = y as usize * buffer.width as usize + x as usize;
    let (r, g, b, a) = match buffer.channels {
        0 => return 0,
        1 => {
            let v = buffer.data[i];
            (v, v, v, 255)
        }
        2 => {
            let v = buffer.data[i];
            (v, v, v, buffer.data[plane + i])
        }
        3 => (
            buffer.data[i],
            buffer.data[plane + i],
            buffer.data[2 * plane + i],
            255,
        ),
        _ => (
            buffer.data[i],
            buffer.data[plane + i],
            buffer.data[2 * plane + i],
            buffer.data[3 * plane + i],
        ),
    };
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// The topmost pixel layer: the last layer (bottom-first order) that is
/// neither a group nor an adjustment.
fn topmost_pixel_layer(doc: &mut Document) -> Option<&mut Layer> {
    doc.layers
        .iter_mut()
        .rev()
        .find(|l| l.adjustment.is_none() && !l.is_group)
}

/// A full-frame raster mask whose coverage is the selection.
fn selection_to_mask(selection: &Selection, doc: &Document) -> LayerMask {
    LayerMask {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: doc.height as i32,
            right: doc.width as i32,
        },
        default_color: 0,
        disabled: false,
        flags: 0,
        data: Some(selection.data.clone()),
    }
}

/// The buffer a wand samples: the composited layer stack when present,
/// otherwise the embedded PSD composite.
fn current_buffer(doc: &Document) -> PixelBuffer {
    if doc.layers.is_empty() {
        doc.composite.clone()
    } else {
        pictura_render::composite_rgba(doc)
    }
}

/// Convert the document to a packed RGBA `QImage`.
fn document_to_image(doc: &Document) -> QImage {
    buffer_to_image(&current_buffer(doc))
}

/// The 4-byte PSD blend key as a `String` (e.g. `"mul "`).
fn blend_key(mode: BlendMode) -> String {
    String::from_utf8_lossy(&mode.to_psd_key()).into_owned()
}

/// Convert a pixel layer's planar channels to an RGBA `QImage`, or `None` for an
/// empty rect, a missing/wrong-sized color channel, or an alpha mismatch.
fn layer_image(layer: &Layer) -> Option<QImage> {
    let width = layer.rect.width();
    let height = layer.rect.height();
    if width <= 0 || height <= 0 {
        return None;
    }
    let (width, height) = (width as u32, height as u32);
    let plane = (width * height) as usize;
    let channel = |id: i16| layer.channels.iter().find(|c| c.id == id).map(|c| &c.data);

    let alpha = channel(-1);
    let buffer = if channel(1).is_none() && channel(2).is_none() {
        let gray = channel(0)?;
        if gray.len() != plane {
            return None;
        }
        let mut data = vec![0u8; plane * 2];
        data[..plane].copy_from_slice(gray);
        match alpha {
            Some(a) if a.len() == plane => data[plane..].copy_from_slice(a),
            Some(_) => return None,
            None => data[plane..].fill(255),
        }
        PixelBuffer {
            width,
            height,
            channels: 2,
            data,
        }
    } else {
        let (r, g, b) = (channel(0)?, channel(1)?, channel(2)?);
        if r.len() != plane || g.len() != plane || b.len() != plane {
            return None;
        }
        let mut data = vec![0u8; plane * 4];
        data[..plane].copy_from_slice(r);
        data[plane..2 * plane].copy_from_slice(g);
        data[2 * plane..3 * plane].copy_from_slice(b);
        match alpha {
            Some(a) if a.len() == plane => data[3 * plane..].copy_from_slice(a),
            Some(_) => return None,
            None => data[3 * plane..].fill(255),
        }
        PixelBuffer {
            width,
            height,
            channels: 4,
            data,
        }
    };
    Some(buffer_to_image(&buffer))
}

/// Convert a planar 8-bit buffer (1 = gray, 2 = gray+alpha, 3 = RGB, 4 = RGBA)
/// to interleaved RGBA8888. Gray replicates across RGB; RGB gets opaque alpha.
fn buffer_to_image(buffer: &PixelBuffer) -> QImage {
    let width = buffer.width as i32;
    let height = buffer.height as i32;
    let plane = buffer.width as usize * buffer.height as usize;
    let channels = buffer.channels as usize;

    let mut rgba = vec![0u8; plane * 4];
    for i in 0..plane {
        let (r, g, b, a) = if channels <= 1 {
            let v = buffer.data[i];
            (v, v, v, 255)
        } else if channels == 2 {
            let v = buffer.data[i];
            (v, v, v, buffer.data[plane + i])
        } else {
            let a = if channels >= 4 {
                buffer.data[3 * plane + i]
            } else {
                255
            };
            (
                buffer.data[i],
                buffer.data[plane + i],
                buffer.data[2 * plane + i],
                a,
            )
        };
        let o = i * 4;
        rgba[o..o + 4].copy_from_slice(&[r, g, b, a]);
    }

    // SAFETY: `rgba` is exactly width*height RGBA8888 bytes, tightly packed.
    unsafe { QImage::from_raw_bytes(rgba, width, height, QImageFormat::Format_RGBA8888) }
}

/// Wrap packed RGBA8888 bytes as a `QImage`.
fn rgba_image(rgba: Vec<u8>, width: i32, height: i32) -> QImage {
    // SAFETY: `rgba` is exactly width*height RGBA8888 bytes.
    unsafe { QImage::from_raw_bytes(rgba, width, height, QImageFormat::Format_RGBA8888) }
}

/// Deterministic gradient so the window always has something to show.
fn test_image() -> QImage {
    let (width, height) = (512i32, 512i32);
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let o = ((y * width + x) * 4) as usize;
            rgba[o] = (x * 255 / width) as u8;
            rgba[o + 1] = (y * 255 / height) as u8;
            rgba[o + 2] = ((x ^ y) & 0xff) as u8;
            rgba[o + 3] = 255;
        }
    }

    rgba_image(rgba, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, BlendMode, Channel, ColorMode, Layer, PsdRect};

    #[test]
    fn converts_planar_rgb_to_rgba() {
        let mut doc = Document::new(2, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.composite.data = vec![10, 20, 30, 40, 50, 60];
        let image = document_to_image(&doc);
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 1);
        assert_eq!(image.pixel_color(0, 0).red(), 10);
        assert_eq!(image.pixel_color(0, 0).green(), 30);
        assert_eq!(image.pixel_color(0, 0).blue(), 50);
        assert_eq!(image.pixel_color(0, 0).alpha(), 255);
    }

    #[test]
    fn layered_document_composites_with_source_alpha() {
        let mut doc = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![Layer {
            name: "red".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 1,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![255],
                },
                Channel {
                    id: 1,
                    data: vec![0],
                },
                Channel {
                    id: 2,
                    data: vec![0],
                },
                Channel {
                    id: -1,
                    data: vec![255],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }];

        let image = document_to_image(&doc);
        assert_eq!(image.pixel_color(0, 0).red(), 255);
        assert_eq!(image.pixel_color(0, 0).alpha(), 255);
        // Uncovered canvas stays transparent, not the embedded composite.
        assert_eq!(image.pixel_color(1, 1).alpha(), 0);
    }

    #[test]
    fn invert_and_visibility_change_composite() {
        let mut doc = Document::new(1, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![Layer {
            name: "red".into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: 1,
                right: 1,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![255],
                },
                Channel {
                    id: 1,
                    data: vec![0],
                },
                Channel {
                    id: 2,
                    data: vec![0],
                },
                Channel {
                    id: -1,
                    data: vec![255],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }];

        let before = document_to_image(&doc);
        assert_eq!(before.pixel_color(0, 0).red(), 255);

        doc.layers
            .push(adjustment_layer("invert", None).expect("known kind"));
        let after = document_to_image(&doc);
        assert_eq!(after.pixel_color(0, 0).red(), 0);
        assert_eq!(after.pixel_color(0, 0).green(), 255);
        assert_eq!(after.pixel_color(0, 0).blue(), 255);

        doc.layers[0].visible = false;
        let hidden = document_to_image(&doc);
        assert_eq!(hidden.pixel_color(0, 0).alpha(), 0);

        assert!(adjustment_layer("bogus", None).is_none());
    }

    fn pixel_layer(name: &str, w: u32, h: u32, rgb: (u8, u8, u8)) -> Layer {
        let n = (w * h) as usize;
        Layer {
            name: name.into(),
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: h as i32,
                right: w as i32,
            },
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![rgb.0; n],
                },
                Channel {
                    id: 1,
                    data: vec![rgb.1; n],
                },
                Channel {
                    id: 2,
                    data: vec![rgb.2; n],
                },
                Channel {
                    id: -1,
                    data: vec![255; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }
    }

    #[test]
    fn selection_becomes_full_frame_mask_that_confines_adjustment() {
        let mut doc = Document::new(4, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel_layer("base", 4, 1, (255, 0, 0))];

        let selection = Selection {
            width: 4,
            height: 1,
            data: vec![255, 255, 0, 0],
        };
        let mask = selection_to_mask(&selection, &doc);
        assert_eq!(mask.rect.right, 4);
        assert_eq!(mask.rect.bottom, 1);
        assert_eq!(mask.data.as_deref(), Some(&[255u8, 255, 0, 0][..]));

        doc.layers
            .push(adjustment_layer("invert", Some(mask)).expect("known kind"));
        let image = document_to_image(&doc);
        // Selected half inverts red -> cyan; unselected half is untouched.
        assert_eq!(image.pixel_color(0, 0).red(), 0);
        assert_eq!(image.pixel_color(0, 0).green(), 255);
        assert_eq!(image.pixel_color(0, 0).blue(), 255);
        assert_eq!(image.pixel_color(3, 0).red(), 255);
        assert_eq!(image.pixel_color(3, 0).blue(), 0);
    }

    #[test]
    fn test_image_is_not_null() {
        assert!(!test_image().is_null());
    }

    #[test]
    fn filter_from_kind_maps_known_and_rejects_unknown() {
        use pictura_filters::{
            Filter, LensType, MezzotintType, NoiseDistribution, PolarKind, RippleSize, ShearFill,
            SpherizeMode, WaveType, ZigZagStyle,
        };

        assert_eq!(
            filter_from_kind("gaussian-blur"),
            Some(Filter::GaussianBlur { radius: 5.0 })
        );
        assert_eq!(
            filter_from_kind("box-blur"),
            Some(Filter::BoxBlur { radius: 3 })
        );
        assert_eq!(
            filter_from_kind("motion-blur"),
            Some(Filter::MotionBlur {
                angle: 0.0,
                distance: 15,
            })
        );
        assert_eq!(
            filter_from_kind("median"),
            Some(Filter::Median { radius: 2 })
        );
        assert_eq!(filter_from_kind("despeckle"), Some(Filter::Despeckle));
        assert_eq!(filter_from_kind("sharpen"), Some(Filter::Sharpen));
        assert_eq!(filter_from_kind("sharpen-more"), Some(Filter::SharpenMore));
        assert_eq!(
            filter_from_kind("unsharp-mask"),
            Some(Filter::UnsharpMask {
                amount: 150.0,
                radius: 1.0,
                threshold: 0,
            })
        );
        assert_eq!(
            filter_from_kind("add-noise"),
            Some(Filter::AddNoise {
                amount: 25.0,
                distribution: NoiseDistribution::Uniform,
                monochromatic: false,
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("maximum"),
            Some(Filter::Maximum { radius: 2 })
        );
        assert_eq!(
            filter_from_kind("minimum"),
            Some(Filter::Minimum { radius: 2 })
        );
        assert_eq!(
            filter_from_kind("offset"),
            Some(Filter::Offset {
                horizontal: 4,
                vertical: 4,
                wrap: true,
                background: [0, 0, 0],
            })
        );
        assert_eq!(
            filter_from_kind("high-pass"),
            Some(Filter::HighPass { radius: 4.0 })
        );
        assert_eq!(
            filter_from_kind("emboss"),
            Some(Filter::Emboss {
                angle: 135.0,
                height: 2.0,
                amount: 100.0,
            })
        );
        assert_eq!(filter_from_kind("find-edges"), Some(Filter::FindEdges));
        assert_eq!(filter_from_kind("solarize"), Some(Filter::Solarize));
        assert_eq!(
            filter_from_kind("mosaic"),
            Some(Filter::Mosaic { cell_size: 10 })
        );
        assert_eq!(
            filter_from_kind("crystallize"),
            Some(Filter::Crystallize {
                cell_size: 10,
                seed: 1,
            })
        );
        assert_eq!(filter_from_kind("facet"), Some(Filter::Facet));
        assert_eq!(filter_from_kind("fragment"), Some(Filter::Fragment));
        assert_eq!(
            filter_from_kind("mezzotint"),
            Some(Filter::Mezzotint {
                kind: MezzotintType::FineDots,
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("pointillize"),
            Some(Filter::Pointillize {
                cell_size: 5,
                background: [0, 0, 0],
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("color-halftone"),
            Some(Filter::ColorHalftone {
                max_radius: 5,
                angles: [108.0, 162.0, 90.0, 45.0],
            })
        );
        assert_eq!(
            filter_from_kind("twirl"),
            Some(Filter::Twirl { angle: 90.0 })
        );
        assert_eq!(
            filter_from_kind("pinch"),
            Some(Filter::Pinch { amount: 50.0 })
        );
        assert_eq!(
            filter_from_kind("spherize"),
            Some(Filter::Spherize {
                amount: 100.0,
                mode: SpherizeMode::Normal,
            })
        );
        assert_eq!(
            filter_from_kind("ripple"),
            Some(Filter::Ripple {
                amount: 100.0,
                size: RippleSize::Medium,
            })
        );
        assert_eq!(
            filter_from_kind("wave"),
            Some(Filter::Wave {
                generators: 5,
                wavelength: (10.0, 120.0),
                amplitude: (5.0, 35.0),
                kind: WaveType::Sine,
                scale: (100.0, 100.0),
                seed: 1,
                repeat_edge: true,
            })
        );
        assert_eq!(
            filter_from_kind("polar-coordinates"),
            Some(Filter::PolarCoordinates {
                kind: PolarKind::RectangularToPolar,
            })
        );
        assert_eq!(
            filter_from_kind("shear"),
            Some(Filter::Shear {
                curve: vec![(-1.0, -0.5), (0.0, 0.0), (1.0, 0.5)],
                fill: ShearFill::RepeatEdgePixels,
            })
        );
        assert_eq!(
            filter_from_kind("zigzag"),
            Some(Filter::ZigZag {
                amount: 50.0,
                ridges: 5,
                style: ZigZagStyle::AroundCenter,
            })
        );
        assert_eq!(
            filter_from_kind("ocean-ripple"),
            Some(Filter::OceanRipple {
                size: 9,
                magnitude: 5,
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("clouds"),
            Some(Filter::Clouds {
                color_a: [0, 0, 0],
                color_b: [255, 255, 255],
                starker: false,
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("difference-clouds"),
            Some(Filter::DifferenceClouds {
                color_a: [0, 0, 0],
                color_b: [255, 255, 255],
                starker: false,
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("fibers"),
            Some(Filter::Fibers {
                variance: 16.0,
                strength: 4.0,
                color_a: [0, 0, 0],
                color_b: [255, 255, 255],
                seed: 1,
            })
        );
        assert_eq!(
            filter_from_kind("lens-flare"),
            Some(Filter::LensFlare {
                brightness: 100.0,
                center: (0.5, 0.5),
                lens: LensType::Zoom,
            })
        );
        assert_eq!(filter_from_kind("bogus"), None);
    }

    #[test]
    fn filter_confines_to_selection_and_skips_adjustment_layer() {
        let mut doc = Document::new(8, 1, ColorMode::Rgb, BitDepth::Eight);
        let mut base = pixel_layer("base", 8, 1, (40, 40, 40));
        for ch in base.channels.iter_mut().filter(|c| c.id >= 0) {
            ch.data = (0..8).map(|x| if x < 4 { 40u8 } else { 200 }).collect();
        }
        // A topmost adjustment must be skipped in favour of the pixel layer.
        doc.layers = vec![base, adjustment_layer("invert", None).expect("known kind")];

        let selection = Selection {
            width: 8,
            height: 1,
            data: vec![255, 255, 255, 0, 0, 0, 0, 0],
        };
        let mask = selection_to_mask(&selection, &doc);
        let layer = topmost_pixel_layer(&mut doc).expect("pixel layer");
        assert_eq!(layer.name, "base");
        let before = layer
            .channels
            .iter()
            .find(|c| c.id == 0)
            .expect("red channel")
            .data
            .clone();

        pictura_render::apply_filter(
            layer,
            &filter_from_kind("gaussian-blur").expect("known kind"),
            Some(&mask),
        )
        .expect("filter applies");

        let after = &layer.channels.iter().find(|c| c.id == 0).unwrap().data;
        assert!(
            after[..4].iter().zip(&before[..4]).any(|(a, b)| a != b),
            "selected pixels should change"
        );
        assert_eq!(&after[4..], &before[4..], "unselected pixels changed");
    }

    #[test]
    fn parse_resample_maps_known_and_rejects_unknown() {
        use pictura_render::Resample;

        assert_eq!(parse_resample("nearest"), Some(Resample::Nearest));
        assert_eq!(parse_resample("bilinear"), Some(Resample::Bilinear));
        assert_eq!(parse_resample("bicubic"), Some(Resample::Bicubic));
        assert_eq!(parse_resample("Bilinear"), None);
        assert_eq!(parse_resample(""), None);
        assert_eq!(parse_resample("gaussian"), None);
    }

    #[test]
    fn parse_anchor_maps_known_and_rejects_unknown() {
        use pictura_render::Anchor;

        assert_eq!(parse_anchor("top-left"), Some(Anchor::TopLeft));
        assert_eq!(parse_anchor("top-center"), Some(Anchor::TopCenter));
        assert_eq!(parse_anchor("top-right"), Some(Anchor::TopRight));
        assert_eq!(parse_anchor("center-left"), Some(Anchor::MiddleLeft));
        assert_eq!(parse_anchor("center"), Some(Anchor::Center));
        assert_eq!(parse_anchor("center-right"), Some(Anchor::MiddleRight));
        assert_eq!(parse_anchor("bottom-left"), Some(Anchor::BottomLeft));
        assert_eq!(parse_anchor("bottom-center"), Some(Anchor::BottomCenter));
        assert_eq!(parse_anchor("bottom-right"), Some(Anchor::BottomRight));
        assert_eq!(parse_anchor("top"), None);
        assert_eq!(parse_anchor("middle"), None);
        assert_eq!(parse_anchor(""), None);
    }

    #[test]
    fn document_ops_wire_parsed_values_and_reject_invalid_params() {
        let mut doc = Document::new(2, 1, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel_layer("base", 2, 1, (0, 0, 0))];
        doc.layers[0].channels = vec![
            Channel {
                id: 0,
                data: vec![255, 0],
            },
            Channel {
                id: 1,
                data: vec![0, 0],
            },
            Channel {
                id: 2,
                data: vec![0, 255],
            },
            Channel {
                id: -1,
                data: vec![255, 255],
            },
        ];

        pictura_render::flip_document(&mut doc, true);
        let plane = 2usize;
        assert_eq!(
            [
                doc.composite.data[0],
                doc.composite.data[plane],
                doc.composite.data[2 * plane]
            ],
            [0, 0, 255],
            "left pixel mirrors to the old right pixel"
        );
        assert_eq!(
            [
                doc.composite.data[1],
                doc.composite.data[plane + 1],
                doc.composite.data[2 * plane + 1]
            ],
            [255, 0, 0],
            "right pixel mirrors to the old left pixel"
        );

        pictura_render::rotate_document(&mut doc, 1).expect("valid rotate");
        assert_eq!((doc.width, doc.height), (1, 2));

        pictura_render::resize_document(&mut doc, 4, 4, parse_resample("bicubic").unwrap())
            .expect("valid resize");
        assert_eq!((doc.width, doc.height), (4, 4));

        let snapshot = doc.composite.data.clone();
        assert!(pictura_render::resize_document(
            &mut doc,
            0,
            4,
            parse_resample("nearest").unwrap()
        )
        .is_err());
        assert_eq!((doc.width, doc.height), (4, 4));
        assert_eq!(
            doc.composite.data, snapshot,
            "failed resize must not mutate"
        );
    }
}
