//! The surface textures CS6 lays under several of its filters.
//!
//! Rough Pastels, Underpainting, Conté Crayon, Texturizer and Glass all carry
//! the same block of controls — Texture, Scaling, Relief, Light and Invert —
//! and all do the same thing with it: read a grey texture as a height map and
//! light the picture as if it were printed on that surface. This is that block,
//! once, for all of them. Rough Pastels is the first to use it.
//!
//! CS6 ships its four textures as small bitmaps. These are procedural height
//! maps drawn to match what those look like at 100%, so there is nothing to
//! load and they tile at any size. CS6's "Load Texture…", which reads a
//! Photoshop file as the texture, is not built.
//!
//! The Filter ▸ Texture family lives here too, starting with Craquelure: it
//! lights the picture as a surface in the same way, but draws that surface
//! itself rather than taking one of the four.

use crate::photorust::pixmap::Pixmap;
pub(crate) use crate::{GrainType, LightDirection as Light, TextureSurface as Texture};
use rayon::prelude::*;

mod grain_to_stained_glass;
pub(crate) use grain_to_stained_glass::*;

/// CS6's ranges for the shared texture controls.
pub const SCALING: std::ops::RangeInclusive<u32> = 50..=200;

pub const RELIEF: std::ops::RangeInclusive<u32> = 0..=50;

impl Light {
    /// Which way the light points from, as a unit step in image coordinates
    /// (y down).
    pub fn towards(self) -> (f32, f32) {
        let d = std::f32::consts::FRAC_1_SQRT_2;
        match self {
            Light::Bottom => (0.0, 1.0),
            Light::BottomLeft => (-d, d),
            Light::Left => (-1.0, 0.0),
            Light::TopLeft => (-d, -d),
            Light::Top => (0.0, -1.0),
            Light::TopRight => (d, -d),
            Light::Right => (1.0, 0.0),
            Light::BottomRight => (d, d),
        }
    }
}

/// The texture as a height field, 0 low and 1 high, one value per pixel.
/// `scaling` is CS6's percentage: at 200 every feature is twice the size.
///
/// Laid by where on the canvas a pixel is, so it lines up across the whole
/// image — and so a crop of the image gets a different part of it.
pub fn height_map(texture: Texture, width: u32, height: u32, scaling: u32) -> Vec<f32> {
    let zoom = 100.0 / scaling.clamp(*SCALING.start(), *SCALING.end()) as f32;
    let width = width as usize;
    let mut field = vec![0.0f32; width * height as usize];
    field
        .par_chunks_exact_mut(width.max(1))
        .enumerate()
        .for_each(|(y, row)| {
            for (x, h) in row.iter_mut().enumerate() {
                let (u, v) = (x as f32 * zoom, y as f32 * zoom);
                *h = match texture {
                    Texture::Brick => brick(u, v),
                    Texture::Burlap => burlap(u, v),
                    Texture::Canvas => canvas(u, v),
                    Texture::Sandstone => sandstone(u, v),
                };
            }
        });
    field
}

/// How much a unit of slope lights or shades the picture at the top of Relief:
/// some of it in proportion to the colour, most of it as plain light and
/// shadow, so the texture shows as clearly in a black as in a white.
pub(crate) const SHADE_BY_COLOUR: f32 = 0.8;

pub(crate) const SHADE_BY_LIGHT: f32 = 150.0;

/// How far the height field is softened before it is lit, in pixels, so a
/// hard-edged texture casts a bevel rather than a one-pixel line.
pub(crate) const BEVEL: f32 = 0.6;

/// Light the picture as if it were printed on `texture`.
///
/// **Scaling** is the size of the texture. **Relief** is how deep it is.
/// **Light** is where the light comes from, and **Invert** turns the surface
/// inside out, so what stood up is sunk.
///
/// Alpha is left alone.
///
/// No GPU path: the height field is drawn and lit in one pass each, and the
/// result is wanted straight back on the CPU by the filter that asked.
pub fn apply_relief(
    pixmap: &mut Pixmap,
    texture: Texture,
    scaling: u32,
    relief: u32,
    light: Light,
    invert: bool,
) {
    apply_relief_weighted(
        pixmap,
        texture,
        scaling,
        relief,
        light,
        invert,
        Finish::default(),
    );
}

/// How a surface is lit, beyond the controls CS6 shows.
#[derive(Clone, Copy, Debug)]
pub struct Finish {
    /// How much less the surface shows in the light than in the dark: 0 lights
    /// every tone alike; at 1 white is untouched and black takes the full
    /// relief.
    pub dark_bias: f32,
    /// How far the height field is softened before it is lit, in pixels.
    pub bevel: f32,
    /// A power on the slope below 1 lifts faint slopes towards the strong
    /// ones, so the surface reads as hard glints and cracks rather than as a
    /// soft emboss.
    pub crisp: f32,
    /// How many levels the low parts of the surface are darkened by, as the
    /// shadow in a groove is, whichever way the light falls. It is what makes
    /// a joint or a valley show under a light that runs along it.
    pub occlusion: f32,
    /// How much of the surface fades out in broad, random patches: 0 is
    /// even everywhere, 1 lets some patches go smooth. See [`patchiness`].
    pub patchy: f32,
    /// How many times deeper the surface is than Relief alone makes it.
    pub gain: f32,
}

impl Default for Finish {
    fn default() -> Finish {
        Finish {
            dark_bias: 0.0,
            bevel: BEVEL,
            crisp: 1.0,
            occlusion: 0.0,
            patchy: 0.0,
            gain: 1.0,
        }
    }
}

