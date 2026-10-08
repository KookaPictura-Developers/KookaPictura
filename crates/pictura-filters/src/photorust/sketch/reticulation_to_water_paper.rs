//! Reticulation, Stamp, Torn Edges, Water Paper (split from `sketch` for file size).

#[allow(unused_imports)]
use super::*;

// Every constant above was fitted together, against the 5th, 25th, 50th,
// 75th and 95th percentiles of CS6's output over the sky, the horse, the sea
// and the sand of `samples/horse-3.jpg` at Density / Foreground / Background
// of 5/10/5, 22/26/24 and 22/50/24 — sixty numbers, matched to about ten
// levels each, with the grain made as described at [`RETIC_CLUMP`]. Change
// one and the others are no longer the fit.

/// Filter ▸ Sketch ▸ Reticulation: the picture as film whose emulsion has
/// clumped, in the two swatches.
///
/// Every pixel's tone is shaken by a **grain** of small, worm-like clumps —
/// white noise blurred to a couple of pixels, so it clumps rather than
/// speckles — and the result is read through a levels curve into the two
/// swatches. The grain rides on the picture rather than replacing it: the
/// sky comes out light with dark worms through it, the horse dark with a
/// lighter web, and midtones a close mesh of the two.
///
/// The picture is read through a tone curve first, and the grain shakes
/// the result. **Density** is how hard the grain shakes it, how fine it is,
/// and how far it lightens the deepest shadows so that a web of grain
/// still shows through them. **Foreground Level** darkens the curve's
/// midtones, setting more of them in the foreground colour. **Background
/// Level** washes the highlights out towards clean background. The result
/// is continuous tone, not two colours — CS6's clumps have soft grey edges,
/// and so does this.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: one small blur of noise and a
/// curve per pixel. The grain is laid by where on the canvas a pixel is, so
/// a preview crop cannot be filtered on its own.
pub fn reticulation(
    pixmap: &mut Pixmap,
    density: u32,
    foreground_level: u32,
    background_level: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let density = density.clamp(*RETIC_DENSITY.start(), *RETIC_DENSITY.end()) as f32;
    let foreground_level =
        foreground_level.clamp(*RETIC_FOREGROUND.start(), *RETIC_FOREGROUND.end()) as f32;
    let background_level =
        background_level.clamp(*RETIC_BACKGROUND.start(), *RETIC_BACKGROUND.end()) as f32;
    let black = RETIC_BLACK + foreground_level * RETIC_BLACK_PER_STEP;
    let white = RETIC_WHITE - background_level * RETIC_WHITE_PER_STEP;
    let bend = RETIC_BEND + foreground_level * RETIC_BEND_PER_STEP;
    let lift = density * RETIC_LIFT_PER_STEP;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The grain: white noise band-passed into worms of one size, and put back
    // to a known spread so Density means the same thing however fine the
    // worms are.
    let noise: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| crate::photorust::artistic::noise((i % w) as i32, (i / w) as i32 + 104_729) - 0.5)
        .collect();
    let clump = RETIC_CLUMP - density * RETIC_CLUMP_PER_STEP;
    let mut grain = noise.clone();
    blur_field(&mut grain, w, h, clump);
    let mut broad = noise;
    blur_field(&mut broad, w, h, clump * 2.0);
    grain
        .par_iter_mut()
        .zip(broad.par_iter())
        .for_each(|(g, b)| *g -= b);
    unit_spread(&mut grain);
    let amount = RETIC_GRAIN + density * RETIC_GRAIN_PER_STEP;
    let span = (white - black).max(1e-3);

    let ink = [
        foreground.r as f32,
        foreground.g as f32,
        foreground.b as f32,
    ];
    let paper = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let grain = &grain;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let tone =
                    (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0;
                let curved = ((tone - black) / span).clamp(0.0, 1.0).powf(bend);
                let curved = lift + (1.0 - lift) * curved;
                let shaken = (curved + grain[y * w + x] * amount).clamp(0.0, 1.0);
                let light = RETIC_DEEPEST + (RETIC_PALEST - RETIC_DEEPEST) * shaken;
                for c in 0..3 {
                    px[c] = (ink[c] + (paper[c] - ink[c]) * light)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: graining the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Stamp, which its two sliders run over.
pub const STAMP_BALANCE: std::ops::RangeInclusive<u32> = 0..=50;

pub const STAMP_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=50;

/// How far the picture is melted before it is cut, in pixels of blur per
/// step of Smoothness. At 5 the horse keeps its mane and the highlights on
/// its flank; at 24 it is a rounded silhouette; by 47 the whole frame is a
/// few great soft shapes.
pub(crate) const STAMP_MELT_PER_STEP: f32 = 0.35;

/// Where the cut falls, as a share of the tone range: at the bottom of
/// Light/Dark Balance, and how far each step raises it.
///
/// The slider covers less of the tone range than it looks: under a quarter
/// of the way up at 0, about two thirds at 50. Read off CS6 on
/// `samples/horse-3.jpg` — at 9 the horse is already mostly ink, with only
/// its highlights left as paper; at 33 the dark streaks of the sea have
/// joined it; at 48 the whole sea has, and the pale sky and the spray are
/// still paper.
pub(crate) const STAMP_CUT: f32 = 0.225;

pub(crate) const STAMP_CUT_PER_STEP: f32 = 0.0083;

/// How hard local contrast is pushed before the melt, and over how wide a
/// neighbourhood, in pixels of blur. A stamp picks up a thin dark line on a
/// light ground — CS6 at Smoothness 5 inks every streak of foam-shadow across
/// the sand — even where the line is not darker than the cut on its own; it
/// is darker than what is round it. Sharpening first is what lets it through.
/// At high Smoothness the melt blurs the sharpening away again, as CS6's
/// great soft shapes show nothing of it.
pub(crate) const STAMP_LOCAL: f32 = 2.0;

pub(crate) const STAMP_LOCAL_REACH: f32 = 6.0;

/// How wide the cut between ink and paper is, as a share of the tone range.
/// Where an edge is steep that is about a pixel of anti-aliasing; where two
/// shapes almost meet the tone is shallow, and the same width becomes the
/// soft grey neck CS6 draws between them.
pub(crate) const STAMP_CUT_SOFT: f32 = 0.015;

/// Filter ▸ Sketch ▸ Stamp: the picture cut as a rubber stamp, in the two
/// swatches.
///
/// The picture's brightness is sharpened a little, so thin dark lines hold
/// — see [`STAMP_LOCAL`] — then melted by **Smoothness** — a Gaussian blur,
/// which is what rounds every shape off the way a stamp's are — and then
/// cut at the tone **Light/Dark Balance** asks for: darker than the cut
/// takes the foreground, lighter the background. Raising the balance moves
/// the cut up the tones and inks more of the picture.
///
/// Alpha is left alone.
///
/// No GPU path. The blur is the only real work and would fit, but the cut
/// is read off it at a width of a few levels, so it has to stay in floating
/// point: the GPU blur works in eight bits, and its rounding would come back
/// as stair-steps along every edge. The blur is local, so a preview crop
/// needs only a margin round it.
pub fn stamp(
    pixmap: &mut Pixmap,
    balance: u32,
    smoothness: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*STAMP_BALANCE.start(), *STAMP_BALANCE.end()) as f32;
    let smoothness = smoothness.clamp(*STAMP_SMOOTHNESS.start(), *STAMP_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    let mut around = tone.clone();
    blur_field(&mut around, w, h, STAMP_LOCAL_REACH);
    tone.par_iter_mut()
        .zip(around.par_iter())
        .for_each(|(t, a)| *t += (*t - a) * STAMP_LOCAL);
    blur_field(&mut tone, w, h, smoothness * STAMP_MELT_PER_STEP);
    let cut = STAMP_CUT + balance * STAMP_CUT_PER_STEP;

    let ink = [
        foreground.r as f32,
        foreground.g as f32,
        foreground.b as f32,
    ];
    let paper = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let tone = &tone;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let light = ((tone[y * w + x] - cut) / STAMP_CUT_SOFT + 0.5).clamp(0.0, 1.0);
                for c in 0..3 {
                    px[c] = (ink[c] + (paper[c] - ink[c]) * light)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: stamping the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Torn Edges, which its three sliders run over.
pub const TORN_BALANCE: std::ops::RangeInclusive<u32> = 0..=50;

pub const TORN_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

pub const TORN_CONTRAST: std::ops::RangeInclusive<u32> = 1..=25;

/// Where the cut falls, as a share of the tone range: at the bottom of Image
/// Balance, and how far each step raises it. Read off CS6 on
/// `samples/horse-3.jpg` — at 12 the horse is inked but its highlights are
/// not; at 25 the horse and a few of the darkest streaks of the sea; at 41
/// the whole sea, with the pale sky still paper.
pub(crate) const TORN_CUT: f32 = 0.06;

pub(crate) const TORN_CUT_PER_STEP: f32 = 0.018;

/// How far the edge of the inked mask is spread, in pixels: this over
/// Smoothness. Named backwards from what it looks like — CS6 at Smoothness 3
/// has broad, fuzzy, torn-felt edges, and at 13 crisp ragged ones — because
/// what Smoothness smooths is the *tear*: the higher it is, the less the
/// edge frays out.
pub(crate) const TORN_SPREAD: f32 = 18.0;

/// How hard the paper's grain pulls the edge about. Across the spread-out
/// edge every pixel is pushed in or out of the ink at random, which is what
/// turns a clean boundary into a torn, fibrous one. Deep inside a shape, and
/// far out in the paper, the grain is not strong enough to change anything,
/// so the ink stays solid and the paper clean — which is what CS6 draws at
/// every setting but the very top of Contrast.
pub(crate) const TORN_RAG: f32 = 0.8;

/// How big a grain of the paper is, in pixels of blur over white noise.
/// CS6's fibres and flecks are clumps two or three pixels across.
pub(crate) const TORN_GRAIN: f32 = 0.7;

/// How steeply the frayed mask is cut into ink and paper, as a floor and per
/// step of Contrast. Low, and the torn fringe is a soft grey blur; high, and
/// there is nothing between ink and paper.
pub(crate) const TORN_GAIN: f32 = 2.0;

pub(crate) const TORN_GAIN_PER_STEP: f32 = 0.6;

/// How deep the grain bites holes right through the ink at the top of
/// Contrast. CS6 shows none at 17 and half the ink gone at 25, so the bite
/// comes in steeply: this, times the Contrast slider's position to the power
/// [`TORN_HOLE_ONSET`].
pub(crate) const TORN_HOLE: f32 = 1.2;

pub(crate) const TORN_HOLE_ONSET: i32 = 8;

/// Filter ▸ Sketch ▸ Torn Edges: the picture torn out of paper, in the two
/// swatches.
///
/// Whatever is darker than the tone **Image Balance** asks for is inked, in
/// the foreground colour; the rest is paper, the background. The edge of the
/// inked mask is spread out by a blur that **Smoothness** narrows, and the
/// paper's grain pushes each pixel across it in or out at random — see
/// [`TORN_RAG`] — so the boundary tears into fibres while the shapes stay
/// solid and the paper stays clean. **Contrast** is how cleanly that is cut:
/// a soft grey fringe at the bottom, pure ink and paper higher up, and at the
/// very top the grain bites holes right through the ink.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: two small blurs and a curve per
/// pixel. The grain is laid by where on the canvas a pixel is, so a preview
/// crop cannot be filtered on its own.
pub fn torn_edges(
    pixmap: &mut Pixmap,
    balance: u32,
    smoothness: u32,
    contrast: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*TORN_BALANCE.start(), *TORN_BALANCE.end()) as f32;
    let smoothness = smoothness.clamp(*TORN_SMOOTHNESS.start(), *TORN_SMOOTHNESS.end()) as f32;
    let contrast = contrast.clamp(*TORN_CONTRAST.start(), *TORN_CONTRAST.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The inked mask, cut at the balance and frayed by the blur.
    let cut = TORN_CUT + balance * TORN_CUT_PER_STEP;
    let mut mask: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| {
            let tone = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
            if tone < cut {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    blur_field(&mut mask, w, h, TORN_SPREAD / smoothness);

    // The paper's grain: 0..1, blurred into small clumps and stretched back
    // out to its old spread so the bite means the same at any clump size.
    let mut grain: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| crate::photorust::artistic::noise((i % w) as i32, (i / w) as i32 + 15_485) - 0.5)
        .collect();
    blur_field(&mut grain, w, h, TORN_GRAIN);
    unit_spread(&mut grain);
    // White noise over 0..1 has a spread of 1/√12.
    grain
        .par_iter_mut()
        .for_each(|g| *g = (0.5 + *g * 0.2887).clamp(0.0, 1.0));
    let grain = &grain;

    let top = (contrast - *TORN_CONTRAST.start() as f32)
        / (*TORN_CONTRAST.end() - *TORN_CONTRAST.start()) as f32;
    let hole = TORN_HOLE * top.powi(TORN_HOLE_ONSET);
    let gain = TORN_GAIN + contrast * TORN_GAIN_PER_STEP;
    let ink = [
        foreground.r as f32,
        foreground.g as f32,
        foreground.b as f32,
    ];
    let paper = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let mask = &mask;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let g = grain[i];
                // The grain pulls only where there is an edge to tear: open
                // paper and the solid middle of a shape have none, so a soft
                // low-Contrast cut leaves neither a grey cast on the paper
                // nor one in the ink.
                let m = mask[i];
                let edge = (4.0 * m * (1.0 - m)).max(0.0).sqrt();
                let torn = m + (g - 0.5) * TORN_RAG * edge - g * hole;
                let inked = ((torn - 0.5) * gain + 0.5).clamp(0.0, 1.0);
                for c in 0..3 {
                    px[c] = (paper[c] + (ink[c] - paper[c]) * inked)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: tearing the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Water Paper, which its three sliders run over.
pub const WATER_FIBER: std::ops::RangeInclusive<u32> = 3..=50;

pub const WATER_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=100;

pub const WATER_CONTRAST: std::ops::RangeInclusive<u32> = 0..=100;

/// How far the colour runs along a fibre, in pixels, per step of Fiber
/// Length — the whole run, both ways. Read off CS6 on `samples/horse-3.jpg`:
/// at 15 the mane bleeds in streaks a dozen pixels long, at 30 about twice
/// that.
pub(crate) const WATER_RUN_PER_STEP: f32 = 0.9;

/// How strongly the wettest fibres carry colour over the rest. The fibres'
/// streaked noise has a spread of one, and a pixel's pull along a fibre is
/// `exp` of this times it. Too little and the bleed is an even cross-shaped
/// blur; too much and it is a scribble of dark scratches, which CS6's soft
/// run of colour is not.
pub(crate) const WATER_SOAK: f32 = 0.9;

/// How deep the paper's weave shows through the pigment, in levels at full
/// darkness. The weave shows in the ink and not on bare paper: CS6's dark
/// horse is cross-hatched and its pale sky is smooth.
pub(crate) const WATER_WEAVE: f32 = 18.0;

/// Brightness as a factor of two to the power of its distance from the
/// middle of the slider over this. Below the middle the picture is scaled
/// down by it; above, it is the gamma the picture is lifted by. A lift rather
/// than a scale, because CS6 at 87 turns the brown horse pink and the sky
/// pale blue but leaves the mane black: the mid-tones rise and the tint
/// survives, where scaling would have burnt it all out to white.
pub(crate) const WATER_BRIGHTNESS_DOUBLING: f32 = 30.0;

/// Contrast as a gain about mid-grey: doubling every this many steps from a
/// gain of one at [`WATER_CONTRAST_FLAT`]. CS6 at 23 is flat and murky, at
/// 34 hazy, and at 94 crushes the darks to black.
pub(crate) const WATER_CONTRAST_DOUBLING: f32 = 30.0;

pub(crate) const WATER_CONTRAST_FLAT: f32 = 50.0;

/// Where the highlights start to roll off rather than clip, as a share of the
/// range. At high Contrast CS6's light areas crowd up towards white but keep
/// their colour — pastel, not blown out.
pub(crate) const WATER_SHOULDER: f32 = 0.7;

/// Filter ▸ Sketch ▸ Water Paper: the picture daubed onto damp, fibrous
/// paper, the colour running along the fibres.
///
/// The paper is two sets of fibres, one running down and one across, each
/// streaked noise as long as **Fiber Length** asks. Every pixel's colour is
/// the average of the picture along the fibres through it, weighted by how
/// wet each fibre is — see [`WATER_SOAK`] — so colour bleeds out of a shape in
/// streaks both ways and the picture takes on the grid of the paper. Pigment
/// then settles into the fibres, which gives the dark parts their weave.
/// **Brightness** sinks the whole picture towards black or lifts its
/// mid-tones towards white, and
/// **Contrast** stretches it about mid-grey, in that order, with the
/// highlights rolling off rather than clipping.
///
/// Unlike the rest of the family it keeps the picture's colours: CS6's Water
/// Paper ignores the swatches.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: a handful of short one-axis sums
/// and a curve per pixel. The fibres are laid by where on the canvas a pixel
/// is, so a preview crop cannot be filtered on its own.
pub fn water_paper(pixmap: &mut Pixmap, fiber: u32, brightness: u32, contrast: u32) {
    if pixmap.is_empty() {
        return;
    }
    let fiber = fiber.clamp(*WATER_FIBER.start(), *WATER_FIBER.end()) as f32;
    let brightness = brightness.clamp(*WATER_BRIGHTNESS.start(), *WATER_BRIGHTNESS.end()) as f32;
    let contrast = contrast.clamp(*WATER_CONTRAST.start(), *WATER_CONTRAST.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let run = fiber * WATER_RUN_PER_STEP;
    let reach = (run / 2.0).round().max(1.0) as i32;

    let down = crate::photorust::artistic::streaked_noise(w, h, run, 31, (0.0, 1.0));
    let across = crate::photorust::artistic::streaked_noise(w, h, run, 32, (1.0, 0.0));
    let wet_down: Vec<f32> = down.par_iter().map(|v| (WATER_SOAK * v).exp()).collect();
    let wet_across: Vec<f32> = across.par_iter().map(|v| (WATER_SOAK * v).exp()).collect();

    // Each pixel's colour, in floats, so the sums below read it cheaply.
    let stride = pixmap.stride();
    let source: Vec<[f32; 3]> = pixmap
        .as_bytes()
        .chunks_exact(4)
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let (source, down, across) = (&source, &down, &across);
    let (wet_down, wet_across) = (&wet_down, &wet_across);

    let lift = 2f32.powf((brightness - 50.0) / WATER_BRIGHTNESS_DOUBLING);
    let gain = 2f32.powf((contrast - WATER_CONTRAST_FLAT) / WATER_CONTRAST_DOUBLING);
    let tone_curve = |v: f32| {
        let v = v.clamp(0.0, 1.0);
        let v = if lift < 1.0 {
            v * lift
        } else {
            v.powf(1.0 / lift)
        };
        let v = (v - 0.5) * gain + 0.5;
        if v > WATER_SHOULDER {
            let room = 1.0 - WATER_SHOULDER;
            1.0 - room * (-(v - WATER_SHOULDER) / room).exp()
        } else {
            v
        }
    };
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                // The colour carried along both fibres through this pixel.
                let mut total = [0.0f32; 3];
                let mut weight = 0.0f32;
                for k in -reach..=reach {
                    let sy = y as i32 + k;
                    if sy >= 0 && sy < h as i32 {
                        let j = sy as usize * w + x;
                        let wt = wet_down[j];
                        for c in 0..3 {
                            total[c] += wt * source[j][c];
                        }
                        weight += wt;
                    }
                    let sx = x as i32 + k;
                    if sx >= 0 && sx < w as i32 {
                        let j = y * w + sx as usize;
                        let wt = wet_across[j];
                        for c in 0..3 {
                            total[c] += wt * source[j][c];
                        }
                        weight += wt;
                    }
                }
                let i = y * w + x;
                let bled = total.map(|t| t / weight);

                // Pigment settles into the fibres, and there is only as much
                // of it to settle as the pixel is dark.
                let tone = (0.299 * bled[0] + 0.587 * bled[1] + 0.114 * bled[2]) / 255.0;
                let weave = down[i].max(across[i]) * WATER_WEAVE * (1.0 - tone);
                for c in 0..3 {
                    let v = tone_curve((bled[c] - weave) / 255.0);
                    px[c] = (v * 255.0).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: wetting the paper does not change the layer's
                // shape.
            }
        });
}
