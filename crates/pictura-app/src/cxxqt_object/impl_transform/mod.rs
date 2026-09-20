mod dispatch;
mod geometry;
mod session;

use super::helpers::*;
use super::helpers_composite::*;
use cxx_qt_lib::QImage;
use pictura_core::{layer_move_locked, Document};

/// Build the move-preview base for layer `index`: the authoritative composite
/// with the layer's rect recomposited as if hidden, then the layer's prior
/// `visible` flag restored, so starting a move never reveals an invisible layer.
pub(super) fn build_move_preview_base(
    doc: &mut Document,
    index: usize,
    gpu_compute: bool,
) -> QImage {
    let rect = doc.layers[index].rect;
    let composite_ok = doc.composite.width == doc.width
        && doc.composite.height == doc.height
        && !doc.composite.data.is_empty();
    let was_visible = doc.layers[index].visible;
    match move_preview_region(rect, doc.width, doc.height, composite_ok) {
        Some((x0, y0, ..)) => {
            doc.layers[index].visible = false;
            let (region, _backend) =
                pictura_render::composite_region_active(doc, rect, gpu_compute);
            doc.layers[index].visible = was_visible;
            let mut base_buffer = doc.composite.clone();
            patch_buffer_region(&mut base_buffer, &region, x0, y0);
            buffer_to_image(&base_buffer)
        }
        None => {
            // ponytail: full-composite fallback for a missing/mismatched
            // composite; the region path covers the common case.
            doc.layers[index].visible = false;
            let base = document_to_image(doc, gpu_compute);
            doc.layers[index].visible = was_visible;
            base
        }
    }
}

/// Duplicate the active top-level pixel layer, repointing `active` at the copy.
pub(super) fn duplicate_move_target(
    doc: &mut Document,
    active: &mut Option<String>,
) -> Option<i32> {
    let index: usize = active.as_deref()?.parse().ok()?;
    active_pixel_layer(doc, active.as_deref())?;
    if layer_move_locked(doc.layers.get(index)?) {
        return None;
    }
    let new_index = pictura_render::duplicate_layer(doc, index as i32);
    *active = Some(new_index.to_string());
    Some(new_index)
}
