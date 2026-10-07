//! Paint Daubs, Plastic Wrap, Poster Edges, Rough Pastels, Smudge Stick (split from `artistic` for file size).

#[allow(unused_imports)]
use super::*;

/// Filter ▸ Artistic ▸ Paint Daubs: the picture repainted in daubs, then
/// sharpened, with the brush deciding the shape of the daub and what the
/// sharpening looks like.
///
/// **Brush Size** is the daub. **Sharpness** is the sharpening laid over it.
/// **Brush Type**: Simple is the two as they are; Wide Sharp and Wide Blurry
/// stretch the daub sideways and sharpen harder or soften it; Light Rough and
/// Dark Rough lay a tooth and keep only the light halo or only the dark one;
/// Sparkle draws contour lines and lights the picture up — see [`sparkle`].
///
/// Alpha is left alone: repainting the picture does not change the layer's
/// shape.
///
/// No GPU path. The daubing is a median over a sliding histogram —
/// sequential along each row by construction, and the reason a fifty-pixel
/// brush is fast enough to offer at all.
pub fn paint_daubs(pixmap: &mut Pixmap, size: u32, sharpness: u32, brush: DaubBrush) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*DAUB_SIZE.start(), *DAUB_SIZE.end());
    let sharpness = sharpness.clamp(*DAUB_SHARPNESS.start(), *DAUB_SHARPNESS.end()) as f32;
    let (across, down) = daub_reach(size, brush);
    let rough = brush.is_rough();

    // Not under Sparkle: its contour lines need the smooth shading the tooth
    // would break up.
    if rough && brush != DaubBrush::Sparkle {
        lay_tooth(pixmap, DAUB_TOOTH, DAUB_TOOTH_SCALE, 0);
    }
    let mut daubs = crate::photorust::convolve::median_of(pixmap, across, down);
    for (daub, original) in daubs
        .as_bytes_mut()
        .chunks_exact_mut(4)
        .zip(pixmap.as_bytes().chunks_exact(4))
    {
        daub[3] = original[3];
    }
    *pixmap = daubs;

    let edge = sharpness * EDGE_PER_STEP;
    let halo = sharpness * ROUGH_PER_STEP;
    match brush {
        DaubBrush::Simple => sharpen(pixmap, edge),
        DaubBrush::WideSharp => sharpen(pixmap, edge * 2.0),
        DaubBrush::WideBlurry => {
            crate::photorust::convolve::gaussian_blur_accelerated(
                pixmap,
                down as f32 * 2.0 * WIDE_BLUR,
            );
            sharpen(pixmap, edge * 0.5);
        }
        DaubBrush::LightRough => rough_edge(pixmap, halo, Halo::Light),
        DaubBrush::DarkRough => rough_edge(pixmap, halo, Halo::Dark),
        DaubBrush::Sparkle => sparkle(pixmap, sharpness),
    }
    if rough {
        lay_tooth(pixmap, DAUB_GRIT, DAUB_GRIT_SCALE, 3);
    }
}

impl DaubBrush {
    /// Whether the brush lays a tooth, which is laid by where on the canvas a
    /// pixel is.
    pub fn is_rough(self) -> bool {
        matches!(
            self,
            DaubBrush::LightRough | DaubBrush::DarkRough | DaubBrush::Sparkle
        )
    }
}

/// How far the daub reaches across and down, in pixels either side.
pub fn daub_reach(size: u32, brush: DaubBrush) -> (u32, u32) {
    let reach = (size as f32 * DAUB_REACH).round().max(1.0);
    match brush {
        DaubBrush::WideSharp | DaubBrush::WideBlurry => (
            (reach * WIDE_STRETCH) as u32,
            (reach / WIDE_STRETCH).max(1.0) as u32,
        ),
        _ => (reach as u32, reach as u32),
    }
}

/// Noise in each colour channel, blurred to specks `scale` pixels across and
/// centred on 127.5. `salt` keeps two uses from being the same noise.
pub(crate) fn blurred_specks(width: u32, height: u32, scale: f32, salt: usize) -> Pixmap {
    let mut specks = Pixmap::new(width, height);
    specks
        .as_bytes_mut()
        .par_chunks_exact_mut(width as usize * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                for c in 0..3 {
                    px[c] = ((speck(x as i32, y as i32, c + salt) * 0.5 + 0.5) * 255.0) as u8;
                }
                px[3] = 255;
            }
        });
    crate::photorust::convolve::gaussian_blur_accelerated(&mut specks, scale);
    specks
}

/// What [`blurred_specks`] at `scale` must be multiplied by to spread about
/// one level. A blur of σ leaves white noise about 1/(2√π·σ) of its spread,
/// and `speck` starts at about 52 levels.
pub(crate) fn speck_gain(scale: f32) -> f32 {
    2.0 * std::f32::consts::PI.sqrt() * scale / 52.0
}

