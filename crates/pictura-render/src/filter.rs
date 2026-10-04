//! M6-C: destructive filter application to a pixel layer's color channels.
//!
//! Contract: `docs/dev/m6c-filter-integration.md`. The filter runs on channels
//! `0,1,2`, and on the transparency channel (`-1`) unless the layer is
//! transparency-locked. [`apply_filter_region`] bounds the work to a document
//! rectangle for viewport-sized previews.

use pictura_adjust::Adjustment;
use pictura_core::{
    layer_pixel_locked, layer_transparency_locked, Layer, LayerMask, PixelBuffer, PsdRect,
};
use pictura_filters::{Filter, FilterError};

use crate::channel;

/// Apply a destructive `filter` to a pixel layer's color channels (0,1,2),
/// gated by an optional document-coordinate coverage `mask` (`None` = full
/// frame). An unlocked layer's transparency is filtered too; a transparency
/// lock preserves `-1` and leaves clear pixels untouched.
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
    let region = layer.rect;
    apply_filter_region(layer, filter, mask, gpu_enabled, region)
}

/// As [`apply_filter`], but only over `region` (clamped to the layer rect).
///
/// A neighborhood filter reads up to its support outside `region`, where the
/// source is clamped as if the region were the whole layer. A preview that
/// passes a viewport-sized region expanded by a support-sized apron is correct
/// across the viewport and cheaper than filtering the whole layer.
pub fn apply_filter_region(
    layer: &mut Layer,
    filter: &Filter,
    mask: Option<&LayerMask>,
    gpu_enabled: bool,
    region: PsdRect,
) -> Result<(), FilterError> {
    apply_op_region(layer, Op::Filter(filter, gpu_enabled), mask, region)
}

/// Apply a destructive Image > Adjustments `adjustment` to a pixel layer's
/// colour channels, gated by an optional coverage `mask`, over `region`
/// (clamped to the layer rect). Unlike a filter it leaves transparency alone,
/// and a transparency lock keeps clear pixels untouched.
pub fn apply_adjustment_region(
    layer: &mut Layer,
    adjustment: &Adjustment,
    mask: Option<&LayerMask>,
    region: PsdRect,
) -> Result<(), FilterError> {
    apply_op_region(layer, Op::Adjustment(adjustment), mask, region)
}

