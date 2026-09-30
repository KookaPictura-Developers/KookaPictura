//! The toning tools' engine, **Dodge**, **Burn**, and **Sponge**
//! (`docs/03-tools/dodge-burn-sponge.md`). In the darkroom, dodging held light
//! back from part of a print so it came out lighter, and burning gave it extra
//! exposure so it came out darker. The Sponge has no darkroom ancestor: it
//! moves colour toward or away from grey.
//!
//! Like the retouch tools they work on what is already there, but one pixel at
//! a time — only the pixel's own tone matters. Three ideas do most of the work:
//!
//! * **Range** — Dodge and Burn act strongest inside one band of the tonal
//!   scale (Shadows, Midtones, Highlights), with overlapping Gaussian falloff
//!   so working across a gradient leaves no seam.
//! * **Protect Tones** — move the pixel's *luminance* and keep its colour,
//!   instead of scaling the channels, which drags a colour toward white or
//!   black by whichever channel saturates first.
//! * **Vibrance** — the Sponge eases off where there is little to do:
//!   saturating what is already vivid, or draining what is nearly grey.
//!
//! A stroke applies its effect once per pixel however many overlapping dabs
//! cover it; working the area again with a new stroke deepens it.
//!
//! Ported from photorust's `core/src/tone.rs`
//! (<https://github.com/perfecto25/photorust>). Behavioural parity only:
//! Adobe's Exposure curve is closed.

use crate::healing::RgbaImage;
use crate::replace::{dab_bounds, grow};
use crate::{tip_coverage, StrokeConfig};
use pictura_core::PsdRect;

/// The band of the tonal scale the tool works hardest in: CS6's Range.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToneRange {
    Shadows,
    #[default]
    Midtones,
    Highlights,
}

impl ToneRange {
    /// 0 Shadows, 1 Midtones, 2 Highlights (the menu order); `None` otherwise.
    pub fn from_i32(v: i32) -> Option<ToneRange> {
        match v {
            0 => Some(ToneRange::Shadows),
            1 => Some(ToneRange::Midtones),
            2 => Some(ToneRange::Highlights),
            _ => None,
        }
    }

    fn centre(self) -> f32 {
        match self {
            ToneRange::Shadows => 0.0,
            ToneRange::Midtones => 0.5,
            ToneRange::Highlights => 1.0,
        }
    }
}

/// Which of the three is stroking.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Lighter.
    #[default]
    Dodge,
    /// Darker.
    Burn,
    /// More or less colourful, by [`SpongeMode`].
    Sponge,
}

/// Which way the Sponge moves colour: its Mode menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpongeMode {
    /// Toward grey.
    #[default]
    Desaturate,
    /// Away from it.
    Saturate,
}

impl SpongeMode {
    /// 0 Desaturate, 1 Saturate; `None` otherwise.
    pub fn from_i32(v: i32) -> Option<SpongeMode> {
        match v {
            0 => Some(SpongeMode::Desaturate),
            1 => Some(SpongeMode::Saturate),
            _ => None,
        }
    }
}

/// The toning options bars. `amount` (0.0–1.0) is Dodge and Burn's Exposure
/// and the Sponge's Flow; `range` and `protect_tones` are Dodge and Burn's,
/// `sponge` and `vibrance` the Sponge's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneOptions {
    pub tone: Tone,
    pub range: ToneRange,
    pub amount: f32,
    pub protect_tones: bool,
    pub sponge: SpongeMode,
    pub vibrance: bool,
}

impl Default for ToneOptions {
    /// Dodge, Midtones, 50 %, Protect Tones on, Desaturate, Vibrance on.
    fn default() -> Self {
        ToneOptions {
            tone: Tone::Dodge,
            range: ToneRange::Midtones,
            amount: 0.5,
            protect_tones: true,
            sponge: SpongeMode::Desaturate,
            vibrance: true,
        }
    }
}

/// How wide a range reaches: wide enough that the three overlap.
const RANGE_SIGMA: f32 = 0.38;

/// What Exposure 100 % is worth in one pass: the fraction of the distance to
/// white a fully covered pixel travels. Taken literally (all the way at 100 %)
/// the tool is unusable; half gives one pass at 50 % about CS6's lift.
const EXPOSURE_SCALE: f32 = 0.5;

