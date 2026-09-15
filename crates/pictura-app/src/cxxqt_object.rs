//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.

use core::pin::Pin;

use crate::history::{History, Snapshot};
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QImageFormat, QString};
use pictura_core::{
    AdjustmentData, BitDepth, BlendMode, ColorMode, Document, Layer, LayerMask, PixelBuffer,
    PsdRect,
};
use pictura_select::Selection;

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
        let snapshot = self.snapshot();
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
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
            self.recomposite();
        }
    }

    pub fn select_all(mut self: Pin<&mut Self>) {
        let dims = self.rust().doc.as_ref().map(|d| (d.width, d.height));
        if let Some((w, h)) = dims {
            let snapshot = self.snapshot();
            self.as_mut().rust_mut().selection = Some(Selection::all(w, h));
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
            self.changed();
        }
    }

    pub fn deselect(mut self: Pin<&mut Self>) {
        let snapshot = self.snapshot();
        self.as_mut().rust_mut().selection = None;
        if let Some(snapshot) = snapshot {
            let mut rust = self.as_mut().rust_mut();
            rust.history.capture(snapshot);
            rust.dirty = true;
        }
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
                let snapshot = self.snapshot();
                self.as_mut().rust_mut().selection = Some(selection);
                if let Some(snapshot) = snapshot {
                    let mut rust = self.as_mut().rust_mut();
                    rust.history.capture(snapshot);
                    rust.dirty = true;
                }
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
        let snapshot = self.snapshot();
        let pushed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                doc.layers.push(layer);
                true
            }
            None => false,
        };
        if pushed {
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
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
        let snapshot = self.snapshot();
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
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
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
        let snapshot = self.snapshot();
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_document(doc, width as u32, height as u32, resample).is_ok()
        };
        if resized {
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
            self.as_mut().rust_mut().selection = None;
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
        let snapshot = self.snapshot();
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_canvas_document(doc, width as u32, height as u32, anchor).is_ok()
        };
        if resized {
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
            self.as_mut().rust_mut().selection = None;
            self.recomposite();
        }
        resized
    }

    pub fn rotate_doc(mut self: Pin<&mut Self>, quarter_turns: i32) -> bool {
        if !(1..=3).contains(&quarter_turns) {
            return false;
        }
        let snapshot = self.snapshot();
        let rotated = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::rotate_document(doc, quarter_turns as u8).is_ok()
        };
        if rotated {
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
            self.as_mut().rust_mut().selection = None;
            self.recomposite();
        }
        rotated
    }

    pub fn flip_doc(mut self: Pin<&mut Self>, horizontal: bool) -> bool {
        let snapshot = self.snapshot();
        let mut rust = self.as_mut().rust_mut();
        let Some(doc) = rust.doc.as_mut() else {
            return false;
        };
        pictura_render::flip_document(doc, horizontal);
        rust.selection = None;
        rust.dirty = true;
        if let Some(snapshot) = snapshot {
            self.as_mut().rust_mut().history.capture(snapshot);
        }
        self.recomposite();
        true
    }

    pub fn undo(mut self: Pin<&mut Self>) -> bool {
        let current = {
            let rust = self.rust();
            let Some(doc) = rust.doc.clone() else {
                return false;
            };
            Snapshot {
                doc,
                selection: rust.selection.clone(),
            }
        };
        let restored = self.as_mut().rust_mut().history.undo(current);
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
        let current = {
            let rust = self.rust();
            let Some(doc) = rust.doc.clone() else {
                return false;
            };
            Snapshot {
                doc,
                selection: rust.selection.clone(),
            }
        };
        let restored = self.as_mut().rust_mut().history.redo(current);
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

    fn snapshot(&self) -> Option<Snapshot> {
        let rust = self.rust();
        let doc = rust.doc.clone()?;
        Some(Snapshot {
            doc,
            selection: rust.selection.clone(),
        })
    }

    pub fn remove_layer(mut self: Pin<&mut Self>, i: i32) {
        let snapshot = self.snapshot();
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
            if let Some(snapshot) = snapshot {
                let mut rust = self.as_mut().rust_mut();
                rust.history.capture(snapshot);
                rust.dirty = true;
            }
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
