//! Native-depth adjustments.
//!
//! [`apply_native`] applies the covered adjustment set to a [`Samples`] store
//! (`u8`/`u16`/`f32`) without quantizing to 8-bit first. The per-sample math
//! mirrors the 8-bit kernels in `tonal.rs` and `color.rs` in the unit domain;
//! the 8-bit `apply` path is untouched and stays the parity surface. Any
//! adjustment outside the covered set is refused with [`AdjustError::Unsupported`]
//! and the store is left unchanged.

use pictura_core::{PixelBuffer, Sample, Samples};

use crate::color::{
    hue_saturation_is_identity, hue_saturation_rgb, selective_color_is_identity,
    selective_color_rgb, skin_bump, validate_hue_saturation, validate_selective_color,
};
use crate::common::{
    hermite_eval, hsl_to_rgb, linear_to_srgb, luma, monotone_tangents, rgb_to_hsl, srgb_to_linear,
};
use crate::lut::sample;
use crate::shadows_highlights;
use crate::tonal::gradient_lut;
use crate::types::{
    AdjustError, Adjustment, AutoKind, BlackWhiteParams, BrightnessContrastParams,
    ChannelMixerParams, ColorBalanceParams, ColorLookupParams, CurvesParams, ExposureParams,
    GradientMapParams, HueSaturationParams, LevelsChannel, Lut3d, PhotoFilterParams,
    SelectiveColorParams, ShadowsHighlightsParams, VibranceParams,
};

type Rgb = [f64; 3];

/// Apply `adjustment` in place to a native-depth sample store whose color planes
/// are `channels` wide (3 or 4; alpha untouched). `width * height * channels`
/// samples must fit the store.
pub fn apply_native(
    adjustment: &Adjustment,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    validate(samples, width, height, channels)?;
    match adjustment {
        Adjustment::Invert => {
            map(samples, width, height, channels, |p| p.map(|x| 1.0 - x));
            Ok(())
        }
        Adjustment::Desaturate => {
            map(samples, width, height, channels, desaturate_pixel);
            Ok(())
        }
        Adjustment::Levels(p) => {
            let f = p.composite().map()?;
            let identity = LevelsChannel {
                input_black: 0,
                input_white: 255,
                gamma: 1.0,
                output_black: 0,
                output_white: 255,
            };
            let r = p.red.unwrap_or(identity).map()?;
            let g = p.green.unwrap_or(identity).map()?;
            let b = p.blue.unwrap_or(identity).map()?;
            map(samples, width, height, channels, |p| {
                [f(r(p[0])), f(g(p[1])), f(b(p[2]))]
            });
            Ok(())
        }
        Adjustment::Curves(p) => curves_native(p, samples, width, height, channels),
        Adjustment::BrightnessContrast(p) => {
            brightness_contrast_native(p, samples, width, height, channels)
        }
        Adjustment::Exposure(p) => {
            let f = exposure_map(p)?;
            if p.exposure == 0.0 && p.offset == 0.0 && p.gamma == 1.0 {
                return Ok(());
            }
            map(samples, width, height, channels, |p| {
                [f(p[0]), f(p[1]), f(p[2])]
            });
            Ok(())
        }
        Adjustment::Posterize(levels) => {
            if *levels == 255 {
                return Ok(());
            }
            let f = posterize_map(*levels)?;
            map(samples, width, height, channels, |p| {
                [f(p[0]), f(p[1]), f(p[2])]
            });
            Ok(())
        }
        Adjustment::Threshold(level) => {
            if *level == 0 {
                return Err(AdjustError::InvalidParams(
                    "threshold level must be 1..=255".into(),
                ));
            }
            let t = *level as f64;
            map(samples, width, height, channels, move |p| {
                let y = luma(p[0] * 255.0, p[1] * 255.0, p[2] * 255.0);
                let v = if y > t { 1.0 } else { 0.0 };
                [v, v, v]
            });
            Ok(())
        }
        Adjustment::GradientMap(p) => gradient_map_native(p, samples, width, height, channels),
        Adjustment::HueSaturation(p) => hue_saturation_native(p, samples, width, height, channels),
        Adjustment::Vibrance(p) => vibrance_native(p, samples, width, height, channels),
        Adjustment::ShadowsHighlights(p) => {
            shadows_highlights_native(p, samples, width, height, channels)
        }
        Adjustment::ColorBalance(p) => color_balance_native(p, samples, width, height, channels),
        Adjustment::BlackWhite(p) => black_white_native(p, samples, width, height, channels),
        Adjustment::PhotoFilter(p) => photo_filter_native(p, samples, width, height, channels),
        Adjustment::ChannelMixer(p) => channel_mixer_native(p, samples, width, height, channels),
        Adjustment::SelectiveColor(p) => {
            selective_color_native(p, samples, width, height, channels)
        }
        Adjustment::Auto(kind) => auto_native(*kind, samples, width, height, channels),
        Adjustment::ColorLookup(p) => color_lookup_native(p, samples, width, height, channels),
        other => Err(AdjustError::Unsupported(format!(
            "native-depth apply does not support {other:?}"
        ))),
    }
}

