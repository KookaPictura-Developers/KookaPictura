//! Lens Flare: photorust's flare model (#225), ported from perfecto25/photorust
//! (`core/src/filters/render.rs`), GPL-3.0, onto the planar buffer.
//!
//! A flare is light *added* to the picture: a hot core in a soft glow, a halo
//! ring, rays (or, for the Movie Prime, an anamorphic streak), and a string of
//! tinted ghosts thrown from the flare through the middle of the frame and out
//! the far side. Every size is a fraction of the half-diagonal and the centre a
//! fraction of the frame, so a flare on a proxy and on the full image are the
//! same picture — the dialog's preview depends on that.

use pictura_core::PixelBuffer;
use rayon::prelude::*;

use crate::{validate, FilterError};

/// CS6's four lenses, in the order its Lens Flare dialog lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LensType {
    /// 50-300mm Zoom, CS6's default: a modest core, a long string of
    /// coloured ghosts down the axis, and the hexagonal aperture in them.
    Zoom,
    /// 35mm Prime: a big soft core with long rays and few ghosts.
    Prime35,
    /// 105mm Prime: a tight hot core, short rays, barely any ghosts.
    Prime105,
    /// Movie Prime: the horizontal blue streak a cinema lens throws.
    MoviePrime,
}

/// One of the discs of light strung along the axis of a flare: a reflection
/// between lens elements, which is why they line up.
struct Ghost {
    /// Position on the line from the flare through the middle of the frame —
    /// 0 is the flare, 1 the centre of the picture, 2 the point opposite.
    at: f32,
    /// Fraction of the half-diagonal.
    radius: f32,
    tint: (f32, f32, f32),
    strength: f32,
    /// A ring of light rather than a filled disc.
    ring: bool,
}

/// Everything that distinguishes one lens from the others. Sizes are
/// fractions of the half-diagonal.
struct Lens {
    core: f32,
    glow: f32,
    glow_strength: f32,
    rays: u32,
    ray_length: f32,
    ray_strength: f32,
    /// The anamorphic streak; zero for the three stills lenses.
    streak: f32,
    streak_strength: f32,
    halo: f32,
    halo_strength: f32,
    ghosts: &'static [Ghost],
}

const fn ghost(at: f32, radius: f32, tint: (f32, f32, f32), strength: f32, ring: bool) -> Ghost {
    Ghost {
        at,
        radius,
        tint,
        strength,
        ring,
    }
}

const ZOOM_GHOSTS: &[Ghost] = &[
    ghost(-0.22, 0.030, (1.00, 0.86, 0.55), 0.30, false),
    ghost(0.34, 0.020, (0.55, 0.85, 1.00), 0.24, false),
    ghost(0.56, 0.046, (0.60, 1.00, 0.75), 0.18, false),
    ghost(0.78, 0.028, (1.00, 0.70, 0.45), 0.26, false),
    ghost(1.00, 0.062, (0.70, 0.75, 1.00), 0.14, true),
    ghost(1.22, 0.034, (1.00, 0.55, 0.55), 0.22, false),
    ghost(1.46, 0.092, (0.50, 0.80, 1.00), 0.12, true),
    ghost(1.72, 0.050, (1.00, 0.85, 0.40), 0.16, false),
];

const PRIME35_GHOSTS: &[Ghost] = &[
    ghost(0.45, 0.036, (0.65, 0.90, 1.00), 0.18, false),
    ghost(0.95, 0.078, (0.85, 0.70, 1.00), 0.12, true),
    ghost(1.38, 0.056, (1.00, 0.75, 0.50), 0.16, false),
];

const PRIME105_GHOSTS: &[Ghost] = &[
    ghost(0.62, 0.022, (0.70, 0.95, 1.00), 0.14, false),
    ghost(1.12, 0.040, (1.00, 0.80, 0.60), 0.10, true),
];

const MOVIE_GHOSTS: &[Ghost] = &[
    ghost(0.70, 0.030, (0.60, 0.80, 1.00), 0.16, false),
    ghost(1.26, 0.062, (0.55, 0.75, 1.00), 0.10, true),
];

