use super::helpers::*;
use super::helpers_composite::*;
use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_select::{CombineMode, Selection};

impl qobject::PictureView {
    pub fn select_all(mut self: Pin<&mut Self>) {
        let dims = self.rust().doc.as_ref().map(|d| (d.width, d.height));
        if let Some((w, h)) = dims {
            self.as_mut().rust_mut().selection = Some(Selection::all(w, h));
            self.as_mut().record("Select All");
            self.changed();
        }
    }

    pub fn deselect(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().selection = None;
        self.as_mut().record("Deselect");
        self.changed();
    }

    pub fn magic_wand(mut self: Pin<&mut Self>, x: i32, y: i32, tolerance: i32) -> bool {
        let picked = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if x < 0 || y < 0 {
                None
            } else {
                let tolerance = tolerance.clamp(0, 255) as u8;
                pictura_select::magic_wand(
                    &current_buffer(doc, rust.gpu_compute),
                    x as u32,
                    y as u32,
                    tolerance,
                    true,
                )
                .ok()
            }
        };
        match picked {
            Some(selection) => {
                self.as_mut().rust_mut().selection = Some(selection);
                self.as_mut().record("Magic Wand");
                self.changed();
                true
            }
            None => false,
        }
    }

    pub fn has_selection(&self) -> bool {
        self.rust().selection.is_some()
    }

    pub fn selection_count(&self) -> i32 {
        self.rust()
            .selection
            .as_ref()
            .map_or(0, |s| s.data.iter().filter(|&&v| v > 0).count() as i32)
    }

    /// Merge a document-sized `shape` into the current selection with `mode`,
    /// capturing history and emitting [`changed`]. Returns false without a doc.
    fn apply_selection(mut self: Pin<&mut Self>, shape: Selection, mode: CombineMode) -> bool {
        {
            let mut rust = self.as_mut().rust_mut();
            if rust.doc.is_none() {
                return false;
            }
            let mut base = rust
                .selection
                .take()
                .unwrap_or_else(|| Selection::none(shape.width, shape.height));
            base.combine_with(&shape, mode);
            rust.selection = Some(base);
        }
        self.as_mut().recomposite();
        self.as_mut().record("Selection");
        true
    }

    pub fn select_rect(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            Selection::rect(doc.width, doc.height, x, y, w, h)
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn select_ellipse(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            Selection::ellipse(doc.width, doc.height, x, y, w, h)
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn begin_lasso(mut self: Pin<&mut Self>, mode: &QString) -> bool {
        if self.rust().doc.is_none() {
            return false;
        }
        let mut rust = self.as_mut().rust_mut();
        rust.pending_lasso_mode = mode.to_string();
        rust.pending_lasso.clear();
        true
    }

    pub fn lasso_add_point(mut self: Pin<&mut Self>, x: i32, y: i32) {
        self.as_mut().rust_mut().pending_lasso.push((x, y));
    }

    pub fn end_lasso(mut self: Pin<&mut Self>) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if rust.pending_lasso.len() < 3 {
                return false;
            }
            Selection::polygon(doc.width, doc.height, &rust.pending_lasso)
        };
        let mode = combine_mode_from(&self.rust().pending_lasso_mode);
        if !self.as_mut().apply_selection(shape, mode) {
            return false;
        }
        let mut rust = self.as_mut().rust_mut();
        rust.pending_lasso.clear();
        rust.pending_lasso_mode.clear();
        true
    }

    pub fn quick_select(
        mut self: Pin<&mut Self>,
        x: i32,
        y: i32,
        tolerance: i32,
        mode: &QString,
    ) -> bool {
        let shape = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            if x < 0 || y < 0 || x as u32 >= doc.width || y as u32 >= doc.height {
                return false;
            }
            let tolerance = tolerance.clamp(0, 255) as u8;
            match pictura_select::magic_wand(
                &current_buffer(doc, rust.gpu_compute),
                x as u32,
                y as u32,
                tolerance,
                true,
            ) {
                Ok(selection) => selection,
                Err(_) => return false,
            }
        };
        let mode = combine_mode_from(&mode.to_string());
        self.as_mut().apply_selection(shape, mode)
    }

    pub fn crop(mut self: Pin<&mut Self>, x: i32, y: i32, w: i32, h: i32) -> bool {
        if w < 1 || h < 1 {
            return false;
        }
        let cropped = {
            let mut rust = self.as_mut().rust_mut();
            let Some(doc) = rust.doc.as_mut() else {
                return false;
            };
            if !pictura_render::crop_document(doc, x, y, w as u32, h as u32) {
                return false;
            }
            // Canvas dimensions changed, so the old selection no longer maps.
            rust.selection = None;
            true
        };
        if cropped {
            self.as_mut().recomposite();
            self.as_mut().record("Crop");
        }
        cropped
    }

    pub fn selection_bounds(&self) -> QString {
        let rust = self.rust();
        let Some(sel) = rust.selection.as_ref() else {
            return QString::default();
        };
        let mut bounds: Option<(u32, u32, u32, u32)> = None;
        for (i, &v) in sel.data.iter().enumerate() {
            if v == 0 {
                continue;
            }
            let x = i as u32 % sel.width;
            let y = i as u32 / sel.width;
            bounds = Some(match bounds {
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                None => (x, y, x, y),
            });
        }
        let Some((x0, y0, x1, y1)) = bounds else {
            return QString::default();
        };
        QString::from(format!("{x0} {y0} {} {}", x1 - x0 + 1, y1 - y0 + 1))
    }
}
