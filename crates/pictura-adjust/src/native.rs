//! Native-depth tonal adjustments.
//!
//! [`apply_native`] applies the tonal map family to a [`Samples`] store
//! (`u8`/`u16`/`f32`) without quantizing to 8-bit first. The per-sample math
//! mirrors the 8-bit kernels in `tonal.rs` in the unit domain; the 8-bit
//! `apply` path is untouched and stays the parity surface. Any adjustment
//! outside the covered set is refused with [`AdjustError::Unsupported`] and the
//! store is left unchanged.

use pictura_core::Samples;

use crate::common::{hermite_eval, linear_to_srgb, luma, monotone_tangents, srgb_to_linear};
use crate::tonal::sample_gradient;
use crate::types::{
    AdjustError, Adjustment, BrightnessContrastParams, CurvesParams, ExposureParams,
    GradientMapParams, LevelsParams,
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
