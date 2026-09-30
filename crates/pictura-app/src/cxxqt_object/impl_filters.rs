use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

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
            // the composited result; a layer with an effect block still gets a
            // correct (if slower) region composite.
            let rect = layer.rect;
            pictura_render::apply_filter(layer, &filter, mask.as_ref(), gpu_compute)
                .is_ok()
                .then_some(rect)
        };
        if let Some(rect) = applied {
            self.as_mut().refresh_region(rect);
            self.as_mut().record("Filter");
            return true;
        }
        false
    }
}
