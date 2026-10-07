//! Filter ▸ Brush Strokes.
//!
//! CS6 keeps this family in the Filter Gallery, as it does Artistic. The
//! Gallery is not built (docs/ROADMAP.md), so the filters live under a
//! Filter ▸ Brush Strokes submenu instead. Accented Edges, Angled Strokes,
//! Crosshatch, Dark Strokes, Ink Outlines and Spatter are built; the other two
//! are listed in the menu and disabled.

use crate::photorust::artistic::{blur_field, noise};
use crate::photorust::pixmap::Pixmap;
pub(crate) use crate::StrokeDirection;
use rayon::prelude::*;

mod ink_to_sumi_e;
pub(crate) use ink_to_sumi_e::*;

/// CS6's ranges for Accented Edges, which its three sliders run over.
pub const ACCENT_WIDTH: std::ops::RangeInclusive<u32> = 1..=14;

pub const ACCENT_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=50;

pub const ACCENT_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// Where Edge Brightness turns from dark accents to light ones. Below it the
/// edges are inked; above it they are chalked, harder the further off it.
pub(crate) const ACCENT_NEUTRAL: f32 = 25.0;

/// How far the picture is smoothed before its edges are found, in pixels per
/// step of Smoothness: what washes out the small detail, so only the broader
/// boundaries are accented.
pub(crate) const ACCENT_SMOOTH_PER_STEP: f32 = 0.45;

/// How much of that smoothing the picture itself keeps under the accents,
/// as a fraction of it.
pub(crate) const ACCENT_BASE_SMOOTH: f32 = 0.35;

/// Which edges are accented, in Sobel units of the smoothed brightness: an
/// edge below the first is left alone, one above the second is accented in
/// full, and between the two the accent rises smoothly. A threshold rather than
/// a gain is what makes CS6's accents crisp lines instead of a haze.
pub(crate) const ACCENT_FROM: f32 = 10.0;

pub(crate) const ACCENT_FULL: f32 = 45.0;

/// How much lower a wide accent sets that threshold, as a fraction per step of
/// Edge Width: a heavy brush picks up boundaries a fine one passes over.
pub(crate) const ACCENT_WIDE_REACH: f32 = 0.04;

/// How far an accent spreads per step of Edge Width, as a radius in pixels of
/// the strongest nearby edge, and how far that is softened.
pub(crate) const ACCENT_SPREAD_PER_STEP: f32 = 0.3;

pub(crate) const ACCENT_SOFT: f32 = 0.6;

/// How a chalk accent lights the picture at full strength: a gain on the
/// colour, which keeps its hue and pushes it towards a vivid, lighter version
/// of itself — the lime of a lit stem, the hot pink of a lit petal — and a
/// lift in levels on top.
pub(crate) const ACCENT_CHALK_GAIN: f32 = 1.6;

pub(crate) const ACCENT_CHALK_LIFT: f32 = 25.0;

pub(crate) const ACCENT_CHALK_SATURATION: f32 = 0.9;

