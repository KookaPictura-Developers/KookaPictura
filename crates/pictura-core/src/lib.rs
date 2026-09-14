//! Core document types for Kooka Pictura.
//!
//! M0 scope is deliberately tiny: an 8-bit RGB/gray composite image. The full
//! layer/channel/mask model is specified in `docs/01-architecture/document-model.md`
//! and is **not** implemented here yet.

/// PSD color modes (`header.color_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Bitmap,
    Grayscale,
    Indexed,
    Rgb,
    Cmyk,
    Multichannel,
    Duotone,
    Lab,
}

/// Bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    One,
    Eight,
    Sixteen,
    ThirtyTwo,
}

/// A planar, row-major, 8-bit-per-channel pixel buffer.
///
/// `data.len() == width * height * channels`. Planar means channel `c` for the
/// whole image comes first, matching the PSD image-data layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<u8>,
}

impl PixelBuffer {
    pub fn new(width: u32, height: u32, channels: u8) -> Self {
        Self {
            width,
            height,
            channels,
            data: vec![0; width as usize * height as usize * channels as usize],
        }
    }

    /// Number of pixels (not samples).
    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }
}

/// A minimal document: dimensions, mode, depth, and one composite image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub mode: ColorMode,
    pub depth: BitDepth,
    pub composite: PixelBuffer,
}

impl Document {
    pub fn new(width: u32, height: u32, mode: ColorMode, depth: BitDepth) -> Self {
        let channels = match mode {
            ColorMode::Grayscale | ColorMode::Bitmap | ColorMode::Duotone => 1,
            _ => 3,
        };
        Self {
            width,
            height,
            mode,
            depth,
            composite: PixelBuffer::new(width, height, channels),
        }
    }
}
