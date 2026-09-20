//! Object-based layer effects: the `lfx2` Drop Shadow, Outer Glow, Inner
//! Shadow and Inner Glow.
//!
//! A layer's effects live in the `lfx2` additional-layer-information block as a
//! `DescriptorBlock2`: the top-level `DrSh` object holds the drop shadow, the
//! `OrGl` object holds the outer glow, the `IrSh` object holds the inner shadow,
//! and the `IrGl` object holds the inner glow. This slice decodes and renders
//! all four.
//!
//! ponytail: the effective angle is the stored `lagl`, not the document
//! global-light resource (1037), which this crate does not decode; `uglg` is
//! carried for a later slice. Contour (`TrnS`), noise (`Nose`) and anti-alias
//! (`AntA`) are not applied. A shadow always composites behind the layer
//! content, so a semi-transparent layer shows it through (the `knocks_out =
//! false` look). The matte is a full-canvas `f32` buffer, matching the
//! compositor's ceiling.
//!
//! A glow has no offset: `GlwT` `PrBL` (Precise) is decoded but rendered as
//! `SfBL` (Softer), `Range` (`Inpr`), contour, noise, jitter (`ShdN`),
//! anti-alias and gradient mode (`Grad`) are ignored, the spread is a max-filter
//! dilate of radius `round(spread / 100 * size)` (not Photoshop's spread-then-
//! blur split), and the exterior is the multiplicative `1 - matte` rather than
//! Photoshop's exact knock-out equation.
//!
//! An inner shadow composites **above** the layer content (interior-only): it is
//! the offset, choke-eroded and blurred inverted matte multiplied by the content
//! matte. ponytail: `knocks_out` (`layerConceals`) is decoded but inert — the
//! confinement is unconditional; the interior effect blends against the
//! layer-over-backdrop result rather than an isolated content buffer, so
//! `Blend Interior Effects As Group` is not modelled; contour (`TrnS`), noise
//! (`Nose`) and anti-alias (`AntA`) are ignored; and the effective angle is the
//! stored `lagl`, not the global-light resource.
//!
//! An inner glow also composites **above** the content: the choke-eroded content
//! matte is blurred and turned into an interior field. ponytail: `Source =
//! Center` lights the interior far from any edge (the libpsd complement-of-edge
//! approximation), not a bounded radius-`size` blob measured from the centroid;
//! `choke` maps to a min-filter erode of radius `round(choke / 100 · size)`
//! rather than libpsd's blur-then-edge-find (the documented 50 % piecewise
//! behavior is not modelled); `Precise` renders as `Softer`; contour (`TrnS`),
//! noise (`Nose`), the `Range` (`Inpr`) remap and anti-alias (`AntA`) are
//! ignored; and the interior effect blends against the layer-over-backdrop
//! result rather than an isolated content buffer.
//!
//! A stroke (`FrFX`) also composites **above** the content: an exact
//! integer-width band at the content edge, outside, inside or straddling it.
//! ponytail: only the solid-colour fill (`PntT` `FrFl`/`SClr`) is decoded and
//! rendered — a gradient or pattern fill is ignored; contour (`TrnS`),
//! anti-alias (`AntA`) and `overprint` are ignored; the colour defaults to
//! black rather than the docs' "foreground `(inferred)`"; and Photoshop's exact
//! inter-effect order among the above-content effects is not modelled.

use pictura_adjust::Adjustment;
use pictura_codec::DescValue;
use pictura_core::{Document, Layer, PixelBuffer};

use crate::composite::{channel, Canvas};

mod glows;
mod shadows;
mod strokes;

pub use glows::{decode_inner_glow, decode_outer_glow, GlowSource, InnerGlow, OuterGlow};
pub use shadows::{decode_drop_shadow, decode_inner_shadow, DropShadow, InnerShadow};
pub use strokes::{decode_stroke, Stroke, StrokePosition};

/// Documented Drop Shadow parameter caps (`docs/05-layers/layer-styles.md`).
const MAX_OPACITY: f32 = 100.0;
const MAX_DISTANCE: f32 = 30_000.0;
const MAX_SPREAD: f32 = 100.0;
const MAX_CHOKE: f32 = 100.0;
const MAX_SIZE: f32 = 250.0;

/// The blur technique stored in `GlwT` (typeID `BETE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowTechnique {
    /// `SfBL`: the ordinary Gaussian blur.
    Softer,
    /// `PrBL`: Photoshop's distance-measure technique, rendered as `Softer`
    /// (a stated ceiling).
    Precise,
}

/// Read a numeric key as `f32`. A value that is non-finite as an `f64`, or
/// finite as an `f64` but overflows to infinity as an `f32`, is rejected.
fn num_or(obj: &DescValue, key: &[u8], default: f32) -> Option<f32> {
    match crate::composite::desc_item(obj, key) {
        None => Some(default),
        Some(DescValue::UnitFloat { value, .. }) if value.is_finite() => finite_f32(*value),
        Some(DescValue::Double(v)) if v.is_finite() => finite_f32(*v),
        Some(_) => None,
    }
}

