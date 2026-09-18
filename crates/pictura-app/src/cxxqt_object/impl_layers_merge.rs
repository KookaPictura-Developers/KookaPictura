//! Merge and flatten bridge commands. Engine: `pictura_render::merge_scope` /
//! `flatten`; each successful command recomposites then records one undo state,
//! and every refusal records nothing and returns 0.

use super::helpers::*;
use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};

impl qobject::PictureView {
    /// Merge Down when `paths` names one layer, Merge Layers when it names two
    /// or more (CS6 shares one command, design D4). Returns the number of
    /// inputs replaced, 0 on refusal.
    pub fn merge_layers(mut self: Pin<&mut Self>, paths: &QStringList) -> i32 {
        let owned = list_of_strings(paths);
        if owned.len() == 1 {
            self.as_mut()
                .merge_scope_changed(pictura_render::MergeScope::Down(&owned[0]), "Merge Down")
        } else {
            self.as_mut()
                .merge_scope_changed(pictura_render::MergeScope::Selected(&owned), "Merge Layers")
        }
    }

    /// Whether [`merge_layers`] would apply to `paths`, without mutating the
    /// document or recording history. The UI gates the menu item on it.
    ///
    /// [`merge_layers`]: qobject::PictureView::merge_layers
    pub fn can_merge_layers(&self, paths: &QStringList) -> bool {
        let owned = list_of_strings(paths);
        if owned.is_empty() {
            return false;
        }
        let Some(doc) = self.rust().doc.as_ref() else {
            return false;
        };
        let scope = if owned.len() == 1 {
            pictura_render::MergeScope::Down(&owned[0])
        } else {
            pictura_render::MergeScope::Selected(&owned)
        };
        pictura_render::can_merge_scope(doc, &scope)
    }

    /// Whether [`merge_clipping_mask`] would apply to `path`, without mutating
    /// the document or recording history. The UI gates the menu item on it.
    ///
    /// [`merge_clipping_mask`]: qobject::PictureView::merge_clipping_mask
    pub fn can_merge_clipping_mask(&self, path: &QString) -> bool {
        let path = path.to_string();
        if path.is_empty() {
            return false;
        }
        let Some(doc) = self.rust().doc.as_ref() else {
            return false;
        };
        pictura_render::can_merge_scope(doc, &pictura_render::MergeScope::ClippingMask(&path))
    }

    /// Merge every eye-visible layer, anchored on `path`. Returns the number of
    /// inputs replaced, 0 on refusal (including a hidden active layer).
    pub fn merge_visible(mut self: Pin<&mut Self>, path: &QString) -> i32 {
        let path = path.to_string();
        self.as_mut()
            .merge_scope_changed(pictura_render::MergeScope::Visible(&path), "Merge Visible")
    }

    /// Collapse the clipping group above `path` into its raster base. Returns
    /// the number of inputs replaced, 0 on refusal.
    pub fn merge_clipping_mask(mut self: Pin<&mut Self>, path: &QString) -> i32 {
        let path = path.to_string();
        self.as_mut().merge_scope_changed(
            pictura_render::MergeScope::ClippingMask(&path),
            "Merge Clipping Mask",
        )
    }

    /// Flatten the whole tree into one opaque Background layer. Returns the
    /// number of layers replaced, 0 on refusal.
    pub fn flatten_image(mut self: Pin<&mut Self>) -> i32 {
        let replaced = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::flatten(doc).map_or(0, |outcome| outcome.replaced),
            None => 0,
        };
        if replaced > 0 {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Flatten Image");
        }
        replaced as i32
    }

    /// `Layer from Background…`: clear the flag and unlock the layer at `path`.
    /// Returns false (no history) when it is not the Background.
    pub fn layer_from_background(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::layer_from_background(doc, &path),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Layer from Background");
        }
        changed
    }

    /// `Background From Layer`: flag the layer at `path`, fill its transparent
    /// pixels with the background color, and move it to the bottom. Returns
    /// false (no history) for a group, an adjustment/fill-content layer, or an
    /// existing Background.
    pub fn background_from_layer(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let path = path.to_string();
        let changed = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => pictura_render::background_from_layer(doc, &path),
            None => false,
        };
        if changed {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record("Background From Layer");
        }
        changed
    }

    fn merge_scope_changed(
        mut self: Pin<&mut Self>,
        scope: pictura_render::MergeScope<'_>,
        label: &str,
    ) -> i32 {
        let replaced = match self.as_mut().rust_mut().doc.as_mut() {
            Some(doc) => {
                pictura_render::merge_scope(doc, scope).map_or(0, |outcome| outcome.replaced)
            }
            None => 0,
        };
        if replaced > 0 {
            self.as_mut().clear_link_sets();
            self.as_mut().recomposite();
            self.as_mut().record(label);
        }
        replaced as i32
    }
}
