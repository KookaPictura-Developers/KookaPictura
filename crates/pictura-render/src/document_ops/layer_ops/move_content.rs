//! Move the pixels covered by a selection mask (design D5). Reuses the
//! `Layer via Copy`/`Layer via Cut` primitives and translates the resulting
//! layer's rectangle.

use pictura_core::{Document, LayerMask};

use super::merge::{merge_scope, MergeScope};
use super::paths::resolve_path_mut;
use super::via::{layer_via_copy, layer_via_cut};
use crate::document_ops::canvas::offset_rect;

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
                    data: vec![value; n],
                },
                Channel {
                    id: 1,
                    data: vec![value; n],
                },
                Channel {
                    id: 2,
                    data: vec![value; n],
                },
                Channel {
                    id: -1,
                    data: vec![255; n],
                },
            ],
            children: Vec::new(),
            is_group: false,
            background: false,
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
            data: Some(data),
        }
    }

    fn alpha(layer: &Layer) -> &[u8] {
        &layer.channels.iter().find(|c| c.id == -1).unwrap().data
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
}
