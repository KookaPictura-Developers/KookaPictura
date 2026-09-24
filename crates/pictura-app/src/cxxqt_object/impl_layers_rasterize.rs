//! Solid/gradient fill creation and the Rasterize subset bridge commands.
//!
//! A fill layer is an opaque `SoCo` or `GdFl` block (solid or gradient), so it
//! lives in the existing adjustment slot and needs no new `Layer` field. Each
//! successful command recomposites then records exactly one undo state; a
//! refusal records nothing.

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

impl qobject::PictureView {
    /// Append a solid-color fill layer for `rgba` (`0xAARRGGBB`) at the top of
    /// the stack. Records one "Color Fill" state on success. Returns the new
    /// path, or empty without a document.
    pub fn add_solid_fill(mut self: Pin<&mut Self>, rgba: u32) -> QString {
        let color = [
            (rgba >> 16) as u8,
            (rgba >> 8) as u8,
            rgba as u8,
            (rgba >> 24) as u8,
        ];
        let created = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::add_solid_fill(doc, "", color),
            None => String::new(),
        };
        if !created.is_empty() {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Color Fill");
        }
        QString::from(created.as_str())
    }

    /// Append a black-to-white Linear gradient fill layer at the top of the
    /// stack. Records one "Gradient Fill" state on success. Returns the new
    /// path, or empty without a document.
    pub fn add_gradient_fill(mut self: Pin<&mut Self>) -> QString {
        let created = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::add_gradient_fill(doc, ""),
            None => String::new(),
        };
        if !created.is_empty() {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Gradient Fill");
        }
        QString::from(created.as_str())
    }

    /// Whether the layer at `path` is a decodable solid-color or gradient fill
    /// layer.
    pub fn layer_is_fill_content(&self, path: &QString) -> bool {
        self.rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_render::resolve_path(doc, &path.to_string()))
            .is_some_and(pictura_render::is_fill_content_layer)
    }

    /// `Rasterize Fill Content`: bake the fill at `path` into pixels and clear
    /// its fill data. Records one "Rasterize Fill Content" state on success.
    /// Returns false (no state) when `path` is not a decodable fill layer.
    pub fn rasterize_fill_content(mut self: Pin<&mut Self>, path: &QString) -> bool {
        self.as_mut()
            .rasterize_path(&path.to_string(), "Rasterize Fill Content")
    }

    /// `Rasterize Layer` on a type layer or a fill-content layer. A type layer
    /// is rasterized through the bundled text backend and records one
    /// "Rasterize Type" state; a fill layer records "Rasterize Layer". Returns
    /// false (no state) for any other kind.
    pub fn rasterize_layer(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        if path.is_empty() {
            return false;
        }
        let (changed, label) = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                if pictura_render::render_text_layer(doc, &path) {
                    (true, "Rasterize Type")
                } else if pictura_render::rasterize_fill_content(doc, &path) {
                    (true, "Rasterize Layer")
                } else {
                    (false, "Rasterize Layer")
                }
            }
            None => (false, "Rasterize Layer"),
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record(label);
        }
        changed
    }

    /// `Rasterize All Layers`: rasterize every fill-content or type layer. Records one
    /// "Rasterize All Layers" state only when at least one was rasterized.
    /// Returns how many were rasterized.
    pub fn rasterize_all_layers(mut self: Pin<&mut Self>) -> i32 {
        let count = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::rasterize_all_layers(doc),
            None => 0,
        };
        if count > 0 {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Rasterize All Layers");
        }
        count as i32
    }

    /// `Rasterize Type`: materialize the type layer at `path` through the
    /// bundled backend and record one "Rasterize Type" state on success.
    /// Returns false (no state) for a non-type layer.
    pub fn rasterize_type(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        if path.is_empty() {
            return false;
        }
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::render_text_layer(doc, &path),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Rasterize Type");
        }
        changed
    }

    /// Whether the layer at `path` is a type layer. Read-only.
    pub fn layer_is_type(&self, path: &QString) -> bool {
        self.rust()
            .doc
            .as_ref()
            .and_then(|doc| pictura_render::resolve_path(doc, &path.to_string()))
            .is_some_and(|layer| layer.is_type())
    }

    fn rasterize_path(mut self: Pin<&mut Self>, path: &str, label: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::rasterize_fill_content(doc, path),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record(label);
        }
        changed
    }
}