impl LensType {
    fn lens(self) -> Lens {
        match self {
            LensType::Zoom => Lens {
                core: 0.013,
                glow: 0.17,
                glow_strength: 0.55,
                rays: 6,
                ray_length: 0.55,
                ray_strength: 0.22,
                streak: 0.0,
                streak_strength: 0.0,
                halo: 0.30,
                halo_strength: 0.12,
                ghosts: ZOOM_GHOSTS,
            },
            LensType::Prime35 => Lens {
                core: 0.019,
                glow: 0.27,
                glow_strength: 0.78,
                rays: 8,
                ray_length: 0.85,
                ray_strength: 0.32,
                streak: 0.0,
                streak_strength: 0.0,
                halo: 0.22,
                halo_strength: 0.10,
                ghosts: PRIME35_GHOSTS,
            },
            LensType::Prime105 => Lens {
                core: 0.010,
                glow: 0.12,
                glow_strength: 0.62,
                rays: 4,
                ray_length: 0.34,
                ray_strength: 0.12,
                streak: 0.0,
                streak_strength: 0.0,
                halo: 0.18,
                halo_strength: 0.08,
                ghosts: PRIME105_GHOSTS,
            },
            LensType::MoviePrime => Lens {
                core: 0.012,
                glow: 0.14,
                glow_strength: 0.50,
                rays: 0,
                ray_length: 0.0,
                ray_strength: 0.0,
                streak: 1.30,
                streak_strength: 0.55,
                halo: 0.16,
                halo_strength: 0.06,
                ghosts: MOVIE_GHOSTS,
            },
        }
    }
}

/// Lens Flare: add a flare centred at `center` (fractions of the width and
/// height, clamped into 0..=1) to the RGB planes, leaving alpha untouched.
/// `brightness` is CS6's 10..=300 %. Deterministic — no seed.
///
/// ponytail: CS6's flare model is closed; this is photorust's, tuned by eye,
/// not fitted to reference output.
pub fn lens_flare(
    buf: &mut PixelBuffer,
    brightness: f64,
    center: (f64, f64),
    kind: LensType,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(10.0..=300.0).contains(&brightness) {
        return Err(FilterError::InvalidParams(format!(
            "lens flare brightness {brightness} out of range 10..=300"
        )));
    }
    let lens = kind.lens();
    let gain = brightness as f32 / 100.0;
    let width = buf.width as f32;
    let height = buf.height as f32;
    let span = 0.5 * width.hypot(height);
    let flare = (
        center.0.clamp(0.0, 1.0) as f32 * width,
        center.1.clamp(0.0, 1.0) as f32 * height,
    );
    let axis = (width * 0.5 - flare.0, height * 0.5 - flare.1);

    let row = buf.width as usize;
    let (r, rest) = buf.data.split_at_mut(n);
    let (g, rest) = rest.split_at_mut(n);
    let b = &mut rest[..n];
    r.par_chunks_exact_mut(row)
        .zip(g.par_chunks_exact_mut(row))
        .zip(b.par_chunks_exact_mut(row))
        .enumerate()
        .for_each(|(y, ((r, g), b))| {
            let py = y as f32 + 0.5;
            for x in 0..row {
                let px = x as f32 + 0.5;
                let light = light_at(&lens, flare, axis, span, (px, py));
                for (out, amount) in [&mut r[x], &mut g[x], &mut b[x]]
                    .into_iter()
                    .zip([light.0, light.1, light.2])
                {
                    *out = (*out as f32 + amount * gain * 255.0).clamp(0.0, 255.0) as u8;
                }
            }
        });
    Ok(())
}

