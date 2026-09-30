//! The Rasterize subset (design D8).
//!
//! Fill content is a layer whose opaque adjustment block decodes through
//! [`crate::decode_adjustment`] to [`Adjustment::SolidFill`] (a `SoCo` block, in
//! either the 4-byte in-house form or the standard PSD descriptor),
//! [`Adjustment::GradientFill`] (a `GdFl` descriptor), or
//! [`Adjustment::PatternFill`] (a `PtFl` descriptor). Type/Shape/Vector
//! Mask/Smart Object/Video/3D layers do not exist in the model, so those
//! commands refuse without mutating.

use pictura_adjust::{Adjustment, GradientFillParams, PatternFillParams};
use pictura_codec::PatternPixels;
use pictura_core::{Channel, Document, Layer};

use super::paths::{flatten_rows, resolve_path_mut};

/// A decodable fill-content layer: a `SoCo`/`GdFl`/`PtFl` block whose payload
/// decodes to a fill [`Adjustment`]. Routing both this predicate and the bakers
/// through [`crate::decode_adjustment`] keeps them from disagreeing.
pub fn is_fill_content_layer(layer: &Layer) -> bool {
    matches!(
        layer.adjustment.as_ref().and_then(crate::decode_adjustment),
        Some(Adjustment::SolidFill(_) | Adjustment::GradientFill(_) | Adjustment::PatternFill(_))
    )
}

/// Rasterize the fill content at `path`: bake the decoded fill into the layer's
/// `0/1/2/-1` channels over its rect, clear the adjustment, and keep the name,
/// rect, blend, opacity, fill, and mask. Returns false (no mutation) when
/// `path` does not resolve to a decodable fill-content layer.
pub fn rasterize_fill_content(doc: &mut Document, path: &str) -> bool {
    // Decode the pattern library once, before taking the mutable layer borrow.
    let patterns = pictura_codec::decode_patterns(doc);
    let Some(layer) = resolve_path_mut(doc, path) else {
        return false;
    };
    let Some(adjustment) = layer.adjustment.as_ref().and_then(crate::decode_adjustment) else {
        return false;
    };
    match adjustment {
        Adjustment::SolidFill(rgba) => bake_solid(layer, rgba),
        Adjustment::GradientFill(params) => bake_gradient(layer, &params),
        Adjustment::PatternFill(params) => bake_pattern(layer, &params, &patterns),
        _ => return false,
    }
    layer.adjustment = None;
    true
}

/// Rasterize every fill-content layer in the tree. Returns how many were
/// rasterized; a nil result leaves the document untouched. Rasterizing keeps
/// the tree shape, so the pre-collected paths stay valid.
/// Rasterize every flattened layer that can be baked: fill content through
/// [`rasterize_fill_content`], else a type layer through the bundled text
/// renderer. Neither operation adds or removes layers, so the path list stays
/// valid. Returns how many were rasterized.
pub fn rasterize_all_layers(doc: &mut Document) -> usize {
    let paths: Vec<String> = flatten_rows(doc)
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    paths
        .iter()
        .filter(|path| rasterize_fill_content(doc, path) || crate::render_text_layer(doc, path))
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
            data: vec![rgba[0]; n].into(),
        },
        Channel {
            id: 1,
            data: vec![rgba[1]; n].into(),
        },
        Channel {
            id: 2,
            data: vec![rgba[2]; n].into(),
        },
        Channel {
            id: -1,
            data: vec![rgba[3]; n].into(),
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
            data: vec![255; px.len()].into(),
        },
    ];
}

/// Overwrite the layer's color channels with the tiled pattern (or the
/// placeholder when `params.pattern_id` is absent) over its rect. The pattern's
/// own alpha bakes into `-1`; an empty rect leaves no channels.
fn bake_pattern(layer: &mut Layer, params: &PatternFillParams, patterns: &[PatternPixels]) {
    let w = layer.rect.width();
    let h = layer.rect.height();
    if w <= 0 || h <= 0 {
        layer.channels = Vec::new();
        return;
    }
    let px =
        crate::fill::pattern_tile_rgba(patterns, params, layer.rect.left, layer.rect.top, w, h);
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
            data: px.iter().map(|p| p[3]).collect(),
        },
    ];
}