/// Filter ▸ Brush Strokes ▸ Accented Edges: the picture's boundaries drawn
/// over it in light or dark.
///
/// The picture is smoothed by **Smoothness**, its edges are found on the
/// smoothed brightness and thresholded into crisp lines, and each line is
/// thickened to **Edge Width**. Then the lines are lit when **Edge
/// Brightness** is above the middle of its range — chalk, which keeps the
/// colour and makes it vivid — and inked black when it is below. Away from the
/// lines the picture is left as it was.
///
/// Alpha is left alone.
///
/// No GPU path. The edge field is floats, spread by a max filter, and each
/// stage would upload and read back (docs/gpu-migration.md).
pub fn accented_edges(pixmap: &mut Pixmap, width: u32, brightness: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let edge_width = width.clamp(*ACCENT_WIDTH.start(), *ACCENT_WIDTH.end()) as f32;
    let brightness = brightness.clamp(*ACCENT_BRIGHTNESS.start(), *ACCENT_BRIGHTNESS.end()) as f32;
    let smoothness = smoothness.clamp(*ACCENT_SMOOTHNESS.start(), *ACCENT_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let smooth = smoothness * ACCENT_SMOOTH_PER_STEP;
    let mut base = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut base, smooth * ACCENT_BASE_SMOOTH);

    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    blur_field(&mut tone, w, h, smooth);

    let mut edges = sobel(&tone, w, h);
    let lower = (1.0 - (edge_width - 1.0) * ACCENT_WIDE_REACH).max(0.3);
    let (from, full) = (ACCENT_FROM * lower, ACCENT_FULL * lower);
    edges.par_iter_mut().for_each(|e| {
        let t = ((*e - from) / (full - from)).clamp(0.0, 1.0);
        *e = t * t * (3.0 - 2.0 * t);
    });
    let spread = (edge_width * ACCENT_SPREAD_PER_STEP).round() as usize;
    widest_nearby(&mut edges, w, h, spread);
    blur_field(&mut edges, w, h, ACCENT_SOFT);

    // -1 is full ink, +1 full chalk.
    let tint = (brightness - ACCENT_NEUTRAL) / ACCENT_NEUTRAL;
    let edges = &edges;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(base.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, (out, base))| {
            for (x, (px, b)) in out
                .chunks_exact_mut(4)
                .zip(base.chunks_exact(4))
                .enumerate()
            {
                let accent = edges[y * w + x] * tint.abs();
                let grey = 0.299 * b[0] as f32 + 0.587 * b[1] as f32 + 0.114 * b[2] as f32;
                let vivid = if tint >= 0.0 {
                    1.0 + ACCENT_CHALK_SATURATION * accent
                } else {
                    1.0
                };
                for c in 0..3 {
                    let v = grey + (b[c] as f32 - grey) * vivid;
                    let v = if tint >= 0.0 {
                        v * (1.0 + ACCENT_CHALK_GAIN * accent) + ACCENT_CHALK_LIFT * accent
                    } else {
                        v * (1.0 - accent)
                    };
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: accenting the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Angled Strokes, which its three sliders run over.
pub const ANGLED_BALANCE: std::ops::RangeInclusive<u32> = 0..=100;

pub const ANGLED_LENGTH: std::ops::RangeInclusive<u32> = 3..=50;

pub const ANGLED_SHARPNESS: std::ops::RangeInclusive<u32> = 0..=10;

/// How far along its stroke a pixel takes its colour from, at most, as a
/// fraction of the stroke; and over how much of the stroke's length the
/// colour is then settled. It is settled by a median along the stroke rather
/// than an average: a median lays each stroke as a flat slab with a crisp end
/// and leaves a smooth sky smooth, where an average blurs both.
pub(crate) const ANGLED_DRAG: f32 = 0.15;

pub(crate) const ANGLED_SMEAR: f32 = 0.7;

/// How wide a stroke is, as a blur in pixels across the noise that drags it.
pub(crate) const ANGLED_WIDTH: f32 = 1.6;

/// The straight lines each stroke leaves, in levels of brightness, in the
/// light and in the dark, and how thick they are as a blur across them.
/// Faint in the light, where the paint is thin; in the dark they are heavy
/// and mostly lighter than the paint round them, as a pastel stick drags pale
/// streaks over a dark ground.
pub(crate) const ANGLED_BRISTLES_LIGHT: f32 = 0.0;

pub(crate) const ANGLED_BRISTLES_DARK: f32 = 7.0;

pub(crate) const ANGLED_BRISTLE_WIDTH: f32 = 0.5;

/// How wide the change-over from one direction to the other is, in
/// lightness, and how far the lightness it is judged on is smoothed, in
/// pixels, so the two directions meet in patches rather than in speckle.
pub(crate) const ANGLED_SWITCH: f32 = 0.08;

pub(crate) const ANGLED_JUDGE: f32 = 2.0;

/// Sharpness: an unsharp mask over the strokes, its scale in pixels, its
/// strength per step, the difference in levels below which it leaves a pixel
/// alone, and the most it moves one.
///
/// Taken on brightness and laid on all three channels alike. Per channel, it
/// amplifies the small colour differences between neighbouring strokes into
/// orange and yellow specks; and without the threshold it turns every ridge
/// into a sawtooth. What CS6's Sharpness crisps is where one stroke ends and
/// the next begins.
pub(crate) const ANGLED_SHARP_SCALE: f32 = 1.2;

pub(crate) const ANGLED_SHARP_PER_STEP: f32 = 0.15;

pub(crate) const ANGLED_SHARP_FROM: f32 = 6.0;

pub(crate) const ANGLED_SHARP_MOST: f32 = 40.0;

/// Filter ▸ Brush Strokes ▸ Angled Strokes: the picture repainted in diagonal
/// strokes, the light parts one way and the dark parts the other.
///
/// Each pixel's colour is dragged a little way along a diagonal and smeared
/// along it, with the fine straight lines of the bristles over it. The light
/// parts of the picture run down to the right, the dark parts up to the
/// right. **Direction Balance** moves the line between them: at 50 it is
/// mid-grey, towards 100 nearly everything runs up to the right, towards 0
/// down. **Stroke Length** is how long the strokes are, and
/// **Sharpness** how crisp.
///
/// Alpha is left alone.
///
/// No GPU path. The strokes are sampled along lines, which would fit, but the
/// two directions and the noise that drags them would each upload and read
/// back, and nothing downstream stays on the device.
pub fn angled_strokes(pixmap: &mut Pixmap, balance: u32, length: u32, sharpness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*ANGLED_BALANCE.start(), *ANGLED_BALANCE.end()) as f32 / 100.0;
    let length = length.clamp(*ANGLED_LENGTH.start(), *ANGLED_LENGTH.end()) as f32;
    let sharpness = sharpness.clamp(*ANGLED_SHARPNESS.start(), *ANGLED_SHARPNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let picture = pixmap.clone();
    let stroke = |angle: f32, salt: usize| lay_strokes(&picture, angle, length, salt);
    let rising = stroke(45.0, 21);
    let falling = stroke(-45.0, 22);
    let bristles = |angle: f32, salt: usize| bristle_lines(w, h, angle, length, salt);
    let (rising_lines, falling_lines) = (bristles(45.0, 23), bristles(-45.0, 24));
    let (rising_lines, falling_lines) = (&rising_lines, &falling_lines);

    let mut judged: Vec<f32> = picture
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    blur_field(&mut judged, w, h, ANGLED_JUDGE);
    let split = balance;
    let judged = &judged;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(rising.as_bytes().par_chunks_exact(stride))
        .zip(falling.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, ((out, up), down))| {
            for (x, ((px, u), d)) in out
                .chunks_exact_mut(4)
                .zip(up.chunks_exact(4))
                .zip(down.chunks_exact(4))
                .enumerate()
            {
                let t = ((judged[y * w + x] - split) / ANGLED_SWITCH * 0.5 + 0.5).clamp(0.0, 1.0);
                // Light falls to the right, dark rises.
                let fall = t * t * (3.0 - 2.0 * t);
                let i = y * w + x;
                let n = rising_lines[i] * (1.0 - fall) + falling_lines[i] * fall;
                let dark = 1.0 - judged[i];
                // In the dark the streaks lean pale: chalk over a dark ground.
                let n = n + dark * n.abs() * 0.5;
                let line = n
                    * (ANGLED_BRISTLES_LIGHT
                        + (ANGLED_BRISTLES_DARK - ANGLED_BRISTLES_LIGHT) * dark * dark);
                for c in 0..3 {
                    let v = d[c] as f32 * fall + u[c] as f32 * (1.0 - fall) + line;
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: restroking the picture does not change the
                // layer's shape.
            }
        });

    let gain = sharpness * ANGLED_SHARP_PER_STEP;
    if gain > 0.0 {
        let sharp = pixmap.clone();
        let mut soft = pixmap.clone();
        crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, ANGLED_SHARP_SCALE);
        pixmap
            .as_bytes_mut()
            .par_chunks_exact_mut(4)
            .zip(sharp.as_bytes().par_chunks_exact(4))
            .zip(soft.as_bytes().par_chunks_exact(4))
            .for_each(|((px, s), b)| {
                let luma =
                    |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
                let diff = luma(s) - luma(b);
                let over = (diff.abs() - ANGLED_SHARP_FROM).max(0.0) * diff.signum();
                let lift = (over * gain).clamp(-ANGLED_SHARP_MOST, ANGLED_SHARP_MOST);
                for c in 0..3 {
                    px[c] = (s[c] as f32 + lift).round().clamp(0.0, 255.0) as u8;
                }
            });
    }
}

/// The picture laid in strokes along `angle`, `length` long: each pixel's
/// colour dragged a little way along the stroke and then settled by a median
/// along it. Laid by where on the canvas a pixel is; `salt` keeps two sets of
/// strokes apart.
pub(crate) fn lay_strokes(picture: &Pixmap, angle: f32, length: f32, salt: usize) -> Pixmap {
    let (w, h) = (picture.width() as usize, picture.height() as usize);
    let radians = angle.to_radians();
    let along = (radians.cos(), -radians.sin());
    let mut drag = crate::photorust::artistic::streaked_noise(w, h, length + 3.0, salt, along);
    // Neighbours share a drag, so a stroke is a few pixels wide rather than a
    // hairline.
    blur_field(&mut drag, w, h, ANGLED_WIDTH);
    unit_spread(&mut drag);
    let mut dragged = picture.clone();
    dragged
        .as_bytes_mut()
        .par_chunks_exact_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let offset = drag[y * w + x].clamp(-2.0, 2.0) * length * ANGLED_DRAG * 0.5;
                let sx = (x as f32 + along.0 * offset)
                    .round()
                    .clamp(0.0, (w - 1) as f32) as usize;
                let sy = (y as f32 + along.1 * offset)
                    .round()
                    .clamp(0.0, (h - 1) as f32) as usize;
                let from = (sy * w + sx) * 4;
                px[..3].copy_from_slice(&picture.as_bytes()[from..from + 3]);
            }
        });
    median_along(&dragged, along, (length * ANGLED_SMEAR).max(1.0))
}

