//! Fibers: photorust's streak model (#224), ported from perfecto25/photorust
//! (`core/src/filters/render.rs`), GPL-3.0, onto the planar buffer.
//!
//! CS6's fibres are noise **stretched down the picture**, two things layered:
//! broad soft *clumps*, light and dark wands tens of pixels across running a
//! long way down, with crisp *hairs* a pixel or two across and much shorter
//! standing on top of them. Isotropic noise smeared downwards, a tone fixed per
//! column, one octave run from wide to narrow, and hard edges at every octave
//! were each tried upstream and each looks wrong; see `fibers`.

use pictura_core::PixelBuffer;
use rayon::prelude::*;

use crate::{validate, FilterError};

/// Highest Variance and Strength the CS6 dialog offers.
const SLIDER_MAX: f64 = 64.0;
const CLUMP_OCTAVES: usize = 3;
const OCTAVES: usize = 5;
/// How wide each octave is, in pixels: three clumps, then two hairs. The gap
/// between eight and two is deliberate: CS6's clumps and hairs are separate
/// things, not one continuous spread of sizes.
const BANDS: [f32; OCTAVES] = [48.0, 20.0, 8.0, 2.0, 1.0];

/// A hash of `seed` and `salt` mapped into 0..1.
fn noise01(seed: u32, salt: u32) -> f32 {
    let mut h = seed.wrapping_mul(0x9E37_79B1) ^ salt.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_F491);
    h ^= h >> 13;
    (h % 65521) as f32 / 65521.0
}

/// Smoothstep-eased lattice value noise in 0..1 over a plane.
fn noise2(seed: u32, x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let at = |i: i32, j: i32| {
        noise01(
            seed,
            (i as u32).wrapping_mul(73_856_093) ^ (j as u32).wrapping_mul(19_349_663),
        )
    };
    let (i, j) = (xi as i32, yi as i32);
    let top = at(i, j) + (at(i + 1, j) - at(i, j)) * sx;
    let bottom = at(i, j + 1) + (at(i + 1, j + 1) - at(i, j + 1)) * sx;
    top + (bottom - top) * sy
}

