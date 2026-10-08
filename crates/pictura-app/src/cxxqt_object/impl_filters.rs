use super::filter_map::filter_from_kind_params;
use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use super::state::{FilterPreview, PictureViewRust};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::layer_pixel_locked;

impl qobject::PictureView {
    pub fn add_adjustment(mut self: Pin<&mut Self>, kind: &QString) -> bool {
        let mask = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            rust.selection
                .as_ref()
                .map(|selection| selection_to_mask(selection, doc))
        };
        let Some(layer) = adjustment_layer(&kind.to_string(), mask) else {
            return false;
        };
        let region = layer_visibility_region(&layer);
        let pushed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                doc.layers.push(layer);
                true
            }
            None => false,
        };
        if pushed {
            // A masked adjustment is confined to its mask; an unmasked one spans
            // the canvas and falls back to a full recomposite.
            match region {
                Some(rect) => self.as_mut().refresh_region(rect),
                None => self.as_mut().recomposite(),
            }
            self.as_mut().record("Adjustment");
        }
        pushed
    }

    pub fn apply_filter(mut self: Pin<&mut Self>, kind: &QString) -> bool {
        let Some(filter) = filter_from_kind(&kind.to_string()) else {
            return false;
        };
        let mask = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            rust.selection
                .as_ref()
                .map(|selection| selection_to_mask(selection, doc))
        };
        let active = self.rust().active_layer.clone();
        let applied = {
            let mut rust = self.as_mut().rust_mut();
            let gpu_compute = rust.gpu_compute;
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if !active_layer_visible(doc, active.as_deref()) {
                return false;
            }
            let Some(layer) = active_pixel_layer_mut(doc, active.as_deref()) else {
                return false;
            };
            // A filter only changes the active layer, so its clamped rect bounds
            // the composited result — unless a layer effect spills past that
            // rect, which needs a full recomposite.
            let region = (!layer_has_effects(layer)).then_some(layer.rect);
            pictura_render::apply_filter(layer, &filter, mask.as_ref(), gpu_compute)
                .is_ok()
                .then_some(region)
        };
        if let Some(region) = applied {
            match region {
                Some(rect) => self.as_mut().refresh_region(rect),
                None => self.as_mut().recomposite(),
            }
            self.as_mut().rust_mut().last_filter = Some((kind.to_string(), Vec::new()));
            self.as_mut().record("Filter");
            return true;
        }
        false
    }
}

/// Apply a filter to the single editable layer, either committing it or
/// rendering a non-committing preview.
///
/// On a preview, the first parameter change snapshots the layer's pixels and
/// every later change re-filters that snapshot, so previews never compound. A
/// commit restores the snapshot first (so it never double-applies) and records
/// the `(kind, params)` as the last filter. Returns the region to refresh, or
/// `None` for an unknown kind, a bad arity, or no editable layer.
pub(super) fn apply_filter_active(
    rust: &mut PictureViewRust,
    kind: &str,
    params: &[f64],
    commit: bool,
) -> Option<Option<pictura_core::PsdRect>> {
    apply_filter_active_region(rust, kind, params, commit, None)
}

/// As [`apply_filter_active`], but a preview is restricted to `preview_region`
/// (a document rect), so dragging a control only filters the visible section.
/// A commit always filters the whole layer.
pub(super) fn apply_filter_active_region(
    rust: &mut PictureViewRust,
    kind: &str,
    params: &[f64],
    commit: bool,
    preview_region: Option<pictura_core::PsdRect>,
) -> Option<Option<pictura_core::PsdRect>> {
    rust.filter_error = None;
    let Some(filter) = filter_from_kind_params(kind, params) else {
        rust.filter_error = Some(format!(
            "filter '{kind}' is not recognised or its parameters are wrong"
        ));
        return None;
    };
    let region = apply_filter_obj_active_region(rust, filter, commit, preview_region)?;
    if commit {
        rust.last_filter = Some((kind.to_string(), params.to_vec()));
    }
    Some(region)
}

/// As [`apply_filter_active_region`], but from a resolved [`pictura_filters::Filter`]
/// instead of a kind/params pair. Filters that are not in the parameterised
/// mapper (HDR Toning) reach the same snapshot/preview/commit core this way.
pub(super) fn apply_filter_obj_active_region(
    rust: &mut PictureViewRust,
    filter: pictura_filters::Filter,
    commit: bool,
    preview_region: Option<pictura_core::PsdRect>,
) -> Option<Option<pictura_core::PsdRect>> {
    apply_op_active_region(rust, &ActiveOp::Filter(filter), commit, preview_region)
}

