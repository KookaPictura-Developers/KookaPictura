//! Native-depth compositing: the high-depth adjustment branch and the
//! source-depth composite output. Split from `composite.rs` for the file-size
//! cap.

use pictura_adjust::{AdjustError, Adjustment};
use pictura_core::{BitDepth, Document, Layer, PixelBuffer, Samples};

use crate::composite::{blend_into, composite_layers, to_u8, Canvas};

/// The layer's native sample for channel `id` at layer-pixel index `li`, in the
/// unit `f32` domain, when the document is a high-depth Grayscale/RGB read
/// (`source_depth` set, `source_mode` unset) and this layer retains a store
/// matching its rect and that depth. `None` otherwise, so the caller keeps the
/// 8-bit channel path. A converted mode (Lab/CMYK) is excluded because its
/// store holds source-mode planes, not working RGB.
pub(crate) fn native_unit(layer: &Layer, doc: &Document, id: i16, li: usize) -> Option<f32> {
    let depth = doc.source_depth?;
    if doc.source_mode.is_some() {
        return None;
    }
    let store = layer.source_channels.as_ref()?;
    if store.rect != layer.rect || store.depth != depth {
        return None;
    }
    let (_, samples) = store.planes.iter().find(|(cid, _)| *cid == id)?;
    sample_unit(samples, li)
}

fn sample_unit(samples: &Samples, i: usize) -> Option<f32> {
    match samples {
        Samples::U16(v) => v.get(i).map(|&x| x as f32 / 65535.0),
        Samples::F32(v) => v.get(i).map(|&x| x.clamp(0.0, 1.0)),
        Samples::U8(v) => v.get(i).map(|&x| x as f32 / 255.0),
    }
}

/// Apply a decoded adjustment to the running backdrop, then gate the result by
/// the layer's mask/opacity/blend (Photoshop applies the adjustment to the
/// backdrop and blends the adjusted result back).
pub(crate) fn composite_adjustment(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
    adjustment: &Adjustment,
) {
    // Fill content is generative: it adds color inside the layer's rect instead
    // of transforming the backdrop, so it takes the normal-content path.
    if let Adjustment::SolidFill(rgba) = adjustment {
        crate::fill::composite_solid_fill(canvas, layer, *rgba);
        return;
    }
    if let Adjustment::GradientFill(params) = adjustment {
        crate::fill::composite_gradient_fill(canvas, layer, params);
        return;
    }
    if let Adjustment::PatternFill(params) = adjustment {
        crate::fill::composite_pattern_fill(canvas, layer, doc, params);
        return;
    }
    let n = canvas.w * canvas.h;
    if n == 0 {
        return;
    }
    // A document read at 16/32 applies the adjustment to the f32 accumulator
    // at native precision. An adjustment outside `apply_native`'s set falls
    // back to the 8-bit path below.
    if doc.source_depth.is_some() {
        match apply_native_rgb(canvas, adjustment) {
            Ok(adjusted) => {
                gate_adjusted(canvas, layer, &adjusted);
                return;
            }
            Err(AdjustError::Unsupported(_)) => {}
            Err(_) => return, // invalid parameters: no-op, matching the 8-bit path
        }
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
    for y in canvas.y0()..canvas.y1() {
        for x in canvas.x0()..canvas.x1() {
            let i = canvas.idx(x as usize, y as usize);
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
            blend_into(canvas, layer, x as usize, y as usize, cs, backdrop_alpha);
        }
    }
}

/// Apply `adjustment` to the canvas RGB at native precision through
/// `apply_native` on an `f32` sample store, returning the adjusted color per
/// pixel. `canvas` (the original backdrop) is left untouched so the layer gate
/// still sees it. Alpha is untouched.
fn apply_native_rgb(
    canvas: &Canvas,
    adjustment: &Adjustment,
) -> Result<Vec<[f32; 3]>, AdjustError> {
    let n = canvas.w * canvas.h;
    let mut samples = Vec::with_capacity(n * 3);
    for p in &canvas.px {
        samples.push(p.r);
    }
    for p in &canvas.px {
        samples.push(p.g);
    }
    for p in &canvas.px {
        samples.push(p.b);
    }
    let mut store = Samples::F32(samples);
    pictura_adjust::apply_native(adjustment, &mut store, canvas.w, canvas.h, 3)?;
    let Samples::F32(v) = store else {
        return Err(AdjustError::InvalidParams(
            "native store changed type".into(),
        ));
    };
    Ok((0..n).map(|i| [v[i], v[n + i], v[2 * n + i]]).collect())
}

/// Gate the adjusted color through the layer's mask/opacity/blend, blending it
/// over the original backdrop still held in `canvas.px`.
fn gate_adjusted(canvas: &mut Canvas, layer: &Layer, adjusted: &[[f32; 3]]) {
    for y in canvas.y0()..canvas.y1() {
        for x in canvas.x0()..canvas.x1() {
            let i = canvas.idx(x as usize, y as usize);
            let backdrop_alpha = canvas.px[i].a;
            if backdrop_alpha <= 0.0 {
                continue;
            }
            blend_into(
                canvas,
                layer,
                x as usize,
                y as usize,
                adjusted[i],
                backdrop_alpha,
            );
        }
    }
}

/// The document's composite at its recorded source depth: `Samples::U16` at 16
/// and `Samples::F32` at 32, in planar RGBA order with straight alpha clamped to
/// `[0, 1]`. `None` for a document with no recorded 16/32-bit source depth.
pub fn composite_native(doc: &Document) -> Option<Samples> {
    let depth = doc.source_depth?;
    if !matches!(depth, BitDepth::Sixteen | BitDepth::ThirtyTwo) {
        return None;
    }
    let mut canvas = Canvas::new(doc.width as usize, doc.height as usize);
    composite_layers(&mut canvas, doc);
    Some(emit_native(&canvas, depth))
}

fn emit_native(canvas: &Canvas, depth: BitDepth) -> Samples {
    let n = canvas.w * canvas.h;
    match depth {
        BitDepth::Sixteen => {
            let mut out = vec![0u16; n * 4];
            for (i, p) in canvas.px.iter().enumerate() {
                out[i] = to_u16(p.r);
                out[n + i] = to_u16(p.g);
                out[2 * n + i] = to_u16(p.b);
                out[3 * n + i] = to_u16(p.a);
            }
            Samples::U16(out)
        }
        _ => {
            let mut out = vec![0f32; n * 4];
            for (i, p) in canvas.px.iter().enumerate() {
                out[i] = p.r.clamp(0.0, 1.0);
                out[n + i] = p.g.clamp(0.0, 1.0);
                out[2 * n + i] = p.b.clamp(0.0, 1.0);
                out[3 * n + i] = p.a.clamp(0.0, 1.0);
            }
            Samples::F32(out)
        }
    }
}

fn to_u16(v: f32) -> u16 {
    (v.clamp(0.0, 1.0) * 65535.0).round() as u16
}