/// The light the flare throws on one pixel, before the brightness gain.
fn light_at(
    lens: &Lens,
    flare: (f32, f32),
    axis: (f32, f32),
    span: f32,
    (px, py): (f32, f32),
) -> (f32, f32, f32) {
    let (dx, dy) = (px - flare.0, py - flare.1);
    let distance = dx.hypot(dy);
    let mut light = (0.0f32, 0.0f32, 0.0f32);

    // An inverse square rather than a bell curve: blinding in the middle and
    // still faintly there a long way out.
    let core = 1.0 / (1.0 + (distance / (lens.core * span)).powi(2));
    add(&mut light, (1.00, 0.98, 0.94), core);
    let glow = (-(distance / (lens.glow * span)).powi(2)).exp();
    add(&mut light, (1.00, 0.95, 0.86), glow * lens.glow_strength);

    let ring = (distance - lens.halo * span) / (0.08 * span);
    add(
        &mut light,
        (0.92, 0.76, 1.00),
        (-ring * ring).exp() * lens.halo_strength,
    );

    if lens.rays > 0 {
        let spokes = lens.rays as f32;
        // How far round between one ray and the next, as -0.5..0.5.
        let between = (dy.atan2(dx) * spokes / std::f32::consts::TAU + 0.5).fract() - 0.5;
        let near = (-(between * 14.0).powi(2)).exp();
        let reach = (-(distance / (lens.ray_length * span)).powi(2)).exp();
        add(
            &mut light,
            (1.00, 0.93, 0.80),
            near * reach * lens.ray_strength,
        );
    }

    if lens.streak_strength > 0.0 {
        let along = (-(dx / (lens.streak * span)).powi(2)).exp();
        let across = (-(dy / (0.012 * span)).powi(2)).exp();
        add(
            &mut light,
            (0.55, 0.72, 1.00),
            along * across * lens.streak_strength,
        );
    }

    for ghost in lens.ghosts {
        let (gx, gy) = (
            px - (flare.0 + axis.0 * ghost.at),
            py - (flare.1 + axis.1 * ghost.at),
        );
        let radius = ghost.radius * span;
        if ghost.ring {
            let edge = (gx.hypot(gy) - radius) / (0.3 * radius);
            add(
                &mut light,
                ghost.tint,
                (-edge * edge).exp() * ghost.strength,
            );
        } else {
            let shape = aperture(gx, gy) / radius;
            let disc = (1.0 - shape * shape).max(0.0);
            add(&mut light, ghost.tint, disc * disc * ghost.strength);
        }
    }
    light
}

fn add(light: &mut (f32, f32, f32), tint: (f32, f32, f32), amount: f32) {
    light.0 += tint.0 * amount;
    light.1 += tint.1 * amount;
    light.2 += tint.2 * amount;
}

