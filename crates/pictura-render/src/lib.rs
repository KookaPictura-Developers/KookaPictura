//! CPU compositing for Kooka Pictura.
//!
//! M2 scope: composite the document layer stack (opacity, raster masks, groups,
//! 27 blend modes) into an image. Spec: `docs/05-layers/blend-modes.md` and the
//! W3C Compositing and Blending Level 1 model. GPU acceleration is M2.5.
//!
//! Contract owned by task M2-A:
//!
//! ```ignore
//! /// 4-channel (R,G,B,A) planar, straight alpha, 8-bit, at document size.
//! pub fn composite_rgba(doc: &pictura_core::Document) -> pictura_core::PixelBuffer;
//! ```
//!
//! # Groups and pass-through
//!
//! A group whose blend mode is [`BlendMode::PassThrough`] is composited as a
//! true pass-through when **opacity is 255 and it has no mask**: its children
//! are composited directly onto the running canvas, so child blend modes see
//! the backdrop outside the group. Any other blend mode is an **isolated**
//! group: children composite onto a private transparent buffer that is then
//! blended as one layer with the group's mode, opacity, and mask.
//!
//! Approximation: a pass-through group with non-255 opacity or a mask falls
//! back to isolated compositing with `PassThrough` treated as `Normal`. Real
//! Photoshop applies the group opacity/mask to the pass-through result while
//! still letting children blend against the parent backdrop; that mixed model
//! is not implemented. Exact pass-through parity is limited to the 255/no-mask
//! case (see the test below).

mod composite;
pub(crate) use composite::{channel, mask_alpha, render_smart_source, sample};
pub use composite::{
    composite_rgba, decode_adjustment, encode_brightness_contrast, encode_gradient_map,
    encode_hue_saturation, encode_invert, encode_photo_filter, encode_posterize,
    encode_solid_color_fill, encode_threshold,
};
pub use pictura_adjust::{GradientFillParams, GradientKind, GradientStop};

mod fill;
pub use fill::encode_gradient_fill;

pub mod gpu;
pub use gpu::{
    composite_active, composite_gpu, composite_gpu_or_cpu, composite_region_active, gpu_available,
    Backend, GpuError,
};

mod filter;
pub use filter::apply_filter;

mod gpu_filter;
pub use gpu_filter::{apply_filter_active, filter_gpu_available};

pub mod document_ops;
pub use document_ops::{
    add_gradient_fill, add_group, add_group_full, add_group_in, add_layer, add_layer_full,
    add_layer_in, add_raster_layer_from_rgba, add_solid_fill, apply_visibility,
    background_from_layer, can_convert_to_smart_object, can_edit_smart_object_contents,
    can_merge_scope, can_merge_target, can_move_path_to, can_rasterize_smart_object,
    can_replace_smart_object_contents, convert_to_smart_object, crop_document,
    delete_hidden_layers, delete_paths, duplicate_layer, duplicate_paths, flatten, flatten_rows,
    flip_document, group_layer, group_paths, is_background, is_fill_content_layer,
    is_visible_in_panel, layer_from_background, layer_via_copy, layer_via_cut, merge_scope,
    move_path, move_path_to, move_selection_content, neutral_color, next_layer_name,
    open_as_smart_object, parent_path, place_smart_object, rasterize_all_fill_content,
    rasterize_fill_content, rasterize_smart_object, rename_path, replace_smart_object_contents,
    resize_canvas_document, resize_document, resolve_path, resolve_path_mut, rotate_document,
    select_similar, set_blend_paths, set_color_paths, set_fill_paths, set_lock_paths,
    set_opacity_paths, set_visible_paths, smart_object_source_bytes, transform_layer,
    translate_layer, translate_layer_active, translate_layer_rect, ungroup_layer, ungroup_paths,
    LayerTransform, MergeError, MergeOutcome, MergeScope, NewLayerSpec,
};

pub use pictura_ops::{Anchor, Resample};

#[cfg(test)]
use composite::blend;

#[cfg(test)]
mod tests;
