//! CPU compositing for Kooka Pictura.
//!
//! M2 scope: composite the document layer stack (opacity, raster masks, groups,
//! 27 blend modes) into an image. Spec: `docs/05-layers/blend-modes.md` and the
//! W3C Compositing and Blending Level 1 model. GPU acceleration is M2.5.
//!
//! Contract owned by task M2-A:
//!
//! ```ignore
//! /// 4-channel (R,G,B,A) planar, straight alpha, 8-bit, at document size.
//! pub fn composite_rgba(doc: &pictura_core::Document) -> pictura_core::PixelBuffer;
//! ```
//!
//! # Groups and pass-through
//!
//! A group whose blend mode is [`BlendMode::PassThrough`] is composited as a
//! true pass-through when **opacity is 255 and it has no mask**: its children
//! are composited directly onto the running canvas, so child blend modes see
//! the backdrop outside the group. Any other blend mode is an **isolated**
//! group: children composite onto a private transparent buffer that is then
//! blended as one layer with the group's mode, opacity, and mask.
//!
//! Approximation: a pass-through group with non-255 opacity or a mask falls
//! back to isolated compositing with `PassThrough` treated as `Normal`. Real
//! Photoshop applies the group opacity/mask to the pass-through result while
//! still letting children blend against the parent backdrop; that mixed model
//! is not implemented. Exact pass-through parity is limited to the 255/no-mask
//! case (see the test below).

use pictura_adjust::{Adjustment, BrightnessContrastParams, HueSaturationParams, LevelsParams};
use pictura_core::{AdjustmentData, BlendMode, ColorMode, Document, Layer, PixelBuffer};

pub mod gpu;
pub use gpu::{
    composite_active, composite_gpu, composite_gpu_or_cpu, gpu_available, Backend, GpuError,
};

mod filter;
pub use filter::apply_filter;

mod gpu_filter;
pub use gpu_filter::{apply_filter_active, filter_gpu_available};

pub mod document_ops;
pub use document_ops::{
    crop_document, flip_document, resize_canvas_document, resize_document, rotate_document,
    translate_layer,
};

pub use pictura_ops::{Anchor, Resample};

/// Composite the document's layer stack.
///
/// Returns a 4-channel (R,G,B,A) planar, straight-alpha, 8-bit buffer at
/// document resolution.
pub fn composite_rgba(doc: &Document) -> PixelBuffer {
    let mut canvas = Canvas::new(doc.width as usize, doc.height as usize);
    for layer in &doc.layers {
        composite_layer(&mut canvas, layer, doc);
    }
    canvas.into_pixel_buffer()
}

/// Straight-alpha RGBA accumulator in normalized `f32`.
///
/// ponytail: one full-document f32 buffer; streaming/tiled compositing is the
/// ceiling to raise when PSB-size docs no longer fit in memory.
#[derive(Clone, Copy, Default)]
struct Px {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct Canvas {
    w: usize,
    h: usize,
    px: Vec<Px>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![Px::default(); w * h],
        }
    }

    fn into_pixel_buffer(self) -> PixelBuffer {
        let plane = self.w * self.h;
        let mut out = PixelBuffer::new(self.w as u32, self.h as u32, 4);
        for (i, p) in self.px.iter().enumerate() {
            out.data[i] = to_u8(p.r);
            out.data[plane + i] = to_u8(p.g);
            out.data[2 * plane + i] = to_u8(p.b);
            out.data[3 * plane + i] = to_u8(p.a);
        }
        out
    }
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn composite_layer(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    if !layer.visible {
        return;
    }
    if layer.is_group {
        // True pass-through: recurse children straight onto the running canvas
        // so their blend modes see content outside the group. Only exact for
        // opacity 255 / no mask; otherwise fall back to isolated compositing
        // (see the crate-level note).
        if matches!(layer.blend, BlendMode::PassThrough)
            && layer.opacity == 255
            && layer.mask.is_none()
        {
            for child in &layer.children {
                composite_layer(canvas, child, doc);
            }
            return;
        }
        let mut inner = Canvas::new(canvas.w, canvas.h);
        for child in &layer.children {
            composite_layer(&mut inner, child, doc);
        }
        composite_canvas(canvas, layer, &inner);
    } else if let Some(data) = &layer.adjustment {
        // An adjustment layer owns no pixels: it transforms the backdrop it is
        // composited over. Unknown/undecodable keys are a no-op (preserved on
        // save, not applied), never an error.
        if let Some(adjustment) = decode_adjustment(data) {
            composite_adjustment(canvas, layer, &adjustment);
        }
    } else {
        composite_pixels(canvas, layer, doc);
    }
}

