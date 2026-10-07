//! Shadows/Highlights: a local tone operator. A blurred luminance base says
//! which areas are shadow and which highlight; inside them each pixel's
//! luminance is lifted (shadows) or pulled down (highlights) along a gamma
//! curve, so the detail within a dark or bright area keeps its contrast while
//! the area as a whole opens up. Hue is kept and saturation follows the
//! correction mildly.
//!
//! ponytail: Photoshop's operator is closed. CS6's hidden "Show More Options"
//! defaults are fixed here (tonal width 50 %, radius 30 px, color correction
//! +20, midtone contrast 0); the base is three box blurs approximating a
//! Gaussian of half the radius, and the curve strengths are tuned by eye against CS6 output.

use crate::common::luma;
use crate::types::{AdjustError, ShadowsHighlightsParams};

const TONAL_WIDTH: f64 = 0.5;
const RADIUS: f64 = 30.0;
const COLOR_CORRECTION: f64 = 0.2;
/// The gamma at amount 100 % in a fully shadow (or highlight) area.
const STRENGTH: f64 = 2.0;
/// Caps how far keeping saturation may scale a nearly neutral pixel's chroma.
const MAX_CHROMA_GAIN: f64 = 3.0;
const SATURATION_KEPT: f64 = 0.5;

pub(crate) fn validate(p: &ShadowsHighlightsParams) -> Result<bool, AdjustError> {
    let (sa, ha) = (p.shadows_amount, p.highlights_amount);
    if !sa.is_finite()
        || !ha.is_finite()
        || !(0.0..=100.0).contains(&sa)
        || !(0.0..=100.0).contains(&ha)
    {
        return Err(AdjustError::InvalidParams(
            "shadows/highlights amounts must be within 0..=100".into(),
        ));
    }
    Ok(sa != 0.0 || ha != 0.0)
}

/// Rec. 601 luminance (`0.0..=1.0`) of `0.0..=1.0` RGB.
pub(crate) fn luminance(rgb: [f64; 3]) -> f64 {
    luma(rgb[0], rgb[1], rgb[2])
}

/// The blurred luminance base of a `width` × `height` plane.
pub(crate) fn base(luminance: &[f64], width: usize, height: usize) -> Vec<f64> {
    let mut plane = luminance.to_vec();
    pictura_core::blur::approx_gaussian(&mut plane, width, height, RADIUS / 2.0);
    plane
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// One pixel through the operator, given its `base` luminance.
pub(crate) fn pixel(p: &ShadowsHighlightsParams, rgb: [f64; 3], base: f64) -> [f64; 3] {
    let l = luminance(rgb);
    let shadow = smoothstep(1.0 - base / TONAL_WIDTH) * p.shadows_amount / 100.0;
    let highlight =
        smoothstep((base - (1.0 - TONAL_WIDTH)) / TONAL_WIDTH) * p.highlights_amount / 100.0;
    let mut target = l;
    if shadow > 0.0 {
        target = target.powf(1.0 / (1.0 + STRENGTH * shadow));
    }
    if highlight > 0.0 {
        target = 1.0 - (1.0 - target).powf(1.0 / (1.0 + STRENGTH * highlight));
    }
    if (target - l).abs() < 1e-9 {
        return rgb;
    }
    // Half of the saturation (chroma relative to the room the lightness
    // leaves) is kept, so moving toward the midtones deepens colour;
    // COLOR_CORRECTION adds a little more.
    let room = |v: f64| (1.0 - (2.0 * v - 1.0).abs()).max(1e-3);
    let gain = (room(target) / room(l)).min(MAX_CHROMA_GAIN);
    let mut chroma = 1.0 + (gain - 1.0) * SATURATION_KEPT + COLOR_CORRECTION * (target - l).abs();
    // Pull the chroma in where a channel would clip, so the luminance lands.
    for c in rgb {
        let d = (c - l) * chroma;
        if target + d > 1.0 {
            chroma = chroma.min((1.0 - target) / (c - l));
        } else if target + d < 0.0 {
            chroma = chroma.min(target / (l - c));
        }
    }
    rgb.map(|c| (target + (c - l) * chroma).clamp(0.0, 1.0))
}
