//! Apply a smart object's `filterFX` chain to its rendered source.
//!
//! Phase 1 models the camera-raw smart filter only. Unknown ids are skipped, so
//! an unsupported stack leaves the caller on its proxy/source path unchanged.

use pictura_adjust::render_pictura_raw;
use pictura_codec::{decode_pictura_raw_settings, CAMERA_RAW_FILTER_ID};
use pictura_core::{
    Document, Layer, LayerMask, PicturaRawSettings, PixelBuffer, PsdRect, SmartFilter,
};

use crate::composite::{blend_into, render_smart_source, Canvas};

/// A decoded smart filter ready to render.
#[derive(Debug, Clone)]
pub enum SmartFilterOp {
    CameraRaw(PicturaRawSettings),
}

/// Decode a [`SmartFilter`]'s options into a renderable op, or `None` for an
/// id this engine does not model.
pub fn decode_smart_filter(filter: &SmartFilter) -> Option<SmartFilterOp> {
    match filter.filter_id {
        CAMERA_RAW_FILTER_ID => Some(SmartFilterOp::CameraRaw(decode_pictura_raw_settings(
            &filter.options,
        ))),
        _ => None,
    }
}

/// Apply the enabled smart filters to `base`, in order, over the colour planes;
/// alpha is preserved.
///
/// `base` covers `rect` (document coordinates). When `mask` is given, each
/// pixel is lerped from the unfiltered `base` toward the filtered result by the
/// mask coverage, so black mask pixels keep the source and white pixels take
/// the filter.
pub fn apply_smart_filter_chain(
    base: &PixelBuffer,
    rect: PsdRect,
    filters: &[SmartFilter],
    mask: Option<&LayerMask>,
) -> PixelBuffer {
    let mut out = base.clone();
    for filter in filters.iter().filter(|f| f.enabled) {
        if let Some(op) = decode_smart_filter(filter) {
            out = apply_op(&out, op);
        }
    }
    if let Some(mask) = mask {
        apply_mask(base, &mut out, rect, mask);
    }
    out
}

fn apply_op(base: &PixelBuffer, op: SmartFilterOp) -> PixelBuffer {
    match op {
        SmartFilterOp::CameraRaw(settings) => {
            render_pictura_raw(base, &settings).unwrap_or_else(|_| base.clone())
        }
    }
}

fn apply_mask(base: &PixelBuffer, out: &mut PixelBuffer, rect: PsdRect, mask: &LayerMask) {
    let w = out.width as i32;
    let plane = (out.width * out.height) as usize;
    for y in 0..out.height as i32 {
        for x in 0..w {
            let t = crate::composite::mask_value(mask, rect.left + x, rect.top + y) as f32 / 255.0;
            let i = (y * w + x) as usize;
            for c in 0..3 {
                let from = base.data[c * plane + i] as f32;
                let to = out.data[c * plane + i] as f32;
                out.data[c * plane + i] = (from + (to - from) * t).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Composite a smart object's filtered source, rendering the source first so an
/// already-baked raster proxy is never filtered a second time.
///
/// Returns `false` (leaving the layer to the normal proxy/source path) unless
/// there is at least one enabled filter and every enabled filter is understood.
pub(crate) fn composite_smart_filtered_source(
    canvas: &mut Canvas,
    layer: &Layer,
    doc: &Document,
) -> bool {
    let Some(so) = layer.smart_object.as_ref() else {
        return false;
    };
    let mut enabled = 0usize;
    for filter in &so.smart_filters {
        if !filter.enabled {
            continue;
        }
        if decode_smart_filter(filter).is_none() {
            return false;
        }
        enabled += 1;
    }
    if enabled == 0 {
        return false;
    }
    let region = PsdRect {
        top: layer.rect.top.max(canvas.y0()),
        left: layer.rect.left.max(canvas.x0()),
        bottom: layer.rect.bottom.min(canvas.y1()),
        right: layer.rect.right.min(canvas.x1()),
    };
    if region.width() <= 0 || region.height() <= 0 {
        return false;
    }
    let Some(base) = render_smart_source(so, layer.rect, region) else {
        return false;
    };
    let src = apply_smart_filter_chain(&base, region, &so.smart_filters, so.filter_mask.as_ref());

    let rw = region.width() as usize;
    let rh = region.height() as usize;
    let plane = rw * rh;
    for by in 0..rh {
        for bx in 0..rw {
            let i = by * rw + bx;
            blend_into(
                canvas,
                layer,
                doc,
                region.left as usize + bx,
                region.top as usize + by,
                [
                    src.data[i] as f32 / 255.0,
                    src.data[plane + i] as f32 / 255.0,
                    src.data[2 * plane + i] as f32 / 255.0,
                ],
                src.data[3 * plane + i] as f32 / 255.0,
            );
        }
    }
    true
}
