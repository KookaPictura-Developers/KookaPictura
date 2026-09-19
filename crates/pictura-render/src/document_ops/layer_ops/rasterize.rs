//! The Rasterize subset (design D8).
//!
//! Fill content is a layer whose opaque adjustment block decodes through
//! [`crate::decode_adjustment`] to [`Adjustment::SolidFill`] (a `SoCo` block, in
//! either the 4-byte in-house form or the standard Photoshop descriptor) or
//! [`Adjustment::GradientFill`] (a `GdFl` descriptor). Type/Shape/Vector
//! Mask/Smart Object/Video/3D layers do not exist in the model, so those
//! commands refuse without mutating.

use pictura_adjust::{Adjustment, GradientFillParams};
use pictura_core::{Channel, Document, Layer};

use super::paths::{flatten_rows, resolve_path, resolve_path_mut};

/// A decodable fill-content layer: a `SoCo`/`GdFl` block whose payload decodes
/// to [`Adjustment::SolidFill`] or [`Adjustment::GradientFill`].
pub fn is_fill_content_layer(layer: &Layer) -> bool {
    layer.adjustment.as_ref().is_some_and(|data| {
        matches!(
            crate::decode_adjustment(data),
            Some(Adjustment::SolidFill(_) | Adjustment::GradientFill(_))
        )
    })
}

/// Rasterize the fill content at `path`: bake the decoded fill into the layer's
/// `0/1/2/-1` channels over its rect, clear the adjustment, and keep the name,
/// rect, blend, opacity, fill, and mask. Returns false (no mutation) when
/// `path` does not resolve to a decodable fill-content layer.
pub fn rasterize_fill_content(doc: &mut Document, path: &str) -> bool {
    let Some(layer) = resolve_path_mut(doc, path) else {
        return false;
    };
    let Some(adjustment) = layer.adjustment.as_ref().and_then(crate::decode_adjustment) else {
        return false;
    };
    match adjustment {
        Adjustment::SolidFill(rgba) => bake_solid(layer, rgba),
        Adjustment::GradientFill(params) => bake_gradient(layer, &params),
        _ => return false,
    }
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

/// Overwrite the layer's color channels with a generated gradient over its
/// rect. Alpha is opaque; an empty rect leaves no channels.
fn bake_gradient(layer: &mut Layer, params: &GradientFillParams) {
    let w = layer.rect.width();
    let h = layer.rect.height();
    if w <= 0 || h <= 0 {
        layer.channels = Vec::new();
        return;
    }
    let px = crate::fill::gradient_rgba(w, h, params);
    layer.channels = vec![
        Channel {
            id: 0,
            data: px.iter().map(|p| p[0]).collect(),
        },
        Channel {
            id: 1,
            data: px.iter().map(|p| p[1]).collect(),
        },
        Channel {
            id: 2,
            data: px.iter().map(|p| p[2]).collect(),
        },
        Channel {
            id: -1,
            data: vec![255; px.len()],
        },
    ];
}