/// Add colour noise of about `levels` spread, in specks `scale` pixels
/// across, each channel on its own. `salt` keeps two coats from being the
/// same noise.
pub(crate) fn lay_tooth(pixmap: &mut Pixmap, levels: f32, scale: f32, salt: usize) {
    let tooth = blurred_specks(pixmap.width(), pixmap.height(), scale, salt);
    let gain = levels * speck_gain(scale);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(tooth.as_bytes().par_chunks_exact(stride))
        .for_each(|(out, tooth)| {
            for (out, tooth) in out.chunks_exact_mut(4).zip(tooth.chunks_exact(4)) {
                for c in 0..3 {
                    let v = out[c] as f32 + (tooth[c] as f32 - 127.5) * gain;
                    out[c] = v.clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}

/// Sparkle's contour lines: how many levels of brightness apart they are
/// drawn, how far and how broadly the wobble pushes them about, how thin they
/// are, and how bright they are per step of Sharpness. The dark lines are
/// drawn at a fraction of the light ones'.
pub(crate) const CONTOUR_STEP: f32 = 8.0;

pub(crate) const CONTOUR_SOFTEN: f32 = 2.0;

pub(crate) const CONTOUR_WOBBLE: f32 = 12.0;

pub(crate) const CONTOUR_WOBBLE_SCALE: f32 = 12.0;

pub(crate) const CONTOUR_THIN: f32 = 8.0;

pub(crate) const CONTOUR_PER_STEP: f32 = 3.0;

pub(crate) const CONTOUR_DARK: f32 = 0.5;

/// Sparkle's sharpening: scale in pixels and strength per step.
pub(crate) const SPARKLE_SCALE: f32 = 1.5;

pub(crate) const SPARKLE_PER_STEP: f32 = 0.1;

/// Sparkle's brilliance: an overall lift (gain after a gamma), then a push
/// towards white that starts at [`BRILLIANCE_FROM`] and leaves the darks alone.
pub(crate) const BRILLIANCE_GAMMA: f32 = 0.85;

pub(crate) const BRILLIANCE_GAIN: f32 = 1.1;

pub(crate) const BRILLIANCE_FROM: f32 = 0.3;

pub(crate) const BRILLIANCE_PUSH: f32 = 0.5;

/// Sparkle: the daubs with contour lines drawn through them, sharpened, and
/// lit up.
///
/// The swirling lines all over CS6's Sparkle, which are densest in the
/// out-of-focus background, are iso-lines of brightness: where the picture
/// shades slowly they are far apart and follow the shading, and a slow random
/// wobble added to the brightness first keeps them from reading as a
/// topographic map. Light lines are drawn stronger than dark ones, which is
/// half of the brilliance; the other half is a tone curve that sends the light
/// parts towards white channel by channel, so the petals go pale rather than
/// merely brighter while the background stays dark.
pub(crate) fn sparkle(pixmap: &mut Pixmap, sharpness: f32) {
    use std::f32::consts::TAU;

    let mut field = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut field, CONTOUR_SOFTEN);
    let wobble = blurred_specks(pixmap.width(), pixmap.height(), CONTOUR_WOBBLE_SCALE, 7);
    let wobble_gain = CONTOUR_WOBBLE * speck_gain(CONTOUR_WOBBLE_SCALE);
    let amp = sharpness * CONTOUR_PER_STEP;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(field.as_bytes().par_chunks_exact(stride))
        .zip(wobble.as_bytes().par_chunks_exact(stride))
        .for_each(|((out, field), wobble)| {
            for ((out, f), wb) in out
                .chunks_exact_mut(4)
                .zip(field.chunks_exact(4))
                .zip(wobble.chunks_exact(4))
            {
                let lum = 0.299 * f[0] as f32 + 0.587 * f[1] as f32 + 0.114 * f[2] as f32;
                let level = lum + (wb[0] as f32 - 127.5) * wobble_gain;
                let wave = (level / CONTOUR_STEP * TAU).cos();
                let line = wave.max(0.0).powf(CONTOUR_THIN)
                    - CONTOUR_DARK * (-wave).max(0.0).powf(CONTOUR_THIN);
                for c in 0..3 {
                    out[c] = (out[c] as f32 + line * amp).clamp(0.0, 255.0).round() as u8;
                }
            }
        });

    let sharp = pixmap.clone();
    let mut soft = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, SPARKLE_SCALE);
    put_back(pixmap, &sharp, &soft, sharpness * SPARKLE_PER_STEP);

    let brilliant: [u8; 256] = std::array::from_fn(|v| {
        let x = ((v as f32 / 255.0).powf(BRILLIANCE_GAMMA) * BRILLIANCE_GAIN).min(1.0);
        let k = ((x - BRILLIANCE_FROM) / (1.0 - BRILLIANCE_FROM)).clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        let x = x + BRILLIANCE_PUSH * k * (1.0 - x);
        (x * 255.0).round() as u8
    });
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(4)
        .for_each(|px| {
            for c in 0..3 {
                px[c] = brilliant[px[c] as usize];
            }
        });
}

/// An unsharp mask at [`EDGE_SCALE`], channel by channel.
pub(crate) fn sharpen(pixmap: &mut Pixmap, gain: f32) {
    if gain <= 0.0 {
        return;
    }
    let sharp = pixmap.clone();
    let mut soft = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, EDGE_SCALE);
    put_back(pixmap, &sharp, &soft, gain);
}

/// Which side of a boundary a Rough brush draws its halo on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Halo {
    Light,
    Dark,
}

