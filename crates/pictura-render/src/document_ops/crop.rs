//! `Image > Crop` and layer translation at document scope.
//!
//! A crop is a canvas resize with a negative offset: the new origin is the
//! clamped rect's top-left, so every rect shifts by `(-x, -y)` and every
//! document channel is re-blitted into the smaller canvas.

use pictura_core::{Document, Layer};

use super::canvas::{extend_channel, offset_rect};
use super::{for_each_layer, recompute};

/// Crop the document to the clamped `width`×`height` rect at `(x, y)`.
///
/// Returns false and leaves the document untouched when the intersection with
/// the canvas is empty.
pub fn crop_document(doc: &mut Document, x: i32, y: i32, width: u32, height: u32) -> bool {
    let old_w = doc.width;
    let old_h = doc.height;
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = x.saturating_add(width as i32).min(old_w as i32);
    let y1 = y.saturating_add(height as i32).min(old_h as i32);
    if x1 <= x0 || y1 <= y0 {
        return false;
    }
    let new_w = (x1 - x0) as u32;
    let new_h = (y1 - y0) as u32;
    let (dx, dy) = (-x0, -y0);

    for_each_layer(&mut doc.layers, &mut |layer| {
        layer.rect = offset_rect(layer.rect, dx, dy);
        if let Some(mask) = &mut layer.mask {
            mask.rect = offset_rect(mask.rect, dx, dy);
        }
    });

    for channel in &mut doc.channels {
        channel.data = extend_channel(&channel.data, old_w, old_h, new_w, new_h, dx, dy);
    }

    doc.width = new_w;
    doc.height = new_h;
    recompute(doc);
    true
}

/// Shift the topmost pixel layer's bounds by `(dx, dy)`.
///
/// Groups and adjustment layers are ignored; no pixels move, the compositor
/// reads the shifted `rect`. Returns false when there is no pixel layer.
pub fn translate_layer(doc: &mut Document, dx: i32, dy: i32) -> bool {
    let Some(layer) = topmost_pixel_layer(&mut doc.layers) else {
        return false;
    };
    layer.rect = offset_rect(layer.rect, dx, dy);
    if let Some(mask) = &mut layer.mask {
        mask.rect = offset_rect(mask.rect, dx, dy);
    }
    recompute(doc);
    true
}

/// Shift the topmost pixel layer's bounds by `(dx, dy)` without recompositing.
///
/// Same rect/mask shift as [`translate_layer`]; the caller is expected to
/// refresh the dirty region through the active backend. Use [`translate_layer`]
/// or [`translate_layer_active`] when a full recomposite is wanted. Returns
/// false when there is no pixel layer.
pub fn translate_layer_rect(doc: &mut Document, dx: i32, dy: i32) -> bool {
    let Some(layer) = topmost_pixel_layer(&mut doc.layers) else {
        return false;
    };
    layer.rect = offset_rect(layer.rect, dx, dy);
    if let Some(mask) = &mut layer.mask {
        mask.rect = offset_rect(mask.rect, dx, dy);
    }
    true
}

/// Shift the topmost pixel layer's bounds by `(dx, dy)` and refresh the
/// composite through the active backend.
///
/// Same rect/mask shift as [`translate_layer`]; only the composite refresh
/// differs — `composite_active` uses the GPU when `gpu_enabled` and an adapter
/// is usable, falling back to the CPU oracle otherwise.
pub fn translate_layer_active(doc: &mut Document, dx: i32, dy: i32, gpu_enabled: bool) -> bool {
    let Some(layer) = topmost_pixel_layer(&mut doc.layers) else {
        return false;
    };
    layer.rect = offset_rect(layer.rect, dx, dy);
    if let Some(mask) = &mut layer.mask {
        mask.rect = offset_rect(mask.rect, dx, dy);
    }
    let (composite, _) = crate::gpu::composite_active(doc, gpu_enabled);
    doc.composite = composite;
    true
}