fn validate(
    samples: &Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    if channels != 3 && channels != 4 {
        return Err(AdjustError::Unsupported(format!(
            "channel count {channels} is not supported (expected 3 or 4)"
        )));
    }
    let n = width
        .checked_mul(height)
        .ok_or_else(|| AdjustError::InvalidParams("buffer size overflow".into()))?;
    if n == 0 {
        return Err(AdjustError::InvalidParams("empty buffer".into()));
    }
    let needed = n
        .checked_mul(channels as usize)
        .ok_or_else(|| AdjustError::InvalidParams("buffer size overflow".into()))?;
    if samples.len() < needed {
        return Err(AdjustError::InvalidParams(
            "buffer length does not match width*height*channels".into(),
        ));
    }
    Ok(())
}

fn map(
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
    f: impl FnMut(Rgb) -> Rgb,
) {
    samples.map_color_planes(width, height, channels, f);
}

fn desaturate_pixel(p: Rgb) -> Rgb {
    let x = [p[0] * 255.0, p[1] * 255.0, p[2] * 255.0];
    let lo = x[0].min(x[1]).min(x[2]);
    let hi = x[0].max(x[1]).max(x[2]);
    let v = (lo + hi) / 2.0 / 255.0;
    [v, v, v]
}

fn exposure_map(p: &ExposureParams) -> Result<impl Fn(f64) -> f64, AdjustError> {
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
    let gain = 2f64.powf(p.exposure);
    let (offset, gamma) = (p.offset, p.gamma);
    Ok(move |u: f64| {
        let lin = (srgb_to_linear(u) * gain + offset).max(0.0);
        linear_to_srgb(lin.powf(gamma))
    })
}

fn posterize_map(levels: u8) -> Result<impl Fn(f64) -> f64, AdjustError> {
    if levels < 2 {
        return Err(AdjustError::InvalidParams(
            "posterize levels must be 2..=255".into(),
        ));
    }
    let denom = (levels - 1) as f64;
    Ok(move |u: f64| {
        let q = (u * 255.0 * denom / 255.0).round();
        (q * 255.0 / denom).round() / 255.0
    })
}

fn shadows_highlights_native(
    p: &ShadowsHighlightsParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    if !shadows_highlights::validate(p)? {
        return Ok(());
    }
    // The 8-bit kernel's operator on unit samples: one pass reads the
    // luminance (writing every pixel back unchanged), the second maps.
    let mut luminance = Vec::with_capacity(width * height);
    map(samples, width, height, channels, |rgb| {
        luminance.push(shadows_highlights::luminance(rgb));
        rgb
    });
    let base = shadows_highlights::base(&luminance, width, height);
    let mut i = 0;
    map(samples, width, height, channels, |rgb| {
        let out = shadows_highlights::pixel(p, rgb, base[i]);
        i += 1;
        out
    });
    Ok(())
}

struct Curve {
    xs: Vec<f64>,
    ys: Vec<f64>,
    ms: Vec<f64>,
}

