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

    /// `File > Place…`: read `file_path`, decode it as a PSD/PSB source, and
    /// append it as a topmost channel-less embedded smart-object layer. Records
    /// one "Place" state on success; empty (no state) when the file is missing,
    /// unreadable, or not a PSD/PSB document.
    pub fn place_smart_object(mut self: Pin<&mut Self>, file_path: &QString) -> QString {
        let path = file_path.to_string();
        let name = std::path::Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let created = match (std::fs::read(&path), self.as_mut().rust_mut().doc.as_mut()) {
            (Ok(bytes), Some(doc)) => pictura_render::place_smart_object(doc, &name, &bytes),
            _ => None,
        };
        let Some(created) = created else {
            return QString::default();
        };
        self.as_mut().clear_link_sets();
        self.as_mut().recomposite();
        self.as_mut().record("Place");
        QString::from(created.as_str())
    }

    /// Whether `path` resolves to a replaceable smart-object layer. Read-only.
    pub fn layer_can_replace_smart_object_contents(&self, path: &QString) -> bool {
        self.rust().doc.as_ref().is_some_and(|doc| {
            pictura_render::can_replace_smart_object_contents(doc, &path.to_string())
        })
    }

    /// `Replace Contents…`: read `file_path`, swap the embedded source of the
    /// smart object at `path`, and keep the layer's transform. Records one
    /// "Replace Contents" state on success; false (no state) when the file is
    /// unreadable, not a PSD/PSB document, or the target is ineligible.
    pub fn replace_smart_object_contents(
        mut self: Pin<&mut Self>,
        path: &QString,
        file_path: &QString,
    ) -> bool {
        let file = file_path.to_string();
        let name = std::path::Path::new(&file)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let changed = match (std::fs::read(&file), self.as_mut().rust_mut().doc.as_mut()) {
            (Ok(bytes), Some(doc)) => {
                pictura_render::replace_smart_object_contents(doc, &path.to_string(), &name, &bytes)
            }
            _ => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Replace Contents");
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