/// One toning stroke's state: the coverage already applied at each pixel, so a
/// pixel under four overlapping dabs is toned once, not four times.
pub(crate) struct ToneBrush {
    options: ToneOptions,
    /// Sized to the layer on the first dab.
    applied: Vec<f32>,
}

impl ToneBrush {
    pub(crate) fn new(options: ToneOptions) -> Self {
        ToneBrush {
            options,
            applied: Vec::new(),
        }
    }

    /// Tone one dab centred at layer-local `(cx, cy)`. Returns the layer-local
    /// rectangle changed. Toning changes colour, never coverage.
    pub(crate) fn dab(
        &mut self,
        img: &mut RgbaImage,
        cfg: &StrokeConfig,
        cx: f32,
        cy: f32,
    ) -> Option<PsdRect> {
        let amount = self.options.amount.clamp(0.0, 1.0);
        if amount <= 0.0 {
            return None;
        }
        let region = dab_bounds(img, cfg, cx, cy)?;
        self.applied.resize(img.data.len(), 0.0);
        let mut dirty = None;
        for y in region.top..region.bottom {
            for x in region.left..region.right {
                let cover = tip_coverage(cfg, x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let i = (y * img.width + x) as usize;
                let dst = img.get(x, y);
                // Only the increment over what this stroke already applied.
                if cover <= self.applied[i] || dst[3] == 0 {
                    continue;
                }
                let weight = (cover - self.applied[i]) * amount;
                let rgb = [dst[0], dst[1], dst[2]].map(|v| v as f32 / 255.0);
                let lifted = match self.options.tone {
                    Tone::Dodge | Tone::Burn => tone(rgb, weight, &self.options),
                    Tone::Sponge => sponge(rgb, weight, &self.options),
                };
                let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                let out = [c(lifted[0]), c(lifted[1]), c(lifted[2]), dst[3]];
                // Too small a step to move an 8-bit channel: keep the increment
                // on offer for the next dab rather than rounding it away, or a
                // soft tip's slow falloff leaves a dotted fringe.
                if out == dst {
                    continue;
                }
                self.applied[i] = cover;
                img.set(x, y, out);
                dirty = Some(grow(dirty, x, y));
            }
        }
        dirty
    }
}

/// Dodge and Burn.
fn tone(rgb: [f32; 3], weight: f32, options: &ToneOptions) -> [f32; 3] {
    let l = luminance(rgb);
    let d = l - options.range.centre();
    let k = weight * EXPOSURE_SCALE * (-(d * d) / (2.0 * RANGE_SIGMA * RANGE_SIGMA)).exp();
    if k <= 0.0 {
        return rgb;
    }
    let lighten = options.tone == Tone::Dodge;
    if options.protect_tones {
        // `l + (1 - l) * k` never passes white and `l * (1 - k)` never passes
        // black, so a protected tone cannot clip.
        let target = if lighten {
            l + (1.0 - l) * k
        } else {
            l * (1.0 - k)
        };
        return shift_luminance(rgb, target);
    }
    // Unprotected, the channels are scaled and whichever saturates first pulls
    // the hue with it: the drift Protect Tones exists to prevent.
    rgb.map(|v| {
        if lighten {
            v + (1.0 - v) * k
        } else {
            v * (1.0 - k)
        }
    })
}

/// The Sponge: the distance from grey (the luminance) scaled down or up.
fn sponge(rgb: [f32; 3], weight: f32, options: &ToneOptions) -> [f32; 3] {
    let l = luminance(rgb);
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    let saturation = if max <= 0.0 { 0.0 } else { (max - min) / max };
    let mut k = weight;
    if options.vibrance {
        k *= match options.sponge {
            SpongeMode::Saturate => 1.0 - saturation,
            SpongeMode::Desaturate => saturation,
        };
    }
    if k <= 0.0 {
        return rgb;
    }
    let scale = match options.sponge {
        SpongeMode::Desaturate => 1.0 - k,
        SpongeMode::Saturate => 1.0 + k,
    };
    rgb.map(|v| (l + (v - l) * scale).clamp(0.0, 1.0))
}

/// Rec. 601 luma.
fn luminance(rgb: [f32; 3]) -> f32 {
    0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]
}

