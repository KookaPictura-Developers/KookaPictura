//! The raster-export bridge: flatten the document composite and write it through
//! Qt's image writers. Free functions over a [`PictureView`] (their own bridge,
//! so the `PictureView` declaration list does not grow).
//!
//! [`PictureView`]: super::qobject::PictureView

use super::helpers_composite::{buffer_to_rgba_bytes, current_buffer};
use super::qobject::PictureView;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("encode_image.h");
        /// Encode packed RGBA8888 to `path` with the named Qt image writer; false without writing on any failure.
        fn encode_image_rgba(
            rgba: &[u8],
            width: i32,
            height: i32,
            path: &str,
            format: &str,
            quality: i32,
            scale: i32,
        ) -> bool;
    }

    unsafe extern "C++" {
        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// The output format remembered from the import source (`"psd"` when native).
        fn output_format(view: &PictureView) -> QString;

        /// Flatten the document composite and write it to `path` through the Qt encode edge. Never changes the document, its path, dirty flag, or history; false without writing on refusal.
        fn export_image(
            view: &PictureView,
            path: &QString,
            format: &QString,
            quality: i32,
            scale: i32,
        ) -> bool;
    }
}

pub fn output_format(view: &PictureView) -> QString {
    QString::from(view.rust().source_format.as_str())
}

pub fn export_image(
    view: &PictureView,
    path: &QString,
    format: &QString,
    quality: i32,
    scale: i32,
) -> bool {
    let path = path.to_string();
    let format = format.to_string();
    let rust = view.rust();
    let Some(doc) = rust.doc.as_ref() else {
        return false;
    };
    let buffer = current_buffer(doc, rust.gpu_compute);
    let srgb = pictura_codec::buffer_to_srgb(doc, &buffer);
    let rgba = buffer_to_rgba_bytes(&srgb);
    ffi::encode_image_rgba(
        &rgba,
        srgb.width as i32,
        srgb.height as i32,
        &path,
        &format,
        quality,
        scale,
    )
}
