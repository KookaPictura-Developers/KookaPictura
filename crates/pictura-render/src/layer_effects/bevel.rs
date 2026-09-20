//! The object-based Bevel & Emboss layer effect (`ebbl`).
//!
//! Only the `Inner` style with the `Smooth` technique renders: the masked
//! content matte `M` is blurred by `size` into a height field, a surface normal
//! is derived from its central-difference gradient scaled by `size · depth/100`,
//! and the Lambertian deviation of that normal from flat (`Angle`/`Altitude`)
//! tints the interior highlight (positive) and shadow (negative), softened by
//! `soften`, above the content.
//!
//! ponytail: only `Inner` + `Smooth` renders — the chisel techniques, the
//! `Outer`/`Emboss`/`Pillow`/`Stroke` styles, edge contour (`MpgS`), gloss
//! contour (`TrnS`), contour range (`Inpr`), anti-alias (`AntA`,
//! `antialiasGloss`), texture (`useTexture`, `InvT`, `Algn`, `Scl `, `Ptrn`),
//! `useShape` and `showInDialog` are decoded/ignored; the effective
//! angle/altitude is the stored `lagl`/`Lald`, not the global-light resource
//! (1037); the height profile is a Gaussian blur of `M` rather than Adobe's
//! distance transform; `scale = size · depth/100` and the `dot(N,L) - sin(alt)`
//! flat-offset are ungrounded model choices; the build region pads by the blur
//! supports only; `Scale Effects`, the exact inter-effect order and the
//! highlight/shadow order are not modelled.

use pictura_codec::DescValue;
use pictura_core::{BlendMode, Document, Layer};

use crate::composite::{blend_parts, desc_item, mask_alpha, Canvas};

use super::{
    blur_matte, bool_or, clamp_finite, clip_rect, content_matte, decode_color, effect_blend_mode,
    num_clamped, num_or, pad_rect, rect_empty, MAX_ALTITUDE, MAX_DEPTH, MAX_OPACITY, MAX_SIZE,
};

/// Bevel style (`bvlS`, typeID `BESl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelStyle {
    Inner,
    Outer,
    Emboss,
    Pillow,
    Stroke,
}

/// Bevel technique (`bvlT`, typeID `bvlT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelTechnique {
    Smooth,
    ChiselHard,
    ChiselSoft,
}

/// Bevel direction (`bvlD`, typeID `BESs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevelDirection {
    Up,
    Down,
}

/// The highlight half of a bevel (`hglM`/`hglC`/`hglO`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BevelHighlight {
    pub mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
}

/// The shadow half of a bevel (`sdwM`/`sdwC`/`sdwO`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BevelShadow {
    pub mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
}

/// The typed bevel & emboss decoded from a layer's `lfx2` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BevelEmboss {
    pub enabled: bool,
    pub present: bool,
    pub style: BevelStyle,
    pub technique: BevelTechnique,
    pub direction: BevelDirection,
    /// Percent, `0..=1000` (`srgR`).
    pub depth: f32,
    /// Pixels, Gaussian radius, `0..=250` (`blur`).
    pub size: f32,
    /// Pixels, `0..=250` (`Sftn`).
    pub soften: f32,
    /// The stored local angle in degrees (`lagl`), unclamped.
    pub angle_deg: f32,
    /// Degrees, `0..=90` (`Lald`).
    pub altitude_deg: f32,
    pub use_global_angle: bool,
    pub highlight: BevelHighlight,
    pub shadow: BevelShadow,
}

/// Read an enum key as its value bytes: absent takes `default`, a wrong typeID
/// rejects. An unknown value is the caller's to map (to its default).
fn enum_bytes(obj: &DescValue, key: &[u8], kind: &[u8], default: &[u8]) -> Option<Vec<u8>> {
    match desc_item(obj, key) {
        None => Some(default.to_vec()),
        Some(DescValue::Enum { kind: k, value }) if k.as_slice() == kind => Some(value.clone()),
        Some(_) => None,
    }
}