/// A destructive edit of the active pixel layer: a filter, or an Image >
/// Adjustments adjustment (colour only, no neighbourhood).
pub(super) enum ActiveOp {
    Filter(pictura_filters::Filter),
    /// A Filter Gallery stack, applied in order.
    Filters(Vec<pictura_filters::Filter>),
    Adjustment(pictura_render::Adjustment),
}

/// Apply `op` to the active pixel layer within the selection, as a preview
/// (re-applied from the pre-preview pixels, restricted to `preview_region`)
/// or a commit. Returns the region to refresh; `None` with
/// `filter_error` set when refused.
pub(super) fn apply_op_active_region(
    rust: &mut PictureViewRust,
    op: &ActiveOp,
    commit: bool,
    preview_region: Option<pictura_core::PsdRect>,
) -> Option<Option<pictura_core::PsdRect>> {
    rust.filter_error = None;
    let (index, gpu_compute) = {
        let Some(doc) = rust.doc.as_ref() else {
            rust.filter_error = Some("there is no document".to_string());
            return None;
        };
        let active = rust.active_layer.as_deref();
        if !active_layer_visible(doc, active) {
            rust.filter_error = Some("the active layer is hidden".to_string());
            return None;
        }
        let Some(layer) = active_pixel_layer(doc, active) else {
            rust.filter_error = Some("select a single pixel layer to filter".to_string());
            return None;
        };
        if layer_pixel_locked(layer) {
            rust.filter_error = Some("the active layer is pixel-locked".to_string());
            return None;
        }
        let Some(index) = active.and_then(|p| p.parse::<usize>().ok()) else {
            rust.filter_error = Some("the active layer has no filterable target".to_string());
            return None;
        };
        (index, rust.gpu_compute)
    };
    let mask = {
        let doc = rust.doc.as_ref()?;
        rust.selection
            .as_ref()
            .map(|selection| selection_to_mask(selection, doc))
    };
    // A preview for another layer must not be silently abandoned: refuse so its
    // baseline stays intact until it is cancelled or committed.
    if rust
        .filter_preview
        .as_ref()
        .is_some_and(|p| p.layer_index != index)
    {
        rust.filter_error = Some("another layer has an open preview; cancel it first".to_string());
        return None;
    }
    let existing = rust
        .filter_preview
        .as_ref()
        .filter(|p| p.layer_index == index)
        .map(|p| p.original.clone());
    let Some(doc) = rust.doc.as_mut() else {
        rust.filter_error = Some("there is no document".to_string());
        return None;
    };
    let Some(layer) = doc.layers.get_mut(index) else {
        rust.filter_error = Some("the active layer no longer exists".to_string());
        return None;
    };
    if layer.is_group || layer.adjustment.is_some() {
        rust.filter_error = Some("the active layer cannot take a filter".to_string());
        return None;
    }
    // Re-filter from the pre-preview pixels, never from the previous preview.
    if let Some(baseline) = &existing {
        *layer = baseline.clone();
    }
    let snapshot = if !commit && existing.is_none() {
        Some(layer.clone())
    } else {
        None
    };
    let region_out = (!layer_has_effects(layer)).then_some(layer.rect);
    let filters = match op {
        ActiveOp::Filter(filter) => std::slice::from_ref(filter),
        ActiveOp::Filters(filters) => filters.as_slice(),
        ActiveOp::Adjustment(adjustment) => {
            // A point operation: the visible section alone is exact.
            let region = match preview_region {
                Some(visible) if !commit => visible,
                _ => layer.rect,
            };
            let applied =
                pictura_render::apply_adjustment_region(layer, adjustment, mask.as_ref(), region);
            return finish_op(rust, applied, commit, index, existing, snapshot, region_out);
        }
    };
    // Expand the visible rect by the support of every filter still to run, so
    // the viewport is exact after the whole stack; the renderer clamps it to
    // the layer.
    let mut apron: i32 = filters.iter().map(pictura_render::preview_apron).sum();
    let mut applied = Ok(());
    for filter in filters {
        applied = match preview_region {
            Some(visible) if !commit => {
                let expanded = pictura_core::PsdRect {
                    top: visible.top - apron,
                    left: visible.left - apron,
                    bottom: visible.bottom + apron,
                    right: visible.right + apron,
                };
                pictura_render::apply_filter_region(
                    layer,
                    filter,
                    mask.as_ref(),
                    gpu_compute,
                    expanded,
                )
            }
            _ => pictura_render::apply_filter(layer, filter, mask.as_ref(), gpu_compute),
        };
        if applied.is_err() {
            break;
        }
        apron -= pictura_render::preview_apron(filter);
    }
    finish_op(rust, applied, commit, index, existing, snapshot, region_out)
}

