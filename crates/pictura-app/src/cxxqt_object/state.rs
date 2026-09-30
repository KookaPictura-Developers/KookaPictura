use crate::history::History;
use cxx_qt_lib::QImage;
use pictura_core::{Document, PixelBuffer, PsdRect};
use pictura_paint::{HealStroke, Stroke};
use pictura_render::{CanvasDamage, ViewPyramid};
use pictura_select::Selection;
use std::collections::HashMap;

/// The live Free Transform gesture mode. `Free` is the existing similarity
/// transform; the other three edit a document-space target quad.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum TransformMode {
    #[default]
    Free,
    Skew,
    Distort,
    Perspective,
}

impl TransformMode {
    /// Parse the command mode name, or `None` for an unknown string.
    pub(super) fn parse(name: &str) -> Option<Self> {
        match name {
            "skew" => Some(Self::Skew),
            "distort" => Some(Self::Distort),
            "perspective" => Some(Self::Perspective),
            _ => None,
        }
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Skew => "skew",
            Self::Distort => "distort",
            Self::Perspective => "perspective",
        }
    }

    pub(super) fn is_projective(self) -> bool {
        self != Self::Free
    }
}

/// The live Free Transform session: the target path, the source rect, the
/// current similarity transform about that rect's centre, and the drag state.
pub struct TransformSession {
    pub(super) path: String,
    pub(super) orig_rect: PsdRect,
    pub(super) scale_x: f64,
    pub(super) scale_y: f64,
    pub(super) angle: f64,
    pub(super) dx: f64,
    pub(super) dy: f64,
    /// Active handle: 0..=7 scale, 8 rotate, 9 move, -1 none.
    pub(super) handle: i32,
    pub(super) press_x: f64,
    pub(super) press_y: f64,
    /// `[scale_x, scale_y, angle, dx, dy]` captured at press.
    pub(super) start: [f64; 5],
    pub(super) dragging: bool,
    /// Gesture mode; `Free` uses the similarity scalars.
    pub(super) mode: TransformMode,
    /// Live target quad for a projective mode; `None` in `Free`.
    pub(super) quad: Option<[(f64, f64); 4]>,
    /// Quad captured at begin/press for a projective gesture.
    pub(super) start_quad: [(f64, f64); 4],
}

/// Backing Rust state for [`super::qobject::PictureView`].
pub struct PictureViewRust {
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
    /// Panel path of the single active layer, or `None` when the selection is
    /// empty or multiple. The shared resolver turns this into the edit target.
    pub(super) active_layer: Option<String>,
    pub(super) dirty: bool,
    /// Output format remembered from the import source, used to preselect the
    /// Save As filter: a lowercased extension (`"png"`) or `"psd"` when native.
    pub(super) source_format: String,
    pub(super) interop: Option<crate::gpu::InteropState>,
    pub(super) pending_lasso: Vec<(i32, i32)>,
    pub(super) pending_lasso_mode: String,
    pub(super) stroke: Option<Stroke>,
    pub(super) stroke_label: String,
    /// Dabs received since the last in-stroke present, waiting for `flush_present`.
    pub(super) pending_present: Option<PsdRect>,
    /// True from the present that opens a frame until its flush runs, so the
    /// frame's later dabs accumulate instead of presenting again.
    pub(super) present_flush_due: bool,
    /// The live healing gesture (Spot Healing Brush / Healing Brush): a
    /// coverage mask accumulated over a layer, healed on release.
    pub(super) heal_stroke: Option<HealStroke>,
    pub(super) move_base: Option<QImage>,
    pub(super) move_layer: Option<QImage>,
    pub(super) move_x: i32,
    pub(super) move_y: i32,
    pub(super) move_opacity: i32,
    pub(super) move_prepared_revision: u64,
    pub(super) move_prepared_layer: i32,
    pub(super) move_preview_cache_hit: bool,
    pub(super) transform_session: Option<TransformSession>,
    pub(super) opacity_preview_changed: bool,
    pub(super) fill_preview_changed: bool,
    pub(super) content_revision: u64,
    pub(super) gpu_compute: bool,
    pub(super) color_policy: pictura_codec::Policy,
    /// Outstanding display damage; the canvas redraws the region `take` reports.
    /// No longer consumed by present: it keys on [`Self::canvas_revision`].
    pub(super) damage: CanvasDamage,
    /// Non-consuming revision of the displayed canvas pixels, bumped whenever
    /// `image` or the pyramid is rebuilt. Lets the shell cache without draining
    /// the damage account before `image` reads it.
    pub(super) canvas_revision: u64,
    /// The halved-level view of the composite, fed by `recomposite` and
    /// `refresh_region`.
    pub(super) pyramid: ViewPyramid,
    /// The 4-plane straight **sRGB** level-0 frame the display image, pyramid,
    /// and canvas crops all share, so every path shows the working-space
    /// conversion rather than the raw composite. The display `QImage` is built
    /// from this frame on demand (`PictureView::image`) rather than cached, so
    /// only one full-resolution frame is held.
    pub(super) level0: Option<PixelBuffer>,
    pub(super) link_sets: HashMap<String, u32>,
    /// Magnetic Lasso edge field, live for one gesture (`magnetic_begin` to
    /// `magnetic_end`): one `f32` per pixel, too costly to rebuild per move.
    pub(super) edge_map: Option<pictura_select::EdgeMap>,
    /// The Ruler tool's measuring line: view state, never saved or undone.
    pub(super) ruler: Option<pictura_core::Ruler>,
}

impl Default for PictureViewRust {
    fn default() -> Self {
        Self {
            doc: None,
            selection: None,
            deselected_selection: None,
            selection_move_origin: None,
            history: History::default(),
            path: None,
            active_layer: None,
            dirty: false,
            source_format: "psd".to_string(),
            interop: None,
            pending_lasso: Vec::new(),
            pending_lasso_mode: String::new(),
            stroke: None,
            stroke_label: String::new(),
            pending_present: None,
            present_flush_due: false,
            heal_stroke: None,
            move_base: None,
            move_layer: None,
            move_x: 0,
            move_y: 0,
            move_opacity: 0,
            move_prepared_revision: 0,
            move_prepared_layer: -1,
            move_preview_cache_hit: false,
            transform_session: None,
            opacity_preview_changed: false,
            fill_preview_changed: false,
            content_revision: 0,
            gpu_compute: true,
            color_policy: pictura_codec::Policy::Preserve,
            damage: CanvasDamage::default(),
            canvas_revision: 0,
            pyramid: ViewPyramid::default(),
            level0: None,
            link_sets: HashMap::new(),
            edge_map: None,
            ruler: None,
        }
    }
}