/// Decode a layer's `lfx2` Bevel & Emboss (`ebbl`).
///
/// A missing `lfx2`, a missing or wrongly-typed `ebbl`, an unknown data
/// version, a wrong-typed or non-finite value, a wrong enum typeID, a
/// non-`RGBC` colour, or a parse error is `None`. Never panics.
pub fn decode_bevel_emboss(layer: &Layer) -> Option<BevelEmboss> {
    let data = &layer.extra_block(b"lfx2")?.data;
    if data.len() < 8 {
        return None;
    }
    let body = data.get(4..)?;
    let obj = pictura_codec::read_descriptor(body).ok()?;
    let ebbl = desc_item(&obj, b"ebbl")?;
    let DescValue::Object { class_id, .. } = ebbl else {
        return None;
    };
    if class_id.as_slice() != b"ebbl" {
        return None;
    }
    let style = match enum_bytes(ebbl, b"bvlS", b"BESl", b"InrB")?.as_slice() {
        b"OtrB" => BevelStyle::Outer,
        b"Embs" => BevelStyle::Emboss,
        b"PlEb" => BevelStyle::Pillow,
        b"strokeEmboss" => BevelStyle::Stroke,
        _ => BevelStyle::Inner,
    };
    let technique = match enum_bytes(ebbl, b"bvlT", b"bvlT", b"SfBL")?.as_slice() {
        b"PrBL" => BevelTechnique::ChiselHard,
        b"Slmt" => BevelTechnique::ChiselSoft,
        _ => BevelTechnique::Smooth,
    };
    let direction = match enum_bytes(ebbl, b"bvlD", b"BESs", b"In  ")?.as_slice() {
        b"Out " => BevelDirection::Down,
        _ => BevelDirection::Up,
    };
    let blend = |key: &[u8], default: BlendMode| -> Option<BlendMode> {
        Some(effect_blend_mode(
            &enum_bytes(ebbl, key, b"BlnM", b"")?,
            default,
        ))
    };
    let color = |key: &[u8], default: [u8; 3]| -> Option<[u8; 3]> {
        match desc_item(ebbl, key) {
            None => Some(default),
            Some(value) => decode_color(value),
        }
    };
    Some(BevelEmboss {
        enabled: bool_or(ebbl, b"enab", false)?,
        present: bool_or(ebbl, b"present", false)?,
        style,
        technique,
        direction,
        depth: num_clamped(ebbl, b"srgR", 100.0, 0.0, MAX_DEPTH)?,
        size: num_clamped(ebbl, b"blur", 5.0, 0.0, MAX_SIZE)?,
        soften: num_clamped(ebbl, b"Sftn", 0.0, 0.0, MAX_SIZE)?,
        angle_deg: num_or(ebbl, b"lagl", 120.0)?,
        altitude_deg: num_clamped(ebbl, b"Lald", 30.0, 0.0, MAX_ALTITUDE)?,
        use_global_angle: bool_or(ebbl, b"uglg", true)?,
        highlight: BevelHighlight {
            mode: blend(b"hglM", BlendMode::Screen)?,
            color: color(b"hglC", [255, 255, 255])?,
            opacity: num_clamped(ebbl, b"hglO", 75.0, 0.0, MAX_OPACITY)?,
        },
        shadow: BevelShadow {
            mode: blend(b"sdwM", BlendMode::Multiply)?,
            color: color(b"sdwC", [0, 0, 0])?,
            opacity: num_clamped(ebbl, b"sdwO", 75.0, 0.0, MAX_OPACITY)?,
        },
    })
}

/// The blur reach of a Gaussian radius: `ceil(3·sigma)`.
fn blur_support(size: f64) -> i64 {
    if size > 0.0 {
        (3.0 * pictura_filters::kernel::sigma_from_radius(size)).ceil() as i64
    } else {
        0
    }
}