fn curve(points: &[(u8, u8)]) -> Result<Curve, AdjustError> {
    if points.len() < 2 {
        return Err(AdjustError::InvalidParams(
            "curves need at least 2 points".into(),
        ));
    }
    if points.len() > 14 {
        return Err(AdjustError::InvalidParams(
            "curves support at most 14 points".into(),
        ));
    }
    let xs: Vec<f64> = points.iter().map(|q| q.0 as f64).collect();
    let ys: Vec<f64> = points.iter().map(|q| q.1 as f64).collect();
    if xs.windows(2).any(|w| w[0] >= w[1]) {
        return Err(AdjustError::InvalidParams(
            "curve inputs must be strictly increasing".into(),
        ));
    }
    let ms = monotone_tangents(&xs, &ys);
    Ok(Curve { xs, ys, ms })
}

fn curve_unit(c: &Curve, u: f64) -> f64 {
    hermite_eval(&c.xs, &c.ys, &c.ms, u * 255.0) / 255.0
}

fn curves_native(
    p: &CurvesParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    let composite = curve(&p.points)?;
    let red = p.red.as_deref().map(curve).transpose()?;
    let green = p.green.as_deref().map(curve).transpose()?;
    let blue = p.blue.as_deref().map(curve).transpose()?;
    if red.is_some() || green.is_some() || blue.is_some() {
        map(samples, width, height, channels, |[r, g, b]| {
            [
                red.as_ref().map_or(r, |c| curve_unit(c, r)),
                green.as_ref().map_or(g, |c| curve_unit(c, g)),
                blue.as_ref().map_or(b, |c| curve_unit(c, b)),
            ]
        });
    }
    map(samples, width, height, channels, move |[r, g, b]| {
        [
            curve_unit(&composite, r),
            curve_unit(&composite, g),
            curve_unit(&composite, b),
        ]
    });
    Ok(())
}

fn brightness_contrast_native(
    p: &BrightnessContrastParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
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
    let brightness = p.brightness as f64;
    let c = p.contrast as f64 / 100.0;
    let legacy = p.use_legacy;
    map(samples, width, height, channels, move |p| {
        p.map(|u| brightness_contrast_pixel(u, brightness, c, legacy))
    });
    Ok(())
}

fn brightness_contrast_pixel(u: f64, brightness: f64, c: f64, legacy: bool) -> f64 {
    if legacy {
        let v = u * 255.0 + brightness;
        (((v - 127.5) * (1.0 + c) + 127.5) / 255.0).clamp(0.0, 1.0)
    } else {
        let gamma = 2f64.powf(brightness / 150.0);
        let y = u.powf(1.0 / gamma);
        hermite_eval(&[0.0, 0.5, 1.0], &[0.0, 0.5, 1.0], &[1.0, 1.0 + c, 1.0], y).clamp(0.0, 1.0)
    }
}

fn gradient_map_native(
    p: &GradientMapParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    // ponytail: luma is quantized to the 8-bit gradient LUT exactly as the
    // 8-bit kernel does, so the mapped colours are a fixed set of u8 colours; a
    // native luma sample would need the gradient interpolation to move off
    // 8-bit too.
    let lut = gradient_lut(p)?;
    map(samples, width, height, channels, move |p| {
        let y = luma(p[0] * 255.0, p[1] * 255.0, p[2] * 255.0);
        let (c, a) = lut[y.round().clamp(0.0, 255.0) as usize];
        [0, 1, 2].map(|i| p[i] + (c[i] as f64 / 255.0 - p[i]) * a)
    });
    Ok(())
}

fn hue_saturation_native(
    p: &HueSaturationParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    validate_hue_saturation(p)?;
    if hue_saturation_is_identity(p) {
        return Ok(());
    }
    map(samples, width, height, channels, |[r, g, b]| {
        hue_saturation_rgb(p, r, g, b)
    });
    Ok(())
}

fn vibrance_native(
    p: &VibranceParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
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
    map(samples, width, height, channels, move |[r, g, b]| {
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let keep = 1.0 - 0.5 * skin_bump(h);
        let s = (s + kv * (1.0 - s) * keep).clamp(0.0, 1.0);
        let s = (s * (1.0 + ks)).clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        [nr, ng, nb]
    });
    Ok(())
}

