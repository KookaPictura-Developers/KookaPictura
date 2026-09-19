//! Object-based layer effects: the `lfx2` Drop Shadow, Outer Glow and Inner
//! Shadow.
//!
//! A layer's effects live in the `lfx2` additional-layer-information block as a
//! `DescriptorBlock2`: the top-level `DrSh` object holds the drop shadow, the
//! `OrGl` object holds the outer glow, and the `IrSh` object holds the inner
//! shadow. This slice decodes and renders all three.
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

use pictura_adjust::Adjustment;
use pictura_codec::DescValue;
use pictura_core::{BlendMode, Document, Layer, PixelBuffer};

use crate::composite::{blend_parts, channel, desc_item, mask_alpha, Canvas};

/// Documented Drop Shadow parameter caps (`docs/05-layers/layer-styles.md`).
const MAX_OPACITY: f32 = 100.0;
const MAX_DISTANCE: f32 = 30_000.0;
const MAX_SPREAD: f32 = 100.0;
const MAX_CHOKE: f32 = 100.0;
const MAX_SIZE: f32 = 250.0;

/// The typed drop shadow decoded from a layer's `lfx2` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropShadow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
    /// The stored local angle in degrees (`lagl`).
    pub angle_deg: f32,
    /// Pixels, `0..=30000`.
    pub distance: f32,
    /// Percent, `0..=100` (`Ckmt`, Photoshop's stored key for Spread).
    pub spread: f32,
    /// Pixels, Gaussian radius, `0..=250`.
    pub size: f32,
    pub use_global_angle: bool,
    pub knocks_out: bool,
}

/// The blur technique stored in `GlwT` (typeID `BETE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowTechnique {
    /// `SfBL`: the ordinary Gaussian blur.
    Softer,
    /// `PrBL`: Photoshop's distance-measure technique, rendered as `Softer`
    /// (a stated ceiling).
    Precise,
}

/// The typed outer glow decoded from a layer's `lfx2` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OuterGlow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
    /// Percent, `0..=100` (`Ckmt`, Photoshop's stored key for Spread).
    pub spread: f32,
    /// Pixels, Gaussian radius, `0..=250`.
    pub size: f32,
    pub technique: GlowTechnique,
}

/// The typed inner shadow decoded from a layer's `lfx2` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InnerShadow {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
    /// The stored local angle in degrees (`lagl`).
    pub angle_deg: f32,
    /// Pixels, `0..=30000`.
    pub distance: f32,
    /// Percent, `0..=100` (`Ckmt`, Photoshop's stored key for Choke, an erode).
    pub choke: f32,
    /// Pixels, Gaussian radius, `0..=250`.
    pub size: f32,
    pub use_global_angle: bool,
    /// `layerConceals`, decoded for symmetry with `DropShadow`; no render effect.
    pub knocks_out: bool,
}

/// Decode a layer's `lfx2` Outer Glow.
///
/// A missing `lfx2`, a missing or wrongly-typed `OrGl`, an unknown data version,
/// a wrong-typed or non-finite value, or a parse error is `None`. Never panics.
pub fn decode_outer_glow(layer: &Layer) -> Option<OuterGlow> {
    let data = &layer.extra_block(b"lfx2")?.data;
    let body = data.get(4..)?;
    let obj = pictura_codec::read_descriptor(body).ok()?;
    let orgl = desc_item(&obj, b"OrGl")?;
    let DescValue::Object { class_id, .. } = orgl else {
        return None;
    };
    if class_id.as_slice() != b"OrGl" {
        return None;
    }
    let blend_mode = match desc_item(orgl, b"Md  ") {
        None => BlendMode::Screen,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"BlnM" => value
            .as_slice()
            .try_into()
            .ok()
            .and_then(BlendMode::from_psd_key)
            .unwrap_or(BlendMode::Screen),
        Some(_) => return None,
    };
    let color = match desc_item(orgl, b"Clr ") {
        None => [255, 255, 190],
        Some(value) => decode_color(value)?,
    };
    let technique = match desc_item(orgl, b"GlwT") {
        None => GlowTechnique::Softer,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"BETE" => {
            if value.as_slice() == b"PrBL" {
                GlowTechnique::Precise
            } else {
                // `SfBL`, or an unknown value, is Softer.
                GlowTechnique::Softer
            }
        }
        Some(_) => return None,
    };
    Some(OuterGlow {
        enabled: bool_or(orgl, b"enab", false)?,
        present: bool_or(orgl, b"present", false)?,
        blend_mode,
        color,
        opacity: num_clamped(orgl, b"Opct", 75.0, 0.0, MAX_OPACITY)?,
        spread: num_clamped(orgl, b"Ckmt", 0.0, 0.0, MAX_SPREAD)?,
        size: num_clamped(orgl, b"blur", 5.0, 0.0, MAX_SIZE)?,
        technique,
    })
}