/// Fine straight lines along `angle`, about a stroke and a half long, a
/// little over a pixel thick, with a spread of one.
pub(crate) fn bristle_lines(w: usize, h: usize, angle: f32, length: f32, salt: usize) -> Vec<f32> {
    bristle_lines_width(w, h, angle, length, salt, ANGLED_BRISTLE_WIDTH)
}

pub(crate) fn bristle_lines_width(
    w: usize,
    h: usize,
    angle: f32,
    length: f32,
    salt: usize,
    width: f32,
) -> Vec<f32> {
    let radians = angle.to_radians();
    let mut lines = crate::photorust::artistic::streaked_noise(
        w,
        h,
        length * 1.5,
        salt,
        (radians.cos(), -radians.sin()),
    );
    if width > 0.0 {
        blur_field(&mut lines, w, h, width);
    }
    unit_spread(&mut lines);
    lines
}

pub(crate) fn unit_spread(field: &mut [f32]) {
    let spread = (field.par_iter().map(|v| v * v).sum::<f32>() / field.len().max(1) as f32).sqrt();
    if spread > 0.0 {
        field.par_iter_mut().for_each(|v| *v /= spread);
    }
}

/// CS6's ranges for Crosshatch, which its three sliders run over.
pub const HATCH_LENGTH: std::ops::RangeInclusive<u32> = 3..=50;

