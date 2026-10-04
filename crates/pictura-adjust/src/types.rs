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
    /// Monotone composite control points in `(input, output)` order, inclusive
    /// of endpoints.
    pub points: Vec<(u8, u8)>,
    /// Optional per-channel curves, each in the same `(input, output)` order as
    /// `points`. Applied to their own plane before the composite `points` curve.
    pub red: Option<Vec<(u8, u8)>>,
    pub green: Option<Vec<(u8, u8)>>,
    pub blue: Option<Vec<(u8, u8)>>,
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

/// Shadows/Highlights (`shdH`): the two always-visible amount sliders in
/// `0..=100`. A pointwise approximation of CS6's local operator (see
/// `docs/04-image-ops/adjustments/shadow-highlight.md`); the surround blur and
/// advanced options are not modelled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowsHighlightsParams {
    pub shadows_amount: f64,
    pub highlights_amount: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorBalanceParams {
    /// Per-band `[cyan-red, magenta-green, yellow-blue]` shifts, -100…+100.
    pub shadows: [f64; 3],
    pub midtones: [f64; 3],
    pub highlights: [f64; 3],
    pub preserve_luminosity: bool,
}

/// How a Selective Color correction scales: relative scales by the source ink,
/// absolute by full scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectiveColorMethod {
    Relative,
    Absolute,
}

/// One CMYK correction pair of a Selective Color family, each `-100..=100`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectiveRange {
    pub c: i16,
    pub m: i16,
    pub y: i16,
    pub k: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectiveColorParams {
    pub method: SelectiveColorMethod,
    /// Nine ranges in the reference's order: reds, yellows, greens, cyans, blues,
    /// magentas, whites, neutrals, blacks.
    pub ranges: [SelectiveRange; 9],
}

impl Default for SelectiveColorParams {
    fn default() -> Self {
        Self {
            method: SelectiveColorMethod::Relative,
            ranges: [SelectiveRange::default(); 9],
        }
    }
}

/// One colour stop of a [`GradientMapParams`] gradient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GradientStop {
    /// PSD gradient position, `0..=4096`.
    pub location: u16,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct GradientMapParams {
    pub stops: Vec<GradientStop>,
    pub reverse: bool,
}

/// The geometry of a gradient fill (`GdFl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientKind {
    Linear,
    Radial,
    Angle,
    Reflected,
    Diamond,
}

/// A gradient fill (`GdFl`): the stops plus the geometry parameters psd-tools
/// reads from the descriptor. Like [`GradientMapParams`] it reuses
/// [`GradientStop`]; unlike a gradient map it is composited over the layer rect
/// rather than applied to the backdrop.
#[derive(Debug, Clone, PartialEq)]
pub struct GradientFillParams {
    pub stops: Vec<GradientStop>,
    pub reverse: bool,
    pub kind: GradientKind,
    pub angle_deg: f32,
    pub scale: f32,
}

/// A pattern fill (`PtFl`): the id of a pattern in the document's pattern
/// library plus the tiling controls. Like [`GradientFillParams`] it is
/// composited over the layer rect rather than applied to the backdrop.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternFillParams {
    pub pattern_id: String,
    /// Tile scale in percent (100 = one pattern pixel per layer pixel).
    pub scale: f32,
    /// Anchor the tile at the layer's top-left when set, else at the document
    /// origin.
    pub link_with_layer: bool,
    /// Pixel offset added to the tiled sample.
    pub origin: (i32, i32),
}

/// The `lookupType` of a Color Lookup (`clrL`) adjustment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLookupKind {
    /// A 3-D LUT (the only kind this engine renders).
    ThreeDLut,
    AbstractProfile,
    DeviceLinkProfile,
}

/// A parsed 3-D lookup table: `size` grid steps per axis and `size³` RGB points
/// in `.CUBE` order (red index varies fastest, then green, then blue), each
/// component in `0.0..=1.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Lut3d {
    pub size: usize,
    pub points: Vec<[f32; 3]>,
}

/// A Color Lookup (`clrL`) adjustment. `lookup` is the parsed embedded `.CUBE`
/// for a 3-D LUT; it is `None` for an abstract-profile or device-link kind, a
/// non-`.CUBE` embedded format, or malformed data, and such a layer is a no-op.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorLookupParams {
    pub kind: ColorLookupKind,
    pub lookup: Option<Lut3d>,
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
    /// Image > Adjustments > Shadows/Highlights: lifts shadows and pulls
    /// highlights by amount, without a local surround.
    ShadowsHighlights(ShadowsHighlightsParams),
    ColorBalance(ColorBalanceParams),
    SelectiveColor(SelectiveColorParams),
    Auto(AutoKind),
    Invert,
    Posterize(u8),
    Threshold(u8),
    Desaturate,
    /// Image > Adjustments > Equalize: spread the combined RGB histogram
    /// evenly over 0..=255 (behavioural approximation; CS6's is unpublished).
    Equalize,
    GradientMap(GradientMapParams),
    /// Gradient fill content (`GdFl`): a generative fill composited over the
    /// layer rect (see `pictura-render`'s `composite_gradient_fill`), refused
    /// by [`apply`] like [`Adjustment::SolidFill`].
    GradientFill(GradientFillParams),
    /// Solid-color fill content (`SoCo`): straight-alpha RGBA. This is not a
    /// destructive adjustment; the renderer composites it generatively (see
    /// `pictura-render`'s `composite_adjustment`), and [`apply`] refuses it.
    SolidFill([u8; 4]),
    /// Pattern fill content (`PtFl`): a generative fill composited over the
    /// layer rect (see `pictura-render`'s `composite_pattern_fill`), refused by
    /// [`apply`] like [`Adjustment::SolidFill`].
    PatternFill(PatternFillParams),
    /// Color Lookup (`clrL`): samples a parsed 3-D lookup, or is a no-op when
    /// [`ColorLookupParams::lookup`] is `None`.
    ColorLookup(ColorLookupParams),
}
