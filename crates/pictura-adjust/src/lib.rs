//! Adjustment operations for Kooka Pictura.
//!
//! M4 scope: the destructive `Image > Adjustments` math as pure functions over a
//! planar 8-bit [`PixelBuffer`] (channels 3 or 4; alpha is never modified).
//! Specs live in `docs/04-image-ops/adjustments/`. Adjustment *layers* (model,
//! PSD serialization, compositor integration) are a later task.
//!
//! Adobe's closed kernels are approximated where the specs say so; each
//! approximation is marked inline. Everything is deterministic and returns
//! [`AdjustError`] instead of panicking on bad input.

use pictura_core::PixelBuffer;

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

const COLOR_CHANNELS: usize = 3;
/// Rec.601 luma weights; the specs leave the exact set undocumented.
const LUMA: [f64; 3] = [0.299, 0.587, 0.114];

/// Apply `adjustment` in place to a planar 8-bit buffer (alpha untouched).
pub fn apply(adjustment: &Adjustment, buf: &mut PixelBuffer) -> Result<(), AdjustError> {
    let n = validate(buf)?;
    match adjustment {
        Adjustment::Levels(p) => levels(p, buf, n),
        Adjustment::Curves(p) => curves(p, buf, n),
        Adjustment::BrightnessContrast(p) => brightness_contrast(p, buf, n),
        Adjustment::Exposure(p) => exposure(p, buf, n),
        Adjustment::HueSaturation(p) => hue_saturation(p, buf, n),
        Adjustment::BlackWhite(p) => black_white(p, buf, n),
        Adjustment::PhotoFilter(p) => photo_filter(p, buf, n),
        Adjustment::ChannelMixer(p) => channel_mixer(p, buf, n),
        Adjustment::Vibrance(p) => vibrance(p, buf, n),
        Adjustment::ColorBalance(p) => color_balance(p, buf, n),
        Adjustment::Auto(kind) => auto(*kind, buf, n),
        Adjustment::Invert => {
            map_float(buf, n, |v| 255 - v);
            Ok(())
        }
        Adjustment::Posterize(levels) => posterize(*levels, buf, n),
        Adjustment::Threshold(level) => threshold(*level, buf, n),
        Adjustment::Desaturate => {
            desaturate(buf, n);
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn validate(buf: &PixelBuffer) -> Result<usize, AdjustError> {
    if buf.channels != 3 && buf.channels != 4 {
        return Err(AdjustError::Unsupported(format!(
            "channel count {} is not supported (expected 3 or 4)",
            buf.channels
        )));
    }
    let n = buf.pixel_count();
    if n == 0 {
        return Err(AdjustError::InvalidParams("empty buffer".into()));
    }
    if buf.data.len() != n * buf.channels as usize {
        return Err(AdjustError::InvalidParams(
            "buffer length does not match width*height*channels".into(),
        ));
    }
    Ok(n)
}

/// Split the planar color channels of a validated buffer. The returned slices
/// are `R`, `G`, `B`; alpha (when present) is left untouched.
fn planes_mut(buf: &mut PixelBuffer, n: usize) -> (&mut [u8], &mut [u8], &mut [u8]) {
    let (r, rest) = buf.data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _a) = rest.split_at_mut(n);
    (r, g, b)
}

fn map_lut(buf: &mut PixelBuffer, n: usize, lut: &[u8; 256]) {
    for c in 0..COLOR_CHANNELS {
        let s = c * n;
        for v in &mut buf.data[s..s + n] {
            *v = lut[*v as usize];
        }
    }
}

fn map_float(buf: &mut PixelBuffer, n: usize, f: impl Fn(u8) -> u8) {
    for c in 0..COLOR_CHANNELS {
        let s = c * n;
        for v in &mut buf.data[s..s + n] {
            *v = f(*v);
        }
    }
}

fn luma(r: f64, g: f64, b: f64) -> f64 {
    LUMA[0] * r + LUMA[1] * g + LUMA[2] * b
}

fn rgb_to_hsl(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-12 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } * 60.0;
    (h.rem_euclid(360.0), s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    if s <= 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    (
        hue2rgb(p, q, hk + 1.0 / 3.0),
        hue2rgb(p, q, hk),
        hue2rgb(p, q, hk - 1.0 / 3.0),
    )
}

fn hue2rgb(p: f64, q: f64, t: f64) -> f64 {
    let t = t.rem_euclid(1.0);
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 0.5 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Monotone cubic Hermite tangents (Fritsch–Carlson limiter).
fn monotone_tangents(xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len();
    let mut d = vec![0.0; n - 1];
    for (i, di) in d.iter_mut().enumerate() {
        *di = (ys[i + 1] - ys[i]) / (xs[i + 1] - xs[i]);
    }
    let mut m = vec![0.0; n];
    if let Some(first) = m.first_mut() {
        *first = d[0];
    }
    if let Some(last) = m.last_mut() {
        *last = d[n - 2];
    }
    for i in 1..n - 1 {
        m[i] = (d[i - 1] + d[i]) / 2.0;
    }
    for i in 0..n - 1 {
        if d[i] == 0.0 {
            m[i] = 0.0;
            m[i + 1] = 0.0;
        } else {
            let a = m[i] / d[i];
            let b = m[i + 1] / d[i];
            let s = a * a + b * b;
            if s > 9.0 {
                let tau = 3.0 / s.sqrt();
                m[i] = tau * a * d[i];
                m[i + 1] = tau * b * d[i];
            }
        }
    }
    m
}

fn hermite_eval(xs: &[f64], ys: &[f64], ms: &[f64], x: f64) -> f64 {
    let n = xs.len();
    if x <= xs[0] {
        return ys[0];
    }
    if x >= xs[n - 1] {
        return ys[n - 1];
    }
    let mut i = 0;
    while x > xs[i + 1] {
        i += 1;
    }
    let h = xs[i + 1] - xs[i];
    let t = (x - xs[i]) / h;
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * ys[i]
        + (t3 - 2.0 * t2 + t) * h * ms[i]
        + (-2.0 * t3 + 3.0 * t2) * ys[i + 1]
        + (t3 - t2) * h * ms[i + 1]
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

// ---------------------------------------------------------------------------
// Tonal adjustments
// ---------------------------------------------------------------------------

fn levels(p: &LevelsParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if !p.gamma.is_finite() || p.gamma <= 0.0 {
        return Err(AdjustError::InvalidParams(
            "levels gamma must be > 0".into(),
        ));
    }
    if p.input_black >= p.input_white {
        return Err(AdjustError::InvalidParams(
            "levels input black must be below input white".into(),
        ));
    }
    let ib = p.input_black as f64;
    let iw = p.input_white as f64;
    let ob = p.output_black as f64;
    let ow = p.output_white as f64;
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        let t = ((i as f64 - ib) / (iw - ib)).clamp(0.0, 1.0);
        let t = t.powf(1.0 / p.gamma);
        *slot = (ob + t * (ow - ob)).round().clamp(0.0, 255.0) as u8;
    }
    map_lut(buf, n, &lut);
    Ok(())
}

fn curves(p: &CurvesParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if p.points.len() < 2 {
        return Err(AdjustError::InvalidParams(
            "curves need at least 2 points".into(),
        ));
    }
    if p.points.len() > 14 {
        return Err(AdjustError::InvalidParams(
            "curves support at most 14 points".into(),
        ));
    }
    let xs: Vec<f64> = p.points.iter().map(|q| q.0 as f64).collect();
    let ys: Vec<f64> = p.points.iter().map(|q| q.1 as f64).collect();
    if xs.windows(2).any(|w| w[0] >= w[1]) {
        return Err(AdjustError::InvalidParams(
            "curve inputs must be strictly increasing".into(),
        ));
    }
    let ms = monotone_tangents(&xs, &ys);
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        *slot = hermite_eval(&xs, &ys, &ms, i as f64)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    map_lut(buf, n, &lut);
    Ok(())
}

fn brightness_contrast(
    p: &BrightnessContrastParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    if !(-150..=150).contains(&p.brightness) {
        return Err(AdjustError::InvalidParams("brightness out of range".into()));
    }
    if !(-50..=100).contains(&p.contrast) {
        return Err(AdjustError::InvalidParams("contrast out of range".into()));
    }
    if p.brightness == 0 && p.contrast == 0 {
        return Ok(());
    }
    let c = p.contrast as f64 / 100.0;
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        let out = if p.use_legacy {
            // Approximation: additive shift in levels, then linear contrast
            // about mid-grey (Adobe's legacy normalization is closed).
            let v = i as f64 + p.brightness as f64;
            ((v - 127.5) * (1.0 + c) + 127.5) / 255.0
        } else {
            let b = p.brightness as f64 / 150.0;
            let gamma = 2f64.powf(b);
            let y = (i as f64 / 255.0).powf(1.0 / gamma);
            // Monotone S-curve through (0,0),(0.5,0.5),(1,1); central slope 1+c.
            let xs = [0.0, 0.5, 1.0];
            let ys = [0.0, 0.5, 1.0];
            let ms = [1.0, 1.0 + c, 1.0];
            hermite_eval(&xs, &ys, &ms, y)
        };
        *slot = (out.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    map_lut(buf, n, &lut);
    Ok(())
}

fn exposure(p: &ExposureParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if !p.exposure.is_finite() || !p.offset.is_finite() {
        return Err(AdjustError::InvalidParams(
            "exposure/offset must be finite".into(),
        ));
    }
    if !p.gamma.is_finite() || p.gamma <= 0.0 {
        return Err(AdjustError::InvalidParams(
            "exposure gamma must be > 0".into(),
        ));
    }
    if p.exposure == 0.0 && p.offset == 0.0 && p.gamma == 1.0 {
        return Ok(());
    }
    let gain = 2f64.powf(p.exposure);
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        // 8-bit values are sRGB-encoded; decode to linear light, apply, re-encode.
        let lin = (srgb_to_linear(i as f64 / 255.0) * gain + p.offset).max(0.0);
        let lin = lin.powf(p.gamma);
        *slot = (linear_to_srgb(lin).clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    map_lut(buf, n, &lut);
    Ok(())
}

fn desaturate(buf: &mut PixelBuffer, n: usize) {
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        // (min + max) / 2 == HSL with S = 0, matching the CS6 spec.
        let v = ((*rv).min(*gv).min(*bv) as u16 + (*rv).max(*gv).max(*bv) as u16) as f64 / 2.0;
        let v = v.round() as u8;
        *rv = v;
        *gv = v;
        *bv = v;
    }
}

fn posterize(levels: u8, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if levels < 2 {
        return Err(AdjustError::InvalidParams(
            "posterize levels must be 2..=255".into(),
        ));
    }
    if levels == 255 {
        // 255 target levels coincide with every 8-bit code (spec: "turns it off").
        return Ok(());
    }
    let denom = (levels - 1) as f64;
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        let q = (i as f64 * denom / 255.0).round();
        *slot = (q * 255.0 / denom).round().clamp(0.0, 255.0) as u8;
    }
    map_lut(buf, n, &lut);
    Ok(())
}

fn threshold(level: u8, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if level == 0 {
        return Err(AdjustError::InvalidParams(
            "threshold level must be 1..=255".into(),
        ));
    }
    let t = level as f64;
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let y = luma(*rv as f64, *gv as f64, *bv as f64);
        // Equality falls to black (spec's boundary is unspecified).
        let v = if y > t { 255 } else { 0 };
        *rv = v;
        *gv = v;
        *bv = v;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Colour adjustments
// ---------------------------------------------------------------------------

fn hue_saturation(
    p: &HueSaturationParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    if !(-180..=180).contains(&p.hue) {
        return Err(AdjustError::InvalidParams("hue must be -180..=180".into()));
    }
    if !(-100..=100).contains(&p.saturation) {
        return Err(AdjustError::InvalidParams(
            "saturation must be -100..=100".into(),
        ));
    }
    if !(-100..=100).contains(&p.lightness) {
        return Err(AdjustError::InvalidParams(
            "lightness must be -100..=100".into(),
        ));
    }
    if p.hue == 0 && p.saturation == 0 && p.lightness == 0 {
        return Ok(());
    }
    let ds = p.saturation as f64 / 100.0;
    let dl = p.lightness as f64 / 100.0;
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (h, s, l) = rgb_to_hsl(*rv as f64 / 255.0, *gv as f64 / 255.0, *bv as f64 / 255.0);
        let h = (h + p.hue as f64).rem_euclid(360.0);
        let s = (s * (1.0 + ds)).clamp(0.0, 1.0);
        let l = if dl >= 0.0 {
            l + dl * (1.0 - l)
        } else {
            l + dl * l
        }
        .clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        *rv = (nr * 255.0).round().clamp(0.0, 255.0) as u8;
        *gv = (ng * 255.0).round().clamp(0.0, 255.0) as u8;
        *bv = (nb * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

fn vibrance(p: &VibranceParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if !(-100..=100).contains(&p.vibrance) {
        return Err(AdjustError::InvalidParams(
            "vibrance must be -100..=100".into(),
        ));
    }
    if !(-100..=100).contains(&p.saturation) {
        return Err(AdjustError::InvalidParams(
            "saturation must be -100..=100".into(),
        ));
    }
    if p.vibrance == 0 && p.saturation == 0 {
        return Ok(());
    }
    let kv = p.vibrance as f64 / 100.0;
    let ks = p.saturation as f64 / 100.0;
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (h, s, l) = rgb_to_hsl(*rv as f64 / 255.0, *gv as f64 / 255.0, *bv as f64 / 255.0);
        // Approximation: boost falls off as (1 - S) (the spec's `S*(1-S)` text
        // contradicts its own acceptance criteria) and is damped in the skin
        // hue band. Adobe's exact falloff/skin model is closed.
        let keep = 1.0 - 0.5 * skin_bump(h);
        let s = (s + kv * (1.0 - s) * keep).clamp(0.0, 1.0);
        let s = (s * (1.0 + ks)).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        *rv = (nr * 255.0).round().clamp(0.0, 255.0) as u8;
        *gv = (ng * 255.0).round().clamp(0.0, 255.0) as u8;
        *bv = (nb * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

/// Smooth bump over the orange/red skin band (~30° centre, 30° half-width).
fn skin_bump(h: f64) -> f64 {
    let d = ((h - 30.0 + 540.0).rem_euclid(360.0) - 180.0).abs();
    if d >= 30.0 {
        0.0
    } else {
        0.5 * (1.0 + (std::f64::consts::PI * d / 30.0).cos())
    }
}

fn color_balance(
    p: &ColorBalanceParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    for band in [&p.shadows, &p.midtones, &p.highlights] {
        if band.iter().any(|v| !v.is_finite() || v.abs() > 100.0) {
            return Err(AdjustError::InvalidParams(
                "color balance values must be -100..=100".into(),
            ));
        }
    }
    if p.shadows == [0.0; 3] && p.midtones == [0.0; 3] && p.highlights == [0.0; 3] {
        return Ok(());
    }
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
        let y = luma(ir, ig, ib) / 255.0;
        // Overlapping parabola windows: shadow=1 at Y=0, highlight=1 at Y=1,
        // midtone peaks at Y=0.5. Adobe's exact falloffs are closed.
        let sh = (1.0 - y) * (1.0 - y);
        let hi = y * y;
        let mid = (1.0 - sh - hi).max(0.0);
        let w = [sh, mid, hi];
        let mut out = [ir, ig, ib];
        for (c, o) in out.iter_mut().enumerate() {
            let delta = w[0] * p.shadows[c] + w[1] * p.midtones[c] + w[2] * p.highlights[c];
            *o = (*o + delta).clamp(0.0, 255.0);
        }
        if p.preserve_luminosity {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        *rv = out[0].round() as u8;
        *gv = out[1].round() as u8;
        *bv = out[2].round() as u8;
    }
    Ok(())
}

fn black_white(p: &BlackWhiteParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    let weights = [p.red, p.yellow, p.green, p.cyan, p.blue, p.magenta];
    if weights
        .iter()
        .any(|v| !v.is_finite() || !(-200.0..=300.0).contains(v))
    {
        return Err(AdjustError::InvalidParams(
            "black & white weights must be -200..=300 percent".into(),
        ));
    }
    if p.tint && p.tint_color == [0, 0, 0] {
        // A black tint is a no-op; avoid tinting everything to black.
        return Err(AdjustError::InvalidParams(
            "tint colour must not be black".into(),
        ));
    }
    let (rw, yw, gw, cw, bw, mw) = (
        p.red / 100.0,
        p.yellow / 100.0,
        p.green / 100.0,
        p.cyan / 100.0,
        p.blue / 100.0,
        p.magenta / 100.0,
    );
    let tint_hsl = if p.tint {
        let (h, s, _) = rgb_to_hsl(
            p.tint_color[0] as f64 / 255.0,
            p.tint_color[1] as f64 / 255.0,
            p.tint_color[2] as f64 / 255.0,
        );
        Some((h, s))
    } else {
        None
    };
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (mut cr, mut cg, mut cb) = (*rv as f64, *gv as f64, *bv as f64);
        let neutral = cr.min(cg).min(cb);
        cr -= neutral;
        cg -= neutral;
        cb -= neutral;
        let mut gray = neutral;
        if cr == 0.0 {
            let cyan = cg.min(cb);
            cg -= cyan;
            cb -= cyan;
            gray += cyan * cw + cg * gw + cb * bw;
        } else if cg == 0.0 {
            let magenta = cr.min(cb);
            cr -= magenta;
            cb -= magenta;
            gray += magenta * mw + cr * rw + cb * bw;
        } else {
            let yellow = cr.min(cg);
            cr -= yellow;
            cg -= yellow;
            gray += yellow * yw + cr * rw + cg * gw;
        }
        let out = gray.round().clamp(0.0, 255.0) as u8;
        let (nr, ng, nb) = match tint_hsl {
            Some((h, s)) => {
                let (tr, tg, tb) = hsl_to_rgb(h, s, out as f64 / 255.0);
                (
                    (tr * 255.0).round() as u8,
                    (tg * 255.0).round() as u8,
                    (tb * 255.0).round() as u8,
                )
            }
            None => (out, out, out),
        };
        *rv = nr;
        *gv = ng;
        *bv = nb;
    }
    Ok(())
}

fn photo_filter(p: &PhotoFilterParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    if !p.density.is_finite() || !(0.0..=100.0).contains(&p.density) {
        return Err(AdjustError::InvalidParams(
            "photo filter density must be 0..=100".into(),
        ));
    }
    if p.density == 0.0 {
        return Ok(());
    }
    let d = p.density / 100.0;
    let f = [p.color[0] as f64, p.color[1] as f64, p.color[2] as f64];
    let (r, g, b) = planes_mut(buf, n);
    for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
        let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
        let mut out = [
            ir * (1.0 - d) + f[0] * d,
            ig * (1.0 - d) + f[1] * d,
            ib * (1.0 - d) + f[2] * d,
        ];
        if p.preserve_luminosity {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        *rv = out[0].round().clamp(0.0, 255.0) as u8;
        *gv = out[1].round().clamp(0.0, 255.0) as u8;
        *bv = out[2].round().clamp(0.0, 255.0) as u8;
    }
    Ok(())
}

fn channel_mixer(
    p: &ChannelMixerParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    let mixes = [&p.red, &p.green, &p.blue];
    if mixes
        .iter()
        .chain(std::iter::once(&&p.constant))
        .any(|m| m.iter().any(|v| !v.is_finite() || v.abs() > 200.0))
    {
        return Err(AdjustError::InvalidParams(
            "channel mixer weights must be -200..=200 percent".into(),
        ));
    }
    let (r, g, b) = planes_mut(buf, n);
    if p.monochrome {
        // Monochrome applies one source mix to every output; use the Red row.
        let w = p.red;
        let c = p.constant[0] / 100.0 * 255.0;
        for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
            let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
            let v = ((w[0] * ir + w[1] * ig + w[2] * ib) / 100.0 + c)
                .round()
                .clamp(0.0, 255.0) as u8;
            *rv = v;
            *gv = v;
            *bv = v;
        }
    } else {
        for ((rv, gv), bv) in r.iter_mut().zip(g.iter_mut()).zip(b.iter_mut()) {
            let (ir, ig, ib) = (*rv as f64, *gv as f64, *bv as f64);
            let mix = |w: &[f64; 3], k: f64| {
                ((w[0] * ir + w[1] * ig + w[2] * ib) / 100.0 + k / 100.0 * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8
            };
            *rv = mix(&p.red, p.constant[0]);
            *gv = mix(&p.green, p.constant[1]);
            *bv = mix(&p.blue, p.constant[2]);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Auto corrections
// ---------------------------------------------------------------------------

fn auto(kind: AutoKind, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
    match kind {
        AutoKind::Tone => auto_per_channel(buf, n, 0.001),
        AutoKind::Contrast => auto_joint(buf, n, 0.005),
        AutoKind::Color => {
            auto_per_channel(buf, n, 0.005);
            snap_neutral_midtones(buf, n);
        }
    }
    Ok(())
}

/// First/last histogram bin that keeps `clip` of the population out.
fn percentile_bounds(hist: &[u64; 256], total: u64, clip: f64) -> Option<(u8, u8)> {
    if total == 0 {
        return None;
    }
    let cut = (total as f64 * clip).floor() as u64;
    let mut lo = 0usize;
    let mut acc = 0u64;
    while lo < 255 {
        if acc + hist[lo] > cut {
            break;
        }
        acc += hist[lo];
        lo += 1;
    }
    let mut hi = 255usize;
    let mut acc = 0u64;
    while hi > 0 {
        if acc + hist[hi] > cut {
            break;
        }
        acc += hist[hi];
        hi -= 1;
    }
    Some((lo as u8, hi as u8))
}

fn stretch_lut(lo: u8, hi: u8) -> [u8; 256] {
    let scale = 255.0 / (hi as f64 - lo as f64);
    let mut lut = [0u8; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        *slot = ((i as f64 - lo as f64) * scale).round().clamp(0.0, 255.0) as u8;
    }
    lut
}

fn auto_per_channel(buf: &mut PixelBuffer, n: usize, clip: f64) {
    let (r, g, b) = planes_mut(buf, n);
    for plane in [r, g, b] {
        let mut hist = [0u64; 256];
        for &v in plane.iter() {
            hist[v as usize] += 1;
        }
        if let Some((lo, hi)) = percentile_bounds(&hist, n as u64, clip) {
            if lo < hi {
                let lut = stretch_lut(lo, hi);
                for v in plane.iter_mut() {
                    *v = lut[*v as usize];
                }
            }
        }
    }
}

fn auto_joint(buf: &mut PixelBuffer, n: usize, clip: f64) {
    let (r, g, b) = planes_mut(buf, n);
    let mut hist = [0u64; 256];
    for plane in [&*r, &*g, &*b] {
        for &v in plane.iter() {
            hist[v as usize] += 1;
        }
    }
    if let Some((lo, hi)) = percentile_bounds(&hist, (n * 3) as u64, clip) {
        if lo < hi {
            let lut = stretch_lut(lo, hi);
            for plane in [r, g, b] {
                for v in plane.iter_mut() {
                    *v = lut[*v as usize];
                }
            }
        }
    }
}

/// Approximation of "Find Dark & Light + Snap Neutral Midtones": per-channel
/// gamma brings the average near-neutral midtone to the shared mean.
fn snap_neutral_midtones(buf: &mut PixelBuffer, n: usize) {
    let (r, g, b) = planes_mut(buf, n);
    let mut sums = [0.0f64; 3];
    let mut count = 0u64;
    for ((rv, gv), bv) in r.iter().zip(g.iter()).zip(b.iter()) {
        let y = luma(*rv as f64, *gv as f64, *bv as f64);
        if (64.0..=192.0).contains(&y) {
            sums[0] += *rv as f64;
            sums[1] += *gv as f64;
            sums[2] += *bv as f64;
            count += 1;
        }
    }
    if count == 0 {
        return;
    }
    let means = [
        sums[0] / count as f64,
        sums[1] / count as f64,
        sums[2] / count as f64,
    ];
    let target = (means[0] + means[1] + means[2]) / 3.0;
    if target <= 0.0 {
        return;
    }
    for (c, plane) in [r, g, b].into_iter().enumerate() {
        let m = means[c];
        if m <= 0.0 || (m / 255.0).ln() == 0.0 {
            continue;
        }
        let gamma = ((m / 255.0).ln() / (target / 255.0).ln()).clamp(0.1, 9.99);
        for v in plane.iter_mut() {
            *v = (255.0 * (*v as f64 / 255.0).powf(1.0 / gamma))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buf3(w: u32, h: u32, px: &[[u8; 3]]) -> PixelBuffer {
        let n = (w * h) as usize;
        assert_eq!(px.len(), n);
        let mut data = vec![0u8; n * 3];
        for (i, p) in px.iter().enumerate() {
            data[i] = p[0];
            data[n + i] = p[1];
            data[2 * n + i] = p[2];
        }
        PixelBuffer {
            width: w,
            height: h,
            channels: 3,
            data,
        }
    }

    fn buf4(px: &[[u8; 4]]) -> PixelBuffer {
        let n = px.len();
        let mut data = vec![0u8; n * 4];
        for (i, p) in px.iter().enumerate() {
            data[i] = p[0];
            data[n + i] = p[1];
            data[2 * n + i] = p[2];
            data[3 * n + i] = p[3];
        }
        PixelBuffer {
            width: n as u32,
            height: 1,
            channels: 4,
            data,
        }
    }

    fn px3(b: &PixelBuffer, i: usize) -> [u8; 3] {
        let n = b.pixel_count();
        [b.data[i], b.data[n + i], b.data[2 * n + i]]
    }

    fn ramp_rgb(from: u8, to: u8, steps: usize) -> PixelBuffer {
        let px: Vec<[u8; 3]> = (0..steps)
            .map(|i| {
                let v = from as f64 + (to as f64 - from as f64) * i as f64 / (steps - 1) as f64;
                let v = v.round() as u8;
                [v, v, v]
            })
            .collect();
        buf3(steps as u32, 1, &px)
    }

    const BW_DEFAULT: BlackWhiteParams = BlackWhiteParams {
        red: 40.0,
        yellow: 60.0,
        green: 40.0,
        cyan: 60.0,
        blue: 20.0,
        magenta: 80.0,
        tint: false,
        tint_color: [0, 0, 0],
    };

    // --- Level / curves / exposure / brightness-contrast -------------------

    #[test]
    fn levels_identity_and_clamps() {
        let mut b = buf3(2, 1, &[[0, 0, 0], [255, 255, 255]]);
        apply(
            &Adjustment::Levels(LevelsParams {
                input_black: 0,
                input_white: 255,
                gamma: 1.0,
                output_black: 0,
                output_white: 255,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0), [0, 0, 0]);
        assert_eq!(px3(&b, 1), [255, 255, 255]);
    }

    #[test]
    fn levels_black_white_point_and_output_range() {
        let mut b = buf3(
            4,
            1,
            &[[3, 3, 3], [124, 124, 124], [243, 243, 243], [250, 250, 250]],
        );
        apply(
            &Adjustment::Levels(LevelsParams {
                input_black: 5,
                input_white: 243,
                gamma: 1.0,
                output_black: 20,
                output_white: 235,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0)[0], 20);
        assert_eq!(px3(&b, 2)[0], 235);
        assert_eq!(px3(&b, 3)[0], 235);
        for i in 0..4 {
            let v = px3(&b, i)[0];
            assert!((20..=235).contains(&v), "value {v} out of output range");
        }
    }

    #[test]
    fn levels_gamma_lightens_midtones() {
        let mut b = buf3(1, 1, &[[124, 124, 124]]);
        apply(
            &Adjustment::Levels(LevelsParams {
                input_black: 0,
                input_white: 255,
                gamma: 2.0,
                output_black: 0,
                output_white: 255,
            }),
            &mut b,
        )
        .unwrap();
        assert!(px3(&b, 0)[0] > 124, "gamma>1 should lighten");
    }

    #[test]
    fn levels_rejects_bad_params() {
        let p = LevelsParams {
            input_black: 200,
            input_white: 100,
            gamma: 1.0,
            output_black: 0,
            output_white: 255,
        };
        let mut b = buf3(1, 1, &[[0, 0, 0]]);
        assert!(apply(&Adjustment::Levels(p), &mut b).is_err());
        let p = LevelsParams {
            input_black: 0,
            input_white: 255,
            gamma: 0.0,
            output_black: 0,
            output_white: 255,
        };
        assert!(apply(&Adjustment::Levels(p), &mut b).is_err());
    }

    #[test]
    fn curves_identity_and_control_point() {
        let mut b = ramp_rgb(0, 255, 256);
        let orig = b.clone();
        apply(
            &Adjustment::Curves(CurvesParams {
                points: vec![(0, 0), (255, 255)],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig, "two-point identity must be bit-exact");
        let mut b = buf3(1, 1, &[[100, 100, 100]]);
        apply(
            &Adjustment::Curves(CurvesParams {
                points: vec![(0, 0), (100, 200), (255, 255)],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0)[0], 200);
    }

    #[test]
    fn curves_monotone_and_validation() {
        let mut b = ramp_rgb(0, 255, 256);
        apply(
            &Adjustment::Curves(CurvesParams {
                points: vec![(0, 0), (64, 32), (192, 224), (255, 255)],
            }),
            &mut b,
        )
        .unwrap();
        for i in 1..256 {
            assert!(px3(&b, i)[0] >= px3(&b, i - 1)[0], "curve must be monotone");
        }
        let mut b = buf3(1, 1, &[[0, 0, 0]]);
        assert!(apply(
            &Adjustment::Curves(CurvesParams {
                points: vec![(0, 0)]
            }),
            &mut b
        )
        .is_err());
        assert!(apply(
            &Adjustment::Curves(CurvesParams {
                points: vec![(0, 0), (0, 10), (255, 255)]
            }),
            &mut b
        )
        .is_err());
    }

    #[test]
    fn bc_identity_both_modes_and_legacy_clip() {
        let b = ramp_rgb(0, 255, 256);
        let orig = b.clone();
        for legacy in [false, true] {
            let mut c = b.clone();
            apply(
                &Adjustment::BrightnessContrast(BrightnessContrastParams {
                    brightness: 0,
                    contrast: 0,
                    use_legacy: legacy,
                }),
                &mut c,
            )
            .unwrap();
            assert_eq!(c, orig);
        }
        let mut b = buf3(2, 1, &[[200, 200, 200], [250, 250, 250]]);
        apply(
            &Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 20,
                contrast: 0,
                use_legacy: true,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0)[0], 220);
        assert_eq!(px3(&b, 1)[0], 255, "legacy must clip");
    }

    #[test]
    fn bc_modern_contrast_is_soft_and_monotone() {
        let mut b = ramp_rgb(0, 255, 256);
        apply(
            &Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 0,
                contrast: 50,
                use_legacy: false,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0)[0], 0);
        assert_eq!(px3(&b, 255)[0], 255);
        assert!(px3(&b, 64)[0] < 64, "shadows should darken");
        assert!(px3(&b, 192)[0] > 192, "highlights should lighten");
        for i in 1..256 {
            assert!(px3(&b, i)[0] >= px3(&b, i - 1)[0]);
        }
    }

    #[test]
    fn exposure_identity_and_ev_gain() {
        let mut b = ramp_rgb(0, 255, 256);
        let orig = b.clone();
        apply(
            &Adjustment::Exposure(ExposureParams {
                exposure: 0.0,
                offset: 0.0,
                gamma: 1.0,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig, "identity exposure must be bit-exact");

        let mut b = buf3(1, 1, &[[128, 128, 128]]);
        apply(
            &Adjustment::Exposure(ExposureParams {
                exposure: 1.0,
                offset: 0.0,
                gamma: 1.0,
            }),
            &mut b,
        )
        .unwrap();
        let out_lin = srgb_to_linear(px3(&b, 0)[0] as f64 / 255.0);
        let in_lin = srgb_to_linear(128.0 / 255.0);
        assert!(
            (out_lin - 2.0 * in_lin).abs() < 0.02,
            "one EV doubles linear light"
        );
    }

    #[test]
    fn exposure_rejects_bad_gamma() {
        let mut b = buf3(1, 1, &[[10, 10, 10]]);
        assert!(apply(
            &Adjustment::Exposure(ExposureParams {
                exposure: 0.0,
                offset: 0.0,
                gamma: 0.0,
            }),
            &mut b
        )
        .is_err());
    }

    // --- Invert / posterize / threshold / desaturate -----------------------

    #[test]
    fn invert_known_and_involution() {
        let mut b = buf3(3, 1, &[[0, 5, 200], [255, 255, 255], [1, 2, 3]]);
        apply(&Adjustment::Invert, &mut b).unwrap();
        assert_eq!(px3(&b, 0), [255, 250, 55]);
        assert_eq!(px3(&b, 1), [0, 0, 0]);
        apply(&Adjustment::Invert, &mut b).unwrap();
        assert_eq!(px3(&b, 0), [0, 5, 200]);
        assert_eq!(px3(&b, 2), [1, 2, 3]);
    }

    #[test]
    fn posterize_levels_and_idempotence() {
        let mut b = buf3(1, 1, &[[64, 64, 64]]);
        apply(&Adjustment::Posterize(4), &mut b).unwrap();
        assert_eq!(px3(&b, 0), [85, 85, 85]);
        let mut b = buf3(
            5,
            1,
            &[
                [0, 0, 0],
                [85, 85, 85],
                [170, 170, 170],
                [200, 200, 200],
                [255, 255, 255],
            ],
        );
        apply(&Adjustment::Posterize(4), &mut b).unwrap();
        assert_eq!(px3(&b, 0)[0], 0);
        assert_eq!(px3(&b, 1)[0], 85);
        assert_eq!(px3(&b, 2)[0], 170);
        assert_eq!(px3(&b, 3)[0], 170);
        assert_eq!(px3(&b, 4)[0], 255);
        let once = b.clone();
        apply(&Adjustment::Posterize(4), &mut b).unwrap();
        assert_eq!(b, once, "posterize must be idempotent");
    }

    #[test]
    fn posterize_two_and_identity_and_invalid() {
        let mut b = buf3(
            4,
            1,
            &[[0, 0, 0], [127, 127, 127], [128, 128, 128], [255, 255, 255]],
        );
        apply(&Adjustment::Posterize(2), &mut b).unwrap();
        assert_eq!(px3(&b, 0)[0], 0);
        assert_eq!(px3(&b, 1)[0], 0);
        assert_eq!(px3(&b, 2)[0], 255);
        assert_eq!(px3(&b, 3)[0], 255);

        let mut b = ramp_rgb(0, 255, 256);
        let orig = b.clone();
        apply(&Adjustment::Posterize(255), &mut b).unwrap();
        assert_eq!(b, orig, "255 levels is the 8-bit identity");

        assert!(apply(&Adjustment::Posterize(0), &mut b).is_err());
        assert!(apply(&Adjustment::Posterize(1), &mut b).is_err());
    }

    #[test]
    fn threshold_binarizes_and_extremes() {
        let mut b = buf3(
            4,
            1,
            &[[255, 0, 0], [0, 0, 0], [255, 255, 255], [128, 128, 128]],
        );
        apply(&Adjustment::Threshold(128), &mut b).unwrap();
        assert_eq!(px3(&b, 0), [0, 0, 0], "red luma 76 is below 128");
        assert_eq!(px3(&b, 1), [0, 0, 0]);
        assert_eq!(px3(&b, 2), [255, 255, 255]);
        assert_eq!(px3(&b, 3), [0, 0, 0], "equality falls to black");

        let mut b = buf3(2, 1, &[[0, 0, 0], [255, 255, 255]]);
        apply(&Adjustment::Threshold(255), &mut b).unwrap();
        assert_eq!(px3(&b, 0), [0, 0, 0]);
        assert_eq!(px3(&b, 1), [0, 0, 0]);
        assert!(apply(&Adjustment::Threshold(0), &mut b).is_err());
    }

    #[test]
    fn desaturate_known_and_neutral() {
        let mut b = buf3(2, 1, &[[12, 104, 22], [77, 77, 77]]);
        apply(&Adjustment::Desaturate, &mut b).unwrap();
        assert_eq!(px3(&b, 0), [58, 58, 58]);
        assert_eq!(px3(&b, 1), [77, 77, 77]);
    }

    #[test]
    fn desaturate_matches_hue_saturation_minus_100() {
        let src = [[12, 104, 22], [200, 100, 50], [3, 250, 128]];
        let mut a = buf3(3, 1, &src);
        let mut c = buf3(3, 1, &src);
        apply(&Adjustment::Desaturate, &mut a).unwrap();
        apply(
            &Adjustment::HueSaturation(HueSaturationParams {
                hue: 0,
                saturation: -100,
                lightness: 0,
            }),
            &mut c,
        )
        .unwrap();
        for i in 0..3 {
            for k in 0..3 {
                let d = px3(&a, i)[k] as i16 - px3(&c, i)[k] as i16;
                assert!(d.abs() <= 1, "desaturate must match H/S -100 within 1 LSB");
            }
        }
    }

    // --- Hue/saturation and vibrance --------------------------------------

    #[test]
    fn hue_saturation_identity_and_desaturate() {
        let mut b = buf3(1, 1, &[[200, 100, 50]]);
        let orig = b.clone();
        apply(
            &Adjustment::HueSaturation(HueSaturationParams {
                hue: 0,
                saturation: 0,
                lightness: 0,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig, "neutral H/S must be bit-exact");

        apply(
            &Adjustment::HueSaturation(HueSaturationParams {
                hue: 0,
                saturation: -100,
                lightness: 0,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0), [125, 125, 125], "S=-100 keeps HSL lightness");
    }

    #[test]
    fn hue_saturation_lightness_and_validation() {
        let mut b = buf3(2, 1, &[[100, 100, 100], [50, 50, 50]]);
        apply(
            &Adjustment::HueSaturation(HueSaturationParams {
                hue: 0,
                saturation: 0,
                lightness: 50,
            }),
            &mut b,
        )
        .unwrap();
        assert!(px3(&b, 0)[0] > 100, "positive lightness adds white");
        assert!(px3(&b, 1)[0] > 50);
        assert!(apply(
            &Adjustment::HueSaturation(HueSaturationParams {
                hue: 200,
                saturation: 0,
                lightness: 0,
            }),
            &mut b
        )
        .is_err());
    }

    #[test]
    fn vibrance_identity_and_diminishing_boost() {
        let mut b = buf3(2, 1, &[[150, 100, 100], [255, 50, 50]]);
        let orig = b.clone();
        apply(
            &Adjustment::Vibrance(VibranceParams {
                vibrance: 0,
                saturation: 0,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig);

        let mut b = buf3(2, 1, &[[150, 100, 100], [255, 50, 50]]);
        apply(
            &Adjustment::Vibrance(VibranceParams {
                vibrance: 50,
                saturation: 0,
            }),
            &mut b,
        )
        .unwrap();
        let (_, s_lo, _) = rgb_to_hsl(
            px3(&b, 0)[0] as f64 / 255.0,
            px3(&b, 0)[1] as f64 / 255.0,
            px3(&b, 0)[2] as f64 / 255.0,
        );
        let (_, s_hi, _) = rgb_to_hsl(
            px3(&b, 1)[0] as f64 / 255.0,
            px3(&b, 1)[1] as f64 / 255.0,
            px3(&b, 1)[2] as f64 / 255.0,
        );
        assert!(s_lo > 0.2, "low-saturation patch should gain");
        assert!(s_hi <= 1.0 + 1e-9);
    }

    #[test]
    fn vibrance_saturation_minus_100_is_gray() {
        let mut b = buf3(1, 1, &[[200, 100, 50]]);
        apply(
            &Adjustment::Vibrance(VibranceParams {
                vibrance: 0,
                saturation: -100,
            }),
            &mut b,
        )
        .unwrap();
        let p = px3(&b, 0);
        assert_eq!(p[0], p[1]);
        assert_eq!(p[1], p[2]);
        assert!(apply(
            &Adjustment::Vibrance(VibranceParams {
                vibrance: 500,
                saturation: 0,
            }),
            &mut b
        )
        .is_err());
    }

    // --- Black & white / photo filter / channel mixer / color balance ------

    #[test]
    fn black_white_neutral_and_red() {
        let mut b = buf3(2, 1, &[[100, 100, 100], [200, 0, 0]]);
        apply(&Adjustment::BlackWhite(BW_DEFAULT), &mut b).unwrap();
        assert_eq!(px3(&b, 0), [100, 100, 100]);
        assert_eq!(px3(&b, 1), [80, 80, 80], "pure red uses the 40% red weight");
    }

    #[test]
    fn black_white_tint_and_validation() {
        let mut p = BW_DEFAULT.clone();
        p.tint = true;
        p.tint_color = [200, 150, 100];
        let mut b = buf3(1, 1, &[[100, 100, 100]]);
        apply(&Adjustment::BlackWhite(p.clone()), &mut b).unwrap();
        let out = px3(&b, 0);
        assert!(out[0] > out[2], "sepia tint should make R > B");

        let mut bad = BW_DEFAULT.clone();
        bad.red = 999.0;
        assert!(apply(&Adjustment::BlackWhite(bad), &mut b).is_err());
    }

    #[test]
    fn photo_filter_warms_and_preserves_luminosity() {
        let mut b = buf3(2, 1, &[[128, 128, 128], [64, 64, 64]]);
        apply(
            &Adjustment::PhotoFilter(PhotoFilterParams {
                color: [255, 180, 80],
                density: 25.0,
                preserve_luminosity: true,
            }),
            &mut b,
        )
        .unwrap();
        for i in 0..2 {
            let out = px3(&b, i);
            assert!(out[0] > out[2], "warming filter should increase R over B");
            let y = luma(out[0] as f64, out[1] as f64, out[2] as f64);
            let orig = if i == 0 { 128.0 } else { 64.0 };
            assert!((y - orig).abs() < 3.0, "luminosity should be preserved");
        }
    }

    #[test]
    fn photo_filter_density_zero_is_noop_and_validates() {
        let mut b = buf3(1, 1, &[[10, 20, 30]]);
        let orig = b.clone();
        apply(
            &Adjustment::PhotoFilter(PhotoFilterParams {
                color: [255, 0, 0],
                density: 0.0,
                preserve_luminosity: false,
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig);
        assert!(apply(
            &Adjustment::PhotoFilter(PhotoFilterParams {
                color: [255, 0, 0],
                density: 150.0,
                preserve_luminosity: false,
            }),
            &mut b
        )
        .is_err());
    }

    #[test]
    fn channel_mixer_known_example_and_identity() {
        let mut b = buf3(1, 1, &[[50, 100, 200]]);
        apply(
            &Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: false,
                red: [50.0, 100.0, 0.0],
                green: [0.0, 100.0, 0.0],
                blue: [0.0, 0.0, 100.0],
                constant: [0.0, 0.0, 0.0],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0), [125, 100, 200]);

        let mut b = buf3(2, 1, &[[10, 20, 30], [200, 100, 50]]);
        let orig = b.clone();
        apply(
            &Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: false,
                red: [100.0, 0.0, 0.0],
                green: [0.0, 100.0, 0.0],
                blue: [0.0, 0.0, 100.0],
                constant: [0.0, 0.0, 0.0],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(b, orig);
    }

    #[test]
    fn channel_mixer_monochrome_and_constant_and_validation() {
        let mut b = buf3(1, 1, &[[255, 0, 0]]);
        apply(
            &Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: true,
                red: [40.0, 40.0, 20.0],
                green: [40.0, 40.0, 20.0],
                blue: [40.0, 40.0, 20.0],
                constant: [0.0, 0.0, 0.0],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0), [102, 102, 102]);

        let mut b = buf3(1, 1, &[[100, 100, 100]]);
        apply(
            &Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: true,
                red: [40.0, 40.0, 20.0],
                green: [40.0, 40.0, 20.0],
                blue: [40.0, 40.0, 20.0],
                constant: [10.0, 10.0, 10.0],
            }),
            &mut b,
        )
        .unwrap();
        assert_eq!(px3(&b, 0)[0], 126, "constant adds 10% white");

        assert!(apply(
            &Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: false,
                red: [999.0, 0.0, 0.0],
                green: [0.0, 100.0, 0.0],
                blue: [0.0, 0.0, 100.0],
                constant: [0.0, 0.0, 0.0],
            }),
            &mut b
        )
        .is_err());
    }

    #[test]
    fn color_balance_midtone_red_and_luminosity() {
        let params = ColorBalanceParams {
            shadows: [0.0, 0.0, 0.0],
            midtones: [100.0, 0.0, 0.0],
            highlights: [0.0, 0.0, 0.0],
            preserve_luminosity: false,
        };
        let mut b = buf3(2, 1, &[[0, 0, 0], [128, 128, 128]]);
        apply(&Adjustment::ColorBalance(params.clone()), &mut b).unwrap();
        assert_eq!(px3(&b, 0), [0, 0, 0], "midtones must not touch black");
        assert!(px3(&b, 1)[0] > 128, "mid grey shifts toward red");

        let mut b = buf3(1, 1, &[[128, 128, 128]]);
        let params_lum = ColorBalanceParams {
            preserve_luminosity: true,
            ..params
        };
        apply(&Adjustment::ColorBalance(params_lum), &mut b).unwrap();
        let out = px3(&b, 0);
        let y = luma(out[0] as f64, out[1] as f64, out[2] as f64);
        assert!((y - 128.0).abs() < 3.0, "luminosity preserved");
    }

    #[test]
    fn color_balance_validation() {
        let mut b = buf3(1, 1, &[[10, 10, 10]]);
        assert!(apply(
            &Adjustment::ColorBalance(ColorBalanceParams {
                shadows: [0.0, 0.0, 0.0],
                midtones: [200.0, 0.0, 0.0],
                highlights: [0.0, 0.0, 0.0],
                preserve_luminosity: false,
            }),
            &mut b
        )
        .is_err());
    }

    // --- Auto --------------------------------------------------------------

    #[test]
    fn auto_tone_stretches_per_channel() {
        let mut b = ramp_rgb(60, 200, 100);
        apply(&Adjustment::Auto(AutoKind::Tone), &mut b).unwrap();
        assert!(px3(&b, 0)[0] <= 2, "dark end maps to black");
        assert!(px3(&b, 99)[0] >= 253, "light end maps to white");
    }

    #[test]
    fn auto_contrast_joint_no_cast() {
        let px: Vec<[u8; 3]> = (0..100)
            .map(|i| {
                let v = 60 + (140 * i / 99) as u8;
                [v, v.saturating_add(0), v]
            })
            .collect();
        let mut b = buf3(100, 1, &px);
        apply(&Adjustment::Auto(AutoKind::Contrast), &mut b).unwrap();
        for i in 0..100 {
            let p = px3(&b, i);
            assert_eq!(p[0], p[1]);
            assert_eq!(p[1], p[2]);
        }
    }

    #[test]
    fn auto_color_neutralizes_midtones() {
        let px: Vec<[u8; 3]> = (0..100)
            .map(|i| {
                let v = 60.0 + 140.0 * i as f64 / 99.0;
                [
                    (v + 15.0).round() as u8,
                    v.round() as u8,
                    (v - 15.0).round() as u8,
                ]
            })
            .collect();
        let mut b = buf3(100, 1, &px);
        apply(&Adjustment::Auto(AutoKind::Color), &mut b).unwrap();
        let (mut sr, mut sg, mut sb) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..100 {
            let p = px3(&b, i);
            sr += p[0] as f64;
            sg += p[1] as f64;
            sb += p[2] as f64;
        }
        let spread = (sr - sg).abs().max((sg - sb).abs());
        assert!(spread < 300.0, "channels should be pulled toward neutral");
    }

    // --- Contract / invariants --------------------------------------------

    #[test]
    fn alpha_is_never_modified() {
        let base = buf4(&[[10, 20, 30, 7], [200, 100, 50, 250], [0, 0, 0, 128]]);
        let n = base.pixel_count();
        let alpha: Vec<u8> = base.data[3 * n..4 * n].to_vec();
        let adjustments = vec![
            Adjustment::Levels(LevelsParams {
                input_black: 5,
                input_white: 250,
                gamma: 1.2,
                output_black: 0,
                output_white: 255,
            }),
            Adjustment::Curves(CurvesParams {
                points: vec![(0, 0), (128, 180), (255, 255)],
            }),
            Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 10,
                contrast: 10,
                use_legacy: false,
            }),
            Adjustment::Exposure(ExposureParams {
                exposure: 0.5,
                offset: 0.0,
                gamma: 1.0,
            }),
            Adjustment::HueSaturation(HueSaturationParams {
                hue: 10,
                saturation: 10,
                lightness: 10,
            }),
            Adjustment::BlackWhite(BW_DEFAULT),
            Adjustment::PhotoFilter(PhotoFilterParams {
                color: [255, 180, 80],
                density: 25.0,
                preserve_luminosity: false,
            }),
            Adjustment::ChannelMixer(ChannelMixerParams {
                monochrome: false,
                red: [100.0, 0.0, 0.0],
                green: [0.0, 100.0, 0.0],
                blue: [0.0, 0.0, 100.0],
                constant: [0.0, 0.0, 0.0],
            }),
            Adjustment::Vibrance(VibranceParams {
                vibrance: 20,
                saturation: 10,
            }),
            Adjustment::ColorBalance(ColorBalanceParams {
                shadows: [0.0, 0.0, 0.0],
                midtones: [20.0, 0.0, 0.0],
                highlights: [0.0, 0.0, 0.0],
                preserve_luminosity: true,
            }),
            Adjustment::Auto(AutoKind::Tone),
            Adjustment::Invert,
            Adjustment::Posterize(4),
            Adjustment::Threshold(128),
            Adjustment::Desaturate,
        ];
        for a in adjustments {
            let mut b = base.clone();
            apply(&a, &mut b).unwrap();
            assert_eq!(&b.data[3 * n..4 * n], &alpha[..], "{a:?} touched alpha");
        }
    }

    #[test]
    fn bad_buffers_error_not_panic() {
        let mut empty = PixelBuffer {
            width: 0,
            height: 0,
            channels: 3,
            data: vec![],
        };
        assert!(apply(&Adjustment::Invert, &mut empty).is_err());

        let mut odd = PixelBuffer {
            width: 1,
            height: 1,
            channels: 2,
            data: vec![0, 0],
        };
        assert!(apply(&Adjustment::Invert, &mut odd).is_err());

        let mut short = PixelBuffer {
            width: 2,
            height: 2,
            channels: 3,
            data: vec![0; 5],
        };
        assert!(apply(&Adjustment::Invert, &mut short).is_err());
    }

    #[test]
    fn deterministic_across_runs() {
        let src = [[13, 200, 45], [250, 3, 128], [66, 66, 66]];
        let mut a = buf3(3, 1, &src);
        let mut c = buf3(3, 1, &src);
        let adj = Adjustment::HueSaturation(HueSaturationParams {
            hue: 37,
            saturation: 21,
            lightness: -9,
        });
        apply(&adj, &mut a).unwrap();
        apply(&adj, &mut c).unwrap();
        assert_eq!(a, c);

        let mut a = buf3(3, 1, &src);
        let mut c = buf3(3, 1, &src);
        apply(&Adjustment::Auto(AutoKind::Color), &mut a).unwrap();
        apply(&Adjustment::Auto(AutoKind::Color), &mut c).unwrap();
        assert_eq!(a, c);
    }
}