fn color_balance_native(
    p: &ColorBalanceParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
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
    let (shadows, midtones, highlights) = (p.shadows, p.midtones, p.highlights);
    let preserve = p.preserve_luminosity;
    map(samples, width, height, channels, move |[r, g, b]| {
        let (ir, ig, ib) = (r * 255.0, g * 255.0, b * 255.0);
        let y = luma(ir, ig, ib) / 255.0;
        let sh = (1.0 - y) * (1.0 - y);
        let hi = y * y;
        let mid = (1.0 - sh - hi).max(0.0);
        let w = [sh, mid, hi];
        let mut out = [ir, ig, ib];
        for (c, o) in out.iter_mut().enumerate() {
            let delta = w[0] * shadows[c] + w[1] * midtones[c] + w[2] * highlights[c];
            *o = (*o + delta).clamp(0.0, 255.0);
        }
        if preserve {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        [out[0] / 255.0, out[1] / 255.0, out[2] / 255.0]
    });
    Ok(())
}

fn black_white_native(
    p: &BlackWhiteParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
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
    map(samples, width, height, channels, move |[r, g, b]| {
        let (mut cr, mut cg, mut cb) = (r * 255.0, g * 255.0, b * 255.0);
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
        let out = gray.round().clamp(0.0, 255.0);
        match tint_hsl {
            Some((h, s)) => {
                let (tr, tg, tb) = hsl_to_rgb(h, s, out / 255.0);
                [tr, tg, tb]
            }
            None => [out / 255.0, out / 255.0, out / 255.0],
        }
    });
    Ok(())
}

fn photo_filter_native(
    p: &PhotoFilterParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
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
    let preserve = p.preserve_luminosity;
    map(samples, width, height, channels, move |[r, g, b]| {
        let (ir, ig, ib) = (r * 255.0, g * 255.0, b * 255.0);
        let mut out = [
            ir * (1.0 - d) + f[0] * d,
            ig * (1.0 - d) + f[1] * d,
            ib * (1.0 - d) + f[2] * d,
        ];
        if preserve {
            let y1 = luma(out[0], out[1], out[2]);
            if y1 > 0.0 {
                let scale = luma(ir, ig, ib) / y1;
                for o in &mut out {
                    *o = (*o * scale).clamp(0.0, 255.0);
                }
            }
        }
        [out[0] / 255.0, out[1] / 255.0, out[2] / 255.0]
    });
    Ok(())
}

fn channel_mixer_native(
    p: &ChannelMixerParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
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
    let (monochrome, red, green, blue, constant) =
        (p.monochrome, p.red, p.green, p.blue, p.constant);
    map(samples, width, height, channels, move |[r, g, b]| {
        let (ir, ig, ib) = (r * 255.0, g * 255.0, b * 255.0);
        if monochrome {
            let c = constant[0] / 100.0 * 255.0;
            let v = ((red[0] * ir + red[1] * ig + red[2] * ib) / 100.0 + c)
                .round()
                .clamp(0.0, 255.0);
            [v / 255.0, v / 255.0, v / 255.0]
        } else {
            let mix = |w: &[f64; 3], k: f64| {
                ((w[0] * ir + w[1] * ig + w[2] * ib) / 100.0 + k / 100.0 * 255.0)
                    .round()
                    .clamp(0.0, 255.0)
            };
            [
                mix(&red, constant[0]) / 255.0,
                mix(&green, constant[1]) / 255.0,
                mix(&blue, constant[2]) / 255.0,
            ]
        }
    });
    Ok(())
}

fn selective_color_native(
    p: &SelectiveColorParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    validate_selective_color(p)?;
    if selective_color_is_identity(p) {
        return Ok(());
    }
    map(samples, width, height, channels, |rgb| {
        selective_color_rgb(p, rgb)
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Auto and Color Lookup at native depth
// ---------------------------------------------------------------------------

/// Number of `f32` histogram bins over the unit interval: the 16-bit level count,
/// so the `u16` and `f32` stretches agree on the percentile resolution.
const F32_BINS: usize = 65536;

fn auto_native(
    kind: AutoKind,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    let n = width * height;
    match samples {
        // An 8-bit store already is the histogram kernel's own resolution, so the
        // 8-bit implementation is the exact native one.
        Samples::U8(data) => {
            let mut buf = PixelBuffer {
                width: width as u32,
                height: height as u32,
                channels,
                data: std::mem::take(data).into(),
            };
            let result = crate::auto::auto(kind, &mut buf, n);
            *data = buf.data.to_vec();
            result
        }
        Samples::U16(data) => {
            auto_u16(kind, data, n);
            Ok(())
        }
        Samples::F32(data) => {
            auto_f32(kind, data, n);
            Ok(())
        }
    }
}

fn auto_u16(kind: AutoKind, data: &mut [u16], n: usize) {
    match kind {
        AutoKind::Tone => auto_per_channel_u16(data, n, 0.001),
        AutoKind::Contrast => auto_joint_u16(data, n, 0.005),
        AutoKind::Color => {
            auto_per_channel_u16(data, n, 0.005);
            snap_neutral_native(data, n);
        }
    }
}

fn auto_f32(kind: AutoKind, data: &mut [f32], n: usize) {
    match kind {
        AutoKind::Tone => auto_per_channel_f32(data, n, 0.001),
        AutoKind::Contrast => auto_joint_f32(data, n, 0.005),
        AutoKind::Color => {
            auto_per_channel_f32(data, n, 0.005);
            snap_neutral_native(data, n);
        }
    }
}

/// First/last histogram bin that keeps `clip` of the population out; the same
/// walk as [`crate::auto::percentile_bounds`] generalized from 256 bins.
fn percentile_bounds_at(hist: &[u64], total: u64, clip: f64) -> Option<(usize, usize)> {
    if total == 0 {
        return None;
    }
    let cut = (total as f64 * clip).floor() as u64;
    let last = hist.len() - 1;
    let mut lo = 0usize;
    let mut acc = 0u64;
    while lo < last {
        if acc + hist[lo] > cut {
            break;
        }
        acc += hist[lo];
        lo += 1;
    }
    let mut hi = last;
    let mut acc = 0u64;
    while hi > 0 {
        if acc + hist[hi] > cut {
            break;
        }
        acc += hist[hi];
        hi -= 1;
    }
    Some((lo, hi))
}

fn stretch_u16(plane: &mut [u16], lo: usize, hi: usize) {
    let scale = 65535.0 / (hi as f64 - lo as f64);
    for v in plane.iter_mut() {
        *v = ((*v as f64 - lo as f64) * scale)
            .round()
            .clamp(0.0, 65535.0) as u16;
    }
}

fn auto_per_channel_u16(data: &mut [u16], n: usize, clip: f64) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    for plane in [r, g, b] {
        let mut hist = vec![0u64; 65536];
        for &v in plane.iter() {
            hist[v as usize] += 1;
        }
        if let Some((lo, hi)) = percentile_bounds_at(&hist, n as u64, clip) {
            if lo < hi {
                stretch_u16(plane, lo, hi);
            }
        }
    }
}

fn auto_joint_u16(data: &mut [u16], n: usize, clip: f64) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    let mut hist = vec![0u64; 65536];
    for plane in [&*r, &*g, &*b] {
        for &v in plane.iter() {
            hist[v as usize] += 1;
        }
    }
    if let Some((lo, hi)) = percentile_bounds_at(&hist, (n * 3) as u64, clip) {
        if lo < hi {
            for plane in [r, g, b] {
                stretch_u16(plane, lo, hi);
            }
        }
    }
}

fn f32_bin(u: f64) -> usize {
    ((u * F32_BINS as f64) as usize).min(F32_BINS - 1)
}

fn stretch_f32(plane: &mut [f32], lo: usize, hi: usize) {
    // Bin centers keep the f32 stretch continuous inside a bin while the
    // percentile extremes land on 0 and 1.
    let center_lo = (lo as f64 + 0.5) / F32_BINS as f64;
    let denom = (hi - lo) as f64 / F32_BINS as f64;
    for x in plane.iter_mut() {
        *x = ((x.to_unit() - center_lo) / denom).clamp(0.0, 1.0) as f32;
    }
}

fn auto_per_channel_f32(data: &mut [f32], n: usize, clip: f64) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    for plane in [r, g, b] {
        let mut hist = vec![0u64; F32_BINS];
        for &x in plane.iter() {
            hist[f32_bin(x.to_unit())] += 1;
        }
        if let Some((lo, hi)) = percentile_bounds_at(&hist, n as u64, clip) {
            if lo < hi {
                stretch_f32(plane, lo, hi);
            }
        }
    }
}

fn auto_joint_f32(data: &mut [f32], n: usize, clip: f64) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    let mut hist = vec![0u64; F32_BINS];
    for plane in [&*r, &*g, &*b] {
        for &x in plane.iter() {
            hist[f32_bin(x.to_unit())] += 1;
        }
    }
    if let Some((lo, hi)) = percentile_bounds_at(&hist, (n * 3) as u64, clip) {
        if lo < hi {
            for plane in [r, g, b] {
                stretch_f32(plane, lo, hi);
            }
        }
    }
}