fn composite_pixels(canvas: &mut Canvas, layer: &Layer, doc: &Document) {
    let lw = layer.rect.width();
    let lh = layer.rect.height();
    if lw <= 0 || lh <= 0 {
        return;
    }
    let lw = lw as usize;
    let x0 = layer.rect.left.max(0);
    let y0 = layer.rect.top.max(0);
    let x1 = layer.rect.right.min(canvas.w as i32);
    let y1 = layer.rect.bottom.min(canvas.h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    let gray = matches!(
        doc.mode,
        ColorMode::Grayscale | ColorMode::Bitmap | ColorMode::Duotone
    );
    let ch0 = channel(layer, 0);
    let ch1 = channel(layer, 1).or(ch0);
    let ch2 = channel(layer, 2).or(ch0);
    let alpha = channel(layer, -1);

    for y in y0..y1 {
        for x in x0..x1 {
            let li = (y - layer.rect.top) as usize * lw + (x - layer.rect.left) as usize;
            let (r, g, b) = if gray {
                let v = sample(ch0, li).unwrap_or(0);
                (v, v, v)
            } else {
                (
                    sample(ch0, li).unwrap_or(0),
                    sample(ch1, li).unwrap_or(0),
                    sample(ch2, li).unwrap_or(0),
                )
            };
            let a = sample(alpha, li).unwrap_or(255);
            blend_into(
                canvas,
                layer,
                x as usize,
                y as usize,
                [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0],
                a as f32 / 255.0,
            );
        }
    }
}

fn composite_canvas(canvas: &mut Canvas, layer: &Layer, inner: &Canvas) {
    for y in 0..canvas.h {
        for x in 0..canvas.w {
            let p = inner.px[y * inner.w + x];
            if p.a > 0.0 {
                blend_into(canvas, layer, x, y, [p.r, p.g, p.b], p.a);
            }
        }
    }
}

/// Decode a raw PSD adjustment block into a destructive [`Adjustment`] for the
/// encodings this crate supports. `None` means "not understood": the caller
/// leaves the backdrop unchanged (no-op), never errors.
///
/// Supported keys: `nvrt`/`invr` (Invert, no payload), `post` (Posterize),
/// `thrs` (Threshold), `brit` (Brightness/Contrast), `levl` (Levels, composite
/// record) and `hue2`/`hue ` (Hue/Saturation). Descriptor/custom payloads
/// (`curv`, `expA`, `vibA`, `blwh`, `phfl`, `mixr`, `gdrm`, `selc`, `clrL`) are
/// preserved on disk but not decoded here.
pub fn decode_adjustment(data: &AdjustmentData) -> Option<Adjustment> {
    match &data.key {
        b"nvrt" | b"invr" => Some(Adjustment::Invert),
        b"post" => be_u16(&data.data, 0)
            .filter(|v| (2..=255).contains(v))
            .map(|v| Adjustment::Posterize(v as u8)),
        b"thrs" => be_u16(&data.data, 0)
            .filter(|v| (1..=255).contains(v))
            .map(|v| Adjustment::Threshold(v as u8)),
        b"brit" => decode_brightness_contrast(&data.data),
        b"levl" => decode_levels(&data.data),
        b"hue2" | b"hue " => decode_hue_saturation(&data.data),
        _ => None,
    }
}

fn be_u16(d: &[u8], at: usize) -> Option<u16> {
    let s = d.get(at..at + 2)?;
    Some(u16::from_be_bytes([s[0], s[1]]))
}

fn be_i16(d: &[u8], at: usize) -> Option<i16> {
    Some(be_u16(d, at)? as i16)
}

/// `brit`: brightness (i16), contrast (i16), mean (i16), lab_only (u8), pad.
///
/// Photoshop's `brit` block carries no explicit "Use Legacy" flag; CS6 defaults
/// new adjustment layers to the modern curve, so decode with `use_legacy:false`.
/// ponytail: legacy-vs-modern detection is not encoded here; revisit if a CS6
/// baseline for legacy `brit` files appears.
fn decode_brightness_contrast(d: &[u8]) -> Option<Adjustment> {
    let brightness = be_i16(d, 0)?;
    let contrast = be_i16(d, 2)?;
    if !(-150..=150).contains(&brightness) || !(-50..=100).contains(&contrast) {
        return None;
    }
    Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
        brightness,
        contrast,
        use_legacy: false,
    }))
}

/// `levl`: `u16 version` (2) then 29 five-`u16` records. Record 0 is the
/// composite channel; this decoder applies it uniformly to R/G/B.
fn decode_levels(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 2 {
        return None;
    }
    let input_black = be_u16(d, 2)?;
    let input_white = be_u16(d, 4)?;
    let output_black = be_u16(d, 6)?;
    let output_white = be_u16(d, 8)?;
    let gamma = be_u16(d, 10)?;
    if input_black >= input_white || gamma == 0 {
        return None;
    }
    if [input_black, input_white, output_black, output_white]
        .into_iter()
        .any(|v| v > 255)
    {
        return None;
    }
    Some(Adjustment::Levels(LevelsParams {
        input_black: input_black as u8,
        input_white: input_white as u8,
        gamma: gamma as f64 / 100.0,
        output_black: output_black as u8,
        output_white: output_white as u8,
    }))
}

/// `hue2` (and legacy `hue `): version `u16`, enable `u8`, pad, colorization
/// (3×i16), then the master Hue/Saturation/Lightness triplet (3×i16).
fn decode_hue_saturation(d: &[u8]) -> Option<Adjustment> {
    if be_u16(d, 0)? != 2 {
        return None;
    }
    let hue = be_i16(d, 10)?;
    let saturation = be_i16(d, 12)?;
    let lightness = be_i16(d, 14)?;
    if !(-180..=180).contains(&hue)
        || !(-100..=100).contains(&saturation)
        || !(-100..=100).contains(&lightness)
    {
        return None;
    }
    Some(Adjustment::HueSaturation(HueSaturationParams {
        hue,
        saturation,
        lightness,
    }))
}

// --- Encoders for the same subset ------------------------------------------
//
// These build the raw `AdjustmentData` the decoder above reads, so the app can
// create adjustment layers in memory. Byte layouts mirror psd-tools' adjustment
// structs (the independent oracle the codec fixtures come from); each is the
// minimal block for the key, not a full re-implementation of Photoshop's writer.

/// `nvrt`: Invert carries no payload.
pub fn encode_invert() -> AdjustmentData {
    AdjustmentData {
        key: *b"nvrt",
        data: Vec::new(),
    }
}

/// `post`: a `u16` levels value (2..=255) plus 2 pad bytes (psd-tools
/// `ShortIntegerElement`, `H2x`). Out-of-range input is clamped.
pub fn encode_posterize(levels: u8) -> AdjustmentData {
    encode_short(*b"post", levels.clamp(2, 255) as u16)
}

/// `thrs`: a `u16` level (1..=255) plus 2 pad bytes. Out-of-range input is clamped.
pub fn encode_threshold(level: u8) -> AdjustmentData {
    encode_short(*b"thrs", level.clamp(1, 255) as u16)
}

