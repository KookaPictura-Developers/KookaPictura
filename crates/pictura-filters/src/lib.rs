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

pub mod artistic;
pub mod blur;
pub mod distort;
pub mod kernel;
pub mod luma;
pub mod noise;
pub mod other;
pub mod pixelate;
pub mod render;
pub mod sharpen;
pub mod stylize;

pub use render::LensType;

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
    Clouds {
        color_a: [u8; 3],
        color_b: [u8; 3],
        starker: bool,
        seed: u64,
    },
    DifferenceClouds {
        color_a: [u8; 3],
        color_b: [u8; 3],
        starker: bool,
        seed: u64,
    },
    Fibers {
        variance: f64,
        strength: f64,
        color_a: [u8; 3],
        color_b: [u8; 3],
        seed: u64,
    },
    LensFlare {
        brightness: f64,
        center: (f64, f64),
        lens: LensType,
    },
    Cutout {
        levels: u8,
        edge_simplicity: u8,
        edge_fidelity: u8,
    },
    FilmGrain {
        grain: u8,
        highlight_area: u8,
        intensity: u8,
        seed: u64,
    },
    NeonGlow {
        glow_size: i32,
        glow_brightness: u8,
        glow_color: [u8; 3],
    },
    PosterEdges {
        edge_thickness: u8,
        edge_intensity: u8,
        posterization: u8,
    },
    PaintDaubs {
        brush_size: u8,
        sharpness: u8,
        brush_type: BrushType,
        seed: u64,
    },
    PaletteKnife {
        stroke_size: u8,
        stroke_detail: u8,
        softness: u8,
        seed: u64,
    },
    PlasticWrap {
        highlight_strength: u8,
        detail: u8,
        smoothness: u8,
    },
    Sponge {
        brush_size: u8,
        definition: u8,
        smoothness: u8,
        seed: u64,
    },
    ColoredPencil {
        pencil_width: u8,
        stroke_pressure: u8,
        paper_brightness: u8,
        foreground: [u8; 3],
        background: [u8; 3],
        seed: u64,
    },
    DryBrush {
        brush_size: u8,
        brush_detail: u8,
        texture: u8,
        seed: u64,
    },
    Fresco {
        brush_size: u8,
        brush_detail: u8,
        texture: u8,
        seed: u64,
    },
    RoughPastels {
        stroke_length: u8,
        stroke_detail: u8,
        texture: TextureOptions,
        foreground: [u8; 3],
        background: [u8; 3],
        seed: u64,
    },
    SmudgeStick {
        stroke_length: u8,
        highlight_area: u8,
        intensity: u8,
        seed: u64,
    },
    Underpainting {
        brush_size: u8,
        texture_coverage: u8,
        texture: TextureOptions,
        seed: u64,
    },
    Watercolor {
        brush_detail: u8,
        shadow_intensity: u8,
        texture: u8,
        foreground: [u8; 3],
        background: [u8; 3],
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
        Filter::Clouds {
            color_a,
            color_b,
            starker,
            seed,
        } => render::clouds(buf, *color_a, *color_b, *starker, *seed),
        Filter::DifferenceClouds {
            color_a,
            color_b,
            starker,
            seed,
        } => render::difference_clouds(buf, *color_a, *color_b, *starker, *seed),
        Filter::Fibers {
            variance,
            strength,
            color_a,
            color_b,
            seed,
        } => render::fibers(buf, *variance, *strength, *color_a, *color_b, *seed),
        Filter::LensFlare {
            brightness,
            center,
            lens,
        } => render::lens_flare(buf, *brightness, *center, *lens),
        Filter::Cutout {
            levels,
            edge_simplicity,
            edge_fidelity,
        } => artistic::cutout(buf, *levels, *edge_simplicity, *edge_fidelity),
        Filter::FilmGrain {
            grain,
            highlight_area,
            intensity,
            seed,
        } => artistic::film_grain(buf, *grain, *highlight_area, *intensity, *seed),
        Filter::NeonGlow {
            glow_size,
            glow_brightness,
            glow_color,
        } => artistic::neon_glow(buf, *glow_size, *glow_brightness, *glow_color),
        Filter::PosterEdges {
            edge_thickness,
            edge_intensity,
            posterization,
        } => artistic::poster_edges(buf, *edge_thickness, *edge_intensity, *posterization),
        Filter::PaintDaubs {
            brush_size,
            sharpness,
            brush_type,
            seed,
        } => artistic::paint_daubs(buf, *brush_size, *sharpness, *brush_type, *seed),
        Filter::PaletteKnife {
            stroke_size,
            stroke_detail,
            softness,
            seed,
        } => artistic::palette_knife(buf, *stroke_size, *stroke_detail, *softness, *seed),
        Filter::PlasticWrap {
            highlight_strength,
            detail,
            smoothness,
        } => artistic::plastic_wrap(buf, *highlight_strength, *detail, *smoothness),
        Filter::Sponge {
            brush_size,
            definition,
            smoothness,
            seed,
        } => artistic::sponge(buf, *brush_size, *definition, *smoothness, *seed),
        Filter::ColoredPencil {
            pencil_width,
            stroke_pressure,
            paper_brightness,
            foreground,
            background,
            seed,
        } => artistic::colored_pencil(
            buf,
            *pencil_width,
            *stroke_pressure,
            *paper_brightness,
            *foreground,
            *background,
            *seed,
        ),
        Filter::DryBrush {
            brush_size,
            brush_detail,
            texture,
            seed,
        } => artistic::dry_brush(buf, *brush_size, *brush_detail, *texture, *seed),
        Filter::Fresco {
            brush_size,
            brush_detail,
            texture,
            seed,
        } => artistic::fresco(buf, *brush_size, *brush_detail, *texture, *seed),
        Filter::RoughPastels {
            stroke_length,
            stroke_detail,
            texture,
            foreground,
            background,
            seed,
        } => artistic::rough_pastels(
            buf,
            *stroke_length,
            *stroke_detail,
            *texture,
            *foreground,
            *background,
            *seed,
        ),
        Filter::SmudgeStick {
            stroke_length,
            highlight_area,
            intensity,
            seed,
        } => artistic::smudge_stick(buf, *stroke_length, *highlight_area, *intensity, *seed),
        Filter::Underpainting {
            brush_size,
            texture_coverage,
            texture,
            seed,
        } => artistic::underpainting(buf, *brush_size, *texture_coverage, *texture, *seed),
        Filter::Watercolor {
            brush_detail,
            shadow_intensity,
            texture,
            foreground,
            background,
            seed,
        } => artistic::watercolor(
            buf,
            *brush_detail,
            *shadow_intensity,
            *texture,
            *foreground,
            *background,
            *seed,
        ),
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