/// Decode a layer's `lfx2` Drop Shadow.
///
/// A missing `lfx2`, a missing or wrongly-typed `DrSh`, an unknown data version,
/// a wrong-typed or non-finite value, or a parse error is `None`. Never panics.
pub fn decode_drop_shadow(layer: &Layer) -> Option<DropShadow> {
    let data = &layer.extra_block(b"lfx2")?.data;
    // `DescriptorBlock2`: a `u32` version, then the version-16 descriptor block
    // whose own leading `u32` `read_descriptor` consumes.
    let body = data.get(4..)?;
    let obj = pictura_codec::read_descriptor(body).ok()?;
    let drsh = desc_item(&obj, b"DrSh")?;
    let DescValue::Object { class_id, .. } = drsh else {
        return None;
    };
    if class_id.as_slice() != b"DrSh" {
        return None;
    }
    let blend_mode = match desc_item(drsh, b"Md  ") {
        None => BlendMode::Normal,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"BlnM" => value
            .as_slice()
            .try_into()
            .ok()
            .and_then(BlendMode::from_psd_key)
            .unwrap_or(BlendMode::Normal),
        Some(_) => return None,
    };
    let color = match desc_item(drsh, b"Clr ") {
        None => [0, 0, 0],
        Some(value) => decode_color(value)?,
    };
    Some(DropShadow {
        enabled: bool_or(drsh, b"enab", false)?,
        present: bool_or(drsh, b"present", false)?,
        blend_mode,
        color,
        opacity: num_clamped(drsh, b"Opct", 75.0, 0.0, MAX_OPACITY)?,
        angle_deg: num_or(drsh, b"lagl", 120.0)?,
        distance: num_clamped(drsh, b"Dstn", 5.0, 0.0, MAX_DISTANCE)?,
        spread: num_clamped(drsh, b"Ckmt", 0.0, 0.0, MAX_SPREAD)?,
        size: num_clamped(drsh, b"blur", 5.0, 0.0, MAX_SIZE)?,
        use_global_angle: bool_or(drsh, b"uglg", true)?,
        knocks_out: bool_or(drsh, b"layerConceals", true)?,
    })
}

/// Decode a layer's `lfx2` Inner Shadow.
///
/// A missing `lfx2`, a missing or wrongly-typed `IrSh`, an unknown data version,
/// a wrong-typed or non-finite value, or a parse error is `None`. Never panics.
pub fn decode_inner_shadow(layer: &Layer) -> Option<InnerShadow> {
    let data = &layer.extra_block(b"lfx2")?.data;
    let body = data.get(4..)?;
    let obj = pictura_codec::read_descriptor(body).ok()?;
    let irsh = desc_item(&obj, b"IrSh")?;
    let DescValue::Object { class_id, .. } = irsh else {
        return None;
    };
    if class_id.as_slice() != b"IrSh" {
        return None;
    }
    let blend_mode = match desc_item(irsh, b"Md  ") {
        None => BlendMode::Multiply,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"BlnM" => value
            .as_slice()
            .try_into()
            .ok()
            .and_then(BlendMode::from_psd_key)
            .unwrap_or(BlendMode::Multiply),
        Some(_) => return None,
    };
    let color = match desc_item(irsh, b"Clr ") {
        None => [0, 0, 0],
        Some(value) => decode_color(value)?,
    };
    Some(InnerShadow {
        enabled: bool_or(irsh, b"enab", false)?,
        present: bool_or(irsh, b"present", false)?,
        blend_mode,
        color,
        opacity: num_clamped(irsh, b"Opct", 75.0, 0.0, MAX_OPACITY)?,
        angle_deg: num_or(irsh, b"lagl", 120.0)?,
        distance: num_clamped(irsh, b"Dstn", 5.0, 0.0, MAX_DISTANCE)?,
        choke: num_clamped(irsh, b"Ckmt", 0.0, 0.0, MAX_CHOKE)?,
        size: num_clamped(irsh, b"blur", 5.0, 0.0, MAX_SIZE)?,
        use_global_angle: bool_or(irsh, b"uglg", true)?,
        knocks_out: bool_or(irsh, b"layerConceals", true)?,
    })
}