/// `brit`: brightness (i16), contrast (i16), mean (i16), lab_only (u8), pad
/// (psd-tools `3HBx`). Inputs are clamped to the decoder's accepted ranges
/// (brightness -150..=150, contrast -50..=100); `use_legacy` is not representable
/// and the decoder always uses the modern curve.
pub fn encode_brightness_contrast(brightness: i16, contrast: i16) -> AdjustmentData {
    let b = brightness.clamp(-150, 150);
    let c = contrast.clamp(-50, 100);
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&b.to_be_bytes());
    data.extend_from_slice(&c.to_be_bytes());
    data.extend_from_slice(&0i16.to_be_bytes());
    data.push(0);
    data.push(0);
    AdjustmentData {
        key: *b"brit",
        data,
    }
}

/// `hue2`: version (2), enable (1), pad, colorization (3×i16), then the master
/// Hue/Saturation/Lightness triplet (3×i16), followed by the six per-band range
/// records (6 × 7 i16) that Photoshop stores. The decoder only reads the version
/// and the master triplet; the trailing records are zeroed so the block matches
/// the real 100-byte layout.
pub fn encode_hue_saturation(hue: i16, saturation: i16, lightness: i16) -> AdjustmentData {
    let hue = hue.clamp(-180, 180);
    let saturation = saturation.clamp(-100, 100);
    let lightness = lightness.clamp(-100, 100);
    let mut data = Vec::with_capacity(100);
    data.extend_from_slice(&2u16.to_be_bytes());
    data.push(1);
    data.push(0);
    data.extend_from_slice(&[0u8; 6]);
    data.extend_from_slice(&hue.to_be_bytes());
    data.extend_from_slice(&saturation.to_be_bytes());
    data.extend_from_slice(&lightness.to_be_bytes());
    data.extend_from_slice(&[0u8; 84]);
    AdjustmentData {
        key: *b"hue2",
        data,
    }
}

/// `H2x` payload: a big-endian `u16` value plus two zero pad bytes.
fn encode_short(key: [u8; 4], value: u16) -> AdjustmentData {
    let mut data = value.to_be_bytes().to_vec();
    data.extend_from_slice(&[0, 0]);
    AdjustmentData { key, data }
}

/// Apply a decoded adjustment to the running backdrop, then gate the result by
/// the layer's mask/opacity/blend (Photoshop applies the adjustment to the
/// backdrop and blends the adjusted result back).
fn composite_adjustment(canvas: &mut Canvas, layer: &Layer, adjustment: &Adjustment) {
    let n = canvas.w * canvas.h;
    if n == 0 {
        return;
    }
    let mut buf = PixelBuffer::new(canvas.w as u32, canvas.h as u32, 3);
    for (i, p) in canvas.px.iter().enumerate() {
        buf.data[i] = to_u8(p.r);
        buf.data[n + i] = to_u8(p.g);
        buf.data[2 * n + i] = to_u8(p.b);
    }
    if pictura_adjust::apply(adjustment, &mut buf).is_err() {
        return; // invalid/unsupported parameters: no-op, never an error
    }
    for y in 0..canvas.h {
        for x in 0..canvas.w {
            let i = y * canvas.w + x;
            // Source coverage is the backdrop's own alpha: an adjustment adds no
            // content where the backdrop is transparent.
            let backdrop_alpha = canvas.px[i].a;
            if backdrop_alpha <= 0.0 {
                continue;
            }
            let cs = [
                buf.data[i] as f32 / 255.0,
                buf.data[n + i] as f32 / 255.0,
                buf.data[2 * n + i] as f32 / 255.0,
            ];
            blend_into(canvas, layer, x, y, cs, backdrop_alpha);
        }
    }
}

/// Composite one source sample over the running backdrop, applying the layer's
/// opacity and mask to the source alpha.
fn blend_into(canvas: &mut Canvas, layer: &Layer, x: usize, y: usize, cs: [f32; 3], src_a: f32) {
    if src_a <= 0.0 {
        return;
    }
    let opacity = layer.opacity as f32 / 255.0;
    let masked = mask_alpha(layer, x as i32, y as i32) as f32 / 255.0;
    let mut as_ = src_a * opacity * masked;
    if as_ <= 0.0 {
        return;
    }

    let i = y * canvas.w + x;
    let cb = canvas.px[i];

    // Dissolve is stochastic: the effective alpha is a threshold on a fixed
    // per-pixel noise field, and passing pixels go fully opaque.
    // ponytail: deterministic splitmix hash, not Adobe's unpublished noise tile;
    // swap when a CS6 dither baseline exists.
    if matches!(layer.blend, BlendMode::Dissolve) {
        if dissolve_noise(x, y) >= as_ {
            return;
        }
        as_ = 1.0;
    }

    let ab = cb.a;
    let b = blend(layer.blend, [cb.r, cb.g, cb.b], cs);
    let ao = as_ + ab * (1.0 - as_);
    if ao <= 0.0 {
        canvas.px[i] = Px::default();
        return;
    }
    let co = |idx: usize, c: f32| {
        ((1.0 - ab) * as_ * cs[idx] + as_ * ab * b[idx] + (1.0 - as_) * ab * c) / ao
    };
    canvas.px[i] = Px {
        r: co(0, cb.r),
        g: co(1, cb.g),
        b: co(2, cb.b),
        a: ao,
    };
}

pub(crate) fn channel(layer: &Layer, id: i16) -> Option<&[u8]> {
    layer
        .channels
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.data.as_slice())
}

fn sample(data: Option<&[u8]>, idx: usize) -> Option<u8> {
    data.and_then(|d| d.get(idx).copied())
}

/// Mask value at a canvas pixel; `255` when there is no usable mask.
fn mask_alpha(layer: &Layer, x: i32, y: i32) -> u8 {
    let Some(mask) = &layer.mask else {
        return 255;
    };
    if mask.disabled {
        return 255;
    }
    let Some(data) = &mask.data else {
        return 255;
    };
    let mw = mask.rect.width();
    let mh = mask.rect.height();
    if mw <= 0 || mh <= 0 {
        return mask.default_color;
    }
    let mx = x - mask.rect.left;
    let my = y - mask.rect.top;
    if mx < 0 || my < 0 || mx >= mw || my >= mh {
        return mask.default_color;
    }
    let idx = my as usize * mw as usize + mx as usize;
    data.get(idx).copied().unwrap_or(mask.default_color)
}

