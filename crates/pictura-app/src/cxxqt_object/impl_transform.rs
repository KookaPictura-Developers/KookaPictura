use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};

impl qobject::PictureView {
    pub fn translate_layer(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer(doc, dx, dy)
        };
        if moved {
            self.as_mut().recomposite();
            self.as_mut().record("Move Layer");
        }
        moved
    }

    pub fn move_preview(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        if dx == 0 && dy == 0 {
            return false;
        }
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer(doc, dx, dy)
        };
        if moved {
            let gpu_compute = self.rust().gpu_compute;
            let image = self
                .rust()
                .doc
                .as_ref()
                .map(|doc| document_to_image(doc, gpu_compute));
            if let Some(image) = image {
                let mut rust = self.as_mut().rust_mut();
                rust.image = image;
                rust.display_dirty = false;
            }
            self.changed();
        }
        moved
    }

    /// Cache the base composite (topmost raster layer hidden), the layer image,
    /// its document-space origin, and opacity. One region composite at drag
    /// start, derived from the authoritative `doc.composite`.
    pub fn begin_move_preview(mut self: Pin<&mut Self>) -> bool {
        let mut guard = self.as_mut().rust_mut();
        let rust = &mut *guard;
        let gpu_compute = rust.gpu_compute;
        let Some(doc) = rust.doc.as_mut() else {
            return false;
        };
        let Some(index) = topmost_pixel_layer_index(doc) else {
            return false;
        };
        let Some(layer_image) = layer_image(&doc.layers[index]) else {
            return false;
        };
        let rect = doc.layers[index].rect;
        let (x, y, opacity) = (rect.left, rect.top, doc.layers[index].opacity as i32);
        // Build the base from the authoritative planar composite, never a
        // possibly-stale cached image: clone it and overwrite the moved layer's
        // rectangle with the region composited with the layer hidden.
        let composite_ok = doc.composite.width == doc.width
            && doc.composite.height == doc.height
            && !doc.composite.data.is_empty();
        let base = match move_preview_region(rect, doc.width, doc.height, composite_ok) {
            Some((x0, y0, ..)) => {
                doc.layers[index].visible = false;
                let (region, _backend) =
                    pictura_render::composite_region_active(doc, rect, gpu_compute);
                doc.layers[index].visible = true;
                let mut base_buffer = doc.composite.clone();
                patch_buffer_region(&mut base_buffer, &region, x0, y0);
                buffer_to_image(&base_buffer)
            }
            None => {
                // ponytail: full-composite fallback for a missing/mismatched
                // composite; the region path covers the common case.
                doc.layers[index].visible = false;
                let base = document_to_image(doc, gpu_compute);
                doc.layers[index].visible = true;
                base
            }
        };
        rust.move_base = Some(base);
        rust.move_layer = Some(layer_image);
        rust.move_x = x;
        rust.move_y = y;
        rust.move_opacity = opacity;
        true
    }

    pub fn move_preview_base(&self) -> QImage {
        self.rust().move_base.clone().unwrap_or_default()
    }

    pub fn move_preview_layer(&self) -> QImage {
        self.rust().move_layer.clone().unwrap_or_default()
    }

    pub fn move_preview_x(&self) -> i32 {
        self.rust().move_x
    }

    pub fn move_preview_y(&self) -> i32 {
        self.rust().move_y
    }

    pub fn move_preview_opacity(&self) -> i32 {
        self.rust().move_opacity
    }

    pub fn end_move_preview(mut self: Pin<&mut Self>) {
        self.as_mut().clear_move_cache();
    }

    pub fn commit_move(mut self: Pin<&mut Self>, dx: i32, dy: i32) -> bool {
        if dx == 0 && dy == 0 {
            return false;
        }
        let before = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            match topmost_pixel_layer_rect(doc) {
                Some(rect) => rect,
                None => return false,
            }
        };
        let moved = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::translate_layer_rect(doc, dx, dy)
        };
        if !moved {
            return false;
        }
        let dirty = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            union_rect(before, topmost_pixel_layer_rect(doc).unwrap_or(before))
        };
        self.as_mut().refresh_region(dirty);
        self.as_mut().record("Move Layer");
        self.as_mut().clear_move_cache();
        true
    }

    pub fn commit_move_legacy(mut self: Pin<&mut Self>) -> bool {
        if self.rust().doc.is_none() {
            return false;
        }
        self.as_mut().recomposite();
        self.as_mut().record("Move Layer");
        true
    }

    fn clear_move_cache(mut self: Pin<&mut Self>) {
        let mut rust = self.as_mut().rust_mut();
        rust.move_base = None;
        rust.move_layer = None;
        rust.move_x = 0;
        rust.move_y = 0;
        rust.move_opacity = 0;
    }

    pub fn resize_image(mut self: Pin<&mut Self>, kind: &QString, width: i32, height: i32) -> bool {
        let Some(resample) = parse_resample(&kind.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_document(doc, width as u32, height as u32, resample).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Image Size");
        }
        resized
    }

    pub fn resize_canvas(
        mut self: Pin<&mut Self>,
        anchor: &QString,
        width: i32,
        height: i32,
    ) -> bool {
        let Some(anchor) = parse_anchor(&anchor.to_string()) else {
            return false;
        };
        if width < 1 || height < 1 {
            return false;
        }
        let resized = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::resize_canvas_document(doc, width as u32, height as u32, anchor).is_ok()
        };
        if resized {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Canvas Size");
        }
        resized
    }

    pub fn rotate_doc(mut self: Pin<&mut Self>, quarter_turns: i32) -> bool {
        if !(1..=3).contains(&quarter_turns) {
            return false;
        }
        let rotated = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::rotate_document(doc, quarter_turns as u8).is_ok()
        };
        if rotated {
            self.as_mut().rust_mut().selection = None;
            self.as_mut().recomposite();
            self.as_mut().record("Rotate");
        }
        rotated
    }

    pub fn flip_doc(mut self: Pin<&mut Self>, horizontal: bool) -> bool {
        {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            pictura_render::flip_document(doc, horizontal);
            rust.selection = None;
        }
        self.as_mut().recomposite();
        self.as_mut().record("Flip");
        true
    }
}
