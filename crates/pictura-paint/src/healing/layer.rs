//! Running a heal on one pixel layer of a document.

use super::RgbaImage;
use crate::stroke::{layer_at_mut, parse_layer_path};
use pictura_core::{layer_pixel_locked, layer_transparency_locked, Document, PsdRect};

#[derive(Debug, PartialEq, Eq)]
pub enum HealError {
    /// The path is not one pixel layer with pixels.
    NoRasterLayer,
    /// The layer's pixels are locked.
    Locked,
}

/// Run `op` on the pixel layer at `path` (a panel path such as `"0"`). `region`
/// and `coverage` are in document pixels; `op` receives the layer's pixels and
/// the region in layer coordinates, and returns what it changed. A transparency
/// lock keeps every pixel's alpha. Returns the document rectangle changed, or
/// `None` when nothing was.
pub fn heal_layer(
    doc: &mut Document,
    path: &str,
    region: PsdRect,
    coverage: &[f32],
    op: impl FnOnce(&mut RgbaImage, PsdRect, &[f32]) -> Option<PsdRect>,
) -> Result<Option<PsdRect>, HealError> {
    let indices = parse_layer_path(path).ok_or(HealError::NoRasterLayer)?;
    let layer = layer_at_mut(doc, &indices).ok_or(HealError::NoRasterLayer)?;
    if layer.is_group || layer.adjustment.is_some() {
        return Err(HealError::NoRasterLayer);
    }
    if layer_pixel_locked(layer) {
        return Err(HealError::Locked);
    }
    let rect = layer.rect;
    let (w, h) = (rect.width(), rect.height());
    let len = (w.max(0) * h.max(0)) as usize;
    let plane = |id: i16| {
        layer
            .channels
            .iter()
            .position(|c| c.id == id && c.data.len() == len)
    };
    let (Some(r), Some(g), Some(b)) = (plane(0), plane(1), plane(2)) else {
        return Err(HealError::NoRasterLayer);
    };
    let alpha = plane(-1);
    let keep_alpha = layer_transparency_locked(layer);
    let data = (0..len)
        .map(|i| {
            let a = alpha.map_or(255, |a| layer.channels[a].data[i]);
            [
                layer.channels[r].data[i],
                layer.channels[g].data[i],
                layer.channels[b].data[i],
                a,
            ]
        })
        .collect();
    let mut img = RgbaImage {
        width: w,
        height: h,
        data,
    };
    let local = PsdRect {
        top: region.top - rect.top,
        left: region.left - rect.left,
        bottom: region.bottom - rect.top,
        right: region.right - rect.left,
    };
    let Some(dirty) = op(&mut img, local, coverage) else {
        return Ok(None);
    };
    for y in dirty.top..dirty.bottom {
        for x in dirty.left..dirty.right {
            let i = (y * w + x) as usize;
            let px = img.data[i];
            for (c, plane) in [r, g, b].into_iter().enumerate() {
                layer.channels[plane].data[i] = px[c];
            }
            if let (Some(a), false) = (alpha, keep_alpha) {
                layer.channels[a].data[i] = px[3];
            }
        }
    }
    Ok(Some(PsdRect {
        top: dirty.top + rect.top,
        left: dirty.left + rect.left,
        bottom: dirty.bottom + rect.top,
        right: dirty.right + rect.left,
    }))
}
