//! The cxx-qt bridge: a Rust `QObject` that owns the image shown by the shell.

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QImageFormat, QString};
use pictura_core::Document;

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
    }
}

/// Backing Rust state for [`qobject::PictureView`].
#[derive(Default)]
pub struct PictureViewRust {
    image: QImage,
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
}

/// Convert the planar composite in a [`Document`] to a packed RGBA `QImage`.
fn document_to_image(doc: &Document) -> QImage {
    let width = doc.width as i32;
    let height = doc.height as i32;
    let plane = doc.width as usize * doc.height as usize;
    let channels = doc.composite.channels as usize;

    let mut rgba = vec![0u8; plane * 4];
    for i in 0..plane {
        let (r, g, b) = if channels <= 1 {
            let v = doc.composite.data[i];
            (v, v, v)
        } else {
            (
                doc.composite.data[i],
                doc.composite.data[plane + i],
                doc.composite.data[2 * plane + i],
            )
        };
        let o = i * 4;
        rgba[o] = r;
        rgba[o + 1] = g;
        rgba[o + 2] = b;
        rgba[o + 3] = 255;
    }

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

    // SAFETY: `rgba` is exactly width*height RGBA8888 bytes.
    unsafe { QImage::from_raw_bytes(rgba, width, height, QImageFormat::Format_RGBA8888) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, ColorMode};

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
    }

    #[test]
    fn test_image_is_not_null() {
        assert!(!test_image().is_null());
    }
}