fn dissolve_noise(x: usize, y: usize) -> f32 {
    let mut z = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ 0xD1B5_4A32_D192_ED03;
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    ((z >> 56) as f32) / 256.0
}

// ---------------------------------------------------------------------------
// Blend functions (W3C Compositing and Blending Level 1 + Photoshop-only modes)
// ---------------------------------------------------------------------------

fn blend(mode: BlendMode, cb: [f32; 3], cs: [f32; 3]) -> [f32; 3] {
    match mode {
        BlendMode::Hue => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        BlendMode::Saturation => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        BlendMode::Color => set_lum(cs, lum(cb)),
        BlendMode::Luminosity => set_lum(cb, lum(cs)),
        BlendMode::DarkerColor => {
            if sum(cb) <= sum(cs) {
                cb
            } else {
                cs
            }
        }
        BlendMode::LighterColor => {
            if sum(cb) >= sum(cs) {
                cb
            } else {
                cs
            }
        }
        _ => [
            sep(mode, cb[0], cs[0]),
            sep(mode, cb[1], cs[1]),
            sep(mode, cb[2], cs[2]),
        ],
    }
}

fn sep(mode: BlendMode, cb: f32, cs: f32) -> f32 {
    match mode {
        BlendMode::Normal | BlendMode::Dissolve => cs,
        BlendMode::Darken => cb.min(cs),
        BlendMode::Multiply => cb * cs,
        BlendMode::ColorBurn => color_burn(cb, cs),
        BlendMode::LinearBurn => (cb + cs - 1.0).max(0.0),
        BlendMode::Lighten => cb.max(cs),
        BlendMode::Screen => 1.0 - (1.0 - cb) * (1.0 - cs),
        BlendMode::ColorDodge => color_dodge(cb, cs),
        BlendMode::LinearDodge => (cb + cs).min(1.0),
        BlendMode::Overlay => hard_light(cs, cb),
        BlendMode::SoftLight => soft_light(cb, cs),
        BlendMode::HardLight => hard_light(cb, cs),
        BlendMode::VividLight => vivid_light(cb, cs),
        BlendMode::LinearLight => (cb + 2.0 * cs - 1.0).clamp(0.0, 1.0),
        BlendMode::PinLight => pin_light(cb, cs),
        BlendMode::HardMix => {
            if cb + cs >= 1.0 {
                1.0
            } else {
                0.0
            }
        }
        BlendMode::Difference => (cb - cs).abs(),
        BlendMode::Exclusion => cb + cs - 2.0 * cb * cs,
        BlendMode::Subtract => (cb - cs).max(0.0),
        BlendMode::Divide => {
            if cs == 0.0 {
                1.0
            } else {
                (cb / cs).min(1.0)
            }
        }
        // Non-separable modes never reach the per-channel path.
        _ => cs,
    }
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if cb >= 1.0 {
        1.0
    } else if cs <= 0.0 {
        0.0
    } else {
        1.0 - ((1.0 - cb) / cs).min(1.0)
    }
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if cb <= 0.0 {
        0.0
    } else if cs >= 1.0 {
        1.0
    } else {
        (cb / (1.0 - cs)).min(1.0)
    }
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        2.0 * cb * cs
    } else {
        1.0 - 2.0 * (1.0 - cb) * (1.0 - cs)
    }
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
    } else {
        cb + (2.0 * cs - 1.0) * (soft_light_d(cb) - cb)
    }
}

fn soft_light_d(x: f32) -> f32 {
    if x <= 0.25 {
        ((16.0 * x - 12.0) * x + 4.0) * x
    } else {
        x.sqrt()
    }
}

fn vivid_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        color_burn(cb, 2.0 * cs)
    } else {
        color_dodge(cb, 2.0 * cs - 1.0)
    }
}

fn pin_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        cb.min(2.0 * cs)
    } else {
        cb.max(2.0 * cs - 1.0)
    }
}

fn lum(c: [f32; 3]) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn sat(c: [f32; 3]) -> f32 {
    let max = c.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let min = c.iter().copied().fold(f32::INFINITY, f32::min);
    max - min
}

fn sum(c: [f32; 3]) -> f32 {
    c[0] + c[1] + c[2]
}

fn set_lum(c: [f32; 3], l: f32) -> [f32; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn clip_color(c: [f32; 3]) -> [f32; 3] {
    let l = lum(c);
    let n = c.iter().copied().fold(f32::INFINITY, f32::min);
    let x = c.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut out = c;
    if n < 0.0 {
        let d = l - n;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * l / d,
                l + (out[1] - l) * l / d,
                l + (out[2] - l) * l / d,
            ];
        }
    }
    if x > 1.0 {
        let d = x - l;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * (1.0 - l) / d,
                l + (out[1] - l) * (1.0 - l) / d,
                l + (out[2] - l) * (1.0 - l) / d,
            ];
        }
    }
    out
}