fn finite_f32(value: f64) -> Option<f32> {
    let v = value as f32;
    v.is_finite().then_some(v)
}

/// Read a numeric key and clamp it to its documented range. Out-of-range but
/// finite values are clamped; non-finite values reject the whole effect.
fn num_clamped(obj: &DescValue, key: &[u8], default: f32, min: f32, max: f32) -> Option<f32> {
    num_or(obj, key, default).map(|v| v.clamp(min, max))
}

fn bool_or(obj: &DescValue, key: &[u8], default: bool) -> Option<bool> {
    match crate::composite::desc_item(obj, key) {
        None => Some(default),
        Some(DescValue::Bool(b)) => Some(*b),
        Some(_) => None,
    }
}

fn decode_color(value: &DescValue) -> Option<[u8; 3]> {
    let DescValue::Object { class_id, .. } = value else {
        return None;
    };
    if class_id.as_slice() != b"RGBC" {
        return None;
    }
    let component = |key: &[u8]| match crate::composite::desc_item(value, key) {
        Some(DescValue::Double(v)) if v.is_finite() => Some(v.round().clamp(0.0, 255.0) as u8),
        Some(DescValue::UnitFloat { value, .. }) if value.is_finite() => {
            Some(value.round().clamp(0.0, 255.0) as u8)
        }
        _ => None,
    };
    Some([
        component(b"Rd  ")?,
        component(b"Grn ")?,
        component(b"Bl  ")?,
    ])
}

/// Composite a layer's enabled, present drop shadow into the running canvas
/// before the layer's own content. Groups and destructive adjustment layers are
/// skipped.
///
/// ponytail: an effect on a group or on a destructive adjustment layer is
/// deferred; fill content still contributes.
pub(crate) fn composite_layer_effects(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    if layer.is_group || is_destructive_adjustment(layer) {
        return;
    }
    if let Some(shadow) = decode_drop_shadow(layer) {
        if shadow.enabled && shadow.present {
            shadows::composite_drop_shadow(canvas, layer, doc, &shadow);
        }
    }
    if let Some(glow) = decode_outer_glow(layer) {
        if glow.enabled && glow.present {
            glows::composite_outer_glow(canvas, layer, doc, &glow);
        }
    }
}

/// Composite a layer's enabled, present inner shadow and inner glow into the
/// running canvas **after** the layer's own content. Groups and destructive
/// adjustment layers are skipped, matching the below-content pass. Inner Shadow
/// is composited before Inner Glow when both are present, and a Stroke is drawn
/// after both.
pub(crate) fn composite_layer_effects_above(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    if layer.is_group || is_destructive_adjustment(layer) {
        return;
    }
    if let Some(shadow) = decode_inner_shadow(layer) {
        if shadow.enabled && shadow.present {
            shadows::composite_inner_shadow(canvas, layer, doc, &shadow);
        }
    }
    if let Some(glow) = decode_inner_glow(layer) {
        if glow.enabled && glow.present {
            glows::composite_inner_glow(canvas, layer, doc, &glow);
        }
    }
    if let Some(stroke) = decode_stroke(layer) {
        if stroke.enabled && stroke.present {
            strokes::composite_stroke(canvas, layer, doc, &stroke);
        }
    }
}

/// A layer whose content is a destructive adjustment (not fill content) has no
/// matte; its effect is deferred.
fn is_destructive_adjustment(layer: &Layer) -> bool {
    match layer.adjustment.as_ref().and_then(crate::decode_adjustment) {
        Some(
            Adjustment::SolidFill(_) | Adjustment::GradientFill(_) | Adjustment::PatternFill(_),
        ) => false,
        Some(_) => true,
        None => false,
    }
}

/// A non-negative, finite value no greater than `max` (`0.0` when non-finite).
fn clamp_finite(value: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, max)
    } else {
        0.0
    }
}

/// The layer's content alpha over the canvas region `(x0, y0, x1, y1)`: a pixel
/// layer's `-1` alpha, a fill's payload alpha, or `1.0` inside the rect of an
/// embedded smart source.
fn content_matte(layer: &Layer, doc: &Document, region: (i32, i32, i32, i32)) -> Vec<f32> {
    let (x0, y0, x1, y1) = region;
    let rw = (x1 - x0) as usize;
    let rh = (y1 - y0) as usize;
    let mut matte = vec![0.0f32; rw * rh];
    if crate::fill::fill_coverage_matte(layer, doc, region, &mut matte) {
        return matte;
    }
    // Pixel/smart content exists only inside the layer rect, intersected here.
    let ix0 = layer.rect.left.max(x0);
    let iy0 = layer.rect.top.max(y0);
    let ix1 = layer.rect.right.min(x1);
    let iy1 = layer.rect.bottom.min(y1);
    if ix1 <= ix0 || iy1 <= iy0 {
        return matte;
    }
    let at = |x: i32, y: i32| (y - y0) as usize * rw + (x - x0) as usize;
    match channel(layer, -1) {
        Some(alpha) => {
            let lw = layer.rect.width();
            if lw <= 0 {
                return matte;
            }
            for y in iy0..iy1 {
                for x in ix0..ix1 {
                    let li = (y - layer.rect.top) as usize * lw as usize
                        + (x - layer.rect.left) as usize;
                    // A missing alpha sample is opaque, matching the content path.
                    matte[at(x, y)] = alpha.get(li).copied().unwrap_or(255) as f32 / 255.0;
                }
            }
        }
        None => {
            // No `-1` channel: the content path treats the layer as opaque, and so
            // does the matte (a channel-less smart source is covered here too).
            for y in iy0..iy1 {
                for x in ix0..ix1 {
                    matte[at(x, y)] = 1.0;
                }
            }
        }
    }
    matte
}

