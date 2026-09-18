use crate::history::History;
use cxx_qt_lib::QImage;
use pictura_core::Document;
use pictura_paint::Stroke;
use pictura_select::Selection;
use std::collections::HashMap;

/// Backing Rust state for [`super::qobject::PictureView`].
pub struct PictureViewRust {
    pub(super) image: QImage,
    pub(super) doc: Option<Document>,
    pub(super) selection: Option<Selection>,
    /// CS6 "last deselected" memory: the selection replaced by Deselect or a
    /// New-mode commit. Restored by `reselect`.
    pub(super) deselected_selection: Option<Selection>,
    /// Selection captured at the start of a move-selection drag; `None` when no
    /// drag is active. Restored by `cancel_selection_move`.
    pub(super) selection_move_origin: Option<Selection>,
    pub(super) history: History,
    pub(super) path: Option<String>,
    pub(super) dirty: bool,
    pub(super) interop: Option<crate::gpu::InteropState>,
    pub(super) pending_lasso: Vec<(i32, i32)>,
    pub(super) pending_lasso_mode: String,
    pub(super) stroke: Option<Stroke>,
    pub(super) stroke_label: String,
    pub(super) move_base: Option<QImage>,
    pub(super) move_layer: Option<QImage>,
    pub(super) move_x: i32,
    pub(super) move_y: i32,
    pub(super) move_opacity: i32,
    pub(super) move_prepared_revision: u64,
    pub(super) move_prepared_layer: i32,
    pub(super) move_preview_cache_hit: bool,
    pub(super) opacity_preview_changed: bool,
    pub(super) fill_preview_changed: bool,
    pub(super) content_revision: u64,
    pub(super) gpu_compute: bool,
    pub(super) display_dirty: bool,
    pub(super) link_sets: HashMap<String, u32>,
}

impl Default for PictureViewRust {
    fn default() -> Self {
        Self {
            image: QImage::default(),
            doc: None,
            selection: None,
            deselected_selection: None,
            selection_move_origin: None,
            history: History::default(),
            path: None,
            dirty: false,
            interop: None,
            pending_lasso: Vec::new(),
            pending_lasso_mode: String::new(),
            stroke: None,
            stroke_label: String::new(),
            move_base: None,
            move_layer: None,
            move_x: 0,
            move_y: 0,
            move_opacity: 0,
            move_prepared_revision: 0,
            move_prepared_layer: -1,
            move_preview_cache_hit: false,
            opacity_preview_changed: false,
            fill_preview_changed: false,
            content_revision: 0,
            gpu_compute: true,
            display_dirty: false,
            link_sets: HashMap::new(),
        }
    }
}
