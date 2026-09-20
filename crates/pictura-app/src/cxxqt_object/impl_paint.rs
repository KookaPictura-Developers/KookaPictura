use super::helpers::*;
use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use pictura_paint::{spacing::SpacingMode, Stroke, StrokeConfig, StrokeSample};

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
            if active_pixel_layer(doc, Some(path)).is_none() {
                return false;
            }
            Stroke::begin_at(doc, path, cfg)
        };
        match begun {
            Ok(stroke) => {
                let mut rust = self.as_mut().rust_mut();
                rust.stroke = Some(stroke);
                rust.stroke_label = if aliased { "Pencil" } else { "Brush" }.to_string();
                true
            }
            Err(_) => false,
        }
    }

    pub fn paint_dab(mut self: Pin<&mut Self>, x: f64, y: f64, pressure: f64) -> bool {
        let changed = {
            let mut rust = self.as_mut().rust_mut();
            match rust.stroke.as_mut() {
                Some(stroke) => stroke.sample(StrokeSample {
                    x: x as f32,
                    y: y as f32,
                    pressure: pressure as f32,
                }),
                None => return false,
            }
        };
        if changed {
            let dirty = self
                .rust()
                .stroke
                .as_ref()
                .and_then(|stroke| stroke.dirty());
            if let Some(rect) = dirty {
                self.as_mut().refresh_region(rect);
            }
        }
        changed
    }

    pub fn end_paint(mut self: Pin<&mut Self>) -> bool {
        let stroke = self.as_mut().rust_mut().stroke.take();
        let label = self.rust().stroke_label.clone();
        match stroke {
            None => false,
            Some(stroke) => match stroke.finish() {
                None => {
                    self.as_mut().recomposite();
                    false
                }
                Some(outcome) => {
                    self.as_mut().rust_mut().doc = Some(outcome.document);
                    self.as_mut().recomposite();
                    self.as_mut().record(&label);
                    true
                }
            },
        }
    }

    pub fn cancel_paint(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().stroke = None;
        self.as_mut().recomposite();
    }

    pub fn is_painting(&self) -> bool {
        self.rust().stroke.is_some()
    }
}
