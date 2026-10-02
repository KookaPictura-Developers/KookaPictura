//! Filter operations for Kooka Pictura.
//!
//! M6 scope: the destructive `Filter > Blur / Sharpen / Noise` math as pure
//! functions over a planar 8-bit [`PixelBuffer`] (channels 3 or 4; alpha is
//! never modified). Specs live in `docs/06-filters/`.
//!
//! The closed kernels are approximated where the specs say so; each
//! approximation is marked inline. Everything is deterministic except Add
//! Noise, which takes a seed. Bad input returns [`FilterError`] instead of
//! panicking.

use pictura_core::PixelBuffer;

pub mod artistic;
pub mod blur;
pub mod brush_strokes;
pub mod distort;
pub mod kernel;
pub mod luma;
pub mod noise;
pub mod oil_paint;
pub mod other;
pub mod pixelate;
pub mod render;
pub mod sharpen;
pub mod sketch;
pub mod stylize;
pub mod texture;

pub use render::LensType;

#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
    #[error("layer is pixel-locked")]
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadialMethod {
    Spin,
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Draft,
    Good,
    Best,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseDistribution {
    Uniform,
    Gaussian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MezzotintType {
    FineDots,
    MediumDots,
    GrainyDots,
    CoarseDots,
    ShortLines,
    MediumLines,
    LongLines,
    ShortStrokes,
    MediumStrokes,
    LongStrokes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpherizeMode {
    Normal,
    HorizontalOnly,
    VerticalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RippleSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveType {
    Sine,
    Triangle,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolarKind {
    RectangularToPolar,
    PolarToRectangular,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShearFill {
    WrapAround,
    RepeatEdgePixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZigZagStyle {
    AroundCenter,
    OutFromCenter,
    PondRipples,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushType {
    Simple,
    LightRough,
    DarkRough,
    WideSharp,
    WideBlurry,
    Sparkle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureSurface {
    Brick,
    Burlap,
    Canvas,
    Sandstone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureOptions {
    pub surface: TextureSurface,
    pub scaling: u8,
    pub relief: u8,
    pub light_direction: u8,
    pub invert: bool,
}

impl Default for TextureOptions {
    fn default() -> Self {
        Self {
            surface: TextureSurface::Canvas,
            scaling: 100,
            relief: 4,
            light_direction: 0,
            invert: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeDirection {
    RightDiagonal,
    Horizontal,
    LeftDiagonal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightDirection {
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalftoneType {
    Dot,
    Line,
    Circle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrainType {
    Regular,
    Soft,
    Sprinkles,
    Clumped,
    Contrasty,
    Enlarged,
    Stippled,
    Horizontal,
    Vertical,
    Speckle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtrudeType {
    Blocks,
    Pyramids,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileFill {
    BackgroundColor,
    ForegroundColor,
    InverseImage,
    UnalteredImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContourEdge {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindMethod {
    Wind,
    Blast,
    Stagger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharpenRemove {
    GaussianBlur,
    LensBlur,
    MotionBlur,
}

/// Smart Sharpen Shadow/Highlight tab controls: how strongly sharpening is
/// damped in dark or light tones, over what tonal `width`, using which blur
/// `radius`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TonalFade {
    pub amount: u8,
    pub width: u8,
    pub radius: u32,
}

impl Default for TonalFade {
    fn default() -> Self {
        Self {
            amount: 0,
            width: 50,
            radius: 1,
        }
    }
}

mod filter;
pub use filter::{apply, Filter};
pub(crate) fn validate(buf: &PixelBuffer) -> Result<usize, FilterError> {
    if buf.channels != 3 && buf.channels != 4 {
        return Err(FilterError::Unsupported(format!(
            "channel count {} is not supported (expected 3 or 4)",
            buf.channels
        )));
    }
    let n = buf.pixel_count();
    if n == 0 {
        return Err(FilterError::InvalidParams("empty buffer".into()));
    }
    if buf.data.len() != n * buf.channels as usize {
        return Err(FilterError::InvalidParams(
            "buffer length does not match width*height*channels".into(),
        ));
    }
    Ok(n)
}
