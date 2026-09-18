use pictura_core::PixelBuffer;

use crate::common::{
    hermite_eval, linear_to_srgb, luma, map_lut, monotone_tangents, planes_mut, srgb_to_linear,
};
use crate::types::{
    AdjustError, BrightnessContrastParams, CurvesParams, ExposureParams, LevelsParams,
};

// ---------------------------------------------------------------------------
// Tonal adjustments
// ---------------------------------------------------------------------------

pub(crate) fn levels(p: &LevelsParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
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

pub(crate) fn curves(p: &CurvesParams, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
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

pub(crate) fn brightness_contrast(
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

pub(crate) fn exposure(
    p: &ExposureParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
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

pub(crate) fn desaturate(buf: &mut PixelBuffer, n: usize) {
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

pub(crate) fn posterize(levels: u8, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
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

pub(crate) fn threshold(level: u8, buf: &mut PixelBuffer, n: usize) -> Result<(), AdjustError> {
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