/// The Rough brushes' halo, taken on brightness and laid on all three
/// channels alike, so that a dark halo goes to black rather than to a deeper
/// shade of whatever it was.
pub(crate) fn rough_edge(pixmap: &mut Pixmap, gain: f32, keep: Halo) {
    if gain <= 0.0 {
        return;
    }
    let mut soft = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, ROUGH_SCALE);
    let lum = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(soft.as_bytes().par_chunks_exact(stride))
        .for_each(|(out, soft)| {
            for (out, soft) in out.chunks_exact_mut(4).zip(soft.chunks_exact(4)) {
                let halo = lum(out) - lum(soft);
                let halo = match keep {
                    Halo::Light => halo.max(0.0),
                    Halo::Dark => halo.min(0.0),
                } * gain;
                for c in 0..3 {
                    out[c] = (out[c] as f32 + halo).clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}

/// Add what `over` has and `under` does not to whatever is in `pixmap`.
pub(crate) fn put_back(pixmap: &mut Pixmap, over: &Pixmap, under: &Pixmap, gain: f32) {
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(over.as_bytes().par_chunks_exact(stride))
        .zip(under.as_bytes().par_chunks_exact(stride))
        .for_each(|((out, over), under)| {
            for ((out, over), under) in out
                .chunks_exact_mut(4)
                .zip(over.chunks_exact(4))
                .zip(under.chunks_exact(4))
            {
                for c in 0..3 {
                    let lift = (over[c] as f32 - under[c] as f32) * gain;
                    out[c] = (out[c] as f32 + lift).clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}

/// How broad a swell counts as the picture's shading rather than its relief,
/// in pixels. The relief is the picture less a blur this wide, so a bright
/// shape stands up out of the wrap and the wrap dips into a trough around it
/// before it settles — and the far wall of that trough is what catches the
/// light in the ring CS6 draws a little way outside every petal.
pub(crate) const WRAP_SHADING: f32 = 8.0;

/// How far the relief is smoothed before it is lit, in pixels: a floor, and
/// a step of Smoothness. Wrap laid over a surface does not follow its every
/// grain; the smoother it is, the broader and fewer its folds.
pub(crate) const WRAP_SMOOTH_FLOOR: f32 = 1.0;

pub(crate) const WRAP_SMOOTH_PER_STEP: f32 = 0.7;

/// How hard the relief is pushed up before it is lit: a floor, and a step of
/// Detail. It goes through `tanh`, so strong relief is capped and faint relief
/// is what the slider really raises — which is what brings up the crinkles
/// in the background as Detail goes up.
pub(crate) const WRAP_DETAIL_FLOOR: f32 = 0.5;

pub(crate) const WRAP_DETAIL_PER_STEP: f32 = 0.08;

/// How steep the relief stands, as a multiplier on its slope.
pub(crate) const WRAP_DEPTH: f32 = 250.0;

/// Where the light is — up and to the left, the convention relief is read
/// by — and how tight the highlight is. High, because plastic is glossy: the
/// highlights are thin bright streaks, not a sheen.
pub(crate) const WRAP_LIGHT: [f32; 3] = [-1.0, -1.0, 1.2];

pub(crate) const WRAP_SHINE: f32 = 50.0;

/// How bright the highlights are at the top of Highlight Strength, and how
/// sharply that grows along the slider. CS6's low settings are barely there,
/// so the growth is steeper than a straight line.
pub(crate) const WRAP_GLOSS: f32 = 5.0;

pub(crate) const WRAP_GLOSS_CURVE: f32 = 1.6;

/// How much the wrap dulls the picture under it at the top of Highlight
/// Strength.
pub(crate) const WRAP_DULL: f32 = 0.25;

/// Filter ▸ Artistic ▸ Plastic Wrap: the picture shrink-wrapped in glossy
/// plastic.
///
/// The picture's brightness is read as a surface — light things stand up,
/// dark things sink — and the surface is lit by one light with a tight
/// specular highlight. Only the highlight is laid back on, towards white, over
/// a slightly dulled copy of the picture; the plastic itself is clear.
///
/// **Highlight Strength** is how bright the highlights are. **Detail** is how
/// much of the picture's faint relief the wrap picks up. **Smoothness** is how
/// broad its folds are.
///
/// Alpha is left alone: wrapping the picture does not change the layer's
/// shape.
///
/// No GPU path. The work is three blurs of a floating-point height field —
/// in bytes the relief is a level or two deep and its slopes would come back
/// as steps — and the backend's blur is over bytes. The lighting is one pass
/// per pixel, which would upload its input and read it straight back.
pub fn plastic_wrap(pixmap: &mut Pixmap, highlight: u32, detail: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let highlight = highlight.clamp(*WRAP_HIGHLIGHT.start(), *WRAP_HIGHLIGHT.end()) as f32;
    let detail = detail.clamp(*WRAP_DETAIL.start(), *WRAP_DETAIL.end()) as f32;
    let smoothness = smoothness.clamp(*WRAP_SMOOTHNESS.start(), *WRAP_SMOOTHNESS.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);

    let brightness: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    let mut shading = brightness.clone();
    blur_field(&mut shading, width, height, WRAP_SHADING);
    let mut relief: Vec<f32> = brightness
        .iter()
        .zip(&shading)
        .map(|(b, s)| b - s)
        .collect();
    blur_field(
        &mut relief,
        width,
        height,
        WRAP_SMOOTH_FLOOR + smoothness * WRAP_SMOOTH_PER_STEP,
    );
    let push = WRAP_DETAIL_FLOOR + detail * WRAP_DETAIL_PER_STEP;
    relief.par_iter_mut().for_each(|r| *r = (*r * push).tanh());

    // The half-way vector between the light and a viewer straight above.
    let light = normalise(WRAP_LIGHT);
    let half = normalise([light[0], light[1], light[2] + 1.0]);
    let gloss = (highlight / *WRAP_HIGHLIGHT.end() as f32).powf(WRAP_GLOSS_CURVE) * WRAP_GLOSS;
    let dull = 1.0 - WRAP_DULL * highlight / *WRAP_HIGHLIGHT.end() as f32;

    let relief = &relief;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            // Central differences, one-sided at the edges.
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, px) in out.chunks_exact_mut(4).enumerate() {
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let dx = (relief[y * width + right] - relief[y * width + left])
                    / (right - left).max(1) as f32;
                let dy =
                    (relief[down * width + x] - relief[up * width + x]) / (down - up).max(1) as f32;
                let normal = normalise([-dx * WRAP_DEPTH, -dy * WRAP_DEPTH, 1.0]);
                let facing =
                    (normal[0] * half[0] + normal[1] * half[1] + normal[2] * half[2]).max(0.0);
                let shine = (facing.powf(WRAP_SHINE) * gloss).min(1.0);
                for c in 0..3 {
                    let under = px[c] as f32 * dull;
                    px[c] = (under + (255.0 - under) * shine).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: wrapping the picture does not change the
                // layer's shape.
            }
        });
}

pub(crate) fn normalise(v: [f32; 3]) -> [f32; 3] {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / length, v[1] / length, v[2] / length]
}

/// A Gaussian blur of a floating-point field, with the edge pixels standing
/// in for what is past them.
pub(crate) fn blur_field(field: &mut [f32], width: usize, height: usize, sigma: f32) {
    if sigma <= 0.0 {
        return;
    }
    let taps = (sigma * 3.0).ceil() as i32;
    let kernel = crate::photorust::convolve::gaussian_kernel_1d(sigma, taps);
    let kernel = &kernel;
    let mut scratch = field.to_vec();
    field
        .par_chunks_exact_mut(width)
        .zip(scratch.par_chunks_exact(width))
        .for_each(|(out, row)| {
            for (x, slot) in out.iter_mut().enumerate() {
                *slot = kernel
                    .iter()
                    .enumerate()
                    .map(|(k, w)| {
                        let at = (x as i32 + k as i32 - taps).clamp(0, width as i32 - 1);
                        w * row[at as usize]
                    })
                    .sum();
            }
        });
    scratch.copy_from_slice(field);
    let scratch = &scratch;
    field
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, out)| {
            for (x, slot) in out.iter_mut().enumerate() {
                *slot = kernel
                    .iter()
                    .enumerate()
                    .map(|(k, w)| {
                        let at = (y as i32 + k as i32 - taps).clamp(0, height as i32 - 1);
                        w * scratch[at as usize * width + x]
                    })
                    .sum();
            }
        });
}

/// How far the picture is smoothed before it is posterized, in pixels. It is
/// what gives the bands blotchy, rounded outlines rather than the ragged ones
/// a photograph's grain would cut.
pub(crate) const POSTER_SETTLE: f32 = 2.0;

/// How many bands of brightness Posterization gives: a floor, plus a step,
/// plus a step that grows with the slider. CS6 is harsh at the bottom of the
/// slider — three bands, so the darkest parts of the background go black — and
/// by the top the banding is barely there.
pub(crate) const POSTER_BANDS: f32 = 3.0;

pub(crate) const POSTER_BANDS_CURVE: f32 = 0.25;

/// Where between two bands a brightness is rounded up rather than down. Over
/// a half, so that a band only goes to black when it is well into the dark:
/// CS6's lowest setting turns a mid-green background bright green with black
/// only in its deepest shadow.
pub(crate) const POSTER_ROUND: f32 = 0.64;

/// The edges: how far round each pixel the ink looks, as a floor and a step
/// of Edge Thickness; how much darker than that a pixel must be before it is
/// inked, in levels; and how much ink a level of darkness is worth, as a floor
/// and a step of Edge Intensity.
///
/// Ink goes where a pixel is darker than what is round it, which is the dark
/// side of every boundary — the background just outside a petal — and along
/// anything thin and dark, like a vein. That is where CS6 draws.
pub(crate) const POSTER_EDGE_SOFTEN: f32 = 0.5;

pub(crate) const POSTER_REACH: f32 = 1.5;

pub(crate) const POSTER_REACH_PER_STEP: f32 = 0.5;

pub(crate) const POSTER_INK_FROM: f32 = 2.0;

pub(crate) const POSTER_INK: f32 = 12.0;

pub(crate) const POSTER_INK_PER_STEP: f32 = 4.0;

/// Filter ▸ Artistic ▸ Poster Edges: the picture posterized, with its edges
/// inked in black.
///
/// **Posterization** bands the picture's brightness and keeps its colour: each
/// pixel is scaled to its band's brightness, so a background comes back as a
/// few flat greens rather than as the few flat primaries per-channel
/// posterizing would give. **Edge Thickness** is how broad the ink is, and
/// **Edge Intensity** how much of the picture it picks up — at the top, every
/// vein is hatched in.
///
/// Alpha is left alone: posterizing the picture does not change the layer's
/// shape.
///
/// No GPU path. What costs is the blurs — the settling one already goes
/// through the backend, and the one the ink measures against is over floats —
/// and the rest is one pass that would upload its input and read it straight
/// back.
pub fn poster_edges(pixmap: &mut Pixmap, thickness: u32, intensity: u32, posterization: u32) {
    if pixmap.is_empty() {
        return;
    }
    let thickness = thickness.clamp(*POSTER_THICKNESS.start(), *POSTER_THICKNESS.end()) as f32;
    let intensity = intensity.clamp(*POSTER_INTENSITY.start(), *POSTER_INTENSITY.end()) as f32;
    let posterization = posterization.clamp(*POSTER_LEVELS.start(), *POSTER_LEVELS.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let brightness_of = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;

    let mut settled = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut settled, POSTER_SETTLE);

    let mut softened = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut softened, POSTER_EDGE_SOFTEN);
    let brightness: Vec<f32> = softened
        .as_bytes()
        .par_chunks_exact(4)
        .map(brightness_of)
        .collect();
    let mut around = brightness.clone();
    blur_field(
        &mut around,
        width,
        height,
        POSTER_REACH + thickness * POSTER_REACH_PER_STEP,
    );

    let steps =
        POSTER_BANDS + posterization + posterization * posterization * POSTER_BANDS_CURVE - 1.0;
    let ink_gain = (POSTER_INK + intensity * POSTER_INK_PER_STEP) / 255.0;
    let (brightness, around) = (&brightness, &around);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(settled.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, (out, settled))| {
            for (x, (px, from)) in out
                .chunks_exact_mut(4)
                .zip(settled.chunks_exact(4))
                .enumerate()
            {
                let level = brightness_of(from);
                let band =
                    (level / 255.0 * steps + POSTER_ROUND).floor().min(steps) * 255.0 / steps;
                let scale = band / level.max(1.0);

                let i = y * width + x;
                let darker = around[i] - brightness[i] - POSTER_INK_FROM;
                let bare = 1.0 - (darker * ink_gain).clamp(0.0, 1.0);

                for c in 0..3 {
                    px[c] = ((from[c] as f32 * scale).min(255.0) * bare).round() as u8;
                }
                // Alpha stands: posterizing the picture does not change the
                // layer's shape.
            }
        });
}

