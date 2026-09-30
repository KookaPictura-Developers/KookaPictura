//! The Eraser (`docs/03-tools/eraser-tools.md`): a Brush stroke that takes
//! pixels away instead of laying colour down.
//!
//! * On an ordinary layer it erases **to transparency** — the stroke's
//!   coverage multiplies the pixel's alpha down, so going over a half-erased
//!   edge deepens it instead of punching through.
//! * On the Background, or a layer whose transparency is locked, there is no
//!   alpha to take, so it paints the **background colour**.
//! * **Erase To History** paints the History Brush's source state back instead.
//!
//! Mode picks the tip: `Brush` is the Brush's soft tip, `Pencil` its hard
//! aliased one, and `Block` a fixed hard square at full strength.
//!
//! The plain Eraser is ported from photorust's erase mode (the Brush painting
//! the background colour; <https://github.com/perfecto25/photorust>), extended
//! with erasing to transparency, Block, and Erase To History from the spec.
//! Behavioural parity only.

use crate::stamp::StampSource;
use crate::stroke::{layer_at, parse_layer_path};
use crate::{PaintError, PaintMode, Stroke, StrokeConfig};
use pictura_core::{layer_transparency_locked, Document, Layer};

/// The Block eraser's side, in document pixels.
// ponytail: CS6's Block is a fixed 16 *screen* pixels; ours ignores the zoom.
pub const BLOCK_SIZE: u32 = 16;

/// The Eraser's Mode menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EraserMode {
    #[default]
    Brush,
    Pencil,
    Block,
}

impl EraserMode {
    /// 0 Brush, 1 Pencil, 2 Block; `None` otherwise.
    pub fn from_i32(v: i32) -> Option<EraserMode> {
        match v {
            0 => Some(EraserMode::Brush),
            1 => Some(EraserMode::Pencil),
            2 => Some(EraserMode::Block),
            _ => None,
        }
    }
}

/// True when erasing `layer` paints the background colour: the Background, a
/// layer with no alpha channel, or a transparency lock.
pub fn erases_to_background(layer: &Layer) -> bool {
    layer.background
        || layer_transparency_locked(layer)
        || !layer.channels.iter().any(|c| c.id == -1)
}

