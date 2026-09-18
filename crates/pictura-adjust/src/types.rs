#[derive(Debug, thiserror::Error)]
pub enum AdjustError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelsParams {
    pub input_black: u8,
    pub input_white: u8,
    pub gamma: f64,
    pub output_black: u8,
    pub output_white: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurvesParams {
    /// Monotone control points in `(input, output)` order, inclusive of endpoints.
    pub points: Vec<(u8, u8)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrightnessContrastParams {
    pub brightness: i16,
    pub contrast: i16,
    pub use_legacy: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExposureParams {
    /// Exposure in stops (EV).
    pub exposure: f64,
    pub offset: f64,
    pub gamma: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HueSaturationParams {
    pub hue: i16,
    pub saturation: i16,
    pub lightness: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlackWhiteParams {
    /// Contributions in percent, matching the CS6 -200…+300 sliders.
    pub red: f64,
    pub yellow: f64,
    pub green: f64,
    pub cyan: f64,
    pub blue: f64,
    pub magenta: f64,
    pub tint: bool,
    /// Tint tone when `tint` is set (spec's Tint colour swatch).
    pub tint_color: [u8; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhotoFilterParams {
    pub color: [u8; 3],
    pub density: f64,
    pub preserve_luminosity: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMixerParams {
    pub monochrome: bool,
    /// Output-channel mixes in percent, as `[red, green, blue]` sources.
    pub red: [f64; 3],
    pub green: [f64; 3],
    pub blue: [f64; 3],
    /// Per-output-channel constant in percent.
    pub constant: [f64; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct VibranceParams {
    pub vibrance: i16,
    pub saturation: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorBalanceParams {
    /// Per-band `[cyan-red, magenta-green, yellow-blue]` shifts, -100…+100.
    pub shadows: [f64; 3],
    pub midtones: [f64; 3],
    pub highlights: [f64; 3],
    pub preserve_luminosity: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoKind {
    Tone,
    Contrast,
    Color,
}

/// A single destructive adjustment.
#[derive(Debug, Clone, PartialEq)]
pub enum Adjustment {
    Levels(LevelsParams),
    Curves(CurvesParams),
    BrightnessContrast(BrightnessContrastParams),
    Exposure(ExposureParams),
    HueSaturation(HueSaturationParams),
    BlackWhite(BlackWhiteParams),
    PhotoFilter(PhotoFilterParams),
    ChannelMixer(ChannelMixerParams),
    Vibrance(VibranceParams),
    ColorBalance(ColorBalanceParams),
    Auto(AutoKind),
    Invert,
    Posterize(u8),
    Threshold(u8),
    Desaturate,
}
