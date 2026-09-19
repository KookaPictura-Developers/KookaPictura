//! Smart-object conversion bridge command.
//!
//! `Convert to Smart Object` authors an embedded source from the layer's raster
//! and attaches it while keeping the raster proxy, so the composite is
//! unchanged. Each successful command records exactly one undo state; a refusal
//! records nothing.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::SmartObjectKind;

impl qobject::PictureView {
    /// Whether `path` resolves to a convertible raster pixel layer. Read-only.
    pub fn layer_can_convert_to_smart_object(&self, path: &QString) -> bool {
        self.rust()
            .doc
            .as_ref()
            .is_some_and(|doc| pictura_render::can_convert_to_smart_object(doc, &path.to_string()))
    }

    /// `Convert to Smart Object`: author an embedded source from `path`'s raster
    /// and attach it, keeping the raster proxy. Records one "Convert to Smart
    /// Object" state on success; false (no state) for an ineligible target.
    pub fn convert_to_smart_object(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::convert_to_smart_object(doc, &path.to_string()),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Convert to Smart Object");
        }
        changed
    }

    /// Whether `path` resolves to a rasterizable smart-object layer. Read-only.
    pub fn layer_can_rasterize_smart_object(&self, path: &QString) -> bool {
        self.rust()
            .doc
            .as_ref()
            .is_some_and(|doc| pictura_render::can_rasterize_smart_object(doc, &path.to_string()))
    }

    /// `Rasterize Smart Object`: consume the object at `path`, keeping its raster
    /// proxy (or materializing the embedded source), and drop the preserved
    /// blocks. Records one "Rasterize Smart Object" state on success; false (no
    /// state) for an ineligible target.
    pub fn rasterize_smart_object(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::rasterize_smart_object(doc, &path.to_string()),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Rasterize Smart Object");
        }
        changed
    }

    /// Self-test probe: `<kind>:<payload-len>` for `path`'s smart object, or
    /// empty when the layer has none. Read-only.
    pub fn layer_smart_object_state(&self, path: &QString) -> QString {
        let state = self
            .rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_render::resolve_path(doc, &path.to_string()))
            .and_then(|layer| layer.smart_object.as_ref())
            .map(|so| {
                let kind = match so.kind {
                    SmartObjectKind::Embedded => "embedded",
                    SmartObjectKind::External => "external",
                    SmartObjectKind::Alias => "alias",
                    SmartObjectKind::Unresolved => "unresolved",
                };
                format!("{kind}:{}", so.payload.as_ref().map_or(0, Vec::len))
            })
            .unwrap_or_default();
        QString::from(state.as_str())
    }
}