/// Begin an Eraser stroke on the layer at `path`. `cfg` carries the tip,
/// Opacity, Flow, and the background colour (`cfg.background`); its `color`
/// and `mode` are replaced. With `history`, the stroke paints that source back
/// (Erase To History) instead of erasing.
pub fn begin_erase(
    doc: &Document,
    path: &str,
    cfg: StrokeConfig,
    mode: EraserMode,
    history: Option<StampSource>,
) -> Result<Stroke, PaintError> {
    let target = parse_layer_path(path)
        .and_then(|indices| layer_at(doc, &indices))
        .ok_or(PaintError::NoRasterLayer)?;
    let mut cfg = StrokeConfig {
        auto_erase: false,
        ..cfg
    };
    match mode {
        EraserMode::Brush => {}
        EraserMode::Pencil => cfg.aliased = true,
        EraserMode::Block => {
            cfg = StrokeConfig {
                diameter: BLOCK_SIZE,
                square: true,
                opacity: 100,
                flow: 100,
                ..cfg
            }
        }
    }
    if let Some(source) = history {
        cfg.mode = PaintMode::Normal;
        return Stroke::begin_source(doc, path, cfg, source);
    }
    if erases_to_background(target) {
        cfg.mode = PaintMode::Normal;
        cfg.color = cfg.background;
    } else {
        cfg.mode = PaintMode::Clear;
    }
    Stroke::begin_at(doc, path, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamp::layer_surface;
    use crate::stroke::layer_rgba;
    use crate::{Rgba, StrokeSample};
    use pictura_core::{BitDepth, Channel, ColorMode, LockFlags, PsdRect};

    const GREY: [u8; 4] = [90, 90, 90, 255];
    const TEAL: Rgba = Rgba {
        r: 0,
        g: 128,
        b: 128,
        a: 255,
    };

    fn doc(background: bool) -> Document {
        let (w, h) = (40, 20);
        let mut doc = Document::new(w as u32, h as u32, ColorMode::Rgb, BitDepth::Eight);
        let ids: &[(i16, usize)] = if background {
            &[(0, 0), (1, 1), (2, 2)]
        } else {
            &[(0, 0), (1, 1), (2, 2), (-1, 3)]
        };
        doc.layers.push(Layer {
            rect: PsdRect {
                top: 0,
                left: 0,
                bottom: h,
                right: w,
            },
            channels: ids
                .iter()
                .map(|&(id, c)| Channel {
                    id,
                    data: vec![GREY[c]; (w * h) as usize],
                })
                .collect(),
            background,
            ..Default::default()
        });
        doc
    }

    fn erase(doc: &Document, mode: EraserMode, history: Option<StampSource>) -> Document {
        let cfg = StrokeConfig {
            diameter: 8,
            background: TEAL,
            ..StrokeConfig::default()
        };
        let mut stroke = begin_erase(doc, "0", cfg, mode, history).expect("begin");
        for x in [8.0, 16.0, 24.0] {
            stroke.sample(StrokeSample {
                x,
                y: 10.0,
                pressure: 1.0,
            });
        }
        stroke.finish().expect("erased").document
    }

    fn pixel(doc: &Document, x: i32, y: i32) -> [u8; 4] {
        layer_rgba(&doc.layers[0]).get(x, y)
    }

    #[test]
    fn a_layer_erases_to_transparency_and_the_background_to_the_background_colour() {
        let layer = erase(&doc(false), EraserMode::Brush, None);
        assert_eq!(pixel(&layer, 16, 10)[3], 0, "the stroke left alpha behind");
        assert_eq!(pixel(&layer, 16, 18), GREY, "erased outside the tip");

        let background = erase(&doc(true), EraserMode::Brush, None);
        assert_eq!(pixel(&background, 16, 10), [0, 128, 128, 255]);
    }

    #[test]
    fn a_transparency_lock_paints_the_background_colour() {
        let mut locked = doc(false);
        locked.layers[0].lock = LockFlags::default().with(LockFlags::TRANSPARENCY, true);
        assert_eq!(
            pixel(&erase(&locked, EraserMode::Brush, None), 16, 10),
            [0, 128, 128, 255]
        );
    }

    #[test]
    fn half_opacity_leaves_half_the_alpha() {
        let cfg = StrokeConfig {
            diameter: 8,
            opacity: 50,
            ..StrokeConfig::default()
        };
        let mut stroke = begin_erase(&doc(false), "0", cfg, EraserMode::Brush, None).unwrap();
        stroke.sample(StrokeSample {
            x: 16.0,
            y: 10.0,
            pressure: 1.0,
        });
        let a = pixel(&stroke.finish().unwrap().document, 16, 10)[3];
        assert!((a as i32 - 128).abs() <= 1, "alpha {a}");
    }

    #[test]
    fn block_erases_a_hard_square_at_full_strength() {
        let cfg = StrokeConfig {
            opacity: 10,
            ..StrokeConfig::default()
        };
        let mut stroke = begin_erase(&doc(false), "0", cfg, EraserMode::Block, None).unwrap();
        stroke.sample(StrokeSample {
            x: 20.0,
            y: 10.0,
            pressure: 1.0,
        });
        let out = stroke.finish().unwrap().document;
        // The square's corner is erased; a round 16 px tip would miss it.
        assert_eq!(pixel(&out, 13, 3)[3], 0);
        assert_eq!(pixel(&out, 11, 10)[3], 255);
    }

    #[test]
    fn pencil_erases_without_a_soft_edge() {
        let out = erase(&doc(false), EraserMode::Pencil, None);
        let layer = layer_rgba(&out.layers[0]);
        assert!(layer.data.iter().all(|p| p[3] == 0 || p[3] == 255));
    }

    #[test]
    fn erase_to_history_paints_the_source_back() {
        let mut past = doc(false);
        for c in &mut past.layers[0].channels {
            if c.id == 0 {
                c.data.fill(250);
            }
        }
        let source = StampSource::new(layer_surface(&past, "0").unwrap(), (0, 0));
        let out = erase(&doc(false), EraserMode::Brush, Some(source));
        assert_eq!(pixel(&out, 16, 10), [250, 90, 90, 255]);
    }
}