/// Record the outcome of [`apply_op_active_region`]: the error, or the open
/// preview's baseline (cleared on a commit).
fn finish_op(
    rust: &mut PictureViewRust,
    applied: Result<(), pictura_filters::FilterError>,
    commit: bool,
    index: usize,
    existing: Option<pictura_core::Layer>,
    snapshot: Option<pictura_core::Layer>,
    region: Option<pictura_core::PsdRect>,
) -> Option<Option<pictura_core::PsdRect>> {
    if let Err(error) = applied {
        rust.filter_error = Some(error.to_string());
        return None;
    }
    if commit {
        rust.filter_preview = None;
    } else {
        let original = existing.or(snapshot).expect("preview baseline");
        rust.filter_preview = Some(FilterPreview {
            layer_index: index,
            original,
        });
    }
    Some(region)
}

/// Discard an open preview, restoring the pre-preview pixels bit-identically.
/// Returns the region to refresh, or `None` when no preview is open.
pub(super) fn cancel_filter_preview(
    rust: &mut PictureViewRust,
) -> Option<Option<pictura_core::PsdRect>> {
    let preview = rust.filter_preview.take()?;
    let doc = rust.doc.as_mut()?;
    let layer = doc.layers.get_mut(preview.layer_index)?;
    *layer = preview.original;
    Some((!layer_has_effects(layer)).then_some(layer.rect))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cxxqt_object::tests::pixel_layer;
    use pictura_core::{Document, LockFlags};

    fn state() -> PictureViewRust {
        let mut doc = Document::new(
            16,
            16,
            pictura_core::ColorMode::Rgb,
            pictura_core::BitDepth::Eight,
        );
        let mut layer = pixel_layer("px", 16, 16, (40, 120, 200));
        // A left/right step so a blur has a gradient to smooth.
        for ch in layer.channels.iter_mut().filter(|c| c.id >= 0) {
            ch.data = (0..16 * 16)
                .map(|i| if i % 16 < 8 { 40u8 } else { 200 })
                .collect();
        }
        doc.layers = vec![layer];
        PictureViewRust {
            doc: Some(doc),
            active_layer: Some("0".to_string()),
            gpu_compute: false,
            ..Default::default()
        }
    }

    fn red(rust: &PictureViewRust) -> Vec<u8> {
        rust.doc.as_ref().unwrap().layers[0]
            .channels
            .iter()
            .find(|c| c.id == 0)
            .unwrap()
            .data
            .to_vec()
    }

    /// A 256×256 layer large enough that a section preview's apron does not
    /// span the whole layer, so a crop changes a positional filter's result.
    fn big_state() -> PictureViewRust {
        let mut doc = Document::new(
            256,
            256,
            pictura_core::ColorMode::Rgb,
            pictura_core::BitDepth::Eight,
        );
        let mut layer = pixel_layer("px", 256, 256, (40, 120, 200));
        for (c, ch) in layer.channels.iter_mut().filter(|c| c.id >= 0).enumerate() {
            let plane: Vec<u8> = (0..256 * 256)
                .map(|i| {
                    let x = i % 256;
                    let y = i / 256;
                    ((x * 7 + y * 13 + c * 53) % 256) as u8
                })
                .collect();
            ch.data = plane.into();
        }
        doc.layers = vec![layer];
        PictureViewRust {
            doc: Some(doc),
            active_layer: Some("0".to_string()),
            gpu_compute: false,
            ..Default::default()
        }
    }

    #[test]
    fn wrong_arity_is_refused_without_mutation() {
        let mut rust = state();
        let before = red(&rust);
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[1.0, 2.0], false).is_none());
        assert!(apply_filter_active(&mut rust, "bogus", &[], false).is_none());
        assert_eq!(red(&rust), before, "refusal must not mutate the layer");
        assert!(rust.filter_preview.is_none());
        assert!(rust.last_filter.is_none());
        assert!(
            rust.filter_error.as_deref().is_some_and(|e| !e.is_empty()),
            "a refusal leaves a reason"
        );
    }

    #[test]
    fn success_clears_the_refusal_reason() {
        let mut rust = state();
        assert!(apply_filter_active(&mut rust, "bogus", &[], false).is_none());
        assert!(rust.filter_error.is_some());
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[2.0], false).is_some());
        assert!(rust.filter_error.is_none(), "success clears the reason");
    }

    #[test]
    fn locked_layer_is_refused() {
        let mut rust = state();
        rust.doc.as_mut().unwrap().layers[0].lock =
            LockFlags::default().with(LockFlags::PIXELS, true);
        let before = red(&rust);
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[2.0], false).is_none());
        assert_eq!(red(&rust), before);
    }

    #[test]
    fn non_pixel_active_layer_is_refused() {
        let mut rust = state();
        let pixel_before = red(&rust);
        // An adjustment layer (no pixels) as the sole active target.
        let adj = adjustment_layer("invert", None).expect("adjustment");
        rust.doc.as_mut().unwrap().layers.push(adj);
        rust.active_layer = Some("1".to_string());
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[2.0], true).is_none());
        assert_eq!(red(&rust), pixel_before, "pixel layer untouched");
        assert!(rust.last_filter.is_none());
    }

    #[test]
    fn preview_on_another_layer_is_refused_and_keeps_baseline() {
        let mut rust = state();
        rust.doc
            .as_mut()
            .unwrap()
            .layers
            .push(pixel_layer("px2", 16, 16, (10, 20, 30)));
        let original = red(&rust);
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[4.0], false).is_some());
        assert!(rust
            .filter_preview
            .as_ref()
            .is_some_and(|p| p.layer_index == 0));
        // Switching the active layer must not steal layer 0's baseline.
        rust.active_layer = Some("1".to_string());
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[4.0], false).is_none());
        assert!(rust
            .filter_preview
            .as_ref()
            .is_some_and(|p| p.layer_index == 0));
        assert!(cancel_filter_preview(&mut rust).is_some());
        assert_eq!(red(&rust), original, "layer 0 baseline restored");
    }

    #[test]
    fn preview_re_filters_the_original_not_the_last_preview() {
        let mut rust = state();
        let original = red(&rust);
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[2.0], false).is_some());
        assert!(rust.filter_preview.is_some(), "preview state established");
        assert!(rust.last_filter.is_none(), "preview commits nothing");
        let after_two = red(&rust);
        // A second, different radius must equal filtering the original once.
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[6.0], false).is_some());
        let after_six = red(&rust);
        let mut expected = state();
        apply_filter_active(&mut expected, "gaussian-blur", &[6.0], false);
        assert_eq!(after_six, red(&expected));
        assert_ne!(after_six, after_two);
        assert_ne!(after_six, original);
    }

    #[test]
    fn cancel_restores_bit_identical() {
        let mut rust = state();
        let original = red(&rust);
        apply_filter_active(&mut rust, "gaussian-blur", &[4.0], false).expect("preview");
        assert_ne!(red(&rust), original, "preview changed pixels");
        let region = cancel_filter_preview(&mut rust).expect("preview open");
        assert!(region.is_some());
        assert_eq!(red(&rust), original, "cancel is bit-identical");
        assert!(rust.filter_preview.is_none());
        assert!(rust.last_filter.is_none());
    }

    #[test]
    fn commit_clears_preview_and_records_last_filter() {
        let mut rust = state();
        apply_filter_active(&mut rust, "gaussian-blur", &[3.0], false).expect("preview");
        assert!(apply_filter_active(&mut rust, "gaussian-blur", &[3.0], true).is_some());
        assert!(rust.filter_preview.is_none(), "commit closes the preview");
        assert_eq!(
            rust.last_filter,
            Some(("gaussian-blur".to_string(), vec![3.0]))
        );
        // Committing after a preview must equal a single fresh apply.
        let mut fresh = state();
        apply_filter_active(&mut fresh, "gaussian-blur", &[3.0], true).expect("commit");
        assert_eq!(red(&rust), red(&fresh), "no double application");
    }

    #[test]
    fn positional_kinds_preview_the_whole_layer() {
        use super::super::filter_tools::filter_preview_needs_whole_layer;

        let viewport = pictura_core::PsdRect {
            top: 200,
            left: 200,
            bottom: 232,
            right: 232,
        };
        for (kind, params) in [("diffuse", vec![0.0f64]), ("lighting-effects", Vec::new())] {
            assert!(filter_preview_needs_whole_layer(kind), "{kind}");

            // A cropped section preview does not match the commit...
            let mut cropped = big_state();
            apply_filter_active_region(&mut cropped, kind, &params, false, Some(viewport))
                .expect("section preview");

            // ...so the bridge escalates the kind to a whole-layer preview.
            let section = (!filter_preview_needs_whole_layer(kind)).then_some(viewport);
            let mut preview = big_state();
            apply_filter_active_region(&mut preview, kind, &params, false, section)
                .expect("whole-layer preview");

            let mut commit = big_state();
            apply_filter_active_region(&mut commit, kind, &params, true, None).expect("commit");

            let (preview_px, commit_px, cropped_px) = (red(&preview), red(&commit), red(&cropped));
            let w = 256usize;
            let mut cropped_differs = false;
            for y in viewport.top..viewport.bottom {
                for x in viewport.left..viewport.right {
                    let i = y as usize * w + x as usize;
                    assert_eq!(
                        preview_px[i], commit_px[i],
                        "{kind} whole-layer preview != commit at ({x},{y})"
                    );
                    if cropped_px[i] != commit_px[i] {
                        cropped_differs = true;
                    }
                }
            }
            assert!(
                cropped_differs,
                "{kind}: a cropped preview unexpectedly matched the commit"
            );
        }
    }
}