/// Read a numeric key as `f32`. A value that is non-finite as an `f64`, or
/// finite as an `f64` but overflows to infinity as an `f32`, is rejected.
fn num_or(obj: &DescValue, key: &[u8], default: f32) -> Option<f32> {
    match desc_item(obj, key) {
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
    match desc_item(obj, key) {
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
    let component = |key: &[u8]| match desc_item(value, key) {
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
            composite_drop_shadow(canvas, layer, doc, &shadow);
        }
    }
    if let Some(glow) = decode_outer_glow(layer) {
        if glow.enabled && glow.present {
            composite_outer_glow(canvas, layer, doc, &glow);
        }
    }
}

/// Composite a layer's enabled, present inner shadow into the running canvas
/// **after** the layer's own content. Groups and destructive adjustment layers
/// are skipped, matching the below-content pass.
pub(crate) fn composite_layer_effects_above(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    if layer.is_group || is_destructive_adjustment(layer) {
        return;
    }
    if let Some(shadow) = decode_inner_shadow(layer) {
        if shadow.enabled && shadow.present {
            composite_inner_shadow(canvas, layer, doc, &shadow);
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

/// Build the shadow matte over the padded source region, dilate it by the
/// spread, blur it by the size, and composite it behind the layer content over
/// the shifted region only.
///
/// ponytail: a canvas-filling layer with the maximum `size` still costs
/// O(canvas · size) because the blur is a naive separable kernel; the upgrade
/// path is a bounded/box-blur approximation or a tighter first-slice size
/// ceiling. The common small-layer and crafted off-canvas cases are bounded.
fn composite_drop_shadow(canvas: &mut Canvas, layer: &Layer, doc: &Document, shadow: &DropShadow) {
    let (w, h) = (canvas.w as i32, canvas.h as i32);
    if w == 0 || h == 0 {
        return;
    }
    let source = clip_rect(layer, w, h);
    if rect_empty(source) {
        return;
    }
    // Clamp again: a hand-built `DropShadow` may not have gone through decode.
    let distance = clamp_finite(shadow.distance, MAX_DISTANCE) as f64;
    let angle = if shadow.angle_deg.is_finite() {
        shadow.angle_deg as f64
    } else {
        0.0
    };
    let theta = angle.to_radians();
    let dx = -distance * theta.cos();
    let dy = distance * theta.sin();
    let spread = clamp_finite(shadow.spread, MAX_SPREAD) as f64;
    let size = clamp_finite(shadow.size, MAX_SIZE) as f64;
    let dilate_radius = (spread / 100.0 * size).round() as i64;
    let blur_support = if size > 0.0 {
        (3.0 * pictura_filters::kernel::sigma_from_radius(size)).ceil() as i64
    } else {
        0
    };
    // Content can only be non-zero inside `source`; dilate and blur spread it by
    // at most `reach`, so the matte is built and processed over `padded` only.
    let padded = pad_rect(source, dilate_radius + blur_support, w, h);
    let (px0, py0, px1, py1) = padded;
    // The shadow at canvas `(x, y)` is the matte at `(x - dx, y - dy)`; only the
    // shifted region can be non-zero, and only its canvas part is composited.
    let ox = dx.round() as i64;
    let oy = dy.round() as i64;
    let cx0 = (px0 as i64 + ox).max(0);
    let cy0 = (py0 as i64 + oy).max(0);
    let cx1 = (px1 as i64 + ox).min(w as i64);
    let cy1 = (py1 as i64 + oy).min(h as i64);
    if cx1 <= cx0 || cy1 <= cy0 {
        return;
    }
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
    let dilated = if dilate_radius > 0 {
        dilate_matte(&matte, pw, ph, dilate_radius)
    } else {
        matte
    };
    let blurred = blur_matte(&dilated, pw, ph, size);
    let color = [
        shadow.color[0] as f32 / 255.0,
        shadow.color[1] as f32 / 255.0,
        shadow.color[2] as f32 / 255.0,
    ];
    let opacity = clamp_finite(shadow.opacity, MAX_OPACITY) / 100.0;
    for y in cy0..cy1 {
        for x in cx0..cx1 {
            let sx = (x - ox - px0 as i64) as usize;
            let sy = (y - oy - py0 as i64) as usize;
            let alpha = blurred[sy * pw + sx] * opacity;
            if alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    color,
                    alpha,
                    shadow.blend_mode,
                );
            }
        }
    }
}

/// Build the glow matte over the padded source region, dilate it by the spread,
/// blur it by the size, knock out the content, and composite it behind the
/// layer content over the padded region only (a glow has no offset).
///
/// ponytail: a canvas-filling layer with the maximum `size` still costs
/// O(canvas · size) because the blur is a naive separable kernel; the common
/// small-layer and crafted off-canvas cases are bounded. `Precise` renders as
/// `Softer`.
fn composite_outer_glow(canvas: &mut Canvas, layer: &Layer, doc: &Document, glow: &OuterGlow) {
    let (w, h) = (canvas.w as i32, canvas.h as i32);
    if w == 0 || h == 0 {
        return;
    }
    let source = clip_rect(layer, w, h);
    if rect_empty(source) {
        return;
    }
    // Clamp again: a hand-built `OuterGlow` may not have gone through decode.
    let spread = clamp_finite(glow.spread, MAX_SPREAD) as f64;
    let size = clamp_finite(glow.size, MAX_SIZE) as f64;
    let dilate_radius = (spread / 100.0 * size).round() as i64;
    let blur_support = if size > 0.0 {
        (3.0 * pictura_filters::kernel::sigma_from_radius(size)).ceil() as i64
    } else {
        0
    };
    // Content can only be non-zero inside `source`; dilate and blur spread it by
    // at most `reach`, so the matte is built and processed over `padded` only.
    let padded = pad_rect(source, dilate_radius + blur_support, w, h);
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
    // Exterior mask: the glow is knocked out where the content is opaque.
    let exterior: Vec<f32> = matte.iter().map(|&m| 1.0 - m).collect();
    let dilated = if dilate_radius > 0 {
        dilate_matte(&matte, pw, ph, dilate_radius)
    } else {
        matte
    };
    let blurred = blur_matte(&dilated, pw, ph, size);
    let color = [
        glow.color[0] as f32 / 255.0,
        glow.color[1] as f32 / 255.0,
        glow.color[2] as f32 / 255.0,
    ];
    let opacity = clamp_finite(glow.opacity, MAX_OPACITY) / 100.0;
    for y in py0..py1 {
        for x in px0..px1 {
            let i = (y - py0) as usize * pw + (x - px0) as usize;
            let alpha = blurred[i] * exterior[i] * opacity;
            if alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    color,
                    alpha,
                    glow.blend_mode,
                );
            }
        }
    }
}

