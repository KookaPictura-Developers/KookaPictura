//! Layer creation and grouping at document scope (M37).
//!
//! Pure functions over `&mut Document`, following the bottom-first convention of
//! `Document::layers` (index 0 is the bottom of the stack).

mod align;
mod clipboard;
mod clipping;
mod create;
mod merge;
#[cfg(test)]
mod merge_tests;
mod move_content;
mod paths;
mod perspective_crop;
mod properties;
mod rasterize;
mod shape_layer;
mod shape_style;
mod smart_object;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_clipboard;
#[cfg(test)]
mod tests_perspective_crop;
#[cfg(test)]
mod tests_via;
mod transform;
mod transform_native;
#[cfg(test)]
mod transform_tests;
mod via;
mod warp;
mod warp_styles;
#[cfg(test)]
mod warp_styles_tests;

pub use align::{align_layers, can_align, can_distribute, distribute_layers, AlignEdge};
pub use clipboard::{
    clear_layer, copy_layer, copy_merged, coverage_bounds, paste_clip, Clip, PasteMode,
};
pub use clipping::{
    can_create_clipping_mask, can_release_clipping_mask, create_clipping_mask,
    release_clipping_mask,
};
pub(crate) use create::insert_node;
pub use create::{
    add_gradient_fill, add_group, add_group_full, add_group_in, add_layer, add_layer_full,
    add_layer_in, add_raster_layer_from_rgba, add_solid_fill, background_from_layer,
    duplicate_layer, group_layer, layer_from_background, neutral_color, next_layer_name,
    ungroup_layer, NewLayerSpec,
};
pub use merge::{
    can_merge_scope, can_merge_target, flatten, is_visible_in_panel, merge_scope, MergeError,
    MergeOutcome, MergeScope,
};
pub use move_content::move_selection_content;
pub use paths::{flatten_rows, is_background, parent_path, resolve_path, resolve_path_mut};
pub use perspective_crop::{perspective_crop, perspective_crop_refusal, perspective_crop_size};
pub use properties::{
    apply_visibility, can_move_path_to, delete_hidden_layers, delete_paths, duplicate_paths,
    group_paths, move_path, move_path_to, rename_path, select_similar, set_blend_paths,
    set_color_paths, set_fill_paths, set_lock_paths, set_opacity_paths, set_visible_paths,
    ungroup_paths,
};
pub use rasterize::{is_fill_content_layer, rasterize_all_layers, rasterize_fill_content};
pub use shape_layer::{
    add_shape_layer, is_shape_layer, layer_live_shape, layer_shape_paths, set_layer_live_shape,
    set_layer_shape_paths, shape_coverage, shape_fill_color,
};
pub use shape_style::{
    resize_shape, set_shape_fill, set_shape_stroke, shape_bounds, shape_fill, shape_stroke,
    ShapeStroke,
};
pub use smart_object::{
    can_convert_to_smart_object, can_edit_smart_object_contents, can_rasterize_smart_object,
    can_replace_smart_object_contents, convert_for_smart_filters, convert_to_smart_object,
    open_as_smart_object, place_smart_object, rasterize_smart_object,
    replace_smart_object_contents, smart_object_source_bytes,
};
pub use transform::{transform_layer, transform_layer_quad, LayerTransform};
pub use via::{layer_via_copy, layer_via_cut};
pub use warp::{identity_mesh, transform_layer_warp, WarpMesh, WarpParams};
pub use warp_styles::{style_mesh, WarpStyle};