/// Distance from the middle of a hexagonal aperture, scaled so the six flat
/// sides are one unit away, rounded a little towards a circle. Three dot
/// products instead of an `atan2`: this runs for every ghost at every pixel.
fn aperture(x: f32, y: f32) -> f32 {
    const COS60: f32 = 0.5;
    const SIN60: f32 = 0.866_025_4;
    let flat = x.abs();
    let left = (x * COS60 + y * SIN60).abs();
    let right = (x * COS60 - y * SIN60).abs();
    let hex = flat.max(left).max(right);
    hex * 0.65 + x.hypot(y) * 0.35
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENSES: [LensType; 4] = [
        LensType::Zoom,
        LensType::Prime35,
        LensType::Prime105,
        LensType::MoviePrime,
    ];

    fn plane(w: u32, h: u32, rgb: u8, alpha: u8) -> PixelBuffer {
        let mut buf = PixelBuffer::new(w, h, 4);
        let n = buf.pixel_count();
        buf.data[..3 * n].fill(rgb);
        buf.data[3 * n..].fill(alpha);
        buf
    }

    fn flare(w: u32, h: u32, at: (f64, f64), brightness: f64, lens: LensType) -> PixelBuffer {
        let mut buf = plane(w, h, 20, 255);
        lens_flare(&mut buf, brightness, at, lens).unwrap();
        buf
    }

    fn red(buf: &PixelBuffer, x: usize, y: usize) -> u8 {
        buf.data[y * buf.width as usize + x]
    }

    #[test]
    fn a_flare_burns_brightest_where_it_is_put() {
        let buf = flare(240, 180, (0.5, 0.5), 100.0, LensType::Zoom);
        assert_eq!(
            red(&buf, 120, 90),
            255,
            "the middle of the flare is not blown out"
        );
        // The far corner is off the axis, so no ghost lands on it either.
        assert!(red(&buf, 4, 174) < 90, "the flare lit the whole frame");
    }

    #[test]
    fn a_flare_goes_where_it_is_told() {
        let buf = flare(240, 180, (0.25, 0.25), 100.0, LensType::Zoom);
        let here = red(&buf, 60, 45);
        // The opposite corner along the diagonal is where the ghosts fall.
        let elsewhere = red(&buf, 180, 45);
        assert!(
            here > elsewhere + 100,
            "{here} at the flare, {elsewhere} away from it"
        );
    }

    #[test]
    fn brightness_scales_the_light_a_flare_adds() {
        // Off the axis and away from the blown-out core.
        let sample = |brightness| {
            red(
                &flare(240, 180, (0.2, 0.2), brightness, LensType::Prime35),
                150,
                40,
            )
        };
        let (dim, mid, bright) = (sample(25.0), sample(100.0), sample(300.0));
        assert!(dim < mid && mid < bright, "{dim}, {mid}, {bright}");
    }

    #[test]
    fn the_ghosts_line_up_through_the_middle_of_the_frame() {
        let mut buf = plane(400, 300, 0, 255);
        lens_flare(&mut buf, 100.0, (0.12, 0.5), LensType::Zoom).unwrap();
        let sum = |y: usize| -> u64 { (200..400).map(|x| red(&buf, x, y) as u64).sum() };
        let (along, across) = (sum(150), sum(30));
        assert!(
            along > across * 2,
            "the ghosts did not follow the axis: {along} along it, {across} off it"
        );
    }

    #[test]
    fn the_four_lenses_throw_different_flares_deterministically() {
        let each: Vec<_> = LENSES
            .into_iter()
            .map(|lens| flare(160, 120, (0.5, 0.4), 100.0, lens).data)
            .collect();
        for (i, one) in each.iter().enumerate() {
            for other in each.iter().skip(i + 1) {
                assert_ne!(one, other, "two lenses threw the same flare");
            }
        }
        for (lens, first) in LENSES.into_iter().zip(&each) {
            assert_eq!(&flare(160, 120, (0.5, 0.4), 100.0, lens).data, first);
        }
    }

    #[test]
    fn a_flare_leaves_alpha_alone() {
        let mut buf = plane(64, 64, 10, 77);
        lens_flare(&mut buf, 300.0, (0.5, 0.5), LensType::Prime35).unwrap();
        let n = buf.pixel_count();
        assert!(buf.data[3 * n..].iter().all(|&a| a == 77));
    }

    #[test]
    fn an_off_frame_center_is_clamped_to_the_edge() {
        let clamped = flare(64, 48, (-3.0, 5.0), 100.0, LensType::Zoom);
        let edge = flare(64, 48, (0.0, 1.0), 100.0, LensType::Zoom);
        assert_eq!(clamped.data, edge.data);
    }

    /// The dialog previews a shrunk proxy, which is only honest because a
    /// flare is placed and sized as a fraction of the frame.
    #[test]
    fn a_flare_is_the_same_picture_at_any_size() {
        let big = flare(600, 400, (0.7, 0.3), 100.0, LensType::Zoom);
        let small = flare(150, 100, (0.7, 0.3), 100.0, LensType::Zoom);
        let mut worst = 0i32;
        for y in 0..100 {
            for x in 0..150 {
                let mut block = 0i32;
                let mut blown = false;
                for dy in 0..4 {
                    for dx in 0..4 {
                        let level = red(&big, x * 4 + dx, y * 4 + dy) as i32;
                        blown |= level >= 250;
                        block += level;
                    }
                }
                // Averaging clipped pixels is not clipping their average.
                if blown {
                    continue;
                }
                worst = worst.max((block / 16 - red(&small, x, y) as i32).abs());
            }
        }
        assert!(
            worst <= 20,
            "proxy and full size disagree by {worst} levels"
        );
    }

    #[test]
    fn lens_flare_brightness_is_validated_and_buffer_untouched() {
        let base = plane(8, 8, 40, 255);
        for brightness in [9.0, 301.0, f64::NAN] {
            let mut buf = base.clone();
            let err = lens_flare(&mut buf, brightness, (0.5, 0.5), LensType::Zoom).unwrap_err();
            assert!(
                matches!(err, FilterError::InvalidParams(_)),
                "brightness {brightness} should be rejected"
            );
            assert_eq!(buf.data, base.data);
        }
    }
}