/// How long a pastel stroke is, in pixels: a floor, so that even Stroke
/// Length 0 is drawn in strokes as CS6's is, and a step of the slider.
///
/// This is the length of the chalk's *grain*. The picture itself is smeared
/// far less — [`PASTEL_SMEAR`] — because CS6's pastel keeps the picture sharp
/// at any Stroke Length: the long streaks are chalk laid over it, not the
/// picture dragged out. Smearing the picture by the stroke's full length
/// turns it into a motion blur.
pub(crate) const PASTEL_STROKE: f32 = 6.0;

pub(crate) const PASTEL_STROKE_PER_STEP: f32 = 0.6;

pub(crate) const PASTEL_SMEAR: f32 = 2.0;

pub(crate) const PASTEL_SMEAR_PER_STEP: f32 = 0.1;

/// Which way the strokes run: up and to the right, as CS6's do, in degrees
/// anticlockwise from the horizontal.
pub(crate) const PASTEL_ANGLE: f32 = 45.0;

/// How far along itself a stroke drags its colour, in stroke lengths, at its
/// furthest. Each stroke takes its colour from a little way up or down its own
/// line, which is what breaks an outline into the ragged, overlapping marks of
/// chalk rather than a clean smear.
pub(crate) const PASTEL_DRAG: f32 = 0.12;

