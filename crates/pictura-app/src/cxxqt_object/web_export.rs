//! File > Save for Web & Devices: the flattened sRGB image, palette reduction,
//! and the GIF and WBMP encoders (`pictura_codec::web`). Free functions (their
//! own bridge). JPEG, PNG-8, and PNG-24 are written by the dialog through Qt.

use super::helpers_composite::{buffer_to_rgba_bytes, current_buffer};
use super::qobject::PictureView;
use cxx_qt::CxxQtType;
use ffi::WebPalette;
use pictura_codec::{ColorReduction, Dither, Indexed, PaletteOptions};

#[cxx_qt::bridge]
pub mod ffi {
    /// Palette settings: `reduction` 0 Perceptual / 1 Selective / 2 Adaptive /
    /// 3 Restrictive (Web) / 4 Black and White / 5 Grayscale; `colors` 2–256;
    /// `dither` 0 None / 1 Diffusion / 2 Pattern / 3 Noise and its `amount`
    /// 0–100 %; `transparency`; the `matte` (`0xAARRGGBB`).
    #[namespace = "pictura"]
    struct WebPalette {
        reduction: i32,
        colors: i32,
        dither: i32,
        amount: i32,
        transparency: bool,
        matte: u32,
    }

    unsafe extern "C++" {
        include!("pictura_app/src/cxxqt_object.cxxqt.h");
        #[namespace = "pictura"]
        type PictureView = super::PictureView;
    }

    #[namespace = "pictura"]
    extern "Rust" {
        /// The flattened document as sRGB RGBA8888, `document_width` x `document_height`; empty without a document.
        fn web_flattened(view: &PictureView) -> Vec<u8>;

        /// Reduce RGBA8888 `width` x `height` to a palette: `[n_hi, n_lo]`, then `n` RGBA entries (alpha 0 marks the transparent entry), then one index per pixel.
        fn web_quantize(rgba: &[u8], width: i32, height: i32, palette: &WebPalette) -> Vec<u8>;

        /// Encode a `web_quantize` result as a GIF89a file, `interlaced` or not.
        fn web_encode_gif(indexed: &[u8], width: i32, height: i32, interlaced: bool) -> Vec<u8>;

        /// Encode RGBA8888 as a black-and-white WBMP with `dither` (as `WebPalette::dither`) at `amount` %.
        fn web_encode_wbmp(
            rgba: &[u8],
            width: i32,
            height: i32,
            dither: i32,
            amount: i32,
        ) -> Vec<u8>;
    }
}

fn dither_of(code: i32) -> Dither {
    match code {
        1 => Dither::Diffusion,
        2 => Dither::Pattern,
        3 => Dither::Noise,
        _ => Dither::None,
    }
}

fn size(rgba: &[u8], width: i32, height: i32) -> Option<(u32, u32)> {
    let (w, h) = (u32::try_from(width).ok()?, u32::try_from(height).ok()?);
    (w > 0 && h > 0 && rgba.len() >= (w as usize) * (h as usize) * 4).then_some((w, h))
}

fn web_flattened(view: &PictureView) -> Vec<u8> {
    let rust = view.rust();
    let Some(doc) = rust.doc.as_ref() else {
        return Vec::new();
    };
    let buffer = current_buffer(doc, rust.gpu_compute);
    buffer_to_rgba_bytes(&pictura_codec::buffer_to_srgb(doc, &buffer))
}

fn web_quantize(rgba: &[u8], width: i32, height: i32, palette: &WebPalette) -> Vec<u8> {
    let Some((w, h)) = size(rgba, width, height) else {
        return Vec::new();
    };
    let reduction = match palette.reduction {
        0 => ColorReduction::Perceptual,
        2 => ColorReduction::Adaptive,
        3 => ColorReduction::Restrictive,
        4 => ColorReduction::BlackAndWhite,
        5 => ColorReduction::Grayscale,
        _ => ColorReduction::Selective,
    };
    let options = PaletteOptions {
        reduction,
        colors: palette.colors.clamp(2, 256) as u16,
        dither: dither_of(palette.dither),
        amount: palette.amount.clamp(0, 100) as u8,
        transparency: palette.transparency,
        matte: [
            (palette.matte >> 16) as u8,
            (palette.matte >> 8) as u8,
            palette.matte as u8,
        ],
    };
    let indexed = pictura_codec::quantize(rgba, w, h, options);
    let n = indexed.palette.len() as u16;
    let mut out = n.to_be_bytes().to_vec();
    out.extend(indexed.palette.iter().flatten());
    out.extend_from_slice(&indexed.indices);
    out
}

fn web_encode_gif(indexed: &[u8], width: i32, height: i32, interlaced: bool) -> Vec<u8> {
    let (Ok(w), Ok(h)) = (u32::try_from(width), u32::try_from(height)) else {
        return Vec::new();
    };
    let Some(n) = indexed
        .get(..2)
        .map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])))
    else {
        return Vec::new();
    };
    let pixels = (w as usize) * (h as usize);
    if n == 0 || indexed.len() != 2 + n * 4 + pixels {
        return Vec::new();
    }
    let palette = indexed[2..2 + n * 4].as_chunks::<4>().0.to_vec();
    let image = Indexed {
        width: w,
        height: h,
        palette,
        indices: indexed[2 + n * 4..].to_vec(),
    };
    pictura_codec::encode_gif(&image, interlaced)
}

fn web_encode_wbmp(rgba: &[u8], width: i32, height: i32, dither: i32, amount: i32) -> Vec<u8> {
    let Some((w, h)) = size(rgba, width, height) else {
        return Vec::new();
    };
    pictura_codec::encode_wbmp(rgba, w, h, dither_of(dither), amount.clamp(0, 100) as u8)
}
