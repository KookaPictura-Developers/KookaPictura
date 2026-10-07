//! Move the pixels covered by a selection mask (design D5). Reuses the
//! `Layer via Copy`/`Layer via Cut` primitives and translates the resulting
//! layer's rectangle.

use pictura_core::{
    layer_move_locked, layer_pixel_locked, layer_transparency_locked, Document, LayerMask, PsdRect,
};

use super::clipboard::{clear_layer, coverage_bounds};
use super::merge::{merge_scope, MergeScope};
use super::paths::{resolve_path, resolve_path_mut};
use super::via::{layer_via_copy, layer_via_cut};
use crate::document_ops::canvas::offset_rect;

/// Lift the pixels covered by `coverage` (document-sized, `0..=255`) off the
/// layer at `source_path` for a transform: copy them into a new layer directly
/// above, trimmed to the coverage bounds, and clear them from the source as
/// Edit > Clear does (a Background clears to white). Unlike a move, a
/// Background may be lifted. Returns the new layer's path; empty for a group,
/// an adjustment, a layer without pixels, a pixel lock, a transparency lock on
/// a layer with alpha, or empty coverage. Merge it back with
/// [`MergeScope::Down`].
pub fn lift_selection(doc: &mut Document, source_path: &str, coverage: &[u8]) -> String {
    if !resolve_path(doc, source_path).is_some_and(can_lift_selection) {
        return String::new();
    }
    let Some(bounds) = coverage_bounds(coverage, doc.width, doc.height) else {
        return String::new();
    };
    let mask = LayerMask {
        rect: PsdRect {
            top: 0,
            left: 0,
            bottom: doc.height as i32,
            right: doc.width as i32,
        },
        data: Some(coverage.to_vec().into()),
        ..Default::default()
    };
    let lifted = layer_via_copy(doc, source_path, &mask);
    if lifted.is_empty() || !clear_layer(doc, source_path, Some(coverage)) {
        return String::new();
    }
    if let Some(layer) = resolve_path_mut(doc, &lifted) {
        trim_layer(layer, bounds);
    }
    lifted
}

/// Whether [`lift_selection`] can take pixels from `layer`: a pixel layer or
/// Background without a pixel lock or a transparency lock on its alpha, and
/// not position-locked unless it is the Background (which always is).
pub fn can_lift_selection(layer: &pictura_core::Layer) -> bool {
    let has_alpha = layer.channels.iter().any(|c| c.id == -1);
    !layer.is_group
        && layer.adjustment.is_none()
        && layer.channels.iter().any(|c| c.id == 0)
        && !layer_pixel_locked(layer)
        && !(has_alpha && layer_transparency_locked(layer))
        && (layer.background || !layer_move_locked(layer))
}

/// Merge a [`lift_selection`] layer at `lifted_path` back down into its source,
/// keeping the source's identity: a Background stays the locked, alpha-less
/// Background, and any layer keeps its name and locks. False when the merge is
/// refused.
pub fn merge_lifted(doc: &mut Document, lifted_path: &str) -> bool {
    let Some(mut segments) = super::paths::parse_path(lifted_path) else {
        return false;
    };
    match segments.last_mut() {
        Some(last) if *last > 0 => *last -= 1,
        _ => return false,
    }
    let Some(source) = resolve_path(doc, &super::paths::format_segments(&segments)) else {
        return false;
    };
    let (name, lock, background) = (source.name.clone(), source.lock, source.background);
    let had_alpha = source.channels.iter().any(|c| c.id == -1);
    let Ok(outcome) = merge_scope(doc, MergeScope::Down(lifted_path)) else {
        return false;
    };
    if let Some(merged) = resolve_path_mut(doc, &outcome.path) {
        merged.name = name;
        merged.lock = lock;
        merged.background = background;
        if !had_alpha {
            merged.channels.retain(|c| c.id != -1);
        }
    }
    true
}

/// Crop `layer`'s channels from its rect to `bounds` (clamped inside it).
fn trim_layer(layer: &mut pictura_core::Layer, bounds: PsdRect) {
    let old = layer.rect;
    let new = PsdRect {
        top: bounds.top.max(old.top),
        left: bounds.left.max(old.left),
        bottom: bounds.bottom.min(old.bottom),
        right: bounds.right.min(old.right),
    };
    if new.width() <= 0 || new.height() <= 0 || new == old {
        return;
    }
    let old_w = old.width() as usize;
    let (w, x0) = (new.width() as usize, (new.left - old.left) as usize);
    for channel in &mut layer.channels {
        let mut data = Vec::with_capacity(w * new.height() as usize);
        for y in new.top..new.bottom {
            let row = (y - old.top) as usize * old_w + x0;
            data.extend_from_slice(&channel.data[row..row + w]);
        }
        channel.data = data.into();
    }
    layer.rect = new;
}