/// How broad the patches [`patchiness`] fades the surface in are, in pixels.
pub(crate) const PATCH_SCALE: f32 = 36.0;

/// How strongly the surface shows at a pixel, 1 fully and less in broad
/// random patches: `patchy` 0 gives 1 everywhere. CS6's surface is not laid
/// evenly — here and there it thins to nothing, and the picture shows through
/// soft — and a surface laid the same everywhere reads as a machine print.
pub fn patchiness(x: usize, y: usize, patchy: f32) -> f32 {
    if patchy <= 0.0 {
        return 1.0;
    }
    let n = 0.6 * value_noise(x as f32, y as f32, PATCH_SCALE, 51)
        + 0.4 * value_noise(x as f32, y as f32, PATCH_SCALE * 0.45, 52);
    // Most of the canvas keeps its surface; the lowest third fades out.
    let t = ((n - 0.3) / 0.25).clamp(0.0, 1.0);
    1.0 - patchy * (1.0 - t * t * (3.0 - 2.0 * t))
}

/// [`apply_relief`], with the [`Finish`] set by the filter. Underpainting's
/// paint is thin over a pale sky and thick where the picture is dark, and its
/// surface is glassy rather than soft, and CS6 shows it that way.
pub fn apply_relief_weighted(
    pixmap: &mut Pixmap,
    texture: Texture,
    scaling: u32,
    relief: u32,
    light: Light,
    invert: bool,
    finish: Finish,
) {
    let relief = relief.clamp(*RELIEF.start(), *RELIEF.end()) as f32 / *RELIEF.end() as f32;
    if relief <= 0.0 || pixmap.is_empty() {
        return;
    }
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let mut field = height_map(texture, pixmap.width(), pixmap.height(), scaling);
    if invert {
        field.par_iter_mut().for_each(|h| *h = 1.0 - *h);
    }
    crate::photorust::artistic::blur_field(&mut field, width, height, finish.bevel);

    let (lx, ly) = light.towards();
    let field = &field;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, px) in out.chunks_exact_mut(4).enumerate() {
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let dx = (field[y * width + right] - field[y * width + left])
                    / (right - left).max(1) as f32;
                let dy =
                    (field[down * width + x] - field[up * width + x]) / (down - up).max(1) as f32;
                // A slope that falls away from the light is lit.
                let lightness =
                    (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0;
                let weight = 1.0 - finish.dark_bias * lightness;
                let slope = -(dx * lx + dy * ly);
                let slope = slope.signum() * slope.abs().powf(finish.crisp);
                let patch = patchiness(x, y, finish.patchy);
                let shade = slope * relief * weight * patch * finish.gain;
                let groove = (0.5 - field[y * width + x]).max(0.0)
                    * 2.0
                    * finish.occlusion
                    * relief
                    * weight
                    * patch;
                for c in 0..3 {
                    let v = px[c] as f32 * (1.0 + shade * SHADE_BY_COLOUR) + shade * SHADE_BY_LIGHT
                        - groove;
                    px[c] = v.clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}

/// Brick: courses 9 pixels high of long bricks, every other course offset by
/// half a brick, with a rough, pitted face.
///
/// What CS6's brick is made of is mostly *face*, not mortar. Lit from below,
/// the deep horizontal joints between courses are what shows — a ruled page of
/// lines. Lit from the side, those joints catch no light at all, the shallow
/// vertical ones barely any, and what is left is the grit of the brick itself:
/// a dense scatter of short vertical ticks, because the pits are taller than
/// they are wide. A regular grid of dashes is what a smooth-faced brick with
/// deep joints gives instead, and it reads as a pattern rather than a surface.
pub(crate) fn brick(u: f32, v: f32) -> f32 {
    const COURSE: f32 = 9.0;
    const LENGTH: f32 = 40.0;
    const MORTAR: f32 = 1.4;
    const JOINT_DEPTH: f32 = 0.35;
    const GRIT: f32 = 0.9;
    let row = (v / COURSE).floor();
    let offset = if row.rem_euclid(2.0) > 0.5 {
        LENGTH / 2.0
    } else {
        0.0
    };
    let fy = v - row * COURSE;
    let fx = (u + offset).rem_euclid(LENGTH);
    let bed = ((fy.min(COURSE - fy) - MORTAR * 0.5) / MORTAR).clamp(0.0, 1.0);
    let head = ((fx.min(LENGTH - fx) - MORTAR * 0.5) / MORTAR).clamp(0.0, 1.0);
    let joints = bed * (1.0 - JOINT_DEPTH * (1.0 - head));
    // Pits twice as tall as they are wide, at two sizes.
    let grit = 0.6 * value_noise(u * 2.0, v, 3.0, 3) + 0.4 * value_noise(u * 2.0, v, 1.5, 5);
    joints * (1.0 - GRIT + GRIT * grit)
}

/// Burlap: coarse sacking, read as CS6 draws it — threads running across,
/// about 7 pixels to a thread, with a groove between each and the next.
///
/// CS6's burlap is not a round-threaded basket weave. Lit from the top it is
/// dark grooves on a ground that keeps the picture's own tone, and the
/// grooves are not straight: each runs a few pixels, then steps up or down
/// where a thread crossing underneath lifts it, and some runs are shallow
/// enough to break the line into dashes. The threads running down show only
/// as those kinks and a faint dip at each crossing. A soft sine weave both
/// ways — the obvious model — reads as knitting.
///
/// Fitted to CS6's Texturizer over `samples/horse-3.jpg`, registered against
/// the source, on how the pattern repeats down a column — every 7 pixels,
/// and about as strongly every 14 — and how quickly it changes along a row.
/// The step is what sets the first: kinks much smaller than this and the
/// grooves line up into a ruled page, much bigger and the rows dissolve.
pub(crate) fn burlap(u: f32, v: f32) -> f32 {
    const PITCH: f32 = 7.2;
    // How long a groove runs between kinks, how far a kink steps it, and how
    // much of a run the step takes.
    const RUN: f32 = 4.0;
    const KINK: f32 = 1.4;
    const WEAVE: f32 = 0.35;
    const DRIFT: f32 = 12.0;
    const STEP: f32 = 0.45;
    // The groove's half-width, and how deep the crossings dip.
    const GROOVE: f32 = 1.3;
    const CROSSING: f32 = 0.15;
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let below = (v / PITCH).floor() as i32;
    let mut h = 1.0f32;
    // The groove above the pixel and the one below; a kink never moves a
    // groove far enough for the next one out to reach.
    for row in [below, below + 1] {
        // The threads running down are straight enough that the crossings
        // line up in columns, half a run apart from one groove to the next
        // as the weave goes over and under. The columns wander, slowly enough that neighbouring grooves keep
        // in step but not so slowly that the cloth reads as ruled.
        let wander = (value_noise(u, row as f32 * PITCH, DRIFT, 26) - 0.5) * 2.0;
        let t = u / RUN + 0.5 * row.rem_euclid(2) as f32 + wander;
        let k = t.floor() as i32;
        let blend = smooth((t - k as f32 - (1.0 - STEP)) / STEP);
        let at = |k: i32| {
            // Up at one crossing and down at the next, the other way round
            // in the next groove, and never quite the same twice. That
            // alternation is why CS6's burlap repeats every two threads more
            // strongly than every one.
            let weave = if (k + row).rem_euclid(2) == 0 {
                1.0
            } else {
                -1.0
            };
            let shift = (WEAVE * weave + (lattice(k, row, 23) - 0.5) * 2.0) * KINK;
            // Most runs are cut deep; one in four or so barely at all.
            let depth = (lattice(k, row, 24) * 1.4 - 0.1).clamp(0.25, 1.0);
            (shift, depth)
        };
        let ((s0, d0), (s1, d1)) = (at(k), at(k + 1));
        let shift = s0 + (s1 - s0) * blend;
        let depth = d0 + (d1 - d0) * blend;
        let d = (v - (row as f32 * PITCH + shift)) / GROOVE;
        h -= depth * (-d * d).exp();
        // The dip where a thread running down passes under, on the thread
        // just below this groove.
        let across = (t - (k + 1) as f32) * RUN;
        let under = (v - row as f32 * PITCH) / PITCH;
        if (0.0..1.0).contains(&under) {
            h -= CROSSING * (-(across * across) / 0.8).exp() * (std::f32::consts::PI * under).sin();
        }
    }
    // Hairy fibre along the threads.
    let fibre = value_noise(u * 0.7, v * 1.4, 1.2, 7);
    (h * (0.75 + 0.25 * fibre)).max(0.0)
}

/// Canvas: a fine plain weave, 3 pixels to a thread, the threads running down
/// standing a little prouder than those running across, over a little grain.
/// The threads wander a pixel or so and vary in thickness, as real canvas
/// does; a perfectly regular weave reads as ruled lines, not as cloth.
pub(crate) fn canvas(u: f32, v: f32) -> f32 {
    use std::f32::consts::PI;
    const PITCH: f32 = 3.0;
    const WANDER: f32 = 0.5;
    let u = u + (value_noise(u, v, 6.0, 41) - 0.5) * 2.0 * WANDER;
    let v = v + (value_noise(u, v, 6.0, 42) - 0.5) * 2.0 * WANDER;
    let down = (u / PITCH * PI).sin().abs();
    let across = (v / PITCH * PI).sin().abs();
    let over = ((u / PITCH).floor() + (v / PITCH).floor()).rem_euclid(2.0);
    let weave = if over > 0.5 {
        0.6 * down + 0.3 * across
    } else {
        0.4 * down + 0.5 * across
    };
    let slub = 0.75 + 0.5 * value_noise(u, v, 3.0, 43);
    (weave * slub + 0.2 * value_noise(u, v, 2.0, 4)) * 0.85
}

/// Sandstone: grain at three scales.
pub(crate) fn sandstone(u: f32, v: f32) -> f32 {
    0.5 * value_noise(u, v, 2.0, 1)
        + 0.3 * value_noise(u, v, 5.0, 2)
        + 0.2 * value_noise(u, v, 12.0, 3)
}

/// Smooth 0..1 noise on a lattice `cell` pixels apart.
pub(crate) fn value_noise(u: f32, v: f32, cell: f32, seed: u32) -> f32 {
    let (gu, gv) = (u / cell, v / cell);
    let (iu, iv) = (gu.floor(), gv.floor());
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let (fu, fv) = (smooth(gu - iu), smooth(gv - iv));
    let at = |du: i32, dv: i32| lattice(iu as i32 + du, iv as i32 + dv, seed);
    let top = at(0, 0) * (1.0 - fu) + at(1, 0) * fu;
    let bottom = at(0, 1) * (1.0 - fu) + at(1, 1) * fu;
    top * (1.0 - fv) + bottom * fv
}

pub(crate) fn lattice(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ seed.wrapping_mul(0xC2B2_AE3D)
        ^ crate::photorust::seed().wrapping_mul(0x27d4_eb2d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    (h & 0xFFFF) as f32 / 65535.0
}

/// CS6's ranges for Craquelure, which its three sliders run over.
pub const CRACK_SPACING: std::ops::RangeInclusive<u32> = 2..=100;

pub const CRACK_DEPTH: std::ops::RangeInclusive<u32> = 0..=10;

pub const CRACK_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=10;

/// How wide a plate of paint is, in pixels, per step of Crack Spacing. Read
/// off CS6 on `samples/horse-3.jpg`: at 15 the plates are a dozen pixels
/// across, at 60 the sky is ruled into blocks about fifty wide.
pub(crate) const CRACK_CELL_PER_STEP: f32 = 0.85;

/// How tall a course of plates is, as a share of their width. CS6's plates
/// are laid in rough courses, wider than they are tall: its sky is ruled by
/// cracks running mostly across, broken up by shorter ones running down.
pub(crate) const CRACK_COURSE: f32 = 0.8;

/// How far each crack between courses is pushed up or down from where an
/// even course would put it, as a share of a course, and how wide a stretch
/// of it moves together, as a share of a plate. Evenly spaced courses are
/// brickwork however broken up they are: CS6's rows are uneven, and a crack
/// running across steps up or down every so often rather than running on at
/// one height.
pub(crate) const CRACK_COURSE_JITTER: f32 = 0.42;

pub(crate) const CRACK_COURSE_STEP: f32 = 2.6;

/// How far apart the cracks that break a course run, as a share of a plate.
pub(crate) const CRACK_BREAK: f32 = 1.5;

/// The courses are laid twice over: once at the plate size and again finer,
/// this size, shut more and drawn fainter. One size of plate repeats; CS6's
/// sky has cracks of every length between its long ones.
pub(crate) const CRACK_FINE: f32 = 0.6;

pub(crate) const CRACK_FINE_GAP: f32 = 0.22;

pub(crate) const CRACK_FINE_FAINT: f32 = 1.8;

/// How far the cracks wander off straight: a slow sway and a kink at half a
/// plate, both as shares of a plate, and a fine wobble in pixels but no more
/// than a share of a plate. CS6's cracks run across and down, but none of
/// them is ruled; wobbled as hard as big ones, small plates lose their
/// direction altogether.
pub(crate) const CRACK_WANDER: f32 = 0.18;

pub(crate) const CRACK_KINK: f32 = 0.12;

pub(crate) const CRACK_WOBBLE: f32 = 1.3;

pub(crate) const CRACK_WOBBLE_MAX: f32 = 0.08;

/// The crumple: cracks along the contours of a fractal noise, which meander
/// every which way at every scale at once. CS6's dark horse is crumpled all
/// over like this, with no two chips alike; a Voronoi diagram's chips are
/// all one size and read as a jigsaw. The noise is three octaves from this
/// share of a plate, and a crack runs every [`CRACK_CONTOUR`] of its value.
pub(crate) const CRACK_CRUMPLE: f32 = 1.3;

pub(crate) const CRACK_CONTOUR: f32 = 0.1;

pub(crate) const CRACK_JAG: f32 = 1.2;

/// The jag grows with the plates, as this share of one, over a grain this
/// share of one: without it a wide spacing's contours are the smooth loops
/// of a contour map rather than cracks.
pub(crate) const CRACK_JAG_PER_CELL: f32 = 0.07;

pub(crate) const CRACK_JAG_GRAIN: f32 = 0.15;

/// Where the courses and the crumple hand over, as tones of the picture
/// softened over a share of a plate: the courses fade in above the first
/// pair and the crumple fades down above the second.
pub(crate) const CRACK_LIGHT_FROM: f32 = 0.25;

pub(crate) const CRACK_LIGHT_FULL: f32 = 0.55;

pub(crate) const CRACK_DARK_FROM: f32 = 0.3;

pub(crate) const CRACK_DARK_FULL: f32 = 0.72;

pub(crate) const CRACK_TONE_SOFTEN: f32 = 0.3;

/// How wide a crack is, and how far a plate's edge rounds down into it, in
/// pixels. CS6's plates are flat, with a narrow bevelled rim: rounding them
/// much further turns the surface into bubble wrap. But a crack is a groove
/// with a floor and two walls, not a scratch — a hairline casts no shadow.
pub(crate) const CRACK_WIDTH: f32 = 1.6;

pub(crate) const CRACK_BEVEL: f32 = 1.1;

/// How wide the dark floor at the bottom of a crack is, in pixels. Narrower
/// than the groove: the rest of its width shows as walls, lit on one side
/// and in shadow on the other, and a floor as wide as the groove runs
/// neighbouring cracks together into blots.
pub(crate) const CRACK_FLOOR_WIDTH: f32 = 1.0;

/// How deep a crack runs, as a floor and a spread along its length over a
/// stretch this share of a plate. A crack of one depth all along is a line
/// ruled on the surface; CS6's open wide here and close to nothing there.
pub(crate) const CRACK_DEEP_FLOOR: f32 = 0.45;

pub(crate) const CRACK_DEEP_SPREAD: f32 = 1.1;

pub(crate) const CRACK_DEEP_STRETCH: f32 = 0.6;

/// Pockets: small pits where the paint has flaked away, this share of a
/// plate across (but at least [`CRACK_POCKET_MIN`] pixels), found where a
/// noise rises above [`CRACK_POCKET_FROM`], and this deep. CS6's sea and
/// horse are pocked all over between the cracks, its pale sky hardly at all,
/// so they come with the crumple.
pub(crate) const CRACK_POCKET: f32 = 0.22;

pub(crate) const CRACK_POCKET_MIN: f32 = 2.5;

pub(crate) const CRACK_POCKET_FROM: f32 = 0.66;

pub(crate) const CRACK_POCKET_SOFT: f32 = 0.08;

pub(crate) const CRACK_POCKET_DEPTH: f32 = 0.8;

/// Shadows: how far the light is traced back towards the top left, in
/// pixels, how steeply it falls per pixel in units of the surface's height,
/// and how dark a full shadow is. The lit rim of a crack or a pocket throws
/// its far wall into shade, which is what makes it read as a hole rather
/// than as a line drawn on.
pub(crate) const CRACK_SHADOW_REACH: usize = 4;

pub(crate) const CRACK_SHADOW_FALL: f32 = 0.22;

pub(crate) const CRACK_SHADOW: f32 = 0.55;

/// How much of the crack network never opens. CS6's cracks almost never
/// close into plates: they are runs that meet now and then in a T or an L
/// and stop short, and a network closed all round reads as jigsaw pieces or
/// brickwork. So every run of crack is gated by a noise along its length,
/// this much of which is shut, and the gate opens over [`CRACK_GAP_SOFT`] so
/// a crack tapers out rather than stopping square. Wider spacing shuts more
/// of it, up to [`CRACK_GAP_WIDE`] more by [`CRACK_GAP_FULL`].
pub(crate) const CRACK_GAP: f32 = 0.38;

pub(crate) const CRACK_GAP_SOFT: f32 = 0.12;

pub(crate) const CRACK_GAP_WIDE: f32 = 0.1;

pub(crate) const CRACK_GAP_FULL: f32 = 60.0;

/// How long a dash of crack runs before its gate can shut, as a share of a
/// plate.
pub(crate) const CRACK_DASH: f32 = 0.7;

/// The share of the cracks that break a course that open at all.
pub(crate) const CRACK_BREAKS_OPEN: f32 = 0.55;

/// How much less of the crumple's length the gate shuts than of the
/// courses': CS6's dark horse is crumpled all over, and gated as hard as the
/// sky it is bare.
pub(crate) const CRACK_CRUMPLE_GAP_EASE: f32 = 0.12;

/// How much the picture's own brightness raises the surface, so the relief
/// follows the photograph's contours as well as the cracks.
pub(crate) const CRACK_PICTURE_RELIEF: f32 = 0.9;

/// How steep the relief is lit at the top of Crack Depth, and how much of
/// that the lightest parts of the picture keep. CS6's pale sky is faintly
/// ruled with thin embossed lines, while its dark horse is crumpled, every
/// chip catching a bright rim.
pub(crate) const CRACK_SHADE: f32 = 1.6;

pub(crate) const CRACK_SHADE_IN_LIGHT: f32 = 0.28;

/// A crack's darkness at the bottom of Crack Brightness, as a share of the
/// picture it runs through, and how far each step lifts it. At CS6's default
/// of 9 a crack is barely darker than its plates, and shows by its relief.
pub(crate) const CRACK_FLOOR: f32 = 0.15;

pub(crate) const CRACK_LIFT_PER_STEP: f32 = 0.095;

/// Filter ▸ Texture ▸ Craquelure: the picture painted onto plaster that has
/// dried and cracked into plates.
///
/// The crack network follows the picture. Where it is light the plates are
/// laid in uneven courses, **Crack Spacing** wide, split by cracks at random
/// and laid again finer over the top — see [`CRACK_COURSE`] — which is what
/// makes CS6's cracks across a pale sky run mostly across and down, at every
/// length. Where it is dark the paint has crumpled instead, and the cracks
/// follow the contours of a fractal noise, meandering every which way — see
/// [`CRACK_CRUMPLE`]. The cracks wander, and much of both networks never
/// opens at all, so the plates run into one another and the cracks stop
/// short — see [`CRACK_GAP`]. The
/// surface is the plates, flat with a narrow bevelled rim, plus the
/// picture's own brightness, and it is lit from
/// the top left as deep as **Crack Depth** asks. **Crack Brightness** is how
/// light the bottom of a crack is: near black at 0, the picture's own colour
/// towards 10.
///
/// Alpha is left alone.
///
/// No GPU path: it would fit — a few noises and a slope per pixel — but it
/// runs in tens of milliseconds on the CPU and its result is wanted straight
/// back by the history, so the upload would cost more than it saved.
pub fn craquelure(pixmap: &mut Pixmap, spacing: u32, depth: u32, brightness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let spacing = spacing.clamp(*CRACK_SPACING.start(), *CRACK_SPACING.end()) as f32;
    let depth = depth.clamp(*CRACK_DEPTH.start(), *CRACK_DEPTH.end()) as f32;
    let brightness = brightness.clamp(*CRACK_BRIGHTNESS.start(), *CRACK_BRIGHTNESS.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let cell = (spacing * CRACK_CELL_PER_STEP).max(2.0);
    let gap = CRACK_GAP + CRACK_GAP_WIDE * (spacing / CRACK_GAP_FULL).min(1.0);
    // How open a crack is, from its gate's noise: a crack that is closing
    // has its distance stretched, so it thins and tapers out.
    let open = |gate: f32| ((gate - gap) / CRACK_GAP_SOFT).clamp(0.0, 1.0);

    // The picture's tone, softened so the network follows its broad shapes
    // rather than its detail: that decides which network cracks where.
    let bytes = pixmap.as_bytes();
    let stride = pixmap.stride();
    let lum: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let p = &bytes[(i / width) * stride + (i % width) * 4..];
            (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0
        })
        .collect();
    let mut tone = lum.clone();
    crate::photorust::artistic::blur_field(&mut tone, width, height, cell * CRACK_TONE_SOFTEN);
    let ramp = |v: f32, from: f32, full: f32| ((v - from) / (full - from)).clamp(0.0, 1.0);

    // The crumple's noise, drawn once so its slope can be read off it, over
    // coordinates jagged by a fine wobble.
    let crumple = cell * CRACK_CRUMPLE;
    let noise: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % width) as f32, (i / width) as f32);
            let grain = cell * CRACK_JAG_GRAIN;
            let jag = cell * CRACK_JAG_PER_CELL;
            let u = x
                + (value_noise(x, y, 3.0, 71) - 0.5) * 2.0 * CRACK_JAG
                + (value_noise(x, y, grain, 76) - 0.5) * 2.0 * jag;
            let v = y
                + (value_noise(x, y, 3.0, 72) - 0.5) * 2.0 * CRACK_JAG
                + (value_noise(x, y, grain, 77) - 0.5) * 2.0 * jag;
            0.55 * value_noise(u, v, crumple, 73)
                + 0.3 * value_noise(u, v, crumple * 0.45, 74)
                + 0.15 * value_noise(u, v, crumple * 0.2, 75)
        })
        .collect();

    // How far a point is from the nearest open crack between or across the
    // courses, for courses `size` wide, `seed` apart from any other laying,
    // shut `extra` more than the gap.
    let courses_at = |xf: f32, yf: f32, size: f32, seed: u32, extra: f32| -> f32 {
        let s = seed * 16;
        let wobble = CRACK_WOBBLE.min(size * CRACK_WOBBLE_MAX);
        let u = xf
            + (value_noise(xf, yf, size, 61 + s) - 0.5) * 2.0 * CRACK_WANDER * size
            + (value_noise(xf, yf, size * 0.5, 68 + s) - 0.5) * 2.0 * CRACK_KINK * size
            + (value_noise(xf, yf, 4.0, 62 + s) - 0.5) * 2.0 * wobble;
        let v = yf
            + (value_noise(xf, yf, size, 63 + s) - 0.5) * 2.0 * CRACK_WANDER * size
            + (value_noise(xf, yf, size * 0.5, 69 + s) - 0.5) * 2.0 * CRACK_KINK * size
            + (value_noise(xf, yf, 4.0, 64 + s) - 0.5) * 2.0 * wobble;
        let seed = seed as i32 * 7919;
        let course = size * CRACK_COURSE;
        // The stretch of courses this point is in, and the uneven heights
        // of the cracks between them there.
        let stretch = (u / (size * CRACK_COURSE_STEP)).floor() as i32 + seed;
        let boundary = |k: i32| {
            (k as f32 + (lattice(k, stretch, 85) - 0.5) * 2.0 * CRACK_COURSE_JITTER) * course
        };
        let mut k = (v / course).floor() as i32;
        if v < boundary(k) {
            k -= 1;
        } else if v >= boundary(k + 1) {
            k += 1;
        }
        let dash = size * CRACK_DASH;
        let mut across = f32::MAX;
        for (b, d) in [(k, v - boundary(k)), (k + 1, boundary(k + 1) - v)] {
            let gate = value_noise(u + b as f32 * 97.0, seed as f32, dash, 81);
            let o = open(gate - extra);
            if o > 0.0 {
                across = across.min(d.abs() / o);
            }
        }
        // The cracks that break the course, only some of which open.
        let slab = size * CRACK_BREAK;
        let column = (u / slab).floor() as i32;
        let mut down = f32::MAX;
        for j in column - 1..=column + 1 {
            if lattice(j, k + seed, 82) > CRACK_BREAKS_OPEN - extra {
                continue;
            }
            let at = (j as f32 + 0.15 + 0.7 * lattice(j, k + seed, 65)) * slab;
            down = down.min((u - at).abs());
        }
        across.min(down)
    };

    // How far each pixel is from the nearest crack that is open there. A
    // network that is fading out has its distances stretched, so its cracks
    // thin and go before they vanish rather than switching off.
    let mut crack = vec![0.0f32; width * height];
    let (tone_ref, noise) = (&tone, &noise);
    crack
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, slot) in row.iter_mut().enumerate() {
                let (xf, yf) = (x as f32, y as f32);
                let i = y * width + x;
                let t = tone_ref[i];
                let light = ramp(t, CRACK_LIGHT_FROM, CRACK_LIGHT_FULL);
                let dark = 1.0 - ramp(t, CRACK_DARK_FROM, CRACK_DARK_FULL);

                let mut courses = f32::MAX;
                if light > 0.0 {
                    courses = courses_at(xf, yf, cell, 0, 0.0).min(
                        courses_at(xf, yf, cell * CRACK_FINE, 1, CRACK_FINE_GAP) * CRACK_FINE_FAINT,
                    ) / light;
                }

                // The crumple: distance to the nearest contour of the noise, as
                // its value's distance over its slope. None of it in the light:
                // faded rather than gone, its contours break up into specks.
                if dark <= 0.0 {
                    *slot = courses;
                    continue;
                }
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let gx = (noise[y * width + right] - noise[y * width + left])
                    / (right - left).max(1) as f32;
                let gy =
                    (noise[down * width + x] - noise[up * width + x]) / (down - up).max(1) as f32;
                let slope = (gx * gx + gy * gy).sqrt().max(1e-4);
                let level = noise[i] / CRACK_CONTOUR;
                let contour = (level - level.round()).abs() * CRACK_CONTOUR / slope;
                let along =
                    open(value_noise(xf, yf, cell * CRACK_DASH, 83) + CRACK_CRUMPLE_GAP_EASE);
                let crumpled = if along > 0.0 {
                    contour / along / dark
                } else {
                    f32::MAX
                };

                *slot = courses.min(crumpled);
            }
        });

    // The surface: plates rounded down into their cracks, on top of the
    // picture's own brightness.
    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let mut surface: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % width) as f32, (i / width) as f32);
            // The groove, deeper in some stretches than others.
            let deep = CRACK_DEEP_FLOOR
                + CRACK_DEEP_SPREAD * value_noise(x, y, cell * CRACK_DEEP_STRETCH, 91);
            let groove = deep * (1.0 - smooth((crack[i] - CRACK_WIDTH * 0.5) / CRACK_BEVEL));
            // The pockets, where the paint has crumpled.
            let dark = 1.0 - ramp(tone[i], CRACK_DARK_FROM, CRACK_DARK_FULL);
            let pocket_size = (cell * CRACK_POCKET).max(CRACK_POCKET_MIN);
            let n = 0.65 * value_noise(x, y, pocket_size, 92)
                + 0.35 * value_noise(x, y, pocket_size * 0.5, 93);
            let pocket =
                smooth((n - CRACK_POCKET_FROM) / CRACK_POCKET_SOFT) * CRACK_POCKET_DEPTH * dark;
            1.0 - groove.max(pocket) + lum[i] * CRACK_PICTURE_RELIEF
        })
        .collect();
    crate::photorust::artistic::blur_field(&mut surface, width, height, 0.5);

    let (lx, ly) = Light::TopLeft.towards();
    let shade_gain = depth / *CRACK_DEPTH.end() as f32 * CRACK_SHADE;
    let floor = CRACK_FLOOR + brightness * CRACK_LIFT_PER_STEP;
    let (surface, crack, lum) = (&surface, &crack, &lum);
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, px) in out.chunks_exact_mut(4).enumerate() {
                let i = y * width + x;
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let dx = (surface[y * width + right] - surface[y * width + left])
                    / (right - left).max(1) as f32;
                let dy = (surface[down * width + x] - surface[up * width + x])
                    / (down - up).max(1) as f32;
                // A slope that falls away from the light is lit.
                let lightness = lum[i];
                let shade = -(dx * lx + dy * ly)
                    * shade_gain
                    * (1.0 - (1.0 - CRACK_SHADE_IN_LIGHT) * lightness);
                // The shadow thrown by whatever stands between this pixel and
                // the light: the lit rim of a crack or a pocket.
                let mut shadow = 0.0f32;
                for k in 1..=CRACK_SHADOW_REACH {
                    let sx = x as f32 + lx * k as f32;
                    let sy = y as f32 + ly * k as f32;
                    if sx < 0.0 || sy < 0.0 {
                        break;
                    }
                    let (sx, sy) = (sx.round() as usize, sy.round() as usize);
                    if sx >= width || sy >= height {
                        break;
                    }
                    let above =
                        surface[sy * width + sx] - surface[i] - CRACK_SHADOW_FALL * k as f32;
                    shadow = shadow.max(above);
                }
                let shadow =
                    1.0 - CRACK_SHADOW * (shadow * shade_gain / CRACK_SHADE).clamp(0.0, 1.0);
                // The bottom of the crack, where the light does not reach.
                let open = 1.0 - smooth(crack[i] / CRACK_FLOOR_WIDTH);
                let tone = (1.0 + (floor - 1.0) * open) * shadow;
                for c in 0..3 {
                    let v = px[c] as f32 * (1.0 + shade * SHADE_BY_COLOUR) + shade * SHADE_BY_LIGHT;
                    px[c] = (v * tone).clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: cracking the paint does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Grain, which its two sliders run over.
pub const GRAIN_INTENSITY: std::ops::RangeInclusive<u32> = 0..=100;

pub const GRAIN_CONTRAST: std::ops::RangeInclusive<u32> = 0..=100;

/// How strong the grain is, as a spread in levels per step of Intensity.
/// Read off CS6 on `samples/horse-3.jpg`: at 40 Regular is a plain but
/// moderate noise over the sky, at 71 the sky is more noise than blue.
pub(crate) const GRAIN_SPREAD_PER_STEP: f32 = 0.8;

/// Contrast is a gain about mid-grey, doubling every this many steps from a
/// gain of one at [`GRAIN_CONTRAST_FLAT`], and it stretches the grain along
/// with the picture: CS6 at 78 turns the horse nearly black and the noise
/// over the sky garish.
pub(crate) const GRAIN_CONTRAST_DOUBLING: f32 = 40.0;

pub(crate) const GRAIN_CONTRAST_FLAT: f32 = 50.0;

/// Each kind's grain: how far the noise is blurred into clumps, in pixels,
/// how strong it is against Regular's, how much harder than the slider it
/// pushes the contrast, and a contrast of its own on top that holds at any
/// slider setting. CS6's Contrasty at the slider's flat middle already has
/// the horse black and the sky bleached, so its contrast cannot all come
/// from the slider. Clumped is harsh too; Enlarged softer; Soft barely there.
pub(crate) struct GrainKind {
    pub(crate) clump: f32,
    pub(crate) strength: f32,
    pub(crate) contrast: f32,
    pub(crate) boost: f32,
}

pub(crate) const GRAIN_REGULAR: GrainKind = GrainKind {
    clump: 0.0,
    strength: 1.0,
    contrast: 1.0,
    boost: 1.0,
};

pub(crate) const GRAIN_SOFT: GrainKind = GrainKind {
    clump: 0.7,
    strength: 0.6,
    contrast: 1.0,
    boost: 1.0,
};

pub(crate) const GRAIN_CLUMPED: GrainKind = GrainKind {
    clump: 2.0,
    strength: 0.5,
    contrast: 1.6,
    boost: 1.4,
};

pub(crate) const GRAIN_CONTRASTY: GrainKind = GrainKind {
    clump: 1.6,
    strength: 0.4,
    contrast: 2.0,
    boost: 1.65,
};

pub(crate) const GRAIN_ENLARGED: GrainKind = GrainKind {
    clump: 2.4,
    strength: 0.55,
    contrast: 1.3,
    boost: 1.15,
};

/// Sprinkles: the share of pixels thrown to the background colour at the top
/// of Intensity. CS6 at 51 has salted the dark horse grey with them.
pub(crate) const GRAIN_SPRINKLE: f32 = 0.7;

/// Speckle inks a coarse screen. CS6 at 200% is a square grid about four
/// pixels to a cell: the grid's lines take the foreground wherever the
/// picture is mid-dark, so the brown horse comes out tiled; the darkest
/// parts fill solid; and whatever is darker than its surroundings — an
/// outline, the dark side of a detail — inks in too. The pale sky stays
/// clean. So a pixel is inked when its tone, less how much darker than its
/// neighbourhood it is, falls below a cut that is higher on the grid's lines
/// and shaken by a little noise, all in levels at the top of Intensity.
pub(crate) const GRAIN_MESH: usize = 4;

pub(crate) const GRAIN_SPECKLE_CUT: f32 = 95.0;

pub(crate) const GRAIN_SPECKLE_ON_MESH: f32 = 125.0;

pub(crate) const GRAIN_SPECKLE_SHAKE: f32 = 55.0;

pub(crate) const GRAIN_SPECKLE_EDGE: f32 = 2.5;

pub(crate) const GRAIN_SPECKLE_NEIGHBOURHOOD: f32 = 2.0;

/// How far Speckle lifts and saturates the picture under its specks at the
/// top of Intensity. CS6's brown horse comes out lighter and redder, its sky
/// and sand paler and brighter.
pub(crate) const GRAIN_SPECKLE_LIFT: f32 = 0.7;

pub(crate) const GRAIN_SPECKLE_SATURATE: f32 = 0.5;

/// Stippled: how hard the noise shakes each pixel before it is cut into one
/// swatch or the other, against Regular's. Enough that the mid-tones come
/// out as an even scatter of both.
pub(crate) const GRAIN_STIPPLE: f32 = 2.2;

/// Horizontal and Vertical are printer lines, as on a worn photocopy: each
/// row (or column) gets its own darkness, carried the whole way along it, so
/// the picture is banded rather than scratched. The bands are one to three
/// pixels thick — the row noise is smoothed over [`GRAIN_BAND`] — and fade
/// in and out over [`GRAIN_BAND_FADE`] pixels along their length, a few rows
/// at a time, keeping at least [`GRAIN_BAND_FLOOR`] of their strength. A fine
/// grit rides on top. A band darkens by [`GRAIN_STREAK_STRENGTH`] against
/// Regular's grain and lightens by [`GRAIN_STREAK_LIGHT`] of that.
pub(crate) const GRAIN_BAND: f32 = 1.8;

pub(crate) const GRAIN_BAND_FADE: f32 = 240.0;

pub(crate) const GRAIN_BAND_FLOOR: f32 = 0.3;

pub(crate) const GRAIN_BAND_GRIT: f32 = 0.5;

pub(crate) const GRAIN_STREAK_STRENGTH: f32 = 2.2;

pub(crate) const GRAIN_STREAK_LIGHT: f32 = 0.35;

/// The streaks' contrast of its own, like [`GrainKind::boost`]: CS6's horse
/// under them is near black and its sky bleached, so the lines vanish in the
/// sky and show hardest over the mid-toned sea.
pub(crate) const GRAIN_STREAK_BOOST: f32 = 1.25;

/// How far Sprinkles lifts the picture towards white at the top of
/// Intensity. CS6 pales the whole picture under it, so the specks show.
pub(crate) const GRAIN_SPECK_LIFT: f32 = 0.6;

/// How much of each channel's grain is shared with the other two. CS6's
/// colour grain is pastel — pink, mint and lilac specks, not pure red, green
/// and blue ones — which is what three channels partly moving together give.
pub(crate) const GRAIN_SHARED: f32 = 0.55;

/// White noise from -1 to 1, laid by where on the canvas a pixel is; `salt`
/// keeps two layers from being the same noise.
pub(crate) fn grain_noise(x: usize, y: usize, salt: i32) -> f32 {
    crate::photorust::artistic::noise(x as i32 + salt * 7919, y as i32 - salt * 104_729) * 2.0 - 1.0
}

#[cfg(test)]
mod tests;