pub const HATCH_SHARPNESS: std::ops::RangeInclusive<u32> = 0..=20;

pub const HATCH_STRENGTH: std::ops::RangeInclusive<u32> = 1..=3;

/// How strong the hatching lines are, in levels, where the picture is busy.
///
/// **The hatching follows texture, not tone.** CS6 lays it plainly over
/// spray and broken water, faintly over the dark horse, and not at all over a
/// clear sky: pencil catches where there is something to catch. So the lines'
/// strength is set by how much detail is round a pixel — [`HATCH_DETAIL`] —
/// and muted in the dark, where a line has little to show against.
pub(crate) const HATCH_LINES: f32 = 45.0;

/// How much detail, in levels of difference from a small blur averaged over a
/// few pixels, counts as fully busy; the scale of that blur and of the
/// average; and how much line a completely smooth area still gets.
pub(crate) const HATCH_DETAIL: f32 = 10.0;

pub(crate) const HATCH_DETAIL_SCALE: f32 = 1.5;

pub(crate) const HATCH_DETAIL_REACH: f32 = 4.0;

pub(crate) const HATCH_DETAIL_FLOOR: f32 = 0.05;

/// How much of the lines' strength the darkest tone keeps.
pub(crate) const HATCH_DARK_MUTE: f32 = 0.35;

/// What share of the diagonal segments carry a line, and how much longer than
/// Stroke Length a line is.
pub(crate) const HATCH_DENSITY: f32 = 0.22;

pub(crate) const HATCH_LINE_LENGTH: f32 = 1.6;

/// How many pixels apart the three channels' lines are at the top of
/// Sharpness.
pub(crate) const HATCH_FRINGE: f32 = 1.0;

/// How much stronger the lines are on each pass after the first.
pub(crate) const HATCH_PER_PASS: f32 = 0.3;

/// How much heavier the lines are per step of Sharpness.
pub(crate) const HATCH_LINES_PER_SHARP: f32 = 0.05;

/// Sharpness: a per-channel unsharp mask, its scale in pixels and strength
/// per step. Per channel on purpose: CS6's high settings, repeated by
/// Strength, amplify the colour differences between lines into the rainbow
/// hatching its screenshots show.
pub(crate) const HATCH_SHARP_SCALE: f32 = 1.0;