/// Move the pixels covered by `mask` from the layer at `source_path` by
/// `(dx, dy)`. With `duplicate` the source stays intact and a new layer sits
/// above it; otherwise the source region is cut and the moved pixels merge back
/// down, so the layer count is unchanged. Returns false on the same refusals as
/// [`layer_via_copy`] / [`layer_via_cut`].
pub fn move_selection_content(
    doc: &mut Document,
    source_path: &str,
    mask: &LayerMask,
    dx: i32,
    dy: i32,
    duplicate: bool,
) -> bool {
    // A locked source cannot contribute pixels: a position lock refuses the
    // move, a pixel lock refuses any mutation, and a transparency lock refuses
    // the cut that clears the source alpha (a duplicate leaves it intact).
    let Some(source) = resolve_path(doc, source_path) else {
        return false;
    };
    if layer_move_locked(source)
        || layer_pixel_locked(source)
        || (!duplicate && layer_transparency_locked(source))
    {
        return false;
    }
    let new_path = if duplicate {
        layer_via_copy(doc, source_path, mask)
    } else {
        layer_via_cut(doc, source_path, mask)
    };
    if new_path.is_empty() {
        return false;
    }
    if let Some(layer) = resolve_path_mut(doc, &new_path) {
        layer.rect = offset_rect(layer.rect, dx, dy);
        if let Some(layer_mask) = &mut layer.mask {
            layer_mask.rect = offset_rect(layer_mask.rect, dx, dy);
        }
    }
    if !duplicate {
        // ponytail: merge_down composites with Normal blending, so a non-Normal
        // source may not land exactly; composite with the source's blend if it
        // ever matters.
        let _ = merge_scope(doc, MergeScope::Down(&new_path));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{
        BitDepth, BlendMode, Channel, ColorLabel, ColorMode, Document, Layer, LayerMask, LockFlags,
        PsdRect,
    };

    fn rect(w: i32, h: i32) -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: h,
            right: w,
        }
    }

    fn pixel_layer(name: &str, w: u32, h: u32, value: u8) -> Layer {
        let n = w as usize * h as usize;
        Layer {
            name: name.into(),
            rect: rect(w as i32, h as i32),
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
                    data: vec![value; n].into(),
                },
                Channel {
                    id: 1,
                    data: vec![value; n].into(),
                },
                Channel {
                    id: 2,
                    data: vec![value; n].into(),
                },
                Channel {
                    id: -1,
                    data: vec![255; n].into(),
                },
            ],
            children: Vec::new(),
            is_group: false,
            background: false,
            ..Default::default()
        }
    }

    fn doc_with(layers: Vec<Layer>) -> Document {
        let mut doc = Document::new(4, 4, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = layers;
        doc
    }

    fn selection_mask(data: Vec<u8>) -> LayerMask {
        LayerMask {
            rect: rect(4, 4),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: Some(data.into()),
            ..Default::default()
        }
    }

    fn alpha(layer: &Layer) -> &[u8] {
        &layer.channels.iter().find(|c| c.id == -1).unwrap().data
    }

    fn square_coverage() -> Vec<u8> {
        let mut coverage = vec![0; 16];
        for i in [5, 6, 9, 10] {
            coverage[i] = 255;
        }
        coverage
    }

    fn channel(layer: &Layer, id: i16) -> &[u8] {
        &layer.channels.iter().find(|c| c.id == id).unwrap().data
    }

    #[test]
    fn lift_copies_the_covered_pixels_trimmed_and_clears_them() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let lifted = lift_selection(&mut doc, "0", &square_coverage());
        assert_eq!(lifted, "1");
        let layer = &doc.layers[1];
        assert_eq!(
            layer.rect,
            PsdRect {
                top: 1,
                left: 1,
                bottom: 3,
                right: 3
            },
            "trimmed to the selection"
        );
        assert_eq!(channel(layer, 0), &[40; 4]);
        assert_eq!(alpha(layer), &[255; 4]);
        let source = alpha(&doc.layers[0]);
        assert_eq!((source[5], source[10], source[0]), (0, 0, 255));

        assert!(merge_scope(&mut doc, MergeScope::Down(&lifted)).is_ok());
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(
            alpha(&doc.layers[0])[5],
            255,
            "an untransformed lift merges back"
        );
    }

    #[test]
    fn a_locked_background_can_be_lifted_and_stays_the_background() {
        let mut background = pixel_layer("Background", 4, 4, 40);
        background.channels.retain(|c| c.id != -1);
        background.background = true;
        background.lock = LockFlags::default()
            .with(LockFlags::TRANSPARENCY, true)
            .with(LockFlags::POSITION, true);
        let mut doc = doc_with(vec![background]);
        let lifted = lift_selection(&mut doc, "0", &square_coverage());
        assert_eq!(lifted, "1");
        assert_eq!(
            channel(&doc.layers[0], 0)[5],
            255,
            "the hole clears to white"
        );
        assert_eq!(channel(&doc.layers[0], 0)[0], 40);

        assert!(merge_lifted(&mut doc, &lifted));
        assert_eq!(doc.layers.len(), 1);
        assert!(doc.layers[0].background);
        assert!(doc.layers[0].lock.contains(LockFlags::POSITION));
        assert!(!doc.layers[0].channels.iter().any(|c| c.id == -1));
        assert_eq!(channel(&doc.layers[0], 0)[5], 40);
    }

    #[test]
    fn lift_refuses_empty_coverage_and_pixel_and_position_locks() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        assert!(lift_selection(&mut doc, "0", &[0; 16]).is_empty());
        doc.layers[0].lock = LockFlags::default().with(LockFlags::PIXELS, true);
        assert!(lift_selection(&mut doc, "0", &square_coverage()).is_empty());
        doc.layers[0].lock = LockFlags::default().with(LockFlags::POSITION, true);
        assert!(lift_selection(&mut doc, "0", &square_coverage()).is_empty());
        assert_eq!(doc.layers.len(), 1);
    }

    #[test]
    fn move_cuts_translates_and_merges_without_changing_layer_count() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let moved =
            move_selection_content(&mut doc, "0", &selection_mask(vec![255; 16]), 2, 0, false);

        assert!(moved);
        assert_eq!(doc.layers.len(), 1, "the moved copy merges back down");
        let data = alpha(&doc.layers[0]);
        for y in 0..4 {
            for x in 0..4 {
                if x < 2 {
                    assert_eq!(data[y * 4 + x], 0, "source ({x},{y}) cleared");
                } else {
                    assert_eq!(data[y * 4 + x], 255, "dest ({x},{y}) has the pixels");
                }
            }
        }
    }

    #[test]
    fn move_duplicate_keeps_source_and_adds_a_layer() {
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let moved =
            move_selection_content(&mut doc, "0", &selection_mask(vec![255; 16]), 2, 0, true);

        assert!(moved);
        assert_eq!(doc.layers.len(), 2, "the duplicate is a new layer");
        assert!(
            alpha(&doc.layers[0]).iter().all(|&v| v == 255),
            "source intact"
        );
        let copy = &doc.layers[1];
        assert_eq!(copy.name, "base copy");
        assert_eq!(
            copy.rect,
            PsdRect {
                top: 0,
                left: 2,
                bottom: 4,
                right: 6,
            },
            "the copy rect translated by dx"
        );
        assert!(
            alpha(copy).iter().all(|&v| v == 255),
            "copy carries the pixels"
        );
    }

    #[test]
    fn empty_mask_refuses_and_zero_coverage_cuts_nothing() {
        let no_data = LayerMask {
            rect: rect(4, 4),
            default_color: 0,
            disabled: false,
            flags: 0,
            data: None,
            ..Default::default()
        };
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let before = doc.clone();
        assert!(!move_selection_content(
            &mut doc, "0", &no_data, 2, 0, false
        ));
        assert_eq!(doc, before, "no mask data refuses without mutation");

        // `layer_via_cut` only refuses a missing data plane, so a present but
        // all-zero mask still cuts nothing: the call reports success and the
        // merge-down leaves the document byte-identical.
        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        let before = doc.clone();
        assert!(move_selection_content(
            &mut doc,
            "0",
            &selection_mask(vec![0; 16]),
            2,
            0,
            false
        ));
        assert_eq!(doc, before, "zero coverage moves nothing");
    }

    #[test]
    fn locked_source_refuses_content_move() {
        let mask = selection_mask(vec![255; 16]);

        for flag in [LockFlags::PIXELS, LockFlags::POSITION] {
            let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
            doc.layers[0].lock = LockFlags::default().with(flag, true);
            let before = doc.clone();
            assert!(
                !move_selection_content(&mut doc, "0", &mask, 2, 0, false),
                "flag {flag} refuses the move"
            );
            assert_eq!(doc, before, "refusal leaves the document unchanged");
        }

        let mut doc = doc_with(vec![pixel_layer("base", 4, 4, 40)]);
        doc.layers[0].lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
        let before = doc.clone();
        assert!(
            !move_selection_content(&mut doc, "0", &mask, 2, 0, false),
            "a transparency lock refuses the cut"
        );
        assert_eq!(doc, before);
        assert!(
            move_selection_content(&mut doc, "0", &mask, 2, 0, true),
            "a duplicate leaves the source alpha intact"
        );
    }
}
