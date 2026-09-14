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
pub mod kernel;
pub mod luma;
pub mod noise;
pub mod other;
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