/// "Snap neutral midtones" in the unit domain: the same `64/255..=192/255` luma
/// window and `0.1..=9.99` gamma clamp as the 8-bit kernel.
fn snap_neutral_native<T: Sample>(data: &mut [T], n: usize) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    let mut sums = [0.0f64; 3];
    let mut count = 0u64;
    for i in 0..n {
        let (ru, gu, bu) = (r[i].to_unit(), g[i].to_unit(), b[i].to_unit());
        let y = luma(ru * 255.0, gu * 255.0, bu * 255.0);
        if (64.0..=192.0).contains(&y) {
            sums[0] += ru;
            sums[1] += gu;
            sums[2] += bu;
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
    if target <= 0.0 || target >= 1.0 {
        return;
    }
    for (c, plane) in [r, g, b].into_iter().enumerate() {
        let m = means[c];
        if m <= 0.0 || m.ln() == 0.0 {
            continue;
        }
        let gamma = (m.ln() / target.ln()).clamp(0.1, 9.99);
        for x in plane.iter_mut() {
            *x = T::from_unit(x.to_unit().powf(1.0 / gamma));
        }
    }
}

fn color_lookup_native(
    params: &ColorLookupParams,
    samples: &mut Samples,
    width: usize,
    height: usize,
    channels: u8,
) -> Result<(), AdjustError> {
    let Some(lut) = &params.lookup else {
        return Ok(());
    };
    if !(2..=64).contains(&lut.size) || lut.points.len() != lut.size * lut.size * lut.size {
        return Err(AdjustError::InvalidParams("invalid 3-D LUT size".into()));
    }
    if lut.points.iter().flatten().any(|v| !v.is_finite()) {
        return Err(AdjustError::InvalidParams(
            "non-finite LUT component".into(),
        ));
    }
    // An 8-bit store is the 3-D LUT sampler's own domain, so mirror its f32
    // arithmetic (the final rounding differs from `from_unit`'s f64 path).
    if let Samples::U8(data) = samples {
        lookup_u8(data, lut, width * height);
        return Ok(());
    }
    samples.map_color_planes(width, height, channels, |[r, g, b]| {
        let s = sample(lut, [r as f32, g as f32, b as f32]);
        [s[0] as f64, s[1] as f64, s[2] as f64]
    });
    Ok(())
}

fn lookup_u8(data: &mut [u8], lut: &Lut3d, n: usize) {
    let (r, rest) = data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let (b, _) = rest.split_at_mut(n);
    for i in 0..n {
        let s = sample(
            lut,
            [
                r[i] as f32 / 255.0,
                g[i] as f32 / 255.0,
                b[i] as f32 / 255.0,
            ],
        );
        r[i] = to_u8(s[0]);
        g[i] = to_u8(s[1]);
        b[i] = to_u8(s[2]);
    }
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}