/// The layer rect clamped to the canvas as `(x0, y0, x1, y1)`; empty when the
/// rect does not intersect the canvas.
fn clip_rect(layer: &Layer, w: i32, h: i32) -> (i32, i32, i32, i32) {
    (
        layer.rect.left.max(0),
        layer.rect.top.max(0),
        layer.rect.right.min(w),
        layer.rect.bottom.min(h),
    )
}

fn rect_empty(r: (i32, i32, i32, i32)) -> bool {
    r.2 <= r.0 || r.3 <= r.1
}

/// Pad a rect by `pad` on every side and clamp it to the canvas.
fn pad_rect(r: (i32, i32, i32, i32), pad: i64, w: i32, h: i32) -> (i32, i32, i32, i32) {
    let pad = pad as i32;
    (
        (r.0 - pad).max(0),
        (r.1 - pad).max(0),
        (r.2 + pad).min(w),
        (r.3 + pad).min(h),
    )
}

/// Grayscale dilation (max filter) of radius `r`, separable over the box
/// window. Spread expands the matte before the blur, up to a hard edge at 100 %.
fn dilate_matte(src: &[f32], w: usize, h: usize, r: i64) -> Vec<f32> {
    let horizontal = axis_max(src, w, h, r, true);
    axis_max(&horizontal, w, h, r, false)
}

fn axis_max(src: &[f32], w: usize, h: usize, r: i64, horizontal: bool) -> Vec<f32> {
    let mut out = vec![0.0f32; src.len()];
    if horizontal {
        for y in 0..h {
            for x in 0..w as i64 {
                let lo = (x - r).max(0) as usize;
                let hi = (x + r).min(w as i64 - 1) as usize;
                let mut v = f32::NEG_INFINITY;
                for k in lo..=hi {
                    v = v.max(src[y * w + k]);
                }
                out[y * w + x as usize] = v;
            }
        }
    } else {
        for x in 0..w {
            for y in 0..h as i64 {
                let lo = (y - r).max(0) as usize;
                let hi = (y + r).min(h as i64 - 1) as usize;
                let mut v = f32::NEG_INFINITY;
                for k in lo..=hi {
                    v = v.max(src[k * w + x]);
                }
                out[y as usize * w + x] = v;
            }
        }
    }
    out
}

/// Grayscale erosion (min filter) of radius `r`, separable over the box window.
/// Choke shrinks the inverted matte before the blur, narrowing the shadow.
fn erode_matte(src: &[f32], w: usize, h: usize, r: i64) -> Vec<f32> {
    let horizontal = axis_min(src, w, h, r, true);
    axis_min(&horizontal, w, h, r, false)
}

fn axis_min(src: &[f32], w: usize, h: usize, r: i64, horizontal: bool) -> Vec<f32> {
    let mut out = vec![0.0f32; src.len()];
    if horizontal {
        for y in 0..h {
            for x in 0..w as i64 {
                let lo = (x - r).max(0) as usize;
                let hi = (x + r).min(w as i64 - 1) as usize;
                let mut v = f32::INFINITY;
                for k in lo..=hi {
                    v = v.min(src[y * w + k]);
                }
                out[y * w + x as usize] = v;
            }
        }
    } else {
        for x in 0..w {
            for y in 0..h as i64 {
                let lo = (y - r).max(0) as usize;
                let hi = (y + r).min(h as i64 - 1) as usize;
                let mut v = f32::INFINITY;
                for k in lo..=hi {
                    v = v.min(src[k * w + x]);
                }
                out[y as usize * w + x] = v;
            }
        }
    }
    out
}

/// Gaussian-blur the matte with the crate blur, carrying the quantized matte in
/// all three planes of a `PixelBuffer`. `size <= 0` is a no-op.
fn blur_matte(src: &[f32], w: usize, h: usize, size: f64) -> Vec<f32> {
    if !size.is_finite() || size <= 0.0 {
        return src.to_vec();
    }
    let n = w * h;
    let mut buf = PixelBuffer::new(w as u32, h as u32, 3);
    for (i, &v) in src.iter().enumerate() {
        let q = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        buf.data[i] = q;
        buf.data[n + i] = q;
        buf.data[2 * n + i] = q;
    }
    if pictura_filters::blur::gaussian(&mut buf, size).is_err() {
        return src.to_vec();
    }
    buf.data[..n].iter().map(|&b| b as f32 / 255.0).collect()
}
