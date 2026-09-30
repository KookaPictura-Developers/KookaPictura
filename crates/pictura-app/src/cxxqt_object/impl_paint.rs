use super::helpers::*;
use super::qobject;
use super::state::PictureViewRust;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_core::PsdRect;
use pictura_paint::{spacing::SpacingMode, Stroke, StrokeConfig, StrokeSample};

impl PictureViewRust {
    /// Fold a dab's `rect` into the frame-bounded present. `Some(rect)` is the
    /// region to present now — the dab that opens a frame — and `None` means it
    /// accumulates until [`Self::take_pending_present`].
    pub(super) fn queue_present(&mut self, rect: PsdRect) -> Option<PsdRect> {
        if self.present_flush_due {
            self.pending_present = Some(match self.pending_present.take() {
                Some(pending) => union_rect(pending, rect),
                None => rect,
            });
            None
        } else {
            self.present_flush_due = true;
            Some(rect)
        }
    }

    /// Clear the frame and hand back the region accumulated since the last
    /// present, if any.
    pub(super) fn take_pending_present(&mut self) -> Option<PsdRect> {
        self.present_flush_due = false;
        self.pending_present.take()
    }

    /// Drop a pending present: the stroke start, the commit refresh and the
    /// cancel restore all supersede whatever it would have shown.
    pub(super) fn clear_pending_present(&mut self) {
        self.pending_present = None;
        self.present_flush_due = false;
    }
}

impl qobject::PictureView {
    pub fn begin_paint(
        mut self: Pin<&mut Self>,
        foreground: u32,
        background: u32,
        diameter: i32,
        hardness: i32,
        roundness: i32,
        angle: i32,
        opacity: i32,
        flow: i32,
        spacing: i32,
        mode: &QString,
        aliased: bool,
        auto_erase: bool,
    ) -> bool {
        {
            let rust = self.rust();
            if rust.doc.is_none() || rust.stroke.is_some() {
                return false;
            }
        }
        let cfg = StrokeConfig {
            color: rgba_from_argb(foreground),
            background: rgba_from_argb(background),
            diameter: diameter.max(0) as u32,
            hardness: hardness.clamp(0, 100) as u8,
            roundness: roundness.clamp(0, 100) as u8,
            angle_deg: angle,
            spacing: SpacingMode::Fixed(spacing.clamp(0, 1000) as u16),
            opacity: opacity.clamp(0, 100) as u8,
            flow: flow.clamp(0, 100) as u8,
            mode: paint_mode_from(&mode.to_string()),
            aliased,
            auto_erase,
            ..StrokeConfig::default()
        };
        let begun = {
            let rust = self.rust();
            let Some(doc) = rust.doc.as_ref() else {
                return false;
            };
            let Some(path) = rust.active_layer.as_deref() else {
                return false;
            };
            if active_pixel_layer(doc, Some(path)).is_none()
                || !active_layer_visible(doc, Some(path))
            {
                return false;
            }
            Stroke::begin_at(doc, path, cfg)
        };
        match begun {
            Ok(stroke) => {
                let mut rust = self.as_mut().rust_mut();
                rust.stroke = Some(stroke);
                rust.stroke_label = if aliased { "Pencil" } else { "Brush" }.to_string();
                // A new stroke opens its own present frame.
                rust.clear_pending_present();
                true
            }
            Err(_) => false,
        }
    }

    pub fn paint_dab(mut self: Pin<&mut Self>, x: f64, y: f64, pressure: f64) -> bool {
        let dirty = {
            let mut rust = self.as_mut().rust_mut();
            let Some(stroke) = rust.stroke.as_mut() else {
                return false;
            };
            if !stroke.sample(StrokeSample {
                x: x as f32,
                y: y as f32,
                pressure: pressure as f32,
            }) {
                return false;
            }
            stroke.take_dirty()
        };
        let Some(rect) = dirty else {
            return false;
        };
        // Frame-bounded present: the dab that opens a frame presents itself and
        // arms the flush; every later dab of that frame accumulates into one
        // pending region that `flush_present` shows on the next event-loop turn.
        let opens_frame = {
            let mut rust = self.as_mut().rust_mut();
            rust.queue_present(rect)
        };
        if let Some(present) = opens_frame {
            self.as_mut().refresh_region(present);
        }
        true
    }

    /// Present the region accumulated since the last in-stroke present, or do
    /// nothing when none is pending. The shell calls it on the next event-loop
    /// turn after a present; a consumer that needs a mid-stroke present sooner
    /// calls it directly.
    pub fn flush_present(mut self: Pin<&mut Self>) -> bool {
        let pending = {
            let mut rust = self.as_mut().rust_mut();
            rust.take_pending_present()
        };
        match pending {
            Some(rect) => {
                self.as_mut().refresh_region(rect);
                true
            }
            None => false,
        }
    }

    pub fn end_paint(mut self: Pin<&mut Self>) -> bool {
        let (stroke, label) = {
            let mut rust = self.as_mut().rust_mut();
            // The commit refresh covers the stroke's whole extent, so it
            // supersedes any region still pending; it runs before `record`.
            rust.clear_pending_present();
            (rust.stroke.take(), rust.stroke_label.clone())
        };
        match stroke {
            None => false,
            Some(stroke) => match stroke.finish() {
                None => {
                    self.as_mut().recomposite();
                    false
                }
                Some(outcome) => {
                    let rect = outcome.dirty;
                    self.as_mut().rust_mut().doc = Some(outcome.document);
                    // The commit is a described change: refresh only the stroke's
                    // extent, which patches the composite, the level-0 frame, the
                    // pyramid, and the canvas region in one pass.
                    self.as_mut().refresh_region(rect);
                    self.as_mut().record(&label);
                    true
                }
            },
        }
    }

    pub fn cancel_paint(mut self: Pin<&mut Self>) {
        // The document composite was never patched mid-stroke, so restoring the
        // stroke's extent from it is enough; only the level-0/pyramid and the
        // displayed canvas carry the in-progress paint. A region still pending
        // is dropped: the restore below repaints the whole extent anyway.
        let dirty = {
            let mut rust = self.as_mut().rust_mut();
            rust.clear_pending_present();
            rust.stroke.take()
        }
        .and_then(|stroke| stroke.finish())
        .map(|outcome| outcome.dirty);
        match dirty {
            Some(rect) => self.as_mut().refresh_region(rect),
            None => self.as_mut().recomposite(),
        }
    }

    pub fn is_painting(&self) -> bool {
        self.rust().stroke.is_some()
    }
}
