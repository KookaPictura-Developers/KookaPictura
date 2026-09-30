//! The Blur tool's engine (`docs/03-tools/smudge-blur-sharpen.md`).
//!
//! Not the Blur *filter*: that is one pass over a whole layer at a chosen
//! radius. This is a brush that softens only where the tip passes, and **the
//! more it passes the softer it gets** — each dab works on what the last one
//! left, so like the Mixer Brush it applies per dab straight into the layer.
//!
//! The kernel is a fixed 3×3 Gaussian: a bigger tip works a wider area by the
//! same amount per dab, and depth comes from working the same spot. A radius
//! that grew with the tip would turn one click of a large brush into a smear.
//!
//! Ported from photorust's `core/src/focus.rs`
//! (<https://github.com/perfecto25/photorust>). Behavioural parity only:
//! Adobe's kernel is closed.

use crate::healing::RgbaImage;
use crate::replace::{dab_bounds, grow};
use crate::{tip_coverage, StrokeConfig};
use pictura_core::nonseparable::{lum, sat, set_lum, set_sat};
use pictura_core::PsdRect;

/// Which part of the pixel the blur may change: CS6's cut-down Mode list for
/// a tool whose source is the destination, worked on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlurMode {
    #[default]
    Normal,
    Darken,
    Lighten,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl BlurMode {
    /// 0 Normal … 6 Luminosity (the menu order); `None` otherwise.
    pub fn from_i32(v: i32) -> Option<BlurMode> {
        [
            BlurMode::Normal,
            BlurMode::Darken,
            BlurMode::Lighten,
            BlurMode::Hue,
            BlurMode::Saturation,
            BlurMode::Color,
            BlurMode::Luminosity,
        ]
        .get(usize::try_from(v).ok()?)
        .copied()
    }
}

/// The Blur options bar: Strength (0.0–1.0, how much one dab applies) and Mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlurOptions {
    pub strength: f32,
    pub mode: BlurMode,
}

impl Default for BlurOptions {
    /// Strength 50 %, Normal.
    fn default() -> Self {
        BlurOptions {
            strength: 0.5,
            mode: BlurMode::Normal,
        }
    }
}

/// The 3×3 Gaussian, in sixteenths.
const KERNEL: [f32; 9] = [1.0, 2.0, 1.0, 2.0, 4.0, 2.0, 1.0, 2.0, 1.0];

/// One Blur stroke's state.
pub(crate) struct BlurBrush {
    options: BlurOptions,
    /// Sample All Layers: the composite, layer-local, the neighbourhood is
    /// read from. `None` reads the layer itself.
    sampled: Option<RgbaImage>,
    /// A transparency lock: colour may soften, coverage may not.
    preserve_alpha: bool,
}

impl BlurBrush {
    pub(crate) fn new(
        options: BlurOptions,
        sampled: Option<RgbaImage>,
        preserve_alpha: bool,
    ) -> Self {
        BlurBrush {
            options,
            sampled,
            preserve_alpha,
        }
    }

    /// Soften one dab centred at layer-local `(cx, cy)`. Returns the
    /// layer-local rectangle changed.
    pub(crate) fn dab(
        &mut self,
        img: &mut RgbaImage,
        cfg: &StrokeConfig,
        cx: f32,
        cy: f32,
    ) -> Option<PsdRect> {
        let strength = self.options.strength.clamp(0.0, 1.0);
        if strength <= 0.0 {
            return None;
        }
        let region = dab_bounds(img, cfg, cx, cy)?;
        // The neighbourhood is read from a copy: blurring in place would feed
        // each pixel its already-softened neighbour and smear the dab in
        // whichever direction the loop runs.
        let snapshot = Snapshot::of(self.sampled.as_ref().unwrap_or(img), region);
        let mut dirty = None;
        for y in region.top..region.bottom {
            for x in region.left..region.right {
                let cover = tip_coverage(cfg, x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let dst = img.get(x, y);
                if cover <= 0.0 || (self.preserve_alpha && dst[3] == 0) {
                    continue;
                }
                let target = restrict(dst, snapshot.average(x, y), self.options.mode);
                let mut out = lerp(dst, target, cover * strength);
                if self.preserve_alpha {
                    out[3] = dst[3];
                }
                if out != dst {
                    img.set(x, y, out);
                    dirty = Some(grow(dirty, x, y));
                }
            }
        }
        dirty
    }
}

/// A copy of `region` grown by one pixel, clipped to the image.
struct Snapshot {
    rect: PsdRect,
    pixels: RgbaImage,
}

impl Snapshot {
    fn of(img: &RgbaImage, region: PsdRect) -> Snapshot {
        let rect = PsdRect {
            top: (region.top - 1).max(0),
            left: (region.left - 1).max(0),
            bottom: (region.bottom + 1).min(img.height),
            right: (region.right + 1).min(img.width),
        };
        let (w, h) = (rect.width().max(0), rect.height().max(0));
        let data = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| {
                let (sx, sy) = (x + rect.left, y + rect.top);
                if sx < img.width && sy < img.height {
                    img.get(sx, sy)
                } else {
                    [0; 4]
                }
            })
            .collect();
        Snapshot {
            rect,
            pixels: RgbaImage {
                width: w,
                height: h,
                data,
            },
        }
    }