/// `rgb` with its luminance moved to `target` by **shifting** all three
/// channels the same distance, which keeps the amount of colour (scaling would
/// magnify it, turning a lightened brown orange). Where a channel would pass
/// white the rest of the way is taken toward the target grey, losing only as
/// much colour as it must.
fn shift_luminance(rgb: [f32; 3], target: f32) -> [f32; 3] {
    let shift = target - luminance(rgb);
    let shifted = rgb.map(|v| v + shift);
    let over = shifted.iter().fold(f32::MIN, |m, v| m.max(*v)) - 1.0;
    let under = -shifted.iter().fold(f32::MAX, |m, v| m.min(*v));
    let excess = over.max(under);
    if excess <= 0.0 {
        return shifted;
    }
    let spread = shifted
        .iter()
        .map(|v| (v - target).abs())
        .fold(0.0f32, f32::max);
    let fade = (excess / spread.max(1e-6)).clamp(0.0, 1.0);
    shifted.map(|v| (v + (target - v) * fade).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fill::tests::{doc, pixel};
    use crate::{Stroke, StrokeSample};
    use pictura_core::Document;

    /// One stroke of a hard 20 px tone through `xs` along row 20 of `d`.
    fn stroke(d: &Document, options: ToneOptions, xs: &[f32]) -> Document {
        let cfg = StrokeConfig {
            diameter: 20,
            ..StrokeConfig::default()
        };
        let mut stroke = Stroke::begin_tone(d, "0", cfg, options).expect("begin");
        for &x in xs {
            stroke.sample(StrokeSample {
                x,
                y: 20.0,
                pressure: 1.0,
            });
        }
        stroke.finish().map_or_else(|| d.clone(), |o| o.document)
    }

    /// `passes` separate one-dab strokes at (20, 20).
    fn passes(d: &Document, options: ToneOptions, passes: usize) -> Document {
        (0..passes).fold(d.clone(), |d, _| stroke(&d, options, &[20.0]))
    }

    fn at(amount: f32) -> ToneOptions {
        ToneOptions {
            amount,
            ..ToneOptions::default()
        }
    }

    fn burn(amount: f32) -> ToneOptions {
        ToneOptions {
            tone: Tone::Burn,
            ..at(amount)
        }
    }

    fn sponge_at(sponge: SpongeMode, vibrance: bool) -> ToneOptions {
        ToneOptions {
            tone: Tone::Sponge,
            sponge,
            vibrance,
            ..at(1.0)
        }
    }

    /// How far `p` is from grey: its red minus its blue.
    fn spread(p: [u8; 4]) -> i32 {
        p[0] as i32 - p[2] as i32
    }

    fn grey(v: u8) -> Document {
        doc(40, 40, [v, v, v, 255])
    }

    #[test]
    fn a_pass_at_the_default_exposure_lifts_gently_and_passes_deepen() {
        let once = pixel(&passes(&grey(90), at(0.5), 1), 20, 20)[0];
        assert!(once > 95 && once < 150, "one pass at 50 % gave {once}");
        let four = pixel(&passes(&grey(100), at(1.0), 4), 20, 20)[0];
        assert!(four > pixel(&passes(&grey(100), at(1.0), 1), 20, 20)[0]);
    }

    #[test]
    fn a_stroke_tones_once_however_many_dabs_cover_a_pixel() {
        let single = pixel(&stroke(&grey(90), at(0.5), &[20.0]), 20, 20)[0] as i32;
        let xs: Vec<f32> = (0..9).map(|i| 16.0 + i as f32).collect();
        let dragged = pixel(&stroke(&grey(90), at(0.5), &xs), 20, 20)[0] as i32;
        assert!(
            (single - dragged).abs() <= 2,
            "{dragged} vs {single}: the dabs compounded"
        );
    }

    #[test]
    fn protect_tones_keeps_the_colour_and_never_clips() {
        let dim = doc(40, 40, [120, 72, 24, 255]);
        let spread = |p: [u8; 4]| p[0] as i32 - p[2] as i32;
        let protected = pixel(&passes(&dim, at(0.6), 1), 20, 20);
        assert!(protected[0] > 120);
        assert!(
            (spread(protected) - 96).abs() <= 4,
            "colour changed: {protected:?}"
        );
        let loose = ToneOptions {
            protect_tones: false,
            ..at(0.6)
        };
        assert!(spread(pixel(&passes(&dim, loose, 1), 20, 20)) < 96);

        let hard = pixel(
            &passes(&doc(40, 40, [180, 90, 60, 255]), at(1.0), 40),
            20,
            20,
        );
        assert_ne!(hard, [255; 4], "a protected dodge clipped to white");
    }

    #[test]
    fn the_range_decides_which_tones_lift() {
        let moved = |range, v: u8| {
            let options = ToneOptions { range, ..at(1.0) };
            pixel(&passes(&grey(v), options, 1), 20, 20)[0] as i32 - v as i32
        };
        assert!(moved(ToneRange::Shadows, 40) > moved(ToneRange::Highlights, 40));
        assert!(moved(ToneRange::Highlights, 215) > moved(ToneRange::Shadows, 215));
        assert_eq!(ToneRange::from_i32(3), None);
    }

    #[test]
    fn toning_never_changes_coverage() {
        let mut d = doc(40, 40, [200, 100, 50, 0]);
        for (i, a) in d.layers[0].channels[3].data.iter_mut().enumerate() {
            if i % 40 < 20 {
                *a = 128;
            }
        }
        let out = passes(&d, at(1.0), 1);
        assert_eq!(pixel(&out, 15, 20)[3], 128);
        assert!(pixel(&out, 15, 20)[0] > 200 || pixel(&out, 15, 20)[1] > 100);
        assert_eq!(pixel(&out, 25, 20), [200, 100, 50, 0]);
    }

    #[test]
    fn burn_darkens_without_clipping_and_highlights_bite_the_highlights() {
        let once = pixel(&passes(&grey(128), burn(1.0), 1), 20, 20)[0];
        assert!(once < 128, "burn did not darken: {once}");
        let hard = pixel(
            &passes(&doc(40, 40, [180, 90, 60, 255]), burn(1.0), 40),
            20,
            20,
        );
        assert_ne!(hard, [0, 0, 0, 255], "a protected burn clipped to black");

        let moved = |range, v: u8| {
            let options = ToneOptions { range, ..burn(1.0) };
            v as i32 - pixel(&passes(&grey(v), options, 1), 20, 20)[0] as i32
        };
        assert!(moved(ToneRange::Highlights, 230) > moved(ToneRange::Highlights, 30) * 3);
    }

    #[test]
    fn the_sponge_drains_and_lifts_colour_but_not_grey() {
        let colour = doc(40, 40, [200, 80, 80, 255]);
        let after = |mode, vibrance| {
            spread(pixel(
                &passes(&colour, sponge_at(mode, vibrance), 3),
                20,
                20,
            ))
        };
        assert!(
            after(SpongeMode::Desaturate, false) < 120,
            "no colour drained"
        );
        assert!(after(SpongeMode::Saturate, false) > 120, "no colour lifted");

        let drained = pixel(
            &passes(&colour, sponge_at(SpongeMode::Desaturate, false), 24),
            20,
            20,
        );
        assert!(
            spread(drained).abs() <= 2,
            "the sponge did not reach grey: {drained:?}"
        );
        let grey = pixel(
            &passes(&grey(128), sponge_at(SpongeMode::Saturate, false), 5),
            20,
            20,
        );
        assert_eq!(grey, [128, 128, 128, 255]);
    }

    #[test]
    fn one_sponge_stroke_at_the_default_flow_is_visible() {
        let options = ToneOptions {
            tone: Tone::Sponge,
            sponge: SpongeMode::Saturate,
            ..ToneOptions::default()
        };
        let px = pixel(
            &passes(&doc(40, 40, [180, 90, 70, 255]), options, 1),
            20,
            20,
        );
        assert!(
            spread(px) - 110 >= 10,
            "the spread only moved to {}",
            spread(px)
        );
    }

    #[test]
    fn vibrance_eases_off_on_colour_already_vivid() {
        let lift = |c: [u8; 4], vibrance| {
            let out = pixel(
                &passes(
                    &doc(40, 40, c),
                    sponge_at(SpongeMode::Saturate, vibrance),
                    1,
                ),
                20,
                20,
            );
            spread(out) - spread(c)
        };
        let (vivid, flat) = ([240, 30, 30, 255], [150, 120, 120, 255]);
        assert!(
            lift(flat, true) > 0,
            "Vibrance refused to lift a flat colour"
        );
        assert!(lift(vivid, true) < lift(vivid, false));
        assert_eq!(SpongeMode::from_i32(2), None);
    }
}