/// How much of the picture's fine detail each stroke carries, as a floor and
/// a step of Stroke Detail, and the scale in pixels below which it counts as
/// detail. High detail is what lays CS6's bright scratches over the horse's
/// highlights.
pub(crate) const PASTEL_DETAIL_SCALE: f32 = 2.0;

pub(crate) const PASTEL_DETAIL_FLOOR: f32 = 0.5;

pub(crate) const PASTEL_DETAIL_PER_STEP: f32 = 0.15;

/// The grain of the chalk, in levels of spread. It is strongest in the dark,
/// where chalk goes on thin and catches only on the tooth of the paper, and
/// faintest in the light, where it is laid thick.
pub(crate) const PASTEL_GRAIN: f32 = 22.0;

pub(crate) const PASTEL_GRAIN_DARK: f32 = 1.2;

/// How much lighter than the picture the pastel is, as a gamma: chalk is
/// opaque and pale, and CS6's pastel is a washed-out copy of the photograph.
pub(crate) const PASTEL_PALE: f32 = 0.8;

/// Filter ▸ Artistic ▸ Rough Pastels: the picture drawn in coloured chalk on
/// a textured surface.
///
/// Two stages, the second shared with the other textured filters.
///
/// 1. **The strokes.** The picture is dragged into diagonal strokes, each
///    carrying some of the fine detail under it, with the grain of the chalk
///    over the top, and then paled. **Stroke Length** is how long the strokes
///    are; **Stroke Detail** how much of the picture they carry.
/// 2. **The surface.** The strokes are lit as if drawn on the chosen texture —
///    see [`crate::photorust::texture::apply_relief`] for Texture, Scaling,
///    Relief, Light and Invert.
///
/// Alpha is left alone.
///
/// No GPU path. The strokes are sampled along a line, which is a fit, but
/// every stage wants the previous one's result on the CPU and each would
/// upload and read back — the per-call cost compositing already showed is not
/// worth paying (docs/gpu-migration.md).
#[allow(clippy::too_many_arguments)]
pub fn rough_pastels(
    pixmap: &mut Pixmap,
    length: u32,
    detail: u32,
    texture: crate::photorust::texture::Texture,
    scaling: u32,
    relief: u32,
    light: crate::photorust::texture::Light,
    invert: bool,
) {
    if pixmap.is_empty() {
        return;
    }
    let length = length.clamp(*PASTEL_LENGTH.start(), *PASTEL_LENGTH.end()) as f32;
    let detail = detail.clamp(*PASTEL_DETAIL.start(), *PASTEL_DETAIL.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let stroke = PASTEL_STROKE + length * PASTEL_STROKE_PER_STEP;
    let (along_x, along_y) = {
        let radians = PASTEL_ANGLE.to_radians();
        (radians.cos(), -radians.sin())
    };

    // Where each stroke takes its colour from.
    let drag = streaked_noise(width, height, stroke * 2.0 + 3.0, 11, (along_x, along_y));
    let picture = pixmap.clone();
    let mut dragged = picture.clone();
    dragged
        .as_bytes_mut()
        .par_chunks_exact_mut(width * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let offset = drag[y * width + x].clamp(-2.0, 2.0) * stroke * PASTEL_DRAG;
                let sx = (x as f32 + along_x * offset)
                    .round()
                    .clamp(0.0, (width - 1) as f32);
                let sy = (y as f32 + along_y * offset)
                    .round()
                    .clamp(0.0, (height - 1) as f32);
                let from = (sy as usize * width + sx as usize) * 4;
                px[..3].copy_from_slice(&picture.as_bytes()[from..from + 3]);
            }
        });
    let mut strokes = dragged;
    let smear = PASTEL_SMEAR + length * PASTEL_SMEAR_PER_STEP;
    crate::photorust::convolve::motion_blur(&mut strokes, PASTEL_ANGLE, smear);

    // The picture's fine detail, drawn along the stroke: a streak of the
    // picture less a streak of its blur, which is the streak of the detail.
    let carried = smear;
    let mut sharp = picture.clone();
    crate::photorust::convolve::motion_blur(&mut sharp, PASTEL_ANGLE, carried);
    let mut soft = picture.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, PASTEL_DETAIL_SCALE);
    crate::photorust::convolve::motion_blur(&mut soft, PASTEL_ANGLE, carried);
    let carry = PASTEL_DETAIL_FLOOR + detail * PASTEL_DETAIL_PER_STEP;

    let grain = streaked_noise(width, height, stroke * 1.5 + 3.0, 12, (along_x, along_y));
    let pale: [f32; 256] = std::array::from_fn(|v| 255.0 * (v as f32 / 255.0).powf(PASTEL_PALE));

    let stride = pixmap.stride();
    let (sharp, soft, grain) = (&sharp, &soft, &grain);
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(strokes.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, (out, strokes))| {
            let (sharp, soft) = (sharp.row(y as u32), soft.row(y as u32));
            for x in 0..width {
                let i = x * 4;
                let mut chalk = [0.0f32; 3];
                for c in 0..3 {
                    chalk[c] =
                        strokes[i + c] as f32 + (sharp[i + c] as f32 - soft[i + c] as f32) * carry;
                }
                let lightness = (0.299 * chalk[0] + 0.587 * chalk[1] + 0.114 * chalk[2]) / 255.0;
                let tooth = grain[y * width + x] * PASTEL_GRAIN * (PASTEL_GRAIN_DARK - lightness);
                for c in 0..3 {
                    let v = (chalk[c] + tooth).clamp(0.0, 255.0);
                    out[i + c] = pale[v.round() as usize].round() as u8;
                }
                // Alpha stands: drawing the picture does not change the
                // layer's shape.
            }
        });

    crate::photorust::texture::apply_relief(pixmap, texture, scaling, relief, light, invert);
}