/// What [`apply_op_region`] runs on the working buffer.
#[derive(Clone, Copy)]
enum Op<'a> {
    Filter(&'a Filter, bool),
    Adjustment(&'a Adjustment),
}

fn apply_op_region(
    layer: &mut Layer,
    op: Op,
    mask: Option<&LayerMask>,
    region: PsdRect,
) -> Result<(), FilterError> {
    let lrect = layer.rect;
    if lrect.width() <= 0 || lrect.height() <= 0 {
        return Ok(());
    }
    if layer_pixel_locked(layer) {
        return Err(FilterError::Locked);
    }
    let clip = intersect_rect(region, lrect);
    if clip.width() <= 0 || clip.height() <= 0 {
        return Ok(());
    }
    let lw = lrect.width() as usize;
    let full_n = lw * lrect.height() as usize;
    let cw = clip.width() as usize;
    let ch = clip.height() as usize;
    let n = cw * ch;
    let ox = (clip.left - lrect.left) as usize;
    let oy = (clip.top - lrect.top) as usize;

    // Validate the layer's full color planes (data length == layer w*h).
    let c0 = channel(layer, 0)
        .ok_or_else(|| FilterError::InvalidParams("layer is missing color channel 0".into()))?;
    if c0.len() != full_n {
        return Err(FilterError::InvalidParams(format!(
            "channel 0 has {} samples, expected {full_n}",
            c0.len()
        )));
    }
    // A Grayscale layer carries only channel 0; a color layer carries all three.
    // A mix (one of 1/2 present) is malformed and refused, as before.
    let grayscale = match (channel(layer, 1), channel(layer, 2)) {
        (Some(c1), Some(c2)) => {
            for (id, data) in [(1, c1), (2, c2)] {
                if data.len() != full_n {
                    return Err(FilterError::InvalidParams(format!(
                        "channel {id} has {} samples, expected {full_n}",
                        data.len()
                    )));
                }
            }
            false
        }
        (None, None) => true,
        _ => {
            return Err(FilterError::InvalidParams(
                "layer is missing a color channel".into(),
            ));
        }
    };

    // Crop the clip out of the layer into a clip-sized working buffer.
    let sources: [&[u8]; 3] = if grayscale {
        [c0, c0, c0]
    } else {
        [c0, channel(layer, 1).unwrap(), channel(layer, 2).unwrap()]
    };
    let mut buf = PixelBuffer::new(cw as u32, ch as u32, 3);
    for (c, src) in sources.into_iter().enumerate() {
        for row in 0..ch {
            let from = (oy + row) * lw + ox;
            let to = c * n + row * cw;
            buf.data[to..to + cw].copy_from_slice(&src[from..from + cw]);
        }
    }

    let orig = buf.clone();
    match op {
        Op::Filter(filter, gpu_enabled) => {
            crate::gpu_filter::apply_filter_active(filter, &mut buf, gpu_enabled)?;
        }
        Op::Adjustment(adjustment) => pictura_adjust::apply(adjustment, &mut buf)
            .map_err(|error| FilterError::InvalidParams(error.to_string()))?,
    }

    // A transparency lock preserves the alpha plane exactly and leaves fully
    // transparent pixels untouched; only opaque pixels take the filter colour.
    // An unlocked layer takes the filter on its transparency too (CS6): run the
    // same kernel over a grey copy of the alpha plane so per-plane kernels
    // smear or alter the layer edge exactly as they do colour.
    // ponytail: the alpha pass reruns the kernel on the CPU; fold it into the
    // working buffer once the kernels accept a fourth plane.
    let alpha_locked = layer_transparency_locked(layer);
    let alpha_original: Option<Vec<u8>> = channel(layer, -1).map(|data| {
        let mut crop = Vec::with_capacity(n);
        for row in 0..ch {
            let from = (oy + row) * lw + ox;
            crop.extend_from_slice(&data[from..from + cw]);
        }
        crop
    });
    let filtered_alpha = match op {
        Op::Filter(filter, _) if !alpha_locked => alpha_original
            .as_ref()
            .map(|alpha| -> Result<Vec<u8>, FilterError> {
                let mut gray = PixelBuffer::new(cw as u32, ch as u32, 3);
                for c in 0..3usize {
                    gray.data[c * n..c * n + n].copy_from_slice(alpha);
                }
                pictura_filters::apply(filter, &mut gray)?;
                Ok(gray.data[0..n].to_vec())
            })
            .transpose()?,
        _ => None,
    };

    let (left, top) = (clip.left, clip.top);
    // A Grayscale result is a single plane; a color result has all three.
    let write_planes: &[i16] = if grayscale { &[0] } else { &[0, 1, 2] };
    for &c in write_planes {
        let plane = c as usize * n;
        let orig_plane = &orig.data[plane..plane + n];
        let filtered_plane = &buf.data[plane..plane + n];
        let out = channel_mut(layer, c).expect("channel presence validated above");
        for ly in 0..ch {
            for lx in 0..cw {
                let i = ly * cw + lx;
                if alpha_locked && alpha_original.as_ref().is_some_and(|a| a[i] == 0) {
                    continue;
                }
                let cov = coverage(mask, left + lx as i32, top + ly as i32);
                if cov == 0 {
                    continue;
                }
                let o = orig_plane[i] as f64;
                let f = filtered_plane[i] as f64;
                let dst = (oy + ly) * lw + ox + lx;
                out[dst] = (o + (f - o) * cov as f64 / 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    // Write the filtered transparency (unlocked layers only).
    if let (Some(source), Some(filtered)) = (alpha_original.as_ref(), filtered_alpha.as_ref()) {
        let out = channel_mut(layer, -1).expect("alpha presence checked above");
        for ly in 0..ch {
            for lx in 0..cw {
                let i = ly * cw + lx;
                let cov = coverage(mask, left + lx as i32, top + ly as i32);
                if cov == 0 {
                    continue;
                }
                let o = source[i] as f64;
                let f = filtered[i] as f64;
                let dst = (oy + ly) * lw + ox + lx;
                out[dst] = (o + (f - o) * cov as f64 / 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Ok(())
}

fn intersect_rect(a: PsdRect, b: PsdRect) -> PsdRect {
    PsdRect {
        top: a.top.max(b.top),
        left: a.left.max(b.left),
        bottom: a.bottom.min(b.bottom),
        right: a.right.min(b.right),
    }
}

/// A conservative document-pixel margin a preview must keep around a viewport
/// so a neighborhood filter's result is exact across the visible area.
pub fn preview_apron(filter: &Filter) -> i32 {
    let radius = match filter {
        Filter::GaussianBlur { radius }
        | Filter::UnsharpMask { radius, .. }
        | Filter::HighPass { radius }
        | Filter::SmartSharpen { radius, .. } => radius.ceil(),
        Filter::BoxBlur { radius }
        | Filter::SurfaceBlur { radius, .. }
        | Filter::Median { radius }
        | Filter::Maximum { radius }
        | Filter::Minimum { radius } => f64::from(*radius),
        Filter::MotionBlur { distance, .. } => f64::from(*distance),
        Filter::RadialBlur { amount, .. } => *amount,
        Filter::Offset {
            horizontal,
            vertical,
            ..
        } => f64::from(horizontal.abs().max(vertical.abs())),
        // Distort, render, texture, and cell filters can scatter or displace
        // widely; keep a generous margin.
        _ => 128.0,
    };
    (radius.ceil() as i32).clamp(8, 512)
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
                    data: plane.clone().into(),
                },
                Channel {
                    id: 1,
                    data: plane.clone().into(),
                },
                Channel {
                    id: 2,
                    data: plane.into(),
                },
                Channel {
                    id: -1,
                    data: vec![alpha; n].into(),
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
            data: Some(vec![0u8; 16].into()),
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
    fn region_apply_matches_full_inside_and_leaves_the_rest() {
        let filter = Filter::GaussianBlur { radius: 2.0 };
        let full = step_layer(32, 32, 255);
        let region = rect(8, 8, 24, 24);
        let mut partial = full.clone();
        apply_filter_region(&mut partial, &filter, None, false, region).unwrap();
        let mut whole = full.clone();
        apply_filter(&mut whole, &filter, None, false).unwrap();

        let w = 32usize;
        // The blurred step edge at the region centre matches the full apply.
        assert_eq!(
            chan(&partial, 0)[16 * w + 16],
            chan(&whole, 0)[16 * w + 16],
            "inside the region matches a full apply"
        );
        // A corner outside the region is untouched.
        assert_eq!(
            chan(&partial, 0)[0],
            chan(&full, 0)[0],
            "outside the region is not filtered"
        );
    }

    #[test]
    fn unlocked_layer_filters_its_transparency() {
        let (w, h) = (8usize, 8usize);
        let mut layer = step_layer(w as i32, h as i32, 255);
        for c in &mut layer.channels {
            if c.id == -1 {
                for i in 0..w * h {
                    c.data[i] = if i % w < w / 2 { 255 } else { 0 };
                }
            }
        }
        let alpha_before = chan(&layer, -1).to_vec();
        apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 1.5 },
            None,
            false,
        )
        .unwrap();
        assert_ne!(
            chan(&layer, -1),
            alpha_before.as_slice(),
            "an unlocked layer's alpha is filtered"
        );
    }

    #[test]
    fn an_adjustment_changes_colour_inside_the_region_and_never_alpha() {
        let (w, h) = (8usize, 8usize);
        let mut layer = step_layer(w as i32, h as i32, 255);
        for c in &mut layer.channels {
            if c.id == -1 {
                for i in 0..w * h {
                    c.data[i] = if i % w < w / 2 { 255 } else { 90 };
                }
            }
        }
        let alpha_before = chan(&layer, -1).to_vec();
        let colour_before = chan(&layer, 0).to_vec();
        let left = PsdRect {
            top: 0,
            left: 0,
            bottom: h as i32,
            right: (w / 2) as i32,
        };
        apply_adjustment_region(&mut layer, &Adjustment::Invert, None, left).unwrap();
        assert_eq!(chan(&layer, -1), alpha_before.as_slice(), "alpha untouched");
        let after = chan(&layer, 0);
        assert_eq!(
            after[0],
            255 - colour_before[0],
            "inside the region inverts"
        );
        assert_eq!(after[w - 1], colour_before[w - 1], "outside it does not");
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

    #[test]
    fn grayscale_single_channel_layer_filters_channel_zero() {
        let (w, h) = (32usize, 32usize);
        let mut layer = step_layer(w as i32, h as i32, 123);
        layer.channels.retain(|c| c.id == 0 || c.id == -1);
        let before = chan(&layer, 0).to_vec();
        let alpha_before = chan(&layer, -1).to_vec();

        apply_filter(
            &mut layer,
            &Filter::GaussianBlur { radius: 3.0 },
            None,
            false,
        )
        .unwrap();

        assert_ne!(
            chan(&layer, 0),
            before.as_slice(),
            "grayscale plane changes"
        );
        assert_eq!(chan(&layer, -1), alpha_before.as_slice(), "alpha untouched");
        assert_eq!(layer.channels.len(), 2, "no color planes are added");
    }

    #[test]
    fn missing_channel_zero_is_invalid_params() {
        let mut layer = step_layer(4, 4, 255);
        layer.channels.retain(|c| c.id == -1);
        let err = apply_filter(&mut layer, &Filter::Sharpen, None, false).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{err:?}");
    }

    #[test]
    fn mixed_color_and_missing_channel_is_invalid_params() {
        // Channel 1 present but channel 2 absent is malformed, not Grayscale.
        let mut layer = step_layer(4, 4, 255);
        layer.channels.retain(|c| c.id != 2);
        let err = apply_filter(&mut layer, &Filter::Sharpen, None, false).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{err:?}");
    }
}
