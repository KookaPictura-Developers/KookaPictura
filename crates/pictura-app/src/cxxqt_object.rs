//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QImageFormat, QString};
use pictura_core::{AdjustmentData, BlendMode, Document, Layer, LayerMask, PixelBuffer, PsdRect};
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

        /// The image to display. Never null.
        #[qinvokable]
        fn image(&self) -> QImage;

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
    interop: Option<crate::gpu::InteropState>,
}

impl qobject::PictureView {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        let loaded = std::fs::read(path.to_string())
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
        ok
    }

    pub fn image(&self) -> QImage {
        self.rust().image.clone()
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
            self.recomposite();
        }
    }

    pub fn select_all(mut self: Pin<&mut Self>) {
        let dims = self.rust().doc.as_ref().map(|d| (d.width, d.height));
        if let Some((w, h)) = dims {
            self.as_mut().rust_mut().selection = Some(Selection::all(w, h));
            self.changed();
        }
    }

    pub fn deselect(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().selection = None;
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
        let pushed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                doc.layers.push(layer);
                true
            }
            None => false,
        };
        if pushed {
            self.recomposite();
        }
        pushed
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
}
