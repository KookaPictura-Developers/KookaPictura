//! `Image > Image Size` at document scope (`IMG-001`).

use super::{for_each_layer, recompute, resample_plane, valid_size};

/// Scale every layer rect, resample its channel planes and mask from the old
/// rect size to the new, then recompute `doc.composite = composite_rgba(doc)`.
pub fn resize_document(
    doc: &mut pictura_core::Document,
    width: u32,
    height: u32,
    resample: pictura_ops::Resample,
) -> Result<(), pictura_ops::OpsError> {
    valid_size(width, height)?;
    let old_w = doc.width;
    let old_h = doc.height;
    if old_w == 0 || old_h == 0 {
        return Err(pictura_ops::OpsError::InvalidParams(
            "document has a zero dimension".into(),
        ));
    }
    let sx = width as f64 / old_w as f64;
    let sy = height as f64 / old_h as f64;

    for_each_layer(&mut doc.layers, &mut |layer| {
        resize_layer(layer, sx, sy, resample);
    });

    for channel in &mut doc.channels {
        channel.data = resample_plane(&channel.data, old_w, old_h, width, height, resample);
    }

    doc.width = width;
    doc.height = height;
    recompute(doc);
    Ok(())
}

/// New dimensions for an old `w×h` rect under scale factors, floored at 1.
fn scaled_dims(w: i32, h: i32, sx: f64, sy: f64) -> (u32, u32) {
    let nw = ((w as f64 * sx).round() as i64).max(1) as u32;
    let nh = ((h as f64 * sy).round() as i64).max(1) as u32;
    (nw, nh)
}

/// Rescale a rect's origin, then size it from the resampled dimensions.
fn scaled_rect(
    rect: pictura_core::PsdRect,
    nw: u32,
    nh: u32,
    sx: f64,
    sy: f64,
) -> pictura_core::PsdRect {
    let top = (rect.top as f64 * sy).round() as i32;
    let left = (rect.left as f64 * sx).round() as i32;
    pictura_core::PsdRect {
        top,
        left,
        bottom: top + nh as i32,
        right: left + nw as i32,
    }
}

fn resize_layer(
    layer: &mut pictura_core::Layer,
    sx: f64,
    sy: f64,
    resample: pictura_ops::Resample,
) {
    let lw = layer.rect.width();
    let lh = layer.rect.height();
    if lw > 0 && lh > 0 {
        let (nw, nh) = scaled_dims(lw, lh, sx, sy);
        for channel in &mut layer.channels {
            channel.data = resample_plane(&channel.data, lw as u32, lh as u32, nw, nh, resample);
        }
        layer.rect = scaled_rect(layer.rect, nw, nh, sx, sy);
    }
    if let Some(mask) = &mut layer.mask {
        let mw = mask.rect.width();
        let mh = mask.rect.height();
        if mw > 0 && mh > 0 {
            if let Some(data) = &mut mask.data {
                let (nw, nh) = scaled_dims(mw, mh, sx, sy);
                *data = resample_plane(data, mw as u32, mh as u32, nw, nh, resample);
                mask.rect = scaled_rect(mask.rect, nw, nh, sx, sy);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{
        BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags,
        PsdRect,
    };

    fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
        PsdRect {
            top,
            left,
            bottom,
            right,
        }
    }

    fn pixel_layer(name: &str, r: PsdRect, mask: Option<LayerMask>) -> Layer {
        let n = (r.width().max(0) * r.height().max(0)) as usize;
        Layer {
            name: name.into(),
            rect: r,
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![10; n],
                },
                Channel {
                    id: 1,
                    data: vec![20; n],
                },
                Channel {
                    id: 2,
                    data: vec![30; n],
                },
                Channel {
                    id: -1,
                    data: vec![255; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        }
    }

    fn sample_doc() -> Document {
        let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
        let mask = LayerMask {
            rect: rect(0, 0, 4, 4),
            default_color: 255,
            disabled: false,
            flags: 0,
            data: Some(vec![128; 16]),
            ..Default::default()
        };
        doc.layers = vec![
            pixel_layer("full", rect(0, 0, 4, 4), None),
            pixel_layer("masked", rect(1, 1, 3, 3), Some(mask)),
        ];
        doc.channels = vec![Channel {
            id: -1,
            data: vec![7; 16],
        }];
        doc
    }

    #[test]
    fn resize_2x_scales_document_layers_mask_and_channel() {
        let mut doc = sample_doc();
        let before = doc.clone();
        resize_document(&mut doc, 8, 8, pictura_ops::Resample::Nearest).unwrap();

        assert_eq!((doc.width, doc.height), (8, 8));

        for (layer, old) in doc.layers.iter().zip(&before.layers) {
            assert!(layer.rect.width() > 0 && layer.rect.height() > 0);
            let area = (layer.rect.width() * layer.rect.height()) as usize;
            for channel in &layer.channels {
                assert_eq!(
                    channel.data.len(),
                    area,
                    "{} channel {}",
                    layer.name,
                    channel.id
                );
            }
            if let (Some(new_mask), Some(old_mask)) = (&layer.mask, &old.mask) {
                assert_eq!(new_mask.rect.width(), old_mask.rect.width() * 2);
                assert_eq!(new_mask.rect.height(), old_mask.rect.height() * 2);
                let marea = (new_mask.rect.width() * new_mask.rect.height()) as usize;
                assert_eq!(new_mask.data.as_ref().unwrap().len(), marea);
            }
        }
        assert_eq!(doc.channels[0].data.len(), 64);
        assert_eq!(doc.composite, crate::composite_rgba(&doc));
    }

    #[test]
    fn zero_width_is_invalid_and_unchanged() {
        let mut doc = sample_doc();
        let before = doc.clone();
        let err = resize_document(&mut doc, 0, 8, pictura_ops::Resample::Bilinear);
        assert!(matches!(err, Err(pictura_ops::OpsError::InvalidParams(_))));
        assert_eq!(doc, before);
    }

    #[test]
    fn double_then_half_restores_dims() {
        let mut doc = sample_doc();
        let (w0, h0) = (doc.width, doc.height);
        let dims0: Vec<(i32, i32)> = doc
            .layers
            .iter()
            .map(|l| (l.rect.width(), l.rect.height()))
            .collect();

        resize_document(&mut doc, 8, 8, pictura_ops::Resample::Nearest).unwrap();
        resize_document(&mut doc, 4, 4, pictura_ops::Resample::Nearest).unwrap();

        assert_eq!((doc.width, doc.height), (w0, h0));
        let dims1: Vec<(i32, i32)> = doc
            .layers
            .iter()
            .map(|l| (l.rect.width(), l.rect.height()))
            .collect();
        assert_eq!(dims0, dims1);
        for layer in &doc.layers {
            let area = (layer.rect.width() * layer.rect.height()) as usize;
            for channel in &layer.channels {
                assert_eq!(channel.data.len(), area);
            }
        }
    }
}
