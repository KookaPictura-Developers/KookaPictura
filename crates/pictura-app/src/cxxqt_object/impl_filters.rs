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
        let pushed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                doc.layers.push(layer);
                true
            }
            None => false,
        };
        if pushed {
            self.as_mut().recomposite();
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
            let Some(layer) = active_pixel_layer_mut(doc, active.as_deref()) else {
                return false;
            };
            pictura_render::apply_filter(layer, &filter, mask.as_ref(), gpu_compute).is_ok()
        };
        if applied {
            self.as_mut().recomposite();
            self.as_mut().record("Filter");
        }
        applied
    }
}