/// White noise averaged along `along` over `length` pixels, then scaled to a
/// spread of one: streaks, each about as long as a stroke. Laid by where on
/// the canvas a pixel is.
pub(crate) fn streaked_noise(
    width: usize,
    height: usize,
    length: f32,
    salt: usize,
    along: (f32, f32),
) -> Vec<f32> {
    let noise: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| speck((i % width) as i32, (i / width) as i32, salt))
        .collect();
    let steps = length.round().max(1.0) as i32;
    let mut streaks = vec![0.0f32; width * height];
    streaks
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, slot) in row.iter_mut().enumerate() {
                let (mut total, mut count) = (0.0f32, 0.0f32);
                for step in -steps / 2..=steps / 2 {
                    let sx = (x as f32 + along.0 * step as f32).round() as i32;
                    let sy = (y as f32 + along.1 * step as f32).round() as i32;
                    if sx < 0 || sy < 0 || sx >= width as i32 || sy >= height as i32 {
                        continue;
                    }
                    total += noise[sy as usize * width + sx as usize];
                    count += 1.0;
                }
                *slot = total / count.max(1.0);
            }
        });
    let spread = (streaks.par_iter().map(|v| v * v).sum::<f32>() / streaks.len() as f32).sqrt();
    if spread > 0.0 {
        streaks.par_iter_mut().for_each(|v| *v /= spread);
    }
    streaks
}