fn set_sat(c: [f32; 3], s: f32) -> [f32; 3] {
    let (mut imin, mut imax) = (0usize, 0usize);
    for i in 1..3 {
        if c[i] < c[imin] {
            imin = i;
        }
        if c[i] > c[imax] {
            imax = i;
        }
    }
    let cmin = c[imin];
    let cmax = c[imax];
    if cmax > cmin {
        let imid = 3 - imin - imax;
        let mut out = c;
        out[imid] = (c[imid] - cmin) * s / (cmax - cmin);
        out[imax] = s;
        out[imin] = 0.0;
        out
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, Channel, LayerMask, PsdRect};

    fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
        PsdRect {
            top,
            left,
            bottom,
            right,
        }
    }

    fn full(w: u32, h: u32) -> PsdRect {
        rect(0, 0, h as i32, w as i32)
    }

    fn solid(
        name: &str,
        r: PsdRect,
        rgb: (u8, u8, u8),
        alpha: u8,
        blend: BlendMode,
        opacity: u8,
    ) -> Layer {
        let w = r.width().max(0) as usize;
        let h = r.height().max(0) as usize;
        let n = w * h;
        Layer {
            name: name.into(),
            rect: r,
            blend,
            opacity,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![rgb.0; n],
                },
                Channel {
                    id: 1,
                    data: vec![rgb.1; n],
                },
                Channel {
                    id: 2,
                    data: vec![rgb.2; n],
                },
                Channel {
                    id: -1,
                    data: vec![alpha; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }
    }

    fn group(
        name: &str,
        blend: BlendMode,
        opacity: u8,
        mask: Option<LayerMask>,
        children: Vec<Layer>,
    ) -> Layer {
        Layer {
            name: name.into(),
            rect: rect(0, 0, 0, 0),
            blend,
            opacity,
            clipping: false,
            visible: true,
            mask,
            adjustment: None,
            channels: Vec::new(),
            children,
            is_group: true,
        }
    }

    fn doc(w: u32, h: u32, layers: Vec<Layer>) -> Document {
        let mut d = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        d.layers = layers;
        d
    }

    fn px(buf: &PixelBuffer, x: u32, y: u32) -> [u8; 4] {
        let plane = (buf.width * buf.height) as usize;
        let i = (y * buf.width + x) as usize;
        [
            buf.data[i],
            buf.data[plane + i],
            buf.data[2 * plane + i],
            buf.data[3 * plane + i],
        ]
    }

    fn assert_blend(mode: BlendMode, cb: [f32; 3], cs: [f32; 3], expected: [f32; 3]) {
        let got = blend(mode, cb, cs);
        for i in 0..3 {
            assert!(
                (got[i] - expected[i]).abs() <= 1.0 / 255.0 + 1e-6,
                "{mode:?}[{i}]: got {} want {}",
                got[i],
                expected[i]
            );
        }
    }

    // --- one test per blend mode, Cb = 0.25, Cs = 0.75 ---------------------

    #[test]
    fn mode_normal() {
        assert_blend(BlendMode::Normal, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    #[test]
    fn mode_dissolve_passes_source_color() {
        assert_blend(BlendMode::Dissolve, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    #[test]
    fn mode_darken() {
        assert_blend(BlendMode::Darken, [0.25; 3], [0.75; 3], [0.25; 3]);
    }

    #[test]
    fn mode_multiply() {
        assert_blend(BlendMode::Multiply, [0.25; 3], [0.75; 3], [0.1875; 3]);
    }

    #[test]
    fn mode_color_burn() {
        assert_blend(BlendMode::ColorBurn, [0.25; 3], [0.75; 3], [0.0; 3]);
    }

    #[test]
    fn mode_linear_burn() {
        assert_blend(BlendMode::LinearBurn, [0.25; 3], [0.75; 3], [0.0; 3]);
    }

    #[test]
    fn mode_darker_color() {
        assert_blend(BlendMode::DarkerColor, [0.25; 3], [0.75; 3], [0.25; 3]);
    }

    #[test]
    fn mode_lighten() {
        assert_blend(BlendMode::Lighten, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    #[test]
    fn mode_screen() {
        assert_blend(BlendMode::Screen, [0.25; 3], [0.75; 3], [0.8125; 3]);
    }

    #[test]
    fn mode_color_dodge() {
        assert_blend(BlendMode::ColorDodge, [0.25; 3], [0.75; 3], [1.0; 3]);
    }

    #[test]
    fn mode_linear_dodge() {
        assert_blend(BlendMode::LinearDodge, [0.25; 3], [0.75; 3], [1.0; 3]);
    }

    #[test]
    fn mode_lighter_color() {
        assert_blend(BlendMode::LighterColor, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    #[test]
    fn mode_overlay() {
        assert_blend(BlendMode::Overlay, [0.25; 3], [0.75; 3], [0.375; 3]);
    }

    #[test]
    fn mode_soft_light() {
        assert_blend(BlendMode::SoftLight, [0.25; 3], [0.75; 3], [0.375; 3]);
    }

    #[test]
    fn mode_hard_light() {
        assert_blend(BlendMode::HardLight, [0.25; 3], [0.75; 3], [0.625; 3]);
    }

    #[test]
    fn mode_vivid_light() {
        assert_blend(BlendMode::VividLight, [0.25; 3], [0.75; 3], [0.5; 3]);
    }

    #[test]
    fn mode_linear_light() {
        assert_blend(BlendMode::LinearLight, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    #[test]
    fn mode_pin_light() {
        assert_blend(BlendMode::PinLight, [0.25; 3], [0.75; 3], [0.5; 3]);
    }

    #[test]
    fn mode_hard_mix() {
        assert_blend(BlendMode::HardMix, [0.25; 3], [0.75; 3], [1.0; 3]);
    }

    #[test]
    fn mode_difference() {
        assert_blend(BlendMode::Difference, [0.25; 3], [0.75; 3], [0.5; 3]);
    }

    #[test]
    fn mode_exclusion() {
        assert_blend(BlendMode::Exclusion, [0.25; 3], [0.75; 3], [0.625; 3]);
    }

    #[test]
    fn mode_subtract() {
        assert_blend(BlendMode::Subtract, [0.25; 3], [0.75; 3], [0.0; 3]);
    }

    #[test]
    fn mode_divide() {
        assert_blend(BlendMode::Divide, [0.25; 3], [0.75; 3], [1.0 / 3.0; 3]);
    }

    #[test]
    fn mode_hue() {
        assert_blend(BlendMode::Hue, [0.25; 3], [0.75; 3], [0.25; 3]);
    }

    #[test]
    fn mode_saturation() {
        assert_blend(BlendMode::Saturation, [0.25; 3], [0.75; 3], [0.25; 3]);
    }

    #[test]
    fn mode_color() {
        assert_blend(BlendMode::Color, [0.25; 3], [0.75; 3], [0.25; 3]);
    }

    #[test]
    fn mode_luminosity() {
        assert_blend(BlendMode::Luminosity, [0.25; 3], [0.75; 3], [0.75; 3]);
    }

    // --- chromatic non-separable check (hand-computed, W3C helpers) ---------

    #[test]
    fn nonseparable_modes_chromatic_w3c_values() {
        let cb = [0.1, 0.5, 0.3];
        let cs = [0.8, 0.4, 0.0];
        let cases = [
            (BlendMode::Hue, [0.52, 0.32, 0.12]),
            (BlendMode::Saturation, [0.0, 0.555_038_8, 0.277_519_4]),
            (BlendMode::Color, [0.601_680_7, 0.300_840_4, 0.0]),
            (BlendMode::Luminosity, [0.218, 0.618, 0.418]),
        ];
        for (mode, expected) in cases {
            let got = blend(mode, cb, cs);
            for i in 0..3 {
                assert!(
                    (got[i] - expected[i]).abs() < 1e-4,
                    "{mode:?}[{i}]: got {} want {}",
                    got[i],
                    expected[i]
                );
            }
        }
    }

    // --- compositing pipeline ----------------------------------------------

    #[test]
    fn multi_layer_source_over_scene() {
        let d = doc(
            2,
            2,
            vec![
                solid(
                    "white",
                    full(2, 2),
                    (255, 255, 255),
                    255,
                    BlendMode::Normal,
                    255,
                ),
                solid(
                    "red",
                    full(2, 2),
                    (255, 0, 0),
                    255,
                    BlendMode::Multiply,
                    255,
                ),
                solid("blue", full(2, 2), (0, 0, 255), 255, BlendMode::Screen, 255),
            ],
        );
        let out = composite_rgba(&d);
        for y in 0..2 {
            for x in 0..2 {
                assert_eq!(px(&out, x, y), [255, 0, 255, 255], "at {x},{y}");
            }
        }
    }

    #[test]
    fn blend_ignored_over_transparent_backdrop() {
        let d = doc(
            1,
            1,
            vec![solid(
                "top",
                full(1, 1),
                (255, 0, 0),
                255,
                BlendMode::Multiply,
                255,
            )],
        );
        assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 255]);
    }

    #[test]
    fn layer_opacity_scales_over_backdrop() {
        // 1x1: base 200 opaque, top 0/200/0 at 50% opacity, Normal.
        let d = doc(
            1,
            1,
            vec![
                solid(
                    "base",
                    full(1, 1),
                    (200, 100, 50),
                    255,
                    BlendMode::Normal,
                    255,
                ),
                solid("top", full(1, 1), (0, 200, 0), 255, BlendMode::Normal, 128),
            ],
        );
        assert_eq!(px(&composite_rgba(&d), 0, 0), [100, 150, 25, 255]);
    }

    #[test]
    fn masked_layer_zeroes_masked_alpha() {
        let lw = 1u32;
        let mut top = solid(
            "masked",
            full(1, 2),
            (255, 0, 0),
            255,
            BlendMode::Normal,
            255,
        );
        top.mask = Some(LayerMask {
            rect: full(1, 2),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: Some(vec![0, 255]),
        });
        let _ = lw;
        let d = doc(1, 2, vec![top]);
        let out = composite_rgba(&d);
        assert_eq!(px(&out, 0, 0), [0, 0, 0, 0]);
        assert_eq!(px(&out, 0, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn disabled_mask_is_ignored() {
        let mut top = solid(
            "masked",
            full(1, 1),
            (255, 0, 0),
            255,
            BlendMode::Normal,
            255,
        );
        top.mask = Some(LayerMask {
            rect: full(1, 1),
            default_color: 0,
            disabled: true,
            flags: 0x02,
            data: Some(vec![0]),
        });
        let d = doc(1, 1, vec![top]);
        assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 255]);
    }

    #[test]
    fn isolated_group_blends_as_single_layer() {
        // Backdrop 200; group Multiply containing an opaque 50 child. Isolation
        // means the child is Normal inside the group, then Multiply applies:
        // 200/255 * 50/255 = 0.1537 -> 39.
        let d = doc(
            1,
            1,
            vec![
                solid(
                    "backdrop",
                    full(1, 1),
                    (200, 200, 200),
                    255,
                    BlendMode::Normal,
                    255,
                ),
                group(
                    "group",
                    BlendMode::Multiply,
                    255,
                    None,
                    vec![solid(
                        "child",
                        full(1, 1),
                        (50, 50, 50),
                        255,
                        BlendMode::Normal,
                        255,
                    )],
                ),
            ],
        );
        assert_eq!(px(&composite_rgba(&d), 0, 0), [39, 39, 39, 255]);
    }

    #[test]
    fn group_opacity_scales_isolated_result() {
        let d = doc(
            1,
            1,
            vec![group(
                "group",
                BlendMode::Normal,
                128,
                None,
                vec![solid(
                    "child",
                    full(1, 1),
                    (255, 0, 0),
                    255,
                    BlendMode::Normal,
                    255,
                )],
            )],
        );
        assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 128]);
    }

    #[test]
    fn pass_through_group_equals_ungrouped_stack() {
        let backdrop = solid(
            "backdrop",
            full(2, 2),
            (200, 120, 40),
            255,
            BlendMode::Normal,
            255,
        );
        let child = solid(
            "child",
            full(2, 2),
            (50, 180, 90),
            255,
            BlendMode::Multiply,
            255,
        );
        let grouped = doc(
            2,
            2,
            vec![
                backdrop.clone(),
                group(
                    "group",
                    BlendMode::PassThrough,
                    255,
                    None,
                    vec![child.clone()],
                ),
            ],
        );
        let ungrouped = doc(2, 2, vec![backdrop, child]);
        assert_eq!(
            composite_rgba(&grouped).data,
            composite_rgba(&ungrouped).data,
            "pass-through group must equal the flattened stack"
        );
    }

    #[test]
    fn pass_through_group_with_opacity_falls_back_to_isolated() {
        // Approximation: pass-through + opacity is isolated with mode Normal.
        let d = doc(
            1,
            1,
            vec![group(
                "group",
                BlendMode::PassThrough,
                128,
                None,
                vec![solid(
                    "child",
                    full(1, 1),
                    (255, 0, 0),
                    255,
                    BlendMode::Normal,
                    255,
                )],
            )],
        );
        assert_eq!(px(&composite_rgba(&d), 0, 0), [255, 0, 0, 128]);
    }

    #[test]
    fn bounds_and_negative_rect_clipping() {
        let mut d = Document::new(2, 2, ColorMode::Rgb, BitDepth::Eight);
        d.layers = vec![
            // Covers only canvas (0,0): local (1,1) of a 2x2 rect anchored at -1,-1.
            solid(
                "neg",
                rect(-1, -1, 1, 1),
                (255, 0, 0),
                255,
                BlendMode::Normal,
                255,
            ),
            // Fully off-canvas, contributes nothing and must not panic.
            solid(
                "off",
                rect(-9, -9, -1, -1),
                (0, 255, 0),
                255,
                BlendMode::Normal,
                255,
            ),
            // Zero-area layer.
            solid(
                "empty",
                rect(1, 1, 1, 1),
                (0, 0, 255),
                255,
                BlendMode::Normal,
                255,
            ),
        ];
        let out = composite_rgba(&d);
        assert_eq!(px(&out, 0, 0), [255, 0, 0, 255]);
        assert_eq!(px(&out, 1, 0), [0, 0, 0, 0]);
        assert_eq!(px(&out, 0, 1), [0, 0, 0, 0]);
        assert_eq!(px(&out, 1, 1), [0, 0, 0, 0]);
    }

    #[test]
    fn positive_offset_rect_lands_in_canvas() {
        let d = doc(
            3,
            2,
            vec![solid(
                "offset",
                rect(1, 1, 2, 2),
                (10, 20, 30),
                255,
                BlendMode::Normal,
                255,
            )],
        );
        let out = composite_rgba(&d);
        assert_eq!(px(&out, 1, 1), [10, 20, 30, 255]);
        assert_eq!(px(&out, 0, 0), [0, 0, 0, 0]);
        assert_eq!(px(&out, 2, 1), [0, 0, 0, 0]);
    }

    #[test]
    fn empty_layer_stack_is_transparent() {
        let out = composite_rgba(&doc(2, 2, Vec::new()));
        assert_eq!(out.channels, 4);
        assert!(out.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn dissolve_is_binary_and_deterministic() {
        let d = doc(
            8,
            8,
            vec![solid(
                "noise",
                full(8, 8),
                (255, 0, 0),
                255,
                BlendMode::Dissolve,
                128,
            )],
        );
        let a = composite_rgba(&d);
        let b = composite_rgba(&d);
        assert_eq!(a.data, b.data, "dissolve must be deterministic");
        let mut sources = 0;
        for y in 0..8 {
            for x in 0..8 {
                let p = px(&a, x, y);
                assert!(
                    p == [255, 0, 0, 255] || p == [0, 0, 0, 0],
                    "dissolve pixel must be binary, got {p:?}"
                );
                if p == [255, 0, 0, 255] {
                    sources += 1;
                }
            }
        }
        assert!(sources > 0 && sources < 64, "expected a mix, got {sources}");
    }

    #[test]
    fn grayscale_layer_replicates_channel() {
        let mut d = Document::new(1, 1, ColorMode::Grayscale, BitDepth::Eight);
        d.layers = vec![Layer {
            name: "gray".into(),
            rect: full(1, 1),
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![Channel {
                id: 0,
                data: vec![120],
            }],
            children: Vec::new(),
            is_group: false,
        }];
        assert_eq!(px(&composite_rgba(&d), 0, 0), [120, 120, 120, 255]);
    }

    // --- M4-B: adjustment layers ------------------------------------------

    fn adjdata(key: [u8; 4], data: Vec<u8>) -> AdjustmentData {
        AdjustmentData { key, data }
    }

    fn adjustment_layer(
        name: &str,
        key: [u8; 4],
        data: Vec<u8>,
        opacity: u8,
        mask: Option<LayerMask>,
    ) -> Layer {
        Layer {
            name: name.into(),
            rect: rect(0, 0, 0, 0),
            blend: BlendMode::Normal,
            opacity,
            clipping: false,
            visible: true,
            mask,
            adjustment: Some(adjdata(key, data)),
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
        }
    }

    fn rgb(buf: &PixelBuffer, x: u32, y: u32) -> [u8; 3] {
        let p = px(buf, x, y);
        [p[0], p[1], p[2]]
    }

    #[test]
    fn decode_adjustment_subset_and_unknown() {
        assert_eq!(
            decode_adjustment(&adjdata(*b"nvrt", vec![])),
            Some(Adjustment::Invert)
        );
        assert_eq!(
            decode_adjustment(&adjdata(*b"invr", vec![])),
            Some(Adjustment::Invert)
        );
        assert_eq!(
            decode_adjustment(&adjdata(*b"post", vec![0, 4, 0, 0])),
            Some(Adjustment::Posterize(4))
        );
        assert_eq!(
            decode_adjustment(&adjdata(*b"thrs", vec![0, 128, 0, 0])),
            Some(Adjustment::Threshold(128))
        );
        assert_eq!(
            decode_adjustment(&adjdata(*b"brit", vec![0, 10, 0, 20, 0, 0, 0, 0])),
            Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 10,
                contrast: 20,
                use_legacy: false,
            }))
        );

        let mut levels = vec![0, 2];
        for v in [5u16, 250, 10, 240, 120] {
            levels.extend_from_slice(&v.to_be_bytes());
        }
        assert_eq!(
            decode_adjustment(&adjdata(*b"levl", levels)),
            Some(Adjustment::Levels(LevelsParams {
                input_black: 5,
                input_white: 250,
                gamma: 1.2,
                output_black: 10,
                output_white: 240,
            }))
        );

        let hue = vec![0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 10, 0, 20, 0, 30];
        assert_eq!(
            decode_adjustment(&adjdata(*b"hue2", hue)),
            Some(Adjustment::HueSaturation(HueSaturationParams {
                hue: 10,
                saturation: 20,
                lightness: 30,
            }))
        );

        // Unknown key and undecodable payloads are a no-op, never an error.
        assert_eq!(decode_adjustment(&adjdata(*b"zzzz", vec![1, 2, 3])), None);
        assert_eq!(decode_adjustment(&adjdata(*b"clrL", vec![1, 2, 3])), None);
        assert_eq!(decode_adjustment(&adjdata(*b"post", vec![0, 0])), None);
        assert_eq!(decode_adjustment(&adjdata(*b"levl", vec![0, 3])), None);
    }

    #[test]
    fn encode_decode_round_trips() {
        assert_eq!(
            decode_adjustment(&encode_invert()),
            Some(Adjustment::Invert)
        );
        assert_eq!(
            decode_adjustment(&encode_posterize(4)),
            Some(Adjustment::Posterize(4))
        );
        assert_eq!(
            decode_adjustment(&encode_threshold(128)),
            Some(Adjustment::Threshold(128))
        );
        assert_eq!(
            decode_adjustment(&encode_brightness_contrast(10, 20)),
            Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 10,
                contrast: 20,
                use_legacy: false,
            }))
        );
        assert_eq!(
            decode_adjustment(&encode_hue_saturation(10, 20, 30)),
            Some(Adjustment::HueSaturation(HueSaturationParams {
                hue: 10,
                saturation: 20,
                lightness: 30,
            }))
        );

        // Byte formats match the psd-tools fixtures (`H2x`, `3HBx`).
        assert_eq!(encode_invert().key, *b"nvrt");
        assert!(encode_invert().data.is_empty());
        assert_eq!(encode_posterize(4).data, [0, 4, 0, 0]);
        assert_eq!(encode_threshold(128).data, [0, 128, 0, 0]);
        assert_eq!(
            encode_brightness_contrast(10, 20).data,
            [0, 10, 0, 20, 0, 0, 0, 0]
        );
        assert_eq!(encode_hue_saturation(10, 20, 30).data.len(), 100);
        assert_eq!(encode_hue_saturation(10, 20, 30).data[0..2], [0, 2]);

        // Out-of-range inputs are clamped to what the decoder accepts.
        assert_eq!(
            decode_adjustment(&encode_posterize(0)),
            Some(Adjustment::Posterize(2))
        );
        assert_eq!(
            decode_adjustment(&encode_threshold(0)),
            Some(Adjustment::Threshold(1))
        );
        assert_eq!(
            decode_adjustment(&encode_brightness_contrast(999, -999)),
            Some(Adjustment::BrightnessContrast(BrightnessContrastParams {
                brightness: 150,
                contrast: -50,
                use_legacy: false,
            }))
        );
    }

    #[test]
    fn invert_adjustment_layer_matches_flattened() {
        // Varied RGB backdrop so Invert is not a uniform all-zero/all-one case.
        let base = solid(
            "base",
            full(4, 2),
            (30, 90, 210),
            255,
            BlendMode::Normal,
            255,
        );
        let with_adj = doc(
            4,
            2,
            vec![
                base.clone(),
                adjustment_layer("invert", *b"nvrt", Vec::new(), 255, None),
            ],
        );
        let out = composite_rgba(&with_adj);
        // Expected: apply the destructive Adjustment to the flattened composite.
        let mut flat = composite_rgba(&doc(4, 2, vec![base]));
        pictura_adjust::apply(&Adjustment::Invert, &mut flat).unwrap();
        for y in 0..2 {
            for x in 0..4 {
                let want = rgb(&flat, x, y);
                let got = rgb(&out, x, y);
                for c in 0..3 {
                    let d = got[c] as i16 - want[c] as i16;
                    assert!(d.abs() <= 1, "at {x},{y}.{c}: got {got:?} want {want:?}");
                }
            }
        }
    }

    #[test]
    fn adjustment_layer_mask_and_opacity_gate() {
        // 2x1: mask fully reveals x=0, hides x=1.
        let base = solid(
            "base",
            full(2, 1),
            (100, 100, 100),
            255,
            BlendMode::Normal,
            255,
        );
        let mut masked = adjustment_layer("invert", *b"nvrt", Vec::new(), 255, None);
        masked.mask = Some(LayerMask {
            rect: full(2, 1),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: Some(vec![255, 0]),
        });
        let out = composite_rgba(&doc(2, 1, vec![base, masked]));
        assert_eq!(rgb(&out, 0, 0), [155, 155, 155], "unmasked pixel inverts");
        assert_eq!(
            rgb(&out, 1, 0),
            [100, 100, 100],
            "masked-out pixel unchanged"
        );

        // Opacity 128 lerps about halfway to the inverted value.
        let base = solid(
            "base",
            full(1, 1),
            (100, 100, 100),
            255,
            BlendMode::Normal,
            255,
        );
        let out = composite_rgba(&doc(
            1,
            1,
            vec![
                base,
                adjustment_layer("invert", *b"nvrt", Vec::new(), 128, None),
            ],
        ));
        let got = rgb(&out, 0, 0)[0] as i16;
        assert!(
            (got - 128).abs() <= 1,
            "50% opacity should land near 128, got {got}"
        );
    }

    #[test]
    fn unknown_adjustment_key_is_noop() {
        let base = solid(
            "base",
            full(2, 2),
            (10, 200, 60),
            255,
            BlendMode::Normal,
            255,
        );
        let plain = composite_rgba(&doc(2, 2, vec![base.clone()]));
        let with_unknown = composite_rgba(&doc(
            2,
            2,
            vec![
                base,
                adjustment_layer("lookup", *b"clrL", vec![1, 2, 3], 255, None),
            ],
        ));
        assert_eq!(
            with_unknown.data, plain.data,
            "undecodable key must be a no-op"
        );
    }
}