    /// The 3×3 Gaussian at image `(x, y)`, clamped at the image edge so a blur
    /// there does not fade into nothing. Weighted in premultiplied colour: a
    /// transparent neighbour adds no colour, so softening a layer's edge does
    /// not draw a dark rim inward.
    fn average(&self, x: i32, y: i32) -> [u8; 4] {
        let (mut r, mut g, mut b, mut a) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for ky in -1..=1 {
            for kx in -1..=1 {
                let w = KERNEL[((ky + 1) * 3 + kx + 1) as usize] / 16.0;
                let sx = (x + kx - self.rect.left).clamp(0, self.pixels.width - 1);
                let sy = (y + ky - self.rect.top).clamp(0, self.pixels.height - 1);
                let p = self.pixels.get(sx, sy);
                let alpha = p[3] as f32 / 255.0;
                r += p[0] as f32 * alpha * w;
                g += p[1] as f32 * alpha * w;
                b += p[2] as f32 * alpha * w;
                a += alpha * w;
            }
        }
        if a <= 1e-6 {
            return [0; 4];
        }
        let c = |v: f32| (v / a).round().clamp(0.0, 255.0) as u8;
        [
            c(r),
            c(g),
            c(b),
            (a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]
    }
}

/// The part of the blurred pixel `mode` lets through. Normal takes it whole,
/// alpha included (which softens a layer's edge); every other mode keeps the
/// pixel's coverage.
fn restrict(dst: [u8; 4], blurred: [u8; 4], mode: BlurMode) -> [u8; 4] {
    if mode == BlurMode::Normal {
        return blurred;
    }
    let unit = |p: [u8; 4]| {
        [
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ]
    };
    let (base, worked) = (unit(dst), unit(blurred));
    let out = match mode {
        BlurMode::Darken => std::array::from_fn(|i| base[i].min(worked[i])),
        BlurMode::Lighten => std::array::from_fn(|i| base[i].max(worked[i])),
        BlurMode::Hue => set_lum(set_sat(worked, sat(base)), lum(base)),
        BlurMode::Saturation => set_lum(set_sat(base, sat(worked)), lum(base)),
        BlurMode::Color => set_lum(worked, lum(base)),
        _ => set_lum(base, lum(worked)),
    };
    let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [c(out[0]), c(out[1]), c(out[2]), dst[3]]
}

fn lerp(from: [u8; 4], to: [u8; 4], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    std::array::from_fn(|i| {
        (from[i] as f32 + (to[i] as f32 - from[i] as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fill::tests::{doc, lock, pixel};
    use crate::stroke::layer_rgba;
    use crate::{PaintError, Stroke, StrokeSample};
    use pictura_core::{BitDepth, Document, LockFlags};

    const WHITE: [u8; 4] = [255; 4];
    const BLACK: [u8; 4] = [0, 0, 0, 255];

    /// A hard black/white edge down the middle of a 40 px square.
    fn edge() -> Document {
        let mut d = doc(40, 40, WHITE);
        for c in &mut d.layers[0].channels[..3] {
            for (i, v) in c.data.iter_mut().enumerate() {
                if i % 40 >= 20 {
                    *v = 0;
                }
            }
        }
        d
    }

    /// `passes` back-and-forth passes of a hard 20 px Blur over (20, 20); the
    /// document unchanged when nothing changed.
    fn blur(
        d: &Document,
        options: BlurOptions,
        sampled: Option<RgbaImage>,
        passes: usize,
    ) -> Document {
        let cfg = StrokeConfig {
            diameter: 20,
            ..StrokeConfig::default()
        };
        let mut stroke = Stroke::begin_blur(d, "0", cfg, options, sampled).expect("begin");
        for _ in 0..passes {
            for x in [20.0, 25.0, 20.0] {
                stroke.sample(StrokeSample {
                    x,
                    y: 20.0,
                    pressure: 1.0,
                });
            }
        }
        stroke.finish().map_or_else(|| d.clone(), |o| o.document)
    }

    fn full() -> BlurOptions {
        BlurOptions {
            strength: 1.0,
            ..BlurOptions::default()
        }
    }

    /// Pixels of row 20 between x = 10 and 30 that are neither black nor white.
    fn spread(d: &Document) -> usize {
        (10..30)
            .filter(|&x| (9..247).contains(&pixel(d, x, 20)[0]))
            .count()
    }

    #[test]
    fn a_dab_softens_the_edge_and_nothing_outside_the_tip() {
        let d = blur(&edge(), full(), None, 1);
        assert!(pixel(&d, 19, 20)[0] < 255, "the light side was untouched");
        assert!(pixel(&d, 20, 20)[0] > 0, "the dark side was untouched");
        assert_eq!(pixel(&d, 2, 2), WHITE);
        assert_eq!(pixel(&d, 38, 38), BLACK);
    }

    #[test]
    fn dwelling_deepens_the_blur() {
        let once = blur(&edge(), full(), None, 1);
        let many = blur(&edge(), full(), None, 8);
        assert!(
            spread(&many) > spread(&once),
            "{} vs {}",
            spread(&many),
            spread(&once)
        );
    }

    #[test]
    fn strength_scales_one_dab() {
        let soft = BlurOptions {
            strength: 0.1,
            ..BlurOptions::default()
        };
        let moved = |d: &Document| 255 - pixel(d, 19, 20)[0] as i32;
        let (weak, strong) = (blur(&edge(), soft, None, 1), blur(&edge(), full(), None, 1));
        assert!(moved(&strong) > moved(&weak) * 2);
        let none = BlurOptions {
            strength: 0.0,
            ..BlurOptions::default()
        };
        assert_eq!(
            layer_rgba(&blur(&edge(), none, None, 1).layers[0]),
            layer_rgba(&edge().layers[0])
        );
    }

    #[test]
    fn a_flat_area_is_left_alone() {
        let flat = doc(40, 40, [123, 45, 67, 255]);
        let cfg = StrokeConfig::default();
        let mut stroke = Stroke::begin_blur(&flat, "0", cfg, full(), None).unwrap();
        stroke.sample(StrokeSample {
            x: 20.0,
            y: 20.0,
            pressure: 1.0,
        });
        assert!(stroke.finish().is_none(), "rounding drifted a flat colour");
    }

    #[test]
    fn a_layer_edge_softens_outward_without_darkening() {
        let mut d = doc(40, 40, [240, 200, 40, 0]);
        for (i, a) in d.layers[0].channels[3].data.iter_mut().enumerate() {
            if i % 40 < 20 {
                *a = 255;
            }
        }
        let out = blur(&d, full(), None, 1);
        let inside = pixel(&out, 18, 20);
        assert!(
            inside[0] > 200 && inside[1] > 160,
            "the edge darkened: {inside:?}"
        );
        assert!(
            pixel(&out, 20, 20)[3] > 0,
            "the edge did not soften outward"
        );

        lock(&mut d, LockFlags::TRANSPARENCY);
        let locked = blur(&d, full(), None, 1);
        assert_eq!(
            pixel(&locked, 20, 20)[3],
            0,
            "the blur spread past the lock"
        );
        assert_eq!(pixel(&locked, 10, 20)[3], 255);
    }

    #[test]
    fn luminosity_blurs_the_shading_and_darken_only_darkens() {
        // Two colours of about equal luminosity: nothing to blur in Luminosity.
        let mut d = doc(40, 40, [200, 100, 100, 255]);
        for (c, v) in [(0, 100u8), (1, 160)] {
            for (i, p) in d.layers[0].channels[c].data.iter_mut().enumerate() {
                if i % 40 >= 20 {
                    *p = v;
                }
            }
        }
        let luminosity = BlurOptions {
            mode: BlurMode::Luminosity,
            ..full()
        };
        let out = blur(&d, luminosity, None, 1);
        let moved = (pixel(&out, 19, 20)[0] as i32 - 200).abs();
        assert!(moved <= 12, "Luminosity blurred the colour: moved {moved}");

        let darken = BlurOptions {
            mode: BlurMode::Darken,
            ..full()
        };
        let out = blur(&edge(), darken, None, 1);
        assert!(pixel(&out, 19, 20)[0] < 255);
        assert_eq!(pixel(&out, 20, 20), BLACK, "Darken lightened the dark side");
    }

    #[test]
    fn sample_all_layers_reads_the_composite_it_is_given() {
        let composite = crate::stamp::layer_surface(&edge(), "0").unwrap();
        let out = blur(&doc(40, 40, WHITE), full(), Some(composite), 1);
        assert!(
            pixel(&out, 21, 20)[0] < 200,
            "the composite's edge was not sampled"
        );
    }

    #[test]
    fn a_deep_document_is_refused_and_modes_round_trip() {
        let mut deep = edge();
        deep.depth = BitDepth::Sixteen;
        let begun = Stroke::begin_blur(&deep, "0", StrokeConfig::default(), full(), None);
        assert!(matches!(begun, Err(PaintError::UnsupportedDepth)));
        assert_eq!(BlurMode::from_i32(6), Some(BlurMode::Luminosity));
        assert_eq!(BlurMode::from_i32(7), None);
    }
}
