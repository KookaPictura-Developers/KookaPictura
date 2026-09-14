//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QImageFormat, QString};
use pictura_core::{Document, PixelBuffer};

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

        /// Try to load a PSD through `pictura-codec`. Returns `false` and falls
        /// back to a generated test image when the file is missing or unsupported.
        #[qinvokable]
        fn open(self: Pin<&mut Self>, path: &QString) -> bool;

        /// The image to display. Never null.
        #[qinvokable]
        fn image(&self) -> QImage;

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
    interop: Option<crate::gpu::InteropState>,
}

impl qobject::PictureView {
    pub fn open(self: Pin<&mut Self>, path: &QString) -> bool {
        let loaded = std::fs::read(path.to_string())
            .ok()
            .and_then(|bytes| pictura_codec::read_psd(&bytes).ok())
            .map(|doc| document_to_image(&doc));

        let ok = loaded.is_some();
        self.rust_mut().image = loaded.unwrap_or_else(test_image);
        ok
    }

    pub fn image(&self) -> QImage {
        self.rust().image.clone()
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
}

/// Convert the document to a packed RGBA `QImage`: the composited layer stack
/// when it has layers, otherwise the embedded PSD composite.
fn document_to_image(doc: &Document) -> QImage {
    if doc.layers.is_empty() {
        buffer_to_image(&doc.composite)
    } else {
        buffer_to_image(&pictura_render::composite_rgba(doc))
    }
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
    fn test_image_is_not_null() {
        assert!(!test_image().is_null());
    }
}