/// Build the masked content matte `M` over the padded source region, form the
/// inverted matte `1 - M`, erode it by the choke radius, blur it by the size,
/// then composite `M · blurred(x - dx, y - dy) · opacity` above the content over
/// `source` only (a sample outside the built region is the exterior value `1`).
///
/// ponytail: a canvas-filling layer with the maximum `size` still costs
/// O(canvas · size) because the blur is a naive separable kernel; the common
/// small-layer and crafted off-canvas cases are bounded. The choke maps to a
/// min-filter erode of radius `round(choke / 100 · size)`.
fn composite_inner_shadow(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
    shadow: &InnerShadow,
) {
    let (w, h) = (canvas.w as i32, canvas.h as i32);
    if w == 0 || h == 0 {
        return;
    }
    let source = clip_rect(layer, w, h);
    if rect_empty(source) {
        return;
    }
    // Clamp again: a hand-built `InnerShadow` may not have gone through decode.
    let distance = clamp_finite(shadow.distance, MAX_DISTANCE) as f64;
    let angle = if shadow.angle_deg.is_finite() {
        shadow.angle_deg as f64
    } else {
        0.0
    };
    let theta = angle.to_radians();
    let dx = -distance * theta.cos();
    let dy = distance * theta.sin();
    let choke = clamp_finite(shadow.choke, MAX_CHOKE) as f64;
    let size = clamp_finite(shadow.size, MAX_SIZE) as f64;
    let erode_radius = (choke / 100.0 * size).round() as i64;
    let blur_support = if size > 0.0 {
        (3.0 * pictura_filters::kernel::sigma_from_radius(size)).ceil() as i64
    } else {
        0
    };
    // Content can only be non-zero inside `source`; erode and blur spread the
    // inverted matte by at most `reach`, so it is built over `padded` only.
    let padded = pad_rect(source, erode_radius + blur_support, w, h);
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
    let inverted: Vec<f32> = matte.iter().map(|&m| 1.0 - m).collect();
    let eroded = if erode_radius > 0 {
        erode_matte(&inverted, pw, ph, erode_radius)
    } else {
        inverted
    };
    let blurred = blur_matte(&eroded, pw, ph, size);
    let color = [
        shadow.color[0] as f32 / 255.0,
        shadow.color[1] as f32 / 255.0,
        shadow.color[2] as f32 / 255.0,
    ];
    let opacity = clamp_finite(shadow.opacity, MAX_OPACITY) / 100.0;
    let ox = dx.round() as i64;
    let oy = dy.round() as i64;
    for y in source.1..source.3 {
        for x in source.0..source.2 {
            let sx = x as i64 - ox - px0 as i64;
            let sy = y as i64 - oy - py0 as i64;
            let b = if sx >= 0 && sy >= 0 && sx < pw as i64 && sy < ph as i64 {
                blurred[sy as usize * pw + sx as usize]
            } else {
                // Beyond the built region there is no content, so `1 - M = 1`.
                1.0
            };
            let m = matte[(y - py0) as usize * pw + (x - px0) as usize];
            let alpha = m * b * opacity;
            if alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    color,
                    alpha,
                    shadow.blend_mode,
                );
            }
        }
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
