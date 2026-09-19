//! Document-scope wrappers around the M10 `pictura-ops` image operations
//! (`IMG-001`, `IMG-002`, `IMG-003`). See `docs/dev/m12-document-ops.md`.

mod canvas;
mod crop;
mod layer_ops;
mod orient;
mod resize;

pub use canvas::resize_canvas_document;
pub use crop::{crop_document, translate_layer, translate_layer_active, translate_layer_rect};
pub use layer_ops::{
    add_gradient_fill, add_group, add_group_full, add_group_in, add_layer, add_layer_full,
    add_layer_in, add_solid_fill, apply_visibility, background_from_layer,
    can_convert_to_smart_object, can_edit_smart_object_contents, can_merge_scope, can_merge_target,
    can_move_path_to, can_rasterize_smart_object, can_replace_smart_object_contents,
    convert_to_smart_object, delete_hidden_layers, delete_paths, duplicate_layer, duplicate_paths,
    flatten, flatten_rows, group_layer, group_paths, is_background, is_fill_content_layer,
    is_visible_in_panel, layer_from_background, layer_via_copy, layer_via_cut, merge_scope,
    move_path, move_path_to, move_selection_content, neutral_color, next_layer_name,
    open_as_smart_object, parent_path, place_smart_object, rasterize_all_fill_content,
    rasterize_fill_content, rasterize_smart_object, rename_path, replace_smart_object_contents,
    resolve_path, resolve_path_mut, select_similar, set_blend_paths, set_color_paths,
    set_fill_paths, set_lock_paths, set_opacity_paths, set_visible_paths,
    smart_object_source_bytes, ungroup_layer, ungroup_paths, MergeError, MergeOutcome, MergeScope,
    NewLayerSpec,
};
pub use orient::{flip_document, rotate_document};
pub use resize::resize_document;

/// Reject a zero width or height with `InvalidParams`.
pub(crate) fn valid_size(width: u32, height: u32) -> Result<(), pictura_ops::OpsError> {
    if width == 0 || height == 0 {
        return Err(pictura_ops::OpsError::InvalidParams(
            "width and height must be >= 1".into(),
        ));
    }
    Ok(())
}

/// Rebuild the document composite from its (already transformed) layer tree.
pub(crate) fn recompute(doc: &mut pictura_core::Document) {
    doc.composite = crate::composite_rgba(doc);
}

/// Visit each layer depth-first, groups before their children.
pub(crate) fn for_each_layer(
    layers: &mut [pictura_core::Layer],
    f: &mut impl FnMut(&mut pictura_core::Layer),
) {
    for layer in layers.iter_mut() {
        f(layer);
        for_each_layer(&mut layer.children, f);
    }
}

/// Resample one planar 8-bit channel from `sw×sh` to `dw×dh`.
///
/// The caller validates the sizes, so a rejected `resize` (which would mean a
/// malformed plane) falls back to passing the input through untouched.
pub(crate) fn resample_plane(
    data: &[u8],
    sw: u32,
    sh: u32,
    dw: u32,
    dh: u32,
    resample: pictura_ops::Resample,
) -> Vec<u8> {
    let mut plane = pictura_core::PixelBuffer::new(sw, sh, 1);
    let n = plane.data.len().min(data.len());
    plane.data[..n].copy_from_slice(&data[..n]);
    pictura_ops::resize(&plane, dw, dh, resample)
        .map(|b| b.data)
        .unwrap_or_else(|_| data.to_vec())
}