pub(crate) const HATCH_SHARP_PER_STEP: f32 = 0.08;

/// The difference in levels below which Sharpness leaves a channel alone, so
/// the faint ridges of a smooth sky are not sharpened into hatching.
pub(crate) const HATCH_SHARP_FROM: f32 = 5.0;

/// Filter ▸ Brush Strokes ▸ Crosshatch: the picture drawn over in strokes
/// running both ways at once.
///
/// Each pass lays the picture in strokes along both diagonals — the same
/// strokes as Angled Strokes — takes the mean of the two, draws fine lines
/// both ways over it, and sharpens it. **Stroke Length** is how long the
/// strokes are, **Sharpness** how hard they are sharpened, and **Strength**
/// how many passes: each one works over what the last one left, so at 3 with
/// a high Sharpness the picture is lost under the hatching.
///
/// Alpha is left alone.
///
/// No GPU path, for Angled Strokes' reasons.
pub fn crosshatch(pixmap: &mut Pixmap, length: u32, sharpness: u32, strength: u32) {
    if pixmap.is_empty() {
        return;
    }
    let length = length.clamp(*HATCH_LENGTH.start(), *HATCH_LENGTH.end()) as f32;
    let sharpness = sharpness.clamp(*HATCH_SHARPNESS.start(), *HATCH_SHARPNESS.end()) as f32;
    let strength = strength.clamp(*HATCH_STRENGTH.start(), *HATCH_STRENGTH.end());
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    for pass in 0..strength as usize {
        let salt = 40 + pass * 4;
        let rising = lay_strokes(pixmap, 45.0, length, salt);
        let falling = lay_strokes(pixmap, -45.0, length, salt + 1);
        // Hairlines, unsoftened, and cut hard: pen, not charcoal.
        let up = pen_lines(w, h, true, length, salt + 2);
        let down = pen_lines(w, h, false, length, salt + 3);
        // At high Sharpness each channel reads its lines a pixel or so off
        // the others, and the lines come apart into CS6's colour fringes.
        let fringe = (sharpness / *HATCH_SHARPNESS.end() as f32 * HATCH_FRINGE).round() as isize;
        let detail = busyness(pixmap);
        let pass_gain = 1.0 + pass as f32 * HATCH_PER_PASS;
        let (up, down, detail) = (&up, &down, &detail);
        let stride = pixmap.stride();
        pixmap
            .as_bytes_mut()
            .par_chunks_exact_mut(stride)
            .zip(rising.as_bytes().par_chunks_exact(stride))
            .zip(falling.as_bytes().par_chunks_exact(stride))
            .enumerate()
            .for_each(|(y, ((out, r), f))| {
                for (x, ((px, r), f)) in out
                    .chunks_exact_mut(4)
                    .zip(r.chunks_exact(4))
                    .zip(f.chunks_exact(4))
                    .enumerate()
                {
                    let mut mean = [0.0f32; 3];
                    for c in 0..3 {
                        mean[c] = (r[c] as f32 + f[c] as f32) * 0.5;
                    }
                    let lightness = (0.299 * mean[0] + 0.587 * mean[1] + 0.114 * mean[2]) / 255.0;
                    let i = y * w + x;
                    let busy = HATCH_DETAIL_FLOOR + (1.0 - HATCH_DETAIL_FLOOR) * detail[i];
                    let amp = HATCH_LINES
                        * busy
                        * (HATCH_DARK_MUTE + (1.0 - HATCH_DARK_MUTE) * lightness)
                        * (1.0 + sharpness * HATCH_LINES_PER_SHARP)
                        * pass_gain;
                    // Each line has a pale side and a dark side. In the light
                    // the dark side shows, as pencil on paper; in the dark,
                    // the pale side, as chalk.
                    for c in 0..3 {
                        // Shifted across the lines, which run diagonally, so
                        // a shift along x moves them.
                        let shift = (c as isize - 1) * fringe;
                        let xs = (x as isize + shift).clamp(0, w as isize - 1) as usize;
                        let j = y * w + xs;
                        let pale = up[j].max(0.0) + down[j].max(0.0);
                        let shade = -(up[j].min(0.0) + down[j].min(0.0));
                        let line = (pale * (1.0 - lightness) - shade * lightness) * amp;
                        px[c] = (mean[c] + line).round().clamp(0.0, 255.0) as u8;
                    }
                    // Alpha stands: hatching the picture does not change the
                    // layer's shape.
                }
            });

        let gain = sharpness * HATCH_SHARP_PER_STEP;
        // The threshold falls away as Sharpness rises: at the top of the
        // slider every difference is sharpened, and the picture goes hard.
        let threshold =
            (HATCH_SHARP_FROM * (1.0 - sharpness / *HATCH_SHARPNESS.end() as f32)).max(2.0);
        if gain > 0.0 {
            let sharp = pixmap.clone();
            let mut soft = pixmap.clone();
            crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, HATCH_SHARP_SCALE);
            pixmap
                .as_bytes_mut()
                .par_chunks_exact_mut(4)
                .zip(sharp.as_bytes().par_chunks_exact(4))
                .zip(soft.as_bytes().par_chunks_exact(4))
                .for_each(|((px, s), b)| {
                    for c in 0..3 {
                        let diff = s[c] as f32 - b[c] as f32;
                        let over = (diff.abs() - threshold).max(0.0) * diff.signum();
                        let v = s[c] as f32 + over * gain;
                        px[c] = v.round().clamp(0.0, 255.0) as u8;
                    }
                });
        }
    }
}