/// How long a smudge is, in pixels: a floor and a step of Stroke Length.
pub(crate) const SMUDGE_STROKE: f32 = 8.0;

pub(crate) const SMUDGE_STROKE_PER_STEP: f32 = 2.0;

/// How big a smudged patch is, in pixels either side: a floor and a step of
/// Stroke Length. CS6's smudging is soft paint with crisp edges between
/// patches, not a streak — a long streak reads as motion blur.
pub(crate) const SMUDGE_PATCH: f32 = 1.0;

pub(crate) const SMUDGE_PATCH_PER_STEP: f32 = 0.35;

/// The fine streaks: how much of the picture's own detail, dragged along the
/// stroke, goes back on, and how long the drag is as a fraction of the stroke;
/// and a light grain along the strokes in the darks, in levels. This is what
/// CS6's smudging is made of close up — dense, thin diagonal streaks over
/// crisp patches. Without them the smear reads as a blur.
pub(crate) const SMUDGE_STREAK: f32 = 0.8;

pub(crate) const SMUDGE_STREAK_LENGTH: f32 = 0.5;

pub(crate) const SMUDGE_GRAIN: f32 = 7.0;

/// Which way the smudges run, in degrees anticlockwise from the horizontal.
pub(crate) const SMUDGE_ANGLE: f32 = 45.0;

/// How much the smudge favours the darker of the picture and its streak —
/// the stick drags dark into light, not light into dark — and how far the
/// smudging is confined to the dark: a power on darkness, so the lights are
/// barely touched.
pub(crate) const SMUDGE_DARK_DRAG: f32 = 0.8;

pub(crate) const SMUDGE_DARK_ONLY: f32 = 0.4;

/// How much of the smear goes over the patches at most. Short of all of it,
/// so a bridle or an eye still reads through the strokes.
pub(crate) const SMUDGE_MIX: f32 = 0.5;

/// How much the tones below the highlights are deepened, most in the
/// midtones and fading to nothing at black and at the highlights. The stick lays dark as well as
/// lifting light: CS6's shadows and midtones come back heavier than the
/// photograph's at any Intensity.
pub(crate) const SMUDGE_DEEPEN: f32 = 0.4;

/// Where the highlights start, as a fraction of white: at Highlight Area 0,
/// and how far down each step takes it. And how soft the start is.
pub(crate) const SMUDGE_HIGHLIGHT_FROM: f32 = 0.9;

pub(crate) const SMUDGE_HIGHLIGHT_PER_STEP: f32 = 0.02;

pub(crate) const SMUDGE_HIGHLIGHT_SOFT: f32 = 0.15;

/// At Intensity 10: how far the highlights are carried towards white, and how
/// much the contrast is raised.
pub(crate) const SMUDGE_LIFT: f32 = 0.9;

pub(crate) const SMUDGE_CONTRAST: f32 = 0.2;

