//! Layer creation and grouping at document scope (M37).
//!
//! Pure functions over `&mut Document`, following the bottom-first convention of
//! `Document::layers` (index 0 is the bottom of the stack).

mod create;
mod paths;
mod properties;
#[cfg(test)]
mod tests;

pub use create::{
    add_group, add_group_in, add_layer, add_layer_in, duplicate_layer, group_layer,
    next_layer_name, ungroup_layer,
};
pub use paths::{flatten_rows, is_background, parent_path, resolve_path, resolve_path_mut};
pub use properties::{
    apply_visibility, delete_paths, duplicate_paths, group_paths, move_path, move_path_to,
    rename_path, set_blend_paths, set_color_paths, set_fill_paths, set_lock_paths,
    set_opacity_paths, set_visible_paths, ungroup_paths,
};
