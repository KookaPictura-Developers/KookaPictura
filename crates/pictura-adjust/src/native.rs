//! Native-depth adjustments.
//!
//! [`apply_native`] applies the covered adjustment set to a [`Samples`] store
//! (`u8`/`u16`/`f32`) without quantizing to 8-bit first. The per-sample math
//! mirrors the 8-bit kernels in `tonal.rs` and `color.rs` in the unit domain;
//! the 8-bit `apply` path is untouched and stays the parity surface. Any
//! adjustment outside the covered set is refused with [`AdjustError::Unsupported`]
//! and the store is left unchanged.

use pictura_core::Samples;

use crate::color::skin_bump;
use crate::common::{
    hermite_eval, hsl_to_rgb, linear_to_srgb, luma, monotone_tangents, rgb_to_hsl, srgb_to_linear,
};
use crate::tonal::sample_gradient;
use crate::types::{
    AdjustError, Adjustment, BlackWhiteParams, BrightnessContrastParams, ChannelMixerParams,
    ColorBalanceParams, CurvesParams, ExposureParams, GradientMapParams, HueSaturationParams,
    LevelsParams, PhotoFilterParams, SelectiveColorMethod, SelectiveColorParams, SelectiveRange,
    VibranceParams,
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
            let f = levels_map(p)?;
            map(samples, width, height, channels, |p| {
                [f(p[0]), f(p[1]), f(p[2])]
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
        Adjustment::ColorBalance(p) => color_balance_native(p, samples, width, height, channels),
        Adjustment::BlackWhite(p) => black_white_native(p, samples, width, height, channels),
        Adjustment::PhotoFilter(p) => photo_filter_native(p, samples, width, height, channels),
        Adjustment::ChannelMixer(p) => channel_mixer_native(p, samples, width, height, channels),
        Adjustment::SelectiveColor(p) => {
            selective_color_native(p, samples, width, height, channels)
        }
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

fn levels_map(p: &LevelsParams) -> Result<impl Fn(f64) -> f64, AdjustError> {
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
    let inv_gamma = 1.0 / p.gamma;
    Ok(move |u: f64| {
        let x = u * 255.0;
        let t = ((x - ib) / (iw - ib)).clamp(0.0, 1.0);
        let t = t.powf(inv_gamma);
        (ob + t * (ow - ob)) / 255.0
    })
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
    if p.stops.len() < 2 {
        return Err(AdjustError::InvalidParams(
            "gradient map needs at least 2 stops".into(),
        ));
    }
    if p.stops.iter().any(|s| s.location > 4096) {
        return Err(AdjustError::InvalidParams(
            "gradient stop location must be <= 4096".into(),
        ));
    }
    if p.stops.windows(2).any(|w| w[0].location >= w[1].location) {
        return Err(AdjustError::InvalidParams(
            "gradient stop locations must be strictly increasing".into(),
        ));
    }
    // ponytail: luma is quantized to the 8-bit gradient LUT exactly as the
    // 8-bit kernel does, so the outputs are a fixed set of u8 colors; a native
    // luma sample would need the gradient interpolation to move off 8-bit too.
    let mut lut = [[0u8; 3]; 256];
    for (i, slot) in lut.iter_mut().enumerate() {
        let l = i as f64 / 255.0;
        let l = if p.reverse { 1.0 - l } else { l };
        *slot = sample_gradient(&p.stops, (l * 4096.0).round());
    }
    map(samples, width, height, channels, move |p| {
        let y = luma(p[0] * 255.0, p[1] * 255.0, p[2] * 255.0);
        let c = lut[y.round().clamp(0.0, 255.0) as usize];
        [
            c[0] as f64 / 255.0,
            c[1] as f64 / 255.0,
            c[2] as f64 / 255.0,
        ]
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
    let hue = p.hue as f64;
    map(samples, width, height, channels, move |[r, g, b]| {
        let (h, s, l) = rgb_to_hsl(r, g, b);
        let h = (h + hue).rem_euclid(360.0);
        let s = (s * (1.0 + ds)).clamp(0.0, 1.0);
        let l = if dl >= 0.0 {
            l + dl * (1.0 - l)
        } else {
            l + dl * l
        }
        .clamp(0.0, 1.0);
        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        [nr, ng, nb]
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
    if p.ranges
        .iter()
        .flat_map(|r| [r.c, r.m, r.y, r.k])
        .any(|v| !(-100..=100).contains(&v))
    {
        return Err(AdjustError::InvalidParams(
            "selective color corrections must be -100..=100".into(),
        ));
    }
    if p.ranges.iter().all(|r| *r == SelectiveRange::default()) {
        return Ok(());
    }
    // ponytail: libpsd's Selective Color is an integer 0–255 -> CMYK -> RGB
    // pipeline, so this kernel rounds unit to 8-bit and gains nothing at native
    // depth; a float CMYK port would be the upgrade.
    map(samples, width, height, channels, |[r, g, b]| {
        let (nr, ng, nb) = selective_color_pixel(
            p,
            (r * 255.0).round() as i32,
            (g * 255.0).round() as i32,
            (b * 255.0).round() as i32,
        );
        [nr as f64 / 255.0, ng as f64 / 255.0, nb as f64 / 255.0]
    });
    Ok(())
}

fn selective_color_pixel(p: &SelectiveColorParams, r: i32, g: i32, b: i32) -> (u8, u8, u8) {
    let (sc, sm, sy, sk) = rgb_to_intcmyk(r, g, b);
    let src = [sc, sm, sy, sk];
    let mut dst = [sc, sm, sy, sk];
    let hue = rgb_to_int_hue(r, g, b);

    for (index, range) in p.ranges.iter().take(6).enumerate() {
        let i = index as i32 + 1;
        let r0 = -105 + i * 60;
        let (r1, r2, r3) = (r0 + 30, r0 + 60, r0 + 90);
        if hue >= r0 && hue < r3 {
            let opacity = if hue < r1 {
                (hue - r0) * 255 / 30
            } else if hue < r2 {
                255
            } else {
                (r3 - hue) * 255 / 30
            };
            add_correction(&p.method, range, opacity, src, &mut dst, 25500);
        }
    }

    for (index, range) in p.ranges.iter().enumerate().skip(6) {
        let selected = match index {
            6 => sk == 0,
            7 => sk > 0 && sk < 255,
            _ => sk == 255,
        };
        if selected {
            add_correction(&p.method, range, 1, src, &mut dst, 100);
        }
    }

    for ink in &mut dst {
        *ink = (*ink).clamp(0, 255);
    }
    intcmyk_to_rgb(dst[0], dst[1], dst[2], dst[3])
}

fn add_correction(
    method: &SelectiveColorMethod,
    range: &SelectiveRange,
    weight: i32,
    src: [i32; 4],
    dst: &mut [i32; 4],
    divisor: i32,
) {
    for (i, (ink, correction)) in dst
        .iter_mut()
        .zip([range.c, range.m, range.y, range.k])
        .enumerate()
    {
        let correction = correction as i32;
        if correction == 0 {
            continue;
        }
        let amount = match method {
            SelectiveColorMethod::Relative => src[i],
            SelectiveColorMethod::Absolute => 255,
        };
        *ink += amount * correction * weight / divisor;
    }
}

fn rgb_to_intcmyk(r: i32, g: i32, b: i32) -> (i32, i32, i32, i32) {
    let (dc, dm, dy) = (255 - r, 255 - g, 255 - b);
    let k = dc.min(dm).min(dy);
    if k < 255 {
        let d = 255 - k;
        (
            (dc - k) * 255 / d,
            (dm - k) * 255 / d,
            (dy - k) * 255 / d,
            k,
        )
    } else {
        (0, 0, 0, k)
    }
}

fn rgb_to_int_hue(r: i32, g: i32, b: i32) -> i32 {
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    if cmax == cmin {
        return 0;
    }
    let d = cmax - cmin;
    let h = if r == cmax {
        (g - b) * 60 / d
    } else if g == cmax {
        120 + (b - r) * 60 / d
    } else {
        240 + (r - g) * 60 / d
    };
    (h + 360) % 360
}

fn intcmyk_to_rgb(c: i32, m: i32, y: i32, k: i32) -> (u8, u8, u8) {
    let channel = |ink: i32| ((65535 - (ink * (255 - k) + (k << 8))) >> 8) as u8;
    (channel(c), channel(m), channel(y))
}