/// Pen lines: one pixel wide, straight, along the rising (`/`) or falling
/// (`\`) diagonal, each a segment of a diagonal chosen at random, so they
/// stand apart with gaps between — rather than a streak on every diagonal,
/// which reads as a screen. Positive values are one side of the pen, negative
/// the other; 0 is no line.
pub(crate) fn pen_lines(w: usize, h: usize, rising: bool, length: f32, salt: usize) -> Vec<f32> {
    let span = (length * HATCH_LINE_LENGTH).max(3.0);
    let hash = |a: i64, b: i64| {
        let mut v = (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (b as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ (salt as u64).wrapping_mul(0x1656_67B1_9E37_79F9)
            ^ (crate::photorust::seed() as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
        v ^= v >> 31;
        v = v.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        v ^= v >> 29;
        (v >> 11) as f32 / (1u64 << 53) as f32
    };
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, slot) in row.iter_mut().enumerate() {
                let (x, y) = (x as i64, y as i64);
                let (diagonal, along) = if rising {
                    (x + y, x - y)
                } else {
                    (x - y, x + y)
                };
                // Each diagonal is cut at its own offset, so the ends do not line
                // up across the picture.
                let offset = hash(diagonal, -1) * span;
                let segment = ((along as f32 + offset) / span).floor() as i64;
                let pick = hash(diagonal, segment);
                if pick < HATCH_DENSITY {
                    let side = if hash(diagonal, segment + 7919) < 0.5 {
                        -1.0
                    } else {
                        1.0
                    };
                    *slot = side * (0.6 + 0.4 * pick / HATCH_DENSITY);
                }
            }
        });
    out
}

/// How busy the picture is round each pixel, 0 smooth to 1 fully busy: its
/// difference from a small blur, averaged over a few pixels.
pub(crate) fn busyness(picture: &Pixmap) -> Vec<f32> {
    let (w, h) = (picture.width() as usize, picture.height() as usize);
    let mut soft = picture.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut soft, HATCH_DETAIL_SCALE);
    let luma = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    let mut detail: Vec<f32> = picture
        .as_bytes()
        .par_chunks_exact(4)
        .zip(soft.as_bytes().par_chunks_exact(4))
        .map(|(p, s)| (luma(p) - luma(s)).abs())
        .collect();
    blur_field(&mut detail, w, h, HATCH_DETAIL_REACH);
    detail
        .par_iter_mut()
        .for_each(|d| *d = (*d / HATCH_DETAIL).min(1.0));
    detail
}

/// CS6's ranges for Dark Strokes, which its three sliders run over.
pub const DARK_BALANCE: std::ops::RangeInclusive<u32> = 0..=10;

pub const DARK_BLACK: std::ops::RangeInclusive<u32> = 0..=10;

pub const DARK_WHITE: std::ops::RangeInclusive<u32> = 0..=10;

/// The strokes: long ones in the light, short ones in the dark, in pixels.
pub(crate) const DARK_LONG: f32 = 14.0;

pub(crate) const DARK_SHORT: f32 = 5.0;