/// Fibers: replace the color planes with vertical streaks blended from
/// `color_b` (background) at 0 to `color_a` (foreground) at 1, leaving alpha
/// untouched.
///
/// `variance` (0..=64) is how far the fibres break up: near zero a few broad
/// soft bands running the whole height, by the middle a thicket of hairs over
/// clumps, at 64 short near-black-and-white spatter; 0 itself is the even
/// blend of the two colours. It shortens both clumps and hairs and shifts the
/// balance towards the hairs. `strength` (1..=64) stretches them lengthwise
/// again. `seed` reproduces one exact arrangement for Randomize.
///
/// ponytail: CS6's fibre algorithm is closed; this is photorust's model tuned
/// by eye against CS6 at 500 %, not fitted to reference output.
pub fn fibers(
    buf: &mut PixelBuffer,
    variance: f64,
    strength: f64,
    color_a: [u8; 3],
    color_b: [u8; 3],
    seed: u64,
) -> Result<(), FilterError> {
    let n = validate(buf)?;
    if !(0.0..=SLIDER_MAX).contains(&variance) {
        return Err(FilterError::InvalidParams(format!(
            "fibers variance {variance} out of range 0..=64"
        )));
    }
    if !(1.0..=SLIDER_MAX).contains(&strength) {
        return Err(FilterError::InvalidParams(format!(
            "fibers strength {strength} out of range 1..=64"
        )));
    }
    let planes = (buf.channels as usize).min(3);
    if variance == 0.0 {
        for c in 0..planes {
            let mid = ((color_a[c] as u16 + color_b[c] as u16) / 2) as u8;
            buf.data[c * n..(c + 1) * n].fill(mid);
        }
        return Ok(());
    }

    let width = buf.width as usize;
    let variance = variance as f32;
    let strength = strength as f32;
    let seed = seed as u32 ^ (seed >> 32) as u32;

    // How far each runs before its tone shifts, before the per-hair jitter
    // below. Variance shortens both, strength stretches both.
    let clump_run = (60.0 / (1.0 + variance * 0.02)) * (strength / 4.0).sqrt();
    let hair_run = (28.0 / (1.0 + variance * 0.04)) * (strength / 4.0).sqrt();
    // How much of the picture is hairs rather than clumps: the other half of
    // what variance does.
    let hair_share = 0.20 + (variance / 64.0).powf(0.6) * 0.45;

    // Where each hair starts and how long its own tone holds. Without this
    // every hair changes tone at the same rows and the result is corduroy.
    // Drawn from the octave's own lattice so it varies per pixel where that
    // octave does and smoothly where it doesn't.
    let mut shift = vec![(0.0f32, 0.0f32); width * OCTAVES];
    shift
        .par_chunks_exact_mut(OCTAVES)
        .enumerate()
        .for_each(|(x, octaves)| {
            for (octave, cell) in octaves.iter_mut().enumerate() {
                let along = x as f32 / BANDS[octave];
                let phase = noise2(seed ^ 0x27d4_eb2d, along, 0.0) * 512.0;
                // Three fifths to eight fifths of the nominal length, so some
                // hairs are stubble and others run on.
                let length = 0.6 + noise2(seed ^ 0x1656_67b1, along, 7.0);
                *cell = (phase, length);
            }
        });

    // The hair octaves' lattices land on every pixel or every other one, so
    // they come out hard-edged while the clump octaves stay smooth gradients.
    let mut field = vec![0.0f32; n];
    field
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, line)| {
            for (x, cell) in line.iter_mut().enumerate() {
                let part = |from: usize, to: usize, run: f32, falloff: f32| {
                    let (mut sum, mut weight, mut total) = (0.0, 1.0, 0.0);
                    for octave in from..to {
                        let band = BANDS[octave];
                        let (phase, length) = shift[x * OCTAVES + octave];
                        sum += noise2(
                            seed.wrapping_add(octave as u32 * 7919),
                            x as f32 / band,
                            (y as f32 + phase) / (run * band.sqrt() * length),
                        ) * weight;
                        total += weight;
                        weight *= falloff;
                    }
                    sum / total
                };
                let clumps = part(0, CLUMP_OCTAVES, clump_run, 0.6);
                let hairs = part(CLUMP_OCTAVES, OCTAVES, hair_run, 1.7);
                *cell = clumps * (1.0 - hair_share) + hairs * hair_share;
            }
        });

    // A sum of noises sits in a narrow band around its mean, which is grey
    // mush; stretch it to the full range, clipping the extremes as CS6 does.
    let count = n as f32;
    let mean = field.iter().sum::<f32>() / count;
    let spread = (field.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / count)
        .sqrt()
        .max(f32::EPSILON);
    let contrast = 0.55 + (variance / 64.0).sqrt() * 0.95;
    // Two and a bit deviations either side of the mean fills the range.
    let stretch = contrast / (4.4 * spread);

    for c in 0..planes {
        let (fg, bg) = (color_a[c] as f32, color_b[c] as f32);
        buf.data[c * n..(c + 1) * n]
            .par_iter_mut()
            .zip(field.par_iter())
            .for_each(|(out, &v)| {
                let v = (0.5 + (v - mean) * stretch).clamp(0.0, 1.0);
                *out = (bg + (fg - bg) * v) as u8;
            });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: [u8; 3] = [0, 0, 0];
    const WHITE: [u8; 3] = [255, 255, 255];

    fn plane(w: u32, h: u32, alpha: u8) -> PixelBuffer {
        let mut buf = PixelBuffer::new(w, h, 4);
        let n = buf.pixel_count();
        buf.data[3 * n..].fill(alpha);
        buf
    }

    fn render(w: u32, h: u32, variance: f64, strength: f64, seed: u64) -> PixelBuffer {
        let mut buf = plane(w, h, 255);
        fibers(&mut buf, variance, strength, WHITE, BLACK, seed).unwrap();
        buf
    }

    /// Sum of |Δ| on the red plane between each pixel and its neighbour
    /// `(dx, dy)` away.
    fn travel(buf: &PixelBuffer, dx: usize, dy: usize) -> u64 {
        let (w, h) = (buf.width as usize, buf.height as usize);
        let mut total = 0;
        for y in 0..h - dy {
            for x in 0..w - dx {
                let a = buf.data[y * w + x];
                let b = buf.data[(y + dy) * w + x + dx];
                total += a.abs_diff(b) as u64;
            }
        }
        total
    }

    #[test]
    fn fibers_are_deterministic_per_seed() {
        assert_eq!(
            render(48, 48, 32.0, 4.0, 7).data,
            render(48, 48, 32.0, 4.0, 7).data
        );
        assert_ne!(
            render(48, 48, 32.0, 4.0, 7).data,
            render(48, 48, 32.0, 4.0, 8).data
        );
    }

    #[test]
    fn fibers_blend_between_the_two_colours_and_keep_alpha() {
        let (a, b) = ([200, 150, 100], [250, 240, 230]);
        let mut buf = plane(64, 64, 123);
        fibers(&mut buf, 32.0, 8.0, a, b, 3).unwrap();
        let n = buf.pixel_count();
        for c in 0..3 {
            let (lo, hi) = (a[c].min(b[c]), a[c].max(b[c]));
            assert!(buf.data[c * n..(c + 1) * n]
                .iter()
                .all(|v| (lo..=hi).contains(v)));
        }
        assert!(buf.data[3 * n..].iter().all(|&v| v == 123));
    }

    #[test]
    fn zero_variance_is_the_even_blend() {
        let mut buf = plane(8, 8, 255);
        fibers(&mut buf, 0.0, 4.0, [200, 0, 10], [100, 255, 11], 1).unwrap();
        let n = buf.pixel_count();
        assert!(buf.data[..n].iter().all(|&v| v == 150));
        assert!(buf.data[n..2 * n].iter().all(|&v| v == 127));
        assert!(buf.data[2 * n..3 * n].iter().all(|&v| v == 10));
    }

    /// Strands stand beside each other with a clean edge rather than fading
    /// into one another, which is what holds up at 500 %.
    #[test]
    fn fibers_have_hard_edges_between_strands() {
        let buf = render(256, 256, 32.0, 4.0, 3);
        let w = 256;
        let steep = (0..256)
            .flat_map(|y| (0..255).map(move |x| (y, x)))
            .filter(|&(y, x)| buf.data[y * w + x].abs_diff(buf.data[y * w + x + 1]) > 64)
            .count();
        assert!(
            steep * 5 > 256 * 255,
            "only {steep} sideways steps are a clean jump"
        );
    }

    #[test]
    fn fibers_run_down_the_picture_not_across_it() {
        let buf = render(128, 128, 12.0, 4.0, 5);
        let (across, down) = (travel(&buf, 1, 0), travel(&buf, 0, 1));
        assert!(across > down * 4, "{across} across vs {down} down");
    }

    #[test]
    fn fibers_break_up_as_variance_climbs() {
        let low = travel(&render(96, 192, 8.0, 4.0, 11), 0, 1);
        let high = travel(&render(96, 192, 48.0, 4.0, 11), 0, 1);
        assert!(high > low * 2, "{low} at 8 vs {high} at 48");
    }

    #[test]
    fn fibers_stretch_with_strength() {
        let short = travel(&render(64, 128, 48.0, 1.0, 5), 0, 1);
        let long = travel(&render(64, 128, 48.0, 48.0, 5), 0, 1);
        assert!(long < short, "{long} at 48 vs {short} at 1");
    }

    #[test]
    fn fibers_reach_both_colours() {
        let buf = render(128, 128, 12.0, 4.0, 2);
        let red = &buf.data[..buf.pixel_count()];
        assert!(*red.iter().min().unwrap() < 16, "no fibre went dark");
        assert!(*red.iter().max().unwrap() > 239, "no fibre went light");
    }

    #[test]
    fn fibers_params_are_validated_and_buffer_untouched() {
        let base = plane(8, 8, 200);
        for (variance, strength) in [
            (-1.0, 4.0),
            (65.0, 4.0),
            (16.0, 0.0),
            (16.0, 65.0),
            (f64::NAN, 4.0),
            (16.0, f64::NAN),
        ] {
            let mut buf = base.clone();
            let err = fibers(&mut buf, variance, strength, BLACK, WHITE, 1).unwrap_err();
            assert!(
                matches!(err, FilterError::InvalidParams(_)),
                "variance {variance} strength {strength} should be rejected"
            );
            assert_eq!(buf.data, base.data);
        }
    }
}
