//! The Rasterize subset (design D8).
//!
//! Only solid-color fill content (`SoCo` with a 4-byte RGBA payload) is a
//! target. Type/Shape/Vector Mask/Smart Object/Video/3D layers do not exist in
//! the model, and gradient/pattern fills and Photoshop's `'Clr '` descriptor
//! are preserved on disk but not decoded, so those commands refuse without
//! mutating.

use pictura_core::{Channel, Document, Layer};

use super::paths::{flatten_rows, resolve_path, resolve_path_mut};

/// A decodable solid-color fill layer: a `SoCo` block carrying exactly the
/// 4-byte RGBA payload this codec writes. A descriptor-shaped `SoCo` is real
/// Photoshop fill data but is not decoded, so it is not a rasterize target.
pub fn is_fill_content_layer(layer: &Layer) -> bool {
    layer
        .adjustment
        .as_ref()
        .is_some_and(|data| data.key == *b"SoCo" && data.data.len() == 4)
}

/// Rasterize the fill content at `path`: bake the solid color into the layer's
/// `0/1/2/-1` channels over its rect, clear the adjustment, and keep the name,
/// rect, blend, opacity, fill, and mask. Returns false (no mutation) when
/// `path` does not resolve to a decodable fill-content layer.
pub fn rasterize_fill_content(doc: &mut Document, path: &str) -> bool {
    let Some(layer) = resolve_path_mut(doc, path) else {
        return false;
    };
    if !is_fill_content_layer(layer) {
        return false;
    }
    let data = layer
        .adjustment
        .as_ref()
        .expect("is_fill_content_layer checked Some");
    let rgba = [data.data[0], data.data[1], data.data[2], data.data[3]];
    bake_solid(layer, rgba);
    layer.adjustment = None;
    true
}

/// Rasterize every fill-content layer in the tree. Returns how many were
/// rasterized; a nil result leaves the document untouched. Rasterizing keeps
/// the tree shape, so the pre-collected paths stay valid.
pub fn rasterize_all_fill_content(doc: &mut Document) -> usize {
    let paths: Vec<String> = flatten_rows(doc)
        .into_iter()
        .filter(|(path, _)| resolve_path(doc, path).is_some_and(is_fill_content_layer))
        .map(|(path, _)| path)
        .collect();
    paths
        .iter()
        .filter(|path| rasterize_fill_content(doc, path))
        .count()
}

/// Overwrite the layer's color channels with `rgba` over its rect.
fn bake_solid(layer: &mut Layer, rgba: [u8; 4]) {
    let w = layer.rect.width();
    let h = layer.rect.height();
    if w <= 0 || h <= 0 {
        layer.channels = Vec::new();
        return;
    }
    let n = w as usize * h as usize;
    layer.channels = vec![
        Channel {
            id: 0,
            data: vec![rgba[0]; n],
        },
        Channel {
            id: 1,
            data: vec![rgba[1]; n],
        },
        Channel {
            id: 2,
            data: vec![rgba[2]; n],
        },
        Channel {
            id: -1,
            data: vec![rgba[3]; n],
        },
    ];
}
