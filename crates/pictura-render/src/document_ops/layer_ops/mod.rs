//! Layer creation and grouping at document scope (M37).
//!
//! Pure functions over `&mut Document`, following the bottom-first convention of
//! `Document::layers` (index 0 is the bottom of the stack).

mod create;
mod merge;
#[cfg(test)]
mod merge_tests;
mod move_content;
mod paths;
mod properties;
mod rasterize;
mod smart_object;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_via;
mod via;

pub use create::{
    add_group, add_group_full, add_group_in, add_layer, add_layer_full, add_layer_in,
    add_solid_fill, background_from_layer, duplicate_layer, group_layer, layer_from_background,
    neutral_color, next_layer_name, ungroup_layer, NewLayerSpec,
};
pub use merge::{
    can_merge_scope, can_merge_target, flatten, is_visible_in_panel, merge_scope, MergeError,
    MergeOutcome, MergeScope,
};
pub use move_content::move_selection_content;
pub use paths::{flatten_rows, is_background, parent_path, resolve_path, resolve_path_mut};
pub use properties::{
    apply_visibility, can_move_path_to, delete_hidden_layers, delete_paths, duplicate_paths,
    group_paths, move_path, move_path_to, rename_path, select_similar, set_blend_paths,
    set_color_paths, set_fill_paths, set_lock_paths, set_opacity_paths, set_visible_paths,
    ungroup_paths,
};
pub use rasterize::{is_fill_content_layer, rasterize_all_fill_content, rasterize_fill_content};
pub use smart_object::{
    can_convert_to_smart_object, can_rasterize_smart_object, can_replace_smart_object_contents,
    convert_to_smart_object, open_as_smart_object, place_smart_object, rasterize_smart_object,
    replace_smart_object_contents, smart_object_source_bytes,
};
pub use via::{layer_via_copy, layer_via_cut};