/// Filter ▸ Artistic ▸ Smudge Stick: the picture's darks smeared along short
/// diagonal strokes, and its lights brightened towards white.
///
/// **Stroke Length** is how long the smudges are. **Highlight Area** is how
/// far down the tones the brightening reaches — at 20 most of a sunlit
/// picture burns out. **Intensity** is how hard the lights are brightened and
/// the contrast raised.
///
/// Alpha is left alone.
///
/// The streaks are mostly the picture's own detail, dragged; the grain along
/// them is kept light, because a heavy one reads as pencil hatching, which is
/// Rough Pastels' look, not this one.
///
/// No GPU path. The smudge is a streak along a line and the rest one pass per
/// pixel, and each would upload its input and read it straight back.
pub fn smudge_stick(pixmap: &mut Pixmap, length: u32, highlight: u32, intensity: u32) {
    if pixmap.is_empty() {
        return;
    }
    let length = length.clamp(*SMUDGE_LENGTH.start(), *SMUDGE_LENGTH.end()) as f32;
    let highlight = highlight.clamp(*SMUDGE_HIGHLIGHT.start(), *SMUDGE_HIGHLIGHT.end()) as f32;
    let intensity = intensity.clamp(*SMUDGE_INTENSITY.start(), *SMUDGE_INTENSITY.end()) as f32
        / *SMUDGE_INTENSITY.end() as f32;
    let width = pixmap.width() as usize;
    let stroke = SMUDGE_STROKE + length * SMUDGE_STROKE_PER_STEP;

    // Patches first — a median keeps their edges and loses the fine detail
    // inside them — then a short drag along the stroke.
    let patch = (SMUDGE_PATCH + length * SMUDGE_PATCH_PER_STEP).round() as u32;
    let patches = crate::photorust::convolve::median_of(pixmap, patch, patch);
    let mut streak = patches.clone();
    crate::photorust::convolve::motion_blur(&mut streak, SMUDGE_ANGLE, stroke);
    // The picture's detail — what the patches lost — dragged along the
    // stroke, held about mid-grey so it survives in bytes.
    let mut detail = pixmap.clone();
    for (d, p) in detail
        .as_bytes_mut()
        .chunks_exact_mut(4)
        .zip(patches.as_bytes().chunks_exact(4))
    {
        for c in 0..3 {
            d[c] = (128 + (d[c] as i32 - p[c] as i32) / 2).clamp(0, 255) as u8;
        }
    }
    crate::photorust::convolve::motion_blur(
        &mut detail,
        SMUDGE_ANGLE,
        stroke * SMUDGE_STREAK_LENGTH,
    );
    let along = {
        let radians = SMUDGE_ANGLE.to_radians();
        (radians.cos(), -radians.sin())
    };
    let grain = streaked_noise(
        width,
        pixmap.height() as usize,
        stroke * SMUDGE_STREAK_LENGTH + 3.0,
        13,
        along,
    );
    let mut darkest = darkest_along(&patches, SMUDGE_ANGLE, stroke);
    crate::photorust::convolve::motion_blur(&mut darkest, SMUDGE_ANGLE, stroke * 0.5);

    let from = (SMUDGE_HIGHLIGHT_FROM - highlight * SMUDGE_HIGHLIGHT_PER_STEP) * 255.0;
    let soft = SMUDGE_HIGHLIGHT_SOFT * 255.0;
    let contrast = 1.0 + intensity * SMUDGE_CONTRAST;
    let lift = intensity * SMUDGE_LIFT;
    let luma = |p: [f32; 3]| 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2];

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(streak.as_bytes().par_chunks_exact(stride))
        .zip(darkest.as_bytes().par_chunks_exact(stride))
        .zip(patches.as_bytes().par_chunks_exact(stride))
        .zip(detail.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, ((((out, streak), darkest), patches), detail))| {
            for x in 0..width {
                let i = x * 4;
                let picture = [out[i] as f32, out[i + 1] as f32, out[i + 2] as f32];
                let dragged = [
                    darkest[i] as f32,
                    darkest[i + 1] as f32,
                    darkest[i + 2] as f32,
                ];
                // The darker of the pixel and the darkest along its stroke, so
                // dark dragged into a light is smudged there too — the dark
                // streaks CS6 lays through spray next to a dark leg.
                let darkness = 1.0 - luma(picture).min(luma(dragged)) / 255.0;
                let own_darkness = 1.0 - luma(picture) / 255.0;
                let smudged = darkness.powf(SMUDGE_DARK_ONLY) * SMUDGE_MIX;
                let mut paint = [0.0f32; 3];
                for c in 0..3 {
                    let s = streak[i + c] as f32;
                    // The darkest along the stroke, so dark is carried along
                    // it as a mark rather than averaged away — and taken from
                    // the patches, not the picture, whose own darks are sharp
                    // and would come back as speckle.
                    let d = darkest[i + c] as f32;
                    let smear = s + (d.min(s) - s) * SMUDGE_DARK_DRAG;
                    // Over the patches, which keep the shapes readable under
                    // the smear, and not over the picture's own grain.
                    let under = picture[c]
                        + (patches[i + c] as f32 - picture[c])
                            * own_darkness.powf(SMUDGE_DARK_ONLY);
                    let streaks = (detail[i + c] as f32 - 128.0) * 2.0 * SMUDGE_STREAK
                        + grain[y * width + x] * SMUDGE_GRAIN * own_darkness;
                    let v = under + (smear - under) * smudged + streaks;
                    paint[c] = (v - 128.0) * contrast + 128.0;
                }
                let below = (1.0 - luma(paint) / from.max(1.0)).clamp(0.0, 1.0);
                // Heaviest in the midtones: the blacks are black already, and
                // taking them further only loses what is in them.
                let deepen = 1.0 - SMUDGE_DEEPEN * 4.0 * below * (1.0 - below);
                for v in paint.iter_mut() {
                    *v *= deepen;
                }
                let over = ((luma(paint) - from) / soft).clamp(0.0, 1.0);
                let lit = over * over * (3.0 - 2.0 * over) * lift;
                for c in 0..3 {
                    let v = paint[c] + (255.0 - paint[c]) * lit;
                    out[i + c] = v.clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: smudging the picture does not change the
                // layer's shape.
            }
        });
}
