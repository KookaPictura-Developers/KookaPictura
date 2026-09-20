use pictura_codec::DescValue;
use pictura_core::{BlendMode, Document, Layer};

use crate::composite::{blend_parts, desc_item, mask_alpha, Canvas};

use super::{
    bool_or, clamp_finite, clip_rect, content_matte, decode_color, dilate_matte, effect_blend_mode,
    erode_matte, num_clamped, pad_rect, rect_empty, MAX_OPACITY, MAX_SIZE,
};

/// The stroke position stored in `Styl` (typeID `FStl`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokePosition {
    /// `OutF`: the band is built outside the content edge.
    Outside,
    /// `InsF`: the band is built inside the content edge.
    Inside,
    /// `CtrF`: the band straddles the content edge.
    Center,
}

/// The typed stroke decoded from a layer's `lfx2` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub enabled: bool,
    pub present: bool,
    pub blend_mode: BlendMode,
    pub color: [u8; 3],
    /// Percent, `0..=100`.
    pub opacity: f32,
    /// Pixels, integer `1..=250`.
    pub size: u32,
    /// `Styl` (typeID `FStl`), default Outside.
    pub position: StrokePosition,
}

/// Decode a layer's `lfx2` Stroke (`FrFX`).
///
/// ponytail: only the solid-colour fill (`PntT` `FrFl`/`SClr`) is decoded; a
/// `GrFl` gradient or `Ptrn` pattern fill is `None`. Contour (`TrnS`),
/// anti-alias (`AntA`) and `overprint` are ignored. The colour defaults to
/// black rather than the docs' "foreground `(inferred)`", which this crate
/// cannot resolve. `size` is rounded to the nearest integer and clamped to
/// `1..=250`.
///
/// A missing `lfx2`, a missing or wrongly-typed `FrFX`, an unknown data
/// version, a wrong-typed or non-finite value, a non-solid fill type, or a
/// parse error is `None`. Never panics.
pub fn decode_stroke(layer: &Layer) -> Option<Stroke> {
    let data = &layer.extra_block(b"lfx2")?.data;
    if data.len() < 8 {
        return None;
    }
    let body = data.get(4..)?;
    let obj = pictura_codec::read_descriptor(body).ok()?;
    let frfx = desc_item(&obj, b"FrFX")?;
    let DescValue::Object { class_id, .. } = frfx else {
        return None;
    };
    if class_id.as_slice() != b"FrFX" {
        return None;
    }
    let blend_mode = match desc_item(frfx, b"Md  ") {
        None => BlendMode::Normal,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"BlnM" => {
            effect_blend_mode(value, BlendMode::Normal)
        }
        Some(_) => return None,
    };
    let color = match desc_item(frfx, b"Clr ") {
        None => [0, 0, 0],
        Some(value) => decode_color(value)?,
    };
    let position = match desc_item(frfx, b"Styl") {
        None => StrokePosition::Outside,
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"FStl" => {
            match value.as_slice() {
                b"InsF" => StrokePosition::Inside,
                b"CtrF" => StrokePosition::Center,
                // `OutF`, or an unknown value, is Outside.
                _ => StrokePosition::Outside,
            }
        }
        Some(_) => return None,
    };
    match desc_item(frfx, b"PntT") {
        None => {}
        Some(DescValue::Enum { kind, value }) if kind.as_slice() == b"FrFl" => {
            // Only the solid-colour fill is rendered in this slice.
            if value.as_slice() != b"SClr" {
                return None;
            }
        }
        Some(_) => return None,
    }
    Some(Stroke {
        enabled: bool_or(frfx, b"enab", false)?,
        present: bool_or(frfx, b"present", false)?,
        blend_mode,
        color,
        opacity: num_clamped(frfx, b"Opct", 100.0, 0.0, MAX_OPACITY)?,
        size: num_clamped(frfx, b"Sz  ", 3.0, 1.0, MAX_SIZE)?.round() as u32,
        position,
    })
}

/// Build the masked content matte `M` over the padded source region, form the
/// band from exact integer max/min filters, and composite `band · opacity/100`
/// tinted by the stroke colour above the content over `padded` only.
///
/// ponytail: the band is composited after the interior effects, so it sits on
/// top of them; Photoshop's exact inter-effect order and the isolated `Blend
/// Interior Effects As Group` composite are not modelled. The max/min filters
/// are separable, so a canvas-filling layer at the maximum size costs
/// `O(canvas · size)` — the existing compositor ceiling.
pub(super) fn composite_stroke(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
    stroke: &Stroke,
) {
    let (w, h) = (canvas.w as i32, canvas.h as i32);
    if w == 0 || h == 0 {
        return;
    }
    let source = clip_rect(layer, w, h);
    if rect_empty(source) {
        return;
    }
    // Clamp again: a hand-built `Stroke` may not have gone through decode.
    let size = clamp_finite(stroke.size as f32, MAX_SIZE) as i64;
    let opacity = clamp_finite(stroke.opacity, MAX_OPACITY) / 100.0;
    if size == 0 || opacity == 0.0 {
        return;
    }
    let (out_r, in_r) = match stroke.position {
        StrokePosition::Outside => (size, 0),
        StrokePosition::Inside => (0, size),
        StrokePosition::Center => ((size + 1) / 2, size / 2),
    };
    // The matte is built over `source` padded by `size` on every side: the pad
    // bounds the outside half of the band and supplies the zero content beyond
    // the source edge that the erode min filter needs.
    let padded = pad_rect(source, size, w, h);
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
    let dilate = |r: i64| {
        if r > 0 {
            dilate_matte(&matte, pw, ph, r)
        } else {
            matte.clone()
        }
    };
    let erode = |r: i64| {
        if r > 0 {
            erode_matte(&matte, pw, ph, r)
        } else {
            matte.clone()
        }
    };
    let band: Vec<f32> = match stroke.position {
        StrokePosition::Outside => dilate(out_r)
            .iter()
            .zip(&matte)
            .map(|(d, m)| (d - m).clamp(0.0, 1.0))
            .collect(),
        StrokePosition::Inside => matte
            .iter()
            .zip(&erode(in_r))
            .map(|(m, e)| (m - e).clamp(0.0, 1.0))
            .collect(),
        StrokePosition::Center => {
            let d = dilate(out_r);
            let e = erode(in_r);
            d.iter()
                .zip(&e)
                .map(|(a, b)| (a - b).clamp(0.0, 1.0))
                .collect()
        }
    };
    let color = [
        stroke.color[0] as f32 / 255.0,
        stroke.color[1] as f32 / 255.0,
        stroke.color[2] as f32 / 255.0,
    ];
    for y in py0..py1 {
        for x in px0..px1 {
            let i = (y - py0) as usize * pw + (x - px0) as usize;
            let alpha = band[i] * opacity;
            if alpha > 0.0 {
                blend_parts(
                    canvas,
                    x as usize,
                    y as usize,
                    color,
                    alpha,
                    stroke.blend_mode,
                );
            }
        }
    }
}
