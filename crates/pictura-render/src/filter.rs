//! M6-C: destructive filter application to a pixel layer's color channels.
//!
//! Contract: `docs/dev/m6c-filter-integration.md`. The filter runs on channels
//! `0,1,2` only; the transparency channel (`-1`) is never touched.

use pictura_core::{layer_pixel_locked, layer_transparency_locked, Layer, LayerMask, PixelBuffer};
use pictura_filters::{Filter, FilterError};

use crate::channel;

/// Apply a destructive `filter` to a pixel layer's color channels (0,1,2),
/// gated by an optional document-coordinate coverage `mask` (`None` = full
/// frame). The transparency channel (`-1`) is never modified.
///
/// When `gpu_enabled` is set the filter runs through
/// [`crate::gpu_filter::apply_filter_active`], which uses the GPU when a kernel
/// exists and falls back to the CPU oracle otherwise.
pub fn apply_filter(
    layer: &mut Layer,
    filter: &Filter,
    mask: Option<&LayerMask>,
    gpu_enabled: bool,
) -> Result<(), FilterError> {
    let lw = layer.rect.width();
    let lh = layer.rect.height();
    if lw <= 0 || lh <= 0 {
        return Ok(());
    }
    if layer_pixel_locked(layer) {
        return Err(FilterError::Locked);
    }
    let (lw, lh) = (lw as usize, lh as usize);
    let n = lw * lh;

    let mut buf = PixelBuffer::new(lw as u32, lh as u32, 3);
    for c in 0..3i16 {
        let data = channel(layer, c).ok_or_else(|| {
            FilterError::InvalidParams(format!("layer is missing color channel {c}"))
        })?;
        if data.len() != n {
            return Err(FilterError::InvalidParams(format!(
                "channel {c} has {} samples, expected {n}",
                data.len()
            )));
        }
        let plane = c as usize * n;
        buf.data[plane..plane + n].copy_from_slice(data);
    }

    let orig = buf.clone();
    crate::gpu_filter::apply_filter_active(filter, &mut buf, gpu_enabled)?;

    // A transparency lock preserves the alpha plane exactly and leaves fully
    // transparent pixels untouched; only opaque pixels take the filter color.
    let alpha = if layer_transparency_locked(layer) {
        channel(layer, -1).map(|d| d.to_vec())
    } else {
        None
    };

    let (left, top) = (layer.rect.left, layer.rect.top);
    for c in 0..3i16 {
        let plane = c as usize * n;
        let orig_plane = &orig.data[plane..plane + n];
        let filtered_plane = &buf.data[plane..plane + n];
        let out = channel_mut(layer, c).expect("channel presence validated above");
        for ly in 0..lh {
            for lx in 0..lw {
                let i = ly * lw + lx;
                if alpha.as_ref().is_some_and(|a| a[i] == 0) {
                    continue;
                }
                let cov = coverage(mask, left + lx as i32, top + ly as i32);
                if cov == 0 {
                    continue;
                }
                let o = orig_plane[i] as f64;
                let f = filtered_plane[i] as f64;
                out[i] = (o + (f - o) * cov as f64 / 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

fn channel_mut(layer: &mut Layer, id: i16) -> Option<&mut [u8]> {
    layer
        .channels
        .iter_mut()
        .find(|c| c.id == id)
        .map(|c| c.data.as_mut_slice())
}

/// Mask coverage at a document pixel. `255` when there is no mask or the mask
/// is disabled; outside the mask rect (or with no decoded data) the mask's
/// `default_color` applies.
fn coverage(mask: Option<&LayerMask>, x: i32, y: i32) -> u8 {
    let Some(mask) = mask else {
        return 255;
    };
    if mask.disabled {
        return 255;
    }
    let (mw, mh) = (mask.rect.width(), mask.rect.height());
    let (mx, my) = (x - mask.rect.left, y - mask.rect.top);
    if mw <= 0 || mh <= 0 || mx < 0 || my < 0 || mx >= mw || my >= mh {
        return mask.default_color;
    }
    let Some(data) = &mask.data else {
        return mask.default_color;
    };
    data.get(my as usize * mw as usize + mx as usize)
        .copied()
        .unwrap_or(mask.default_color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BlendMode, Channel, ColorLabel, LockFlags, PsdRect};

    fn rect(top: i32, left: i32, bottom: i32, right: i32) -> PsdRect {
        PsdRect {
            top,
            left,
            bottom,
            right,
        }
    }

    fn step_layer(w: i32, h: i32, alpha: u8) -> Layer {
        let n = (w * h) as usize;
        let plane: Vec<u8> = (0..n)
            .map(|i| if (i as i32 % w) < w / 2 { 40 } else { 200 })
            .collect();
        Layer {
            name: "px".into(),
            rect: rect(0, 0, h, w),
            blend: BlendMode::Normal,
            opacity: 255,
            fill: 255,
            lock: LockFlags::default(),
            color: ColorLabel::None,
            clipping: false,
            visible: true,
            mask: None,
            adjustment: None,
            channels: vec![
                Channel {
                    id: 0,
                    data: plane.clone(),
                },
                Channel {
                    id: 1,
                    data: plane.clone(),
                },
                Channel { id: 2, data: plane },
                Channel {
                    id: -1,
                    data: vec![alpha; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        }
    }

    fn chan(layer: &Layer, id: i16) -> &[u8] {
        &layer.channels.iter().find(|c| c.id == id).unwrap().data
    }

    #[test]
    fn gaussian_blur_changes_interior_leaves_alpha_and_meta() {
        let mut layer = step_layer(32, 32, 123);
        let rect_before = layer.rect;
        let blend_before = layer.blend;
        let opacity_before = layer.opacity;
        let alpha_before = chan(&layer, -1).to_vec();
        let before = chan(&layer, 0).to_vec();

        apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 5.0 },
            None,
            false,
        )
        .unwrap();

        assert_ne!(chan(&layer, 0), before.as_slice());
        let edge = 15usize * 32 + 16;
        assert_ne!(chan(&layer, 0)[edge], before[edge]);
        assert_eq!(chan(&layer, -1), alpha_before.as_slice());
        assert_eq!(layer.rect, rect_before);
        assert_eq!(layer.blend, blend_before);
        assert_eq!(layer.opacity, opacity_before);
        assert!(layer.mask.is_none());
    }

    #[test]
    fn mask_confines_change_to_left_half() {
        let (w, h) = (8usize, 8usize);
        let mut layer = step_layer(w as i32, h as i32, 255);
        let before = chan(&layer, 0).to_vec();
        let mask = LayerMask {
            rect: rect(0, 0, h as i32, w as i32),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: Some(
                (0..w * h)
                    .map(|i| if i % w < w / 2 { 255 } else { 0 })
                    .collect(),
            ),
            ..Default::default()
        };

        apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 2.0 },
            Some(&mask),
            false,
        )
        .unwrap();

        let after = chan(&layer, 0);
        let inside = 3 * w + 3;
        assert_ne!(
            after[inside], before[inside],
            "left-half pixel should change"
        );
        for y in 0..h {
            for x in w / 2..w {
                let i = y * w + x;
                assert_eq!(
                    after[i], before[i],
                    "pixel ({x},{y}) outside coverage changed"
                );
            }
        }
    }

    #[test]
    fn disabled_mask_reports_full_coverage() {
        let mask = LayerMask {
            rect: rect(0, 0, 4, 4),
            default_color: 0,
            disabled: true,
            flags: 0,
            data: Some(vec![0u8; 16]),
            ..Default::default()
        };
        assert_eq!(
            coverage(Some(&mask), 0, 0),
            255,
            "disabled mask must not gate"
        );
        assert_eq!(coverage(Some(&mask), 3, 3), 255);

        let enabled = LayerMask {
            disabled: false,
            ..mask
        };
        assert_eq!(
            coverage(Some(&enabled), 1, 1),
            0,
            "enabled mask still gates"
        );
    }

    #[test]
    fn missing_color_channel_is_invalid_params() {
        let mut layer = step_layer(4, 4, 255);
        layer.channels.retain(|c| c.id != 2);
        let err = apply_filter(&mut layer, &Filter::Sharpen, None, false).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{err:?}");
    }

    #[test]
    fn wrong_length_channel_is_invalid_params() {
        let mut layer = step_layer(4, 4, 255);
        for c in &mut layer.channels {
            if c.id == 1 {
                c.data.truncate(3);
            }
        }
        let err = apply_filter(&mut layer, &Filter::Sharpen, None, false).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{err:?}");
    }

    #[test]
    fn empty_layer_rect_is_noop() {
        let mut layer = step_layer(4, 4, 255);
        layer.rect = rect(0, 0, 4, 0);
        assert!(apply_filter(&mut layer, &Filter::Median { radius: 200 }, None, false).is_ok());
        layer.rect = rect(0, 0, 0, 4);
        assert!(apply_filter(&mut layer, &Filter::Median { radius: 200 }, None, false).is_ok());
    }

    #[test]
    fn filter_parameter_error_propagates() {
        let mut layer = step_layer(4, 4, 255);
        let err =
            apply_filter(&mut layer, &Filter::Median { radius: 200 }, None, false).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{err:?}");
    }

    #[test]
    fn pixel_locked_layer_is_refused_and_unchanged() {
        let mut layer = step_layer(8, 8, 255);
        layer.lock = LockFlags::default().with(LockFlags::PIXELS, true);
        let before = layer.clone();
        let err = apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 2.0 },
            None,
            false,
        )
        .unwrap_err();
        assert!(matches!(err, FilterError::Locked), "{err:?}");
        assert_eq!(layer, before, "refusal leaves every channel bit-identical");
    }

    #[test]
    fn transparency_lock_keeps_alpha_and_skips_clear_pixels() {
        let (w, h) = (4usize, 4usize);
        let mut layer = step_layer(w as i32, h as i32, 255);
        for c in &mut layer.channels {
            if c.id == -1 {
                c.data[w + 1] = 0;
                c.data[w + 2] = 180;
            }
        }
        layer.lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
        let alpha_before = chan(&layer, -1).to_vec();
        let before = chan(&layer, 0).to_vec();

        apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 2.0 },
            None,
            false,
        )
        .unwrap();

        assert_eq!(
            chan(&layer, -1),
            alpha_before.as_slice(),
            "alpha is bit-identical"
        );
        assert_eq!(
            chan(&layer, 0)[w + 1],
            before[w + 1],
            "a fully transparent pixel is untouched"
        );
        assert_eq!(chan(&layer, -1)[w + 2], 180);
        assert!(
            chan(&layer, 0)
                .iter()
                .enumerate()
                .any(|(i, v)| { alpha_before[i] > 0 && *v != before[i] }),
            "an opaque pixel still takes the filter"
        );
    }

    #[test]
    fn position_and_transparency_locks_do_not_block_a_filter() {
        for flag in [LockFlags::POSITION, LockFlags::TRANSPARENCY] {
            let mut layer = step_layer(8, 8, 255);
            layer.lock = LockFlags::default().with(flag, true);
            assert!(
                apply_filter(
                    &mut layer,
                    &Filter::GaussianBlur { radius: 2.0 },
                    None,
                    false
                )
                .is_ok(),
                "flag {flag} must not refuse an opaque-preserving filter"
            );
        }
    }
}