/// Where the picture goes to black, as a fraction of white: a floor, and how
/// far each step of Black Intensity and of Balance raises it; and how soft
/// the fall is. At the top of both, everything short of the sky and the spray
/// is black, as CS6's is.
pub(crate) const DARK_BLACK_FROM: f32 = 0.12;

pub(crate) const DARK_BLACK_PER_STEP: f32 = 0.037;

pub(crate) const DARK_BALANCE_PER_STEP: f32 = 0.028;

pub(crate) const DARK_BLACK_SOFT: f32 = 0.08;

/// Where the lights start going to white, as a fraction of white at White
/// Intensity 0, how far down each step takes it, and how far towards white
/// the lightest tone is carried at 10.
pub(crate) const DARK_WHITE_FROM: f32 = 0.85;

pub(crate) const DARK_WHITE_PER_STEP: f32 = 0.03;

pub(crate) const DARK_WHITE_LIFT: f32 = 0.85;

/// How much each step of Black Intensity deepens the midtones, as a power.
pub(crate) const DARK_DEEPEN_PER_STEP: f32 = 0.4;

/// How much the tones between are pushed apart: a contrast gain at full
/// Black and White Intensity together.
pub(crate) const DARK_CONTRAST: f32 = 0.35;

/// Filter ▸ Brush Strokes ▸ Dark Strokes: the dark parts of the picture
/// painted in short strokes and driven to black, the light parts in long
/// strokes the other way and driven to white.
///
/// The strokes are Angled Strokes': the dark run down to the right, short;
/// the light run up to the right, long. Then **Black Intensity** and
/// **Balance** decide how much of the picture goes to black — the line rises
/// with both — and **White Intensity** how far the lights are carried to
/// white. Because the black is decided on the stroked picture, its edge is
/// cut into strokes, which is the ragged diagonal edge CS6 shows.
///
/// Alpha is left alone.
///
/// No GPU path, for Angled Strokes' reasons.
pub fn dark_strokes(pixmap: &mut Pixmap, balance: u32, black: u32, white: u32) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*DARK_BALANCE.start(), *DARK_BALANCE.end()) as f32;
    let black = black.clamp(*DARK_BLACK.start(), *DARK_BLACK.end()) as f32;
    let white = white.clamp(*DARK_WHITE.start(), *DARK_WHITE.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let light_strokes = lay_strokes(pixmap, 45.0, DARK_LONG, 61);
    let dark_strokes = lay_strokes(pixmap, -45.0, DARK_SHORT, 62);

    let luma = |p: &[u8]| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
    let mut judged: Vec<f32> = pixmap.as_bytes().par_chunks_exact(4).map(luma).collect();
    blur_field(&mut judged, w, h, ANGLED_JUDGE);
    let judged = &judged;

    let black_from =
        DARK_BLACK_FROM + black * DARK_BLACK_PER_STEP + balance * DARK_BALANCE_PER_STEP;
    let white_from = DARK_WHITE_FROM - white * DARK_WHITE_PER_STEP;
    let lift = white / 10.0 * DARK_WHITE_LIFT;
    let contrast = 1.0 + (black + white) / 20.0 * DARK_CONTRAST;
    let pivot = (black_from + white_from) * 0.5 * 255.0;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(light_strokes.as_bytes().par_chunks_exact(stride))
        .zip(dark_strokes.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, ((out, l), d))| {
            for (x, ((px, l), d)) in out
                .chunks_exact_mut(4)
                .zip(l.chunks_exact(4))
                .zip(d.chunks_exact(4))
                .enumerate()
            {
                let t = ((judged[y * w + x] - 0.5) / ANGLED_SWITCH * 0.5 + 0.5).clamp(0.0, 1.0);
                let lit = t * t * (3.0 - 2.0 * t);
                let mut paint = [0.0f32; 3];
                for c in 0..3 {
                    let v = l[c] as f32 * lit + d[c] as f32 * (1.0 - lit);
                    paint[c] = ((v - pivot) * contrast + pivot).clamp(0.0, 255.0);
                }
                let level = (0.299 * paint[0] + 0.587 * paint[1] + 0.114 * paint[2]) / 255.0;
                // The midtones deepened on the way down: a power on the
                // tone, fading out towards the lights.
                let below = (1.0 - level / white_from.max(0.01)).clamp(0.0, 1.0);
                let deepen = level.max(0.01).powf(black * DARK_DEEPEN_PER_STEP * below);
                for v in paint.iter_mut() {
                    *v *= deepen;
                }
                let level = level * deepen;
                let k = ((level - black_from) / DARK_BLACK_SOFT * 0.5 + 0.5).clamp(0.0, 1.0);
                let kept = k * k * (3.0 - 2.0 * k);
                let u = ((level - white_from) / (1.0 - white_from).max(0.01)).clamp(0.0, 1.0);
                let whiten = u * lift;
                for c in 0..3 {
                    let v = paint[c] * kept;
                    px[c] = (v + (255.0 - v) * whiten).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: restroking the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Ink Outlines, which its three sliders run over.
pub const INK_LENGTH: std::ops::RangeInclusive<u32> = 0..=50;

pub const INK_DARK: std::ops::RangeInclusive<u32> = 0..=50;

pub const INK_LIGHT: std::ops::RangeInclusive<u32> = 0..=50;

/// How long a stroke is, in pixels: a floor and a step of Stroke Length.
pub(crate) const INK_LINE: f32 = 4.0;

pub(crate) const INK_LINE_PER_STEP: f32 = 0.5;

/// The scale in pixels at which the picture's detail is read as something to
/// draw, and how wide a reach decides whether a place is in shadow.
pub(crate) const INK_DETAIL_SCALE: f32 = 1.3;

pub(crate) const INK_BROAD_SCALE: f32 = 10.0;

/// Which way the strokes run, in degrees anticlockwise from the horizontal.
/// Light and dark run the opposite way to each other — see [`ink_outlines`].
pub(crate) const INK_LIGHT_ANGLE: f32 = 45.0;

pub(crate) const INK_DARK_ANGLE: f32 = -45.0;

/// Which marks are drawn at all, in levels of detail: below the first nothing
/// is drawn, above the second the mark is drawn in full.
pub(crate) const INK_MARK_FROM: f32 = 1.0;

pub(crate) const INK_MARK_FULL: f32 = 5.0;

/// How much of a full mark a step of Dark Intensity inks, and a step of Light
/// Intensity draws in white.
pub(crate) const INK_PER_STEP: f32 = 0.04;

pub(crate) const INK_CHALK_PER_STEP: f32 = 0.02;

/// The outline: how far the tone is softened before its edges are found — a
/// pixel or less, so the line comes out thin — how strong an edge must be, in
/// Sobel units, before a line is drawn round it and where it is drawn in
/// full, and how black it is drawn at Dark Intensity 0 and per step.
pub(crate) const INK_OUTLINE_SOFT: f32 = 0.8;

pub(crate) const INK_OUTLINE_FROM: f32 = 6.0;

pub(crate) const INK_OUTLINE_FULL: f32 = 26.0;

pub(crate) const INK_OUTLINE_FLOOR: f32 = 0.55;

pub(crate) const INK_OUTLINE_PER_STEP: f32 = 0.009;

/// The rim: how white it is at Light Intensity 0 and per step, and how wide
/// the ring outside a filled shape is, in pixels.
pub(crate) const INK_RIM_FLOOR: f32 = 0.6;

pub(crate) const INK_RIM_PER_STEP: f32 = 0.011;

pub(crate) const INK_RIM_WIDTH: usize = 2;

pub(crate) const INK_RIM_CLOSE: usize = 26;

/// How wide a reach decides whether a place is within a dark mass. Wider
/// than the shadow's own: a gloss can be twenty pixels across and still be a
/// gloss on something black.
pub(crate) const INK_WITHIN_SCALE: f32 = 26.0;

/// How far the paint is taken down inside a dark mass, so that what is lit
/// within it reads as a mark on black rather than as a hole in it.
pub(crate) const INK_INSIDE_DAMP: f32 = 0.95;

/// How much darker again the broad reach must be for a place to count as a
/// dark mass rather than a dark line.
pub(crate) const INK_BROAD_ALLOW: f32 = 0.06;

/// How dark a place must be, as a fraction of white, before it fills solid:
/// at Dark Intensity 0 and per step. The shadow is judged over a wide reach,
/// so a lit flank inside a dark body fills with the body.
pub(crate) const INK_BLACK_FROM: f32 = 0.08;

pub(crate) const INK_BLACK_PER_STEP: f32 = 0.008;

pub(crate) const INK_EDGE_SOFT: f32 = 0.10;

/// How far the two Intensity sliders together drive the picture's contrast at
/// the top of both, and the tone it turns about.
pub(crate) const INK_CONTRAST: f32 = 1.1;

pub(crate) const INK_PIVOT: f32 = 140.0;

#[cfg(test)]
mod tests;
