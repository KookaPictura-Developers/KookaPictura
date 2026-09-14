//! Filter operations for Kooka Pictura.
//!
//! M6 scope: the destructive `Filter > Blur / Sharpen / Noise` math as pure
//! functions over a planar 8-bit [`PixelBuffer`] (channels 3 or 4; alpha is
//! never modified). Specs live in `docs/06-filters/`.
//!
//! Adobe's closed kernels are approximated where the specs say so; each
//! approximation is marked inline. Everything is deterministic except Add
//! Noise, which takes a seed. Bad input returns [`FilterError`] instead of
//! panicking.

use pictura_core::PixelBuffer;

pub mod blur;
pub mod distort;
pub mod kernel;
pub mod luma;
pub mod noise;
pub mod other;
pub mod pixelate;
pub mod sharpen;
pub mod stylize;

#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    GaussianBlur {
        radius: f64,
    },
    BoxBlur {
        radius: u32,
    },
    MotionBlur {
        angle: f64,
        distance: u32,
    },
    RadialBlur {
        method: RadialMethod,
        amount: f64,
        quality: Quality,
    },
    Average,
    Blur,
    BlurMore,
    SurfaceBlur {
        radius: u32,
        threshold: u8,
    },
    Sharpen,
    SharpenMore,
    SharpenEdges,
    UnsharpMask {
        amount: f64,
        radius: f64,
        threshold: u8,
    },
    AddNoise {
        amount: f64,
        distribution: NoiseDistribution,
        monochromatic: bool,
        seed: u64,
    },
    Median {
        radius: u32,
    },
    Despeckle,
    Maximum {
        radius: u32,
    },
    Minimum {
        radius: u32,
    },
    Offset {
        horizontal: i32,
        vertical: i32,
        wrap: bool,
        background: [u8; 3],
    },
    HighPass {
        radius: f64,
    },
    Custom {
        kernel: [[f64; 5]; 5],
        scale: f64,
        offset: f64,
    },
    Emboss {
        angle: f64,
        height: f64,
        amount: f64,
    },
    FindEdges,
    Solarize,
    Mosaic {
        cell_size: u32,
    },
    Crystallize {
        cell_size: u32,
        seed: u64,
    },
    Facet,
    Fragment,
    Mezzotint {
        kind: MezzotintType,
        seed: u64,
    },
    Pointillize {
        cell_size: u32,
        background: [u8; 3],
        seed: u64,
    },
    ColorHalftone {
        max_radius: u32,
        angles: [f64; 4],
    },
    Twirl {
        angle: f64,
    },
    Pinch {
        amount: f64,
    },
    Spherize {
        amount: f64,
        mode: SpherizeMode,
    },
    Ripple {
        amount: f64,
        size: RippleSize,
    },
    Wave {
        generators: u32,
        wavelength: (f64, f64),
        amplitude: (f64, f64),
        kind: WaveType,
        scale: (f64, f64),
        seed: u64,
        repeat_edge: bool,
    },
    PolarCoordinates {
        kind: PolarKind,
    },
    Shear {
        curve: Vec<(f64, f64)>,
        fill: ShearFill,
    },
    ZigZag {
        amount: f64,
        ridges: u32,
        style: ZigZagStyle,
    },
    OceanRipple {
        size: u32,
        magnitude: u32,
        seed: u64,
    },
}

/// Apply `filter` in place (planar 8-bit; channels 3 or 4; alpha untouched).
pub fn apply(filter: &Filter, buf: &mut PixelBuffer) -> Result<(), FilterError> {
    validate(buf)?;
    match filter {
        Filter::GaussianBlur { radius } => blur::gaussian(buf, *radius),
        Filter::BoxBlur { radius } => blur::r#box(buf, *radius),
        Filter::MotionBlur { angle, distance } => blur::motion(buf, *angle, *distance),
        Filter::RadialBlur {
            method,
            amount,
            quality,
        } => blur::radial(buf, *method, *amount, *quality),
        Filter::Average => blur::average(buf),
        Filter::Blur => blur::simple(buf, false),
        Filter::BlurMore => blur::simple(buf, true),
        Filter::SurfaceBlur { radius, threshold } => blur::surface(buf, *radius, *threshold),
        Filter::Sharpen => sharpen::sharpen(buf),
        Filter::SharpenMore => sharpen::sharpen_more(buf),
        Filter::SharpenEdges => sharpen::edges(buf),
        Filter::UnsharpMask {
            amount,
            radius,
            threshold,
        } => sharpen::unsharp_mask(buf, *amount, *radius, *threshold),
        Filter::AddNoise {
            amount,
            distribution,
            monochromatic,
            seed,
        } => noise::add(buf, *amount, *distribution, *monochromatic, *seed),
        Filter::Median { radius } => noise::median(buf, *radius),
        Filter::Despeckle => noise::despeckle(buf),
        Filter::Maximum { radius } => other::maximum(buf, *radius),
        Filter::Minimum { radius } => other::minimum(buf, *radius),
        Filter::Offset {
            horizontal,
            vertical,
            wrap,
            background,
        } => other::offset(buf, *horizontal, *vertical, *wrap, *background),
        Filter::HighPass { radius } => other::high_pass(buf, *radius),
        Filter::Custom {
            kernel,
            scale,
            offset,
        } => other::custom(buf, kernel, *scale, *offset),
        Filter::Emboss {
            angle,
            height,
            amount,
        } => stylize::emboss(buf, *angle, *height, *amount),
        Filter::FindEdges => stylize::find_edges(buf),
        Filter::Solarize => stylize::solarize(buf),
        Filter::Mosaic { cell_size } => pixelate::mosaic(buf, *cell_size),
        Filter::Crystallize { cell_size, seed } => pixelate::crystallize(buf, *cell_size, *seed),
        Filter::Facet => pixelate::facet(buf),
        Filter::Fragment => pixelate::fragment(buf),
        Filter::Mezzotint { kind, seed } => pixelate::mezzotint(buf, *kind, *seed),
        Filter::Pointillize {
            cell_size,
            background,
            seed,
        } => pixelate::pointillize(buf, *cell_size, *background, *seed),
        Filter::ColorHalftone { max_radius, angles } => {
            pixelate::color_halftone(buf, *max_radius, *angles)
        }
        Filter::Twirl { angle } => distort::twirl(buf, *angle),
        Filter::Pinch { amount } => distort::pinch(buf, *amount),
        Filter::Spherize { amount, mode } => distort::spherize(buf, *amount, *mode),
        Filter::Ripple { amount, size } => distort::ripple(buf, *amount, *size),
        Filter::Wave {
            generators,
            wavelength,
            amplitude,
            kind,
            scale,
            seed,
            repeat_edge,
        } => distort::wave(
            buf,
            *generators,
            *wavelength,
            *amplitude,
            *kind,
            *scale,
            *seed,
            *repeat_edge,
        ),
        Filter::PolarCoordinates { kind } => distort::polar_coordinates(buf, *kind),
        Filter::Shear { curve, fill } => distort::shear(buf, curve, *fill),
        Filter::ZigZag {
            amount,
            ridges,
            style,
        } => distort::zigzag(buf, *amount, *ridges, *style),
        Filter::OceanRipple {
            size,
            magnitude,
            seed,
        } => distort::ocean_ripple(buf, *size, *magnitude, *seed),
    }
}

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