fn topmost_pixel_layer(layers: &mut [Layer]) -> Option<&mut Layer> {
    layers
        .iter_mut()
        .rev()
        .find(|l| l.adjustment.is_none() && !l.is_group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, BlendMode, Channel, ColorMode, LayerMask, PsdRect};

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

    fn pixel_layer(name: &str, r: PsdRect, mask: Option<LayerMask>) -> Layer {
        let n = (r.width().max(0) * r.height().max(0)) as usize;
        let ramp = |offset: u8| (0..n).map(|i| (i as u8).wrapping_add(offset)).collect();
        Layer {
            name: name.into(),
            rect: r,
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: ramp(0),
                },
                Channel {
                    id: 1,
                    data: ramp(100),
                },
                Channel {
                    id: 2,
                    data: ramp(200),
                },
                Channel {
                    id: -1,
                    data: vec![255; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
        }
    }

    fn sample_doc() -> Document {
        let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel_layer("layer", full(4, 4), None)];
        doc.channels = vec![Channel {
            id: -1,
            data: (0..16u8).collect(),
        }];
        doc
    }

    #[test]
    fn crop_remaps_pixel_and_resizes_document() {
        let mut doc = sample_doc();
        assert!(crop_document(&mut doc, 1, 1, 2, 2));

        assert_eq!((doc.width, doc.height), (2, 2));
        assert_eq!(doc.channels[0].data, vec![5, 6, 9, 10]);
        assert_eq!(doc.layers[0].rect, rect(-1, -1, 3, 3));

        // The layer rect moved to (-1,-1), so doc (0,0) samples local (1,1) = 5.
        let plane = 4usize;
        assert_eq!(doc.composite.width, 2);
        assert_eq!(doc.composite.height, 2);
        assert_eq!(doc.composite.data[0], 5);
        assert_eq!(doc.composite.data[plane], 105);
        assert_eq!(doc.composite.data[2 * plane], 205);
    }

    #[test]
    fn crop_clamps_and_shifts_out_of_bounds_rect() {
        let mut doc = sample_doc();
        assert!(crop_document(&mut doc, 3, 3, 4, 4));
        assert_eq!((doc.width, doc.height), (1, 1));
        assert_eq!(doc.channels[0].data, vec![15]);
        assert_eq!(doc.layers[0].rect, rect(-3, -3, 1, 1));
        assert_eq!(doc.composite.data[0], 15, "old (3,3) becomes the origin");
    }

    #[test]
    fn crop_empty_rect_is_rejected_and_unchanged() {
        let mut doc = sample_doc();
        let before = doc.clone();
        assert!(!crop_document(&mut doc, 10, 10, 2, 2));
        assert!(!crop_document(&mut doc, 0, 0, 0, 2));
        assert!(!crop_document(&mut doc, 0, 0, 2, 0));
        assert_eq!(doc, before);
    }

    #[test]
    fn crop_shifts_layer_mask() {
        let mask = LayerMask {
            rect: full(4, 4),
            default_color: 255,
            disabled: false,
            flags: 0,
            data: Some(vec![128; 16]),
        };
        let mut doc = sample_doc();
        doc.layers[0] = pixel_layer("masked", full(4, 4), Some(mask));
        assert!(crop_document(&mut doc, 1, 1, 2, 2));
        assert_eq!(
            doc.layers[0].mask.as_ref().unwrap().rect,
            rect(-1, -1, 3, 3)
        );
    }

    #[test]
    fn translate_layer_shifts_rect_and_recomposites() {
        let mut doc = sample_doc();
        let before = doc.composite.data[0];
        assert_eq!(before, 0, "old origin holds layer local (0,0)");

        assert!(translate_layer(&mut doc, 1, 1));
        assert_eq!(doc.layers[0].rect, rect(1, 1, 5, 5));

        let plane = 16usize;
        // Doc (1,1) now samples layer local (0,0); doc (2,1) samples local (1,0).
        assert_eq!(doc.composite.data[5], 0);
        assert_eq!(doc.composite.data[6], 1);
        assert_eq!(doc.composite.data[2 * 4 + 1], 4);
        // The old origin is no longer covered by the shifted layer.
        assert_eq!(doc.composite.data[3 * plane], 0, "uncovered alpha");
    }

    #[test]
    fn translate_layer_active_cpu_matches_translate_layer() {
        let mut active = sample_doc();
        let mut oracle = sample_doc();
        assert!(translate_layer_active(&mut active, 1, 1, false));
        assert!(translate_layer(&mut oracle, 1, 1));
        assert_eq!(active, oracle);
        assert_eq!(active.composite.data, oracle.composite.data);
    }

    #[test]
    fn translate_layer_rect_shifts_without_recompositing() {
        let mut doc = sample_doc();
        let before = doc.composite.clone();
        let mask = LayerMask {
            rect: full(4, 4),
            default_color: 255,
            disabled: false,
            flags: 0,
            data: Some(vec![128; 16]),
        };
        doc.layers[0] = pixel_layer("masked", full(4, 4), Some(mask));

        assert!(translate_layer_rect(&mut doc, 2, -1));
        assert_eq!(doc.layers[0].rect, rect(-1, 2, 3, 6));
        assert_eq!(doc.layers[0].mask.as_ref().unwrap().rect, rect(-1, 2, 3, 6));
        assert_eq!(doc.composite.data, before.data, "no recomposite");
    }

    #[test]
    fn translate_layer_rejects_non_pixel_layers() {
        let mut doc = sample_doc();
        doc.layers = vec![Layer {
            name: "adj".into(),
            rect: full(4, 4),
            blend: BlendMode::Normal,
            opacity: 255,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: Some(pictura_core::AdjustmentData {
                key: *b"nvrt",
                data: vec![],
            }),
            channels: Vec::new(),
            children: Vec::new(),
            is_group: false,
        }];
        let before = doc.clone();
        assert!(!translate_layer(&mut doc, 2, 2));
        assert_eq!(doc, before);
    }

    /// Manual M31 evidence: a small region composite against the full 4000²
    /// composite through the active backend. Ignored by default (it allocates a
    /// 4000² layer); run with `--ignored --nocapture`.
    #[test]
    #[ignore = "manual 4000x4000 region-vs-full timing measurement"]
    fn region_move_timing_4000() {
        use std::time::Instant;

        let (w, h) = (4000u32, 4000u32);
        let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![pixel_layer(
            "big",
            PsdRect {
                top: 0,
                left: 0,
                bottom: h as i32,
                right: w as i32,
            },
            None,
        )];
        doc.composite = crate::composite_rgba(&doc);

        let full = Instant::now();
        let _ = crate::composite_active(&doc, true);
        let full_ms = full.elapsed().as_secs_f64() * 1000.0;

        let rect = PsdRect {
            top: 1000,
            left: 1000,
            bottom: 1064,
            right: 1064,
        };
        let region = Instant::now();
        let (buffer, backend) = crate::gpu::composite_region_active(&doc, rect, true);
        let region_ms = region.elapsed().as_secs_f64() * 1000.0;

        assert_eq!((buffer.width, buffer.height), (64, 64));
        eprintln!(
            "m31 timing 4000^2 backend={backend:?} full={full_ms:.1}ms region64={region_ms:.3}ms"
        );
    }
}