/// Build the masked content matte `M` over the padded source region, form the
/// height field `blur(M, size)`, light its central-difference normal from
/// `angle`/`altitude`, soften the signed shading, and composite the positive
/// (highlight) and negative (shadow) parts above the content over `source` only.
///
/// ponytail: a canvas-filling layer with the maximum `size`/`soften` still costs
/// `O(canvas · size)` because the blur is a naive separable kernel; the common
/// small-layer and crafted off-canvas cases are bounded.
pub(super) fn composite_bevel_emboss(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
    bevel: &BevelEmboss,
) {
    if bevel.style != BevelStyle::Inner || bevel.technique != BevelTechnique::Smooth {
        return;
    }
    let (w, h) = (canvas.w as i32, canvas.h as i32);
    if w == 0 || h == 0 {
        return;
    }
    let source = clip_rect(layer, w, h);
    if rect_empty(source) {
        return;
    }
    // Clamp again: a hand-built `BevelEmboss` may not have gone through decode.
    let size = clamp_finite(bevel.size, MAX_SIZE) as f64;
    let soften = clamp_finite(bevel.soften, MAX_SIZE) as f64;
    let depth = clamp_finite(bevel.depth, MAX_DEPTH) as f64;
    let altitude = clamp_finite(bevel.altitude_deg, MAX_ALTITUDE) as f64;
    let angle = if bevel.angle_deg.is_finite() {
        bevel.angle_deg as f64
    } else {
        0.0
    };
    let hi_opacity = clamp_finite(bevel.highlight.opacity, MAX_OPACITY) / 100.0;
    let sh_opacity = clamp_finite(bevel.shadow.opacity, MAX_OPACITY) / 100.0;
    if hi_opacity <= 0.0 && sh_opacity <= 0.0 {
        return;
    }
    // Content can only be non-zero inside `source`; the blur spreads it by the
    // blur supports, so the matte is built and processed over `padded` only.
    let padded = pad_rect(source, blur_support(size) + blur_support(soften) + 1, w, h);
    if rect_empty(padded) {
        return;
    }
    let (px0, py0, px1, py1) = padded;
    let pw = (px1 - px0) as usize;
    let ph = (py1 - py0) as usize;
    let mut matte = content_matte(layer, doc, padded);
    for y in py0..py1 {
        for x in px0..px1 {
            matte[(y - py0) as usize * pw + (x - px0) as usize] *=
                mask_alpha(layer, x, y) as f32 / 255.0;
        }
    }
    if matte.iter().all(|&v| v <= 0.0) {
        return;
    }
    let height = blur_matte(&matte, pw, ph, size);
    let sample = |x: i64, y: i64| -> f64 {
        let sx = x - px0 as i64;
        let sy = y - py0 as i64;
        if sx >= 0 && sy >= 0 && sx < pw as i64 && sy < ph as i64 {
            height[sy as usize * pw + sx as usize] as f64
        } else {
            0.0
        }
    };
    let theta = angle.to_radians();
    let phi = altitude.to_radians();
    let (lx, ly, lz) = (phi.cos() * theta.cos(), -phi.cos() * theta.sin(), phi.sin());
    let scale = size.max(1.0) * depth / 100.0;
    let down = matches!(bevel.direction, BevelDirection::Down);
    let sw = (source.2 - source.0) as usize;
    let sh = (source.3 - source.1) as usize;
    let mut shading = vec![0.0f32; sw * sh];
    for y in source.1..source.3 {
        for x in source.0..source.2 {
            let gx = (sample(x as i64 + 1, y as i64) - sample(x as i64 - 1, y as i64)) / 2.0;
            let gy = (sample(x as i64, y as i64 + 1) - sample(x as i64, y as i64 - 1)) / 2.0;
            let (nx, ny) = (-gx * scale, -gy * scale);
            let inv = 1.0 / (nx * nx + ny * ny + 1.0).sqrt();
            let mut s = (nx * lx + ny * ly + lz) * inv - phi.sin();
            if down {
                s = -s;
            }
            shading[(y - source.1) as usize * sw + (x - source.0) as usize] = s as f32;
        }
    }
    if soften > 0.0 {
        let mapped: Vec<f32> = shading
            .iter()
            .map(|&s| (((s as f64 + 1.0) / 2.0).clamp(0.0, 1.0)) as f32)
            .collect();
        for (dst, &b) in shading
            .iter_mut()
            .zip(blur_matte(&mapped, sw, sh, soften).iter())
        {
            *dst = b * 2.0 - 1.0;
        }
    }
    let shadow_color = [
        bevel.shadow.color[0] as f32 / 255.0,
        bevel.shadow.color[1] as f32 / 255.0,
        bevel.shadow.color[2] as f32 / 255.0,
    ];
    let highlight_color = [
        bevel.highlight.color[0] as f32 / 255.0,
        bevel.highlight.color[1] as f32 / 255.0,
        bevel.highlight.color[2] as f32 / 255.0,
    ];
    for y in source.1..source.3 {
        for x in source.0..source.2 {
            let m = matte[(y - py0) as usize * pw + (x - px0) as usize];
            let s = shading[(y - source.1) as usize * sw + (x - source.0) as usize];
            let sh_alpha = m * (-s).max(0.0) * sh_opacity;
            if sh_alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    shadow_color,
                    sh_alpha,
                    bevel.shadow.mode,
                );
            }
            let hi_alpha = m * s.max(0.0) * hi_opacity;
            if hi_alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    highlight_color,
                    hi_alpha,
                    bevel.highlight.mode,
                );
            }
        }
    }
}
