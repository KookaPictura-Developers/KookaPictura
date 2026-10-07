//! Image > Adjustments > Replace Color (issue #164).
//!
//! A pointwise HSL shift feathered by a colour-match weight: each pixel's
//! weight is the best Chebyshev-distance match to any sample, optionally
//! localised around where the colour was picked. The kernel mirrors
//! photorust's Replace Color. Selection confinement is applied by the
//! renderer's coverage mask, so it is not modelled here; native-depth apply
//! refuses the operation (the [`crate::native`] catch-all).

use pictura_core::PixelBuffer;

use crate::common::{hsl_to_rgb, rgb_to_hsl};
use crate::types::AdjustError;

/// One eyedropper reading: the document position it was taken at (negative
/// when it was not read off the canvas) plus its RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaceColorSample {
    pub x: i32,
    pub y: i32,
    pub rgb: [u8; 3],
}

/// Replace Color's full parameter set. `fuzziness` is `0..=200`; `hue` is
/// degrees `-180..=180`; `saturation` / `lightness` are `-100..=100`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceColorParams {
    pub samples: Vec<ReplaceColorSample>,
    pub fuzziness: f64,
    pub localized: bool,
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

/// Spatial falloff width for Localized Color Clusters, as a fraction of the
/// image diagonal, squared.
pub(crate) fn sigma_sq(w: u32, h: u32) -> f64 {
    let diag = ((w as f64) * (w as f64) + (h as f64) * (h as f64))
        .sqrt()
        .max(1.0);
    let sigma = diag * 0.18;
    sigma * sigma
}

/// Selection weight for one pixel: the best match over all samples, so adding
/// a sample widens the selection rather than averaging it away.
#[allow(clippy::too_many_arguments)]
pub(crate) fn match_weight(
    r: u8,
    g: u8,
    b: u8,
    x: i32,
    y: i32,
    samples: &[ReplaceColorSample],
    fuzziness: f64,
    localized: bool,
    sigma: f64,
) -> f64 {
    let mut best = 0.0f64;
    for s in samples {
        let d = (r as i32 - s.rgb[0] as i32)
            .abs()
            .max((g as i32 - s.rgb[1] as i32).abs())
            .max((b as i32 - s.rgb[2] as i32).abs()) as f64;
        let mut w = if fuzziness <= 0.0 {
            if d == 0.0 {
                1.0
            } else {
                0.0
            }
        } else {
            (1.0 - d / fuzziness).clamp(0.0, 1.0)
        };
        // A sample with no canvas position (the foreground colour the dialog
        // opens on) has no cluster to stay near.
        if localized && w > 0.0 && s.x >= 0 && s.y >= 0 {
            let dx = (x - s.x) as f64;
            let dy = (y - s.y) as f64;
            w *= (-(dx * dx + dy * dy) / (2.0 * sigma)).exp();
        }
        best = best.max(w);
    }
    best
}

fn check_params(p: &ReplaceColorParams) -> Result<(), AdjustError> {
    if !p.fuzziness.is_finite() || !(0.0..=200.0).contains(&p.fuzziness) {
        return Err(AdjustError::InvalidParams(
            "fuzziness must be 0..=200".into(),
        ));
    }
    if !p.hue.is_finite() || !(-180.0..=180.0).contains(&p.hue) {
        return Err(AdjustError::InvalidParams("hue must be -180..=180".into()));
    }
    if !p.saturation.is_finite() || !(-100.0..=100.0).contains(&p.saturation) {
        return Err(AdjustError::InvalidParams(
            "saturation must be -100..=100".into(),
        ));
    }
    if !p.lightness.is_finite() || !(-100.0..=100.0).contains(&p.lightness) {
        return Err(AdjustError::InvalidParams(
            "lightness must be -100..=100".into(),
        ));
    }
    Ok(())
}

/// Apply Replace Color in place to a planar buffer (alpha untouched). `n` is
/// the pixel count from [`crate::common::validate`]. Empty samples is a no-op.
pub(crate) fn replace_color(
    p: &ReplaceColorParams,
    buf: &mut PixelBuffer,
    n: usize,
) -> Result<(), AdjustError> {
    check_params(p)?;
    if p.samples.is_empty() {
        return Ok(());
    }
    let sigma = sigma_sq(buf.width, buf.height);
    let sat = p.saturation / 100.0;
    let light = p.lightness / 100.0;
    let width = buf.width as usize;
    for i in 0..n {
        let x = (i % width) as i32;
        let y = (i / width) as i32;
        let (r, g, b) = (buf.data[i], buf.data[n + i], buf.data[2 * n + i]);
        let weight = match_weight(r, g, b, x, y, &p.samples, p.fuzziness, p.localized, sigma);
        if weight <= 0.0 {
            continue;
        }
        let (hh, ss, ll) = rgb_to_hsl(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
        let nh = (hh + p.hue).rem_euclid(360.0);
        let ns = if sat >= 0.0 {
            ss + (1.0 - ss) * sat
        } else {
            ss * (1.0 + sat)
        };
        let nl = if light >= 0.0 {
            ll + (1.0 - ll) * light
        } else {
            ll * (1.0 + light)
        };
        let (nr, ng, nb) = hsl_to_rgb(nh, ns.clamp(0.0, 1.0), nl.clamp(0.0, 1.0));
        let blend = |orig: u8, new: f64| -> u8 {
            let o = orig as f64 / 255.0;
            ((o + (new - o) * weight).clamp(0.0, 1.0) * 255.0 + 0.5) as u8
        };
        buf.data[i] = blend(r, nr);
        buf.data[n + i] = blend(g, ng);
        buf.data[2 * n + i] = blend(b, nb);
    }
    Ok(())
}

/// The Replace Color selection mask, fitted into a `size` box for the dialog's
/// preview. White is fully selected, black not. A one-channel buffer; empty
/// for degenerate input.
pub fn replace_color_mask(buf: &PixelBuffer, p: &ReplaceColorParams, size: usize) -> PixelBuffer {
    let (sw, sh) = (buf.width as usize, buf.height as usize);
    if sw == 0 || sh == 0 || size == 0 {
        return PixelBuffer {
            width: 0,
            height: 0,
            channels: 1,
            data: Vec::new().into(),
        };
    }
    let scale = (size as f64 / sw as f64).min(size as f64 / sh as f64);
    let tw = ((sw as f64 * scale).round() as usize).max(1);
    let th = ((sh as f64 * scale).round() as usize).max(1);
    let sigma = sigma_sq(buf.width, buf.height);
    let plane = sw * sh;
    let mut out = vec![0u8; tw * th];
    for y in 0..th {
        let sy = ((y as f64 / scale) as usize).min(sh - 1);
        for x in 0..tw {
            let sx = ((x as f64 / scale) as usize).min(sw - 1);
            let at = sy * sw + sx;
            let transparent = buf.channels == 4 && buf.data[3 * plane + at] == 0;
            let w = if transparent {
                0.0
            } else {
                match_weight(
                    buf.data[at],
                    buf.data[plane + at],
                    buf.data[2 * plane + at],
                    sx as i32,
                    sy as i32,
                    &p.samples,
                    p.fuzziness,
                    p.localized,
                    sigma,
                )
            };
            out[y * tw + x] = (w * 255.0 + 0.5) as u8;
        }
    }
    PixelBuffer {
        width: tw as u32,
        height: th as u32,
        channels: 1,
        data: out.into(),
    }
}
