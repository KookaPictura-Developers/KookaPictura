//! Neighbourhood filters: convolution kernels and separable blurs.
//!
//! Convolution runs on **premultiplied** colour. Blurring straight-alpha RGB
//! lets the colour of fully transparent pixels bleed into visible ones, which
//! shows up as dark halos around soft edges.

use crate::photorust::pixmap::{Pixmap, Rgba8};
use rayon::prelude::*;

/// Spread the brightest values about: every pixel takes the largest value
/// within `radius` of it, per channel.
///
/// A maximum over a box is separable the same way a blur is, so this is two
/// passes of one dimension each and costs `O(r)` per pixel rather than
/// `O(r²)`. Glowing Edges thickens its lines with it; Colored Pencil spreads
/// its detail mask out over the region the detail sits in.
pub(crate) fn dilate(source: &Pixmap, radius: i32) -> Pixmap {
    let pass = |input: &Pixmap, horizontal: bool| {
        let width = input.width() as i32;
        let height = input.height() as i32;
        let mut out = Pixmap::new(input.width(), input.height());
        let stride = out.stride();
        out.as_bytes_mut()
            .par_chunks_exact_mut(stride)
            .enumerate()
            .for_each(|(row, out)| {
                let y = row as i32;
                for x in 0..width {
                    let mut best = [0u8; 3];
                    for step in -radius..=radius {
                        let p = if horizontal {
                            input.get((x + step).clamp(0, width - 1), y)
                        } else {
                            input.get(x, (y + step).clamp(0, height - 1))
                        };
                        best[0] = best[0].max(p.r);
                        best[1] = best[1].max(p.g);
                        best[2] = best[2].max(p.b);
                    }
                    let i = x as usize * 4;
                    out[i..i + 3].copy_from_slice(&best);
                    out[i + 3] = 255;
                }
            });
        out
    };
    pass(&pass(source, true), false)
}

/// Gaussian blur with the given standard-deviation-like `radius`, in pixels.
///
/// Implemented as two 1-D passes; a 2-D Gaussian is separable, so this is
/// `O(r)` per pixel rather than `O(r²)`.
pub fn gaussian_blur(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 || pixmap.is_empty() {
        return;
    }
    let sigma = radius.max(0.01);
    // Three sigma captures ~99.7% of the kernel's mass; going wider costs time
    // for no visible change.
    let taps = (sigma * 3.0).ceil() as i32;
    let kernel = gaussian_kernel_1d(sigma, taps);

    pixmap.premultiply();
    blur_pass(pixmap, &kernel, taps, true);
    blur_pass(pixmap, &kernel, taps, false);
    pixmap.unpremultiply();
}

/// Gaussian blur through the active rendering backend.
///
/// Use this from anything the user waits on. [`gaussian_blur`] above stays the
/// CPU reference: the GPU backend falls back to it, and the parity tests
/// compare against it, so it must not itself dispatch or the two would recurse.
pub fn gaussian_blur_accelerated(pixmap: &mut Pixmap, radius: f32) {
    // Kooka's GPU filters live in pictura-render; this crate is the CPU path.
    gaussian_blur(pixmap, radius);
}

/// Shared with the GPU backend, which must use byte-identical weights or its
/// output will drift from the CPU reference.
pub(crate) fn gaussian_kernel_1d(sigma: f32, taps: i32) -> Vec<f32> {
    let mut k = Vec::with_capacity((taps * 2 + 1) as usize);
    let two_sigma_sq = 2.0 * sigma * sigma;
    for i in -taps..=taps {
        let x = i as f32;
        k.push((-(x * x) / two_sigma_sq).exp());
    }
    let sum: f32 = k.iter().sum();
    for v in k.iter_mut() {
        *v /= sum;
    }
    k
}

/// One separable pass. `horizontal` selects the axis.
fn blur_pass(pixmap: &mut Pixmap, kernel: &[f32], taps: i32, horizontal: bool) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let src = pixmap.clone();
    let src_ref = &src;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out_row)| {
            let y = y as i32;
            for x in 0..width {
                let mut acc = [0.0f32; 4];
                for (i, &w) in kernel.iter().enumerate() {
                    let d = i as i32 - taps;
                    let (sx, sy) = if horizontal {
                        ((x + d).clamp(0, width - 1), y)
                    } else {
                        (x, (y + d).clamp(0, height - 1))
                    };
                    let p = src_ref.get(sx, sy);
                    acc[0] += p.r as f32 * w;
                    acc[1] += p.g as f32 * w;
                    acc[2] += p.b as f32 * w;
                    acc[3] += p.a as f32 * w;
                }
                let i = x as usize * 4;
                for c in 0..4 {
                    out_row[i + c] = (acc[c].clamp(0.0, 255.0) + 0.5) as u8;
                }
            }
        });
}

/// Box blur — a flat kernel. Cheaper than Gaussian; used for live previews.
pub fn box_blur(pixmap: &mut Pixmap, radius: u32) {
    if radius == 0 || pixmap.is_empty() {
        return;
    }
    let taps = radius as i32;
    let n = (taps * 2 + 1) as f32;
    let kernel = vec![1.0 / n; (taps * 2 + 1) as usize];

    pixmap.premultiply();
    blur_pass(pixmap, &kernel, taps, true);
    blur_pass(pixmap, &kernel, taps, false);
    pixmap.unpremultiply();
}

/// Average every pixel within `radius` — a *circular* window, not a square.
///
/// This is the shape a lens actually throws an out-of-focus point into, which
/// is why it is the blur Smart Sharpen removes when you tell it the softness
/// came from a lens. A Gaussian's long tail and a box's corners both give the
/// wrong halo when you sharpen against them.
///
/// Done with a summed-area table, because a disc is not separable: written the
/// obvious way it would cost `(2r+1)²` reads per pixel, and CS6 allows a
/// radius of 64. A disc is a stack of horizontal spans, and a table of running
/// sums answers "what is in this span" in constant time, so a pixel costs
/// `O(r)` — one span per row of the disc.
pub fn disc_blur(pixmap: &mut Pixmap, radius: u32) {
    if radius == 0 || pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let reach = radius as i32;

    pixmap.premultiply();

    // (height + 1) × (width + 1) so that a span sum needs no bounds test, and
    // `u32` because the largest total a channel can reach is 255 × the pixel
    // count, which stays inside it for any image this program can open.
    let stride_sat = (width + 1) * 4;
    let mut sat = vec![0u32; (height + 1) * stride_sat];
    {
        let src = pixmap.as_bytes();
        for y in 0..height {
            let above = y * stride_sat;
            let here = (y + 1) * stride_sat;
            let mut running = [0u32; 4];
            for x in 0..width {
                let i = (y * width + x) * 4;
                for c in 0..4 {
                    running[c] += src[i + c] as u32;
                    sat[here + (x + 1) * 4 + c] = sat[above + (x + 1) * 4 + c] + running[c];
                }
            }
        }
    }

    // Half-widths of the disc, one per row offset, worked out once.
    let spans: Vec<i32> = (-reach..=reach)
        .map(|dy| (((reach * reach - dy * dy) as f32).sqrt()) as i32)
        .collect();

    let sat_ref = &sat;
    let spans_ref = &spans;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width as i32 {
                let mut total = [0u64; 4];
                let mut count = 0u64;
                for (index, &half) in spans_ref.iter().enumerate() {
                    let yy = y + index as i32 - reach;
                    if yy < 0 || yy >= height as i32 {
                        continue;
                    }
                    let x0 = (x - half).max(0) as usize;
                    let x1 = (x + half).min(width as i32 - 1) as usize;
                    let top = yy as usize * stride_sat;
                    let bottom = (yy as usize + 1) * stride_sat;
                    for c in 0..4 {
                        // Grouped so that neither half goes negative on the
                        // way: the two added corners always outweigh the two
                        // subtracted ones, but `a - b - d` on its own need
                        // not, and these are unsigned.
                        let plus = sat_ref[bottom + (x1 + 1) * 4 + c] as u64
                            + sat_ref[top + x0 * 4 + c] as u64;
                        let minus = sat_ref[bottom + x0 * 4 + c] as u64
                            + sat_ref[top + (x1 + 1) * 4 + c] as u64;
                        total[c] += plus - minus;
                    }
                    count += (x1 - x0 + 1) as u64;
                }
                let i = x as usize * 4;
                let n = count.max(1);
                for c in 0..4 {
                    out[i + c] = (total[c] / n) as u8;
                }
            }
        });

    pixmap.unpremultiply();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::photorust::pixmap::Rgba8;

    #[test]
    fn blur_of_a_flat_image_is_unchanged() {
        let color = Rgba8::new(120, 130, 140, 255);
        let mut pm = Pixmap::filled(16, 16, color);
        gaussian_blur(&mut pm, 3.0);
        // Clamp-to-edge sampling means even border pixels see only `color`.
        for y in 0..16 {
            for x in 0..16 {
                let p = pm.get(x, y);
                assert!(
                    (p.r as i32 - 120).abs() <= 1
                        && (p.g as i32 - 130).abs() <= 1
                        && (p.b as i32 - 140).abs() <= 1,
                    "({},{}) drifted to {:?}",
                    x,
                    y,
                    p
                );
            }
        }
    }

    #[test]
    fn blur_spreads_a_single_dot() {
        let mut pm = Pixmap::new(9, 9);
        pm.set(4, 4, Rgba8::WHITE);
        gaussian_blur(&mut pm, 2.0);
        // Energy moved outward: neighbours are no longer empty...
        assert!(pm.get(3, 4).a > 0, "blur did not spread");
        // ...and the centre gave some up.
        assert!(pm.get(4, 4).a < 255);
    }

    #[test]
    fn blur_does_not_bleed_color_from_transparent_pixels() {
        // A red dot on a transparent field. If the blur ran on straight alpha,
        // the transparent (0,0,0,0) neighbours would darken the result.
        let mut pm = Pixmap::new(9, 9);
        pm.set(4, 4, Rgba8::new(255, 0, 0, 255));
        gaussian_blur(&mut pm, 1.5);
        let p = pm.get(4, 4);
        assert!(p.r > 200, "red channel darkened to {}", p.r);
        assert!(p.g < 40 && p.b < 40, "color bled: {:?}", p);
    }

    #[test]
    fn zero_radius_blur_is_a_no_op() {
        let mut pm = Pixmap::filled(4, 4, Rgba8::new(1, 2, 3, 255));
        let before = pm.as_bytes().to_vec();
        gaussian_blur(&mut pm, 0.0);
        box_blur(&mut pm, 0);
        assert_eq!(pm.as_bytes(), &before[..]);
    }

    #[test]
    fn box_blur_matches_flat_input() {
        let mut pm = Pixmap::filled(8, 8, Rgba8::new(50, 60, 70, 255));
        box_blur(&mut pm, 2);
        let p = pm.get(4, 4);
        assert!((p.r as i32 - 50).abs() <= 1 && (p.b as i32 - 70).abs() <= 1);
    }
}

/// Replace every pixel with the average of the whole image — Filter ▸ Blur ▸
/// Average.
///
/// Not a convolution at all despite living here: it is one mean over the
/// region, which is why it takes no radius. Alpha is averaged with the
/// colour so a partly transparent layer flattens to one even wash rather
/// than developing edges where its coverage changed.
pub fn average(pixmap: &mut Pixmap) {
    let count = (pixmap.width() as u64) * (pixmap.height() as u64);
    if count == 0 {
        return;
    }
    let (mut r, mut g, mut b, mut a) = (0u64, 0u64, 0u64, 0u64);
    for y in 0..pixmap.height() as i32 {
        for x in 0..pixmap.width() as i32 {
            let px = pixmap.get(x, y);
            r += px.r as u64;
            g += px.g as u64;
            b += px.b as u64;
            a += px.a as u64;
        }
    }
    let mean = Rgba8::new(
        (r / count) as u8,
        (g / count) as u8,
        (b / count) as u8,
        (a / count) as u8,
    );
    for y in 0..pixmap.height() as i32 {
        for x in 0..pixmap.width() as i32 {
            pixmap.set(x, y, mean);
        }
    }
}

/// Smear the image along a line — Filter ▸ Blur ▸ Motion Blur.
///
/// `angle` is in degrees anticlockwise from the horizontal and `distance` is
/// the length of the smear in pixels. Sampled along the line rather than
/// built as a 2-D kernel: the kernel would be mostly zeroes, and the cost
/// would grow with the square of the distance instead of with the distance.
pub fn motion_blur(pixmap: &mut Pixmap, angle: f32, distance: f32) {
    let steps = distance.max(0.0).round() as i32;
    if steps < 1 || pixmap.is_empty() {
        return;
    }
    let radians = angle.to_radians();
    let (dx, dy) = (radians.cos(), -radians.sin());

    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let (mut r, mut g, mut b, mut a, mut n) = (0f32, 0f32, 0f32, 0f32, 0f32);
                // Centred on the pixel, so the smear runs equally both ways
                // and the image does not appear to shift as the distance is
                // raised.
                for step in -steps / 2..=steps / 2 {
                    let sx = (x as f32 + dx * step as f32).round() as i32;
                    let sy = (y as f32 + dy * step as f32).round() as i32;
                    if sx < 0 || sy < 0 || sx >= width || sy >= height {
                        continue;
                    }
                    let px = src.get(sx, sy);
                    r += px.r as f32;
                    g += px.g as f32;
                    b += px.b as f32;
                    a += px.a as f32;
                    n += 1.0;
                }
                if n > 0.0 {
                    let i = x as usize * 4;
                    out[i] = (r / n).round() as u8;
                    out[i + 1] = (g / n).round() as u8;
                    out[i + 2] = (b / n).round() as u8;
                    out[i + 3] = (a / n).round() as u8;
                }
            }
        });
}

/// The median of every pixel within `radius` — Filter ▸ Noise ▸ Median.
///
/// Unlike a mean, a median throws away outliers rather than smearing them
/// about, which is why it is the tool for a speck of dust or a stuck pixel:
/// one wrong value among a few hundred cannot move the middle one. At larger
/// radii it flattens the picture into poster-like patches, which is the look
/// CS6's reference image shows.
///
/// Built on the same sliding histogram as [`surface_blur`], and for the same
/// reason: the obvious way costs `(2r+1)²` reads per pixel and CS6 allows a
/// radius of 100. Sliding the window costs `O(r)` and the median is then read
/// off by walking the tally until half the window is behind you.
pub fn median_filter(pixmap: &mut Pixmap, radius: u32) {
    if radius == 0 || pixmap.is_empty() {
        return;
    }
    let medians = median_of(pixmap, radius, radius);
    pixmap.as_bytes_mut().copy_from_slice(medians.as_bytes());
}

/// Filter ▸ Noise ▸ Dust & Scratches.
///
/// The median again, but only where the pixel is far enough from it to be
/// worth suspecting. `threshold` is how different a pixel has to be before it
/// is treated as a speck rather than as detail — at 0 every pixel is replaced
/// and this is exactly Median, and at 255 nothing is, which is what makes the
/// slider a way to keep the picture while losing the dust.
pub fn dust_and_scratches(pixmap: &mut Pixmap, radius: u32, threshold: u32) {
    if radius == 0 || pixmap.is_empty() {
        return;
    }
    let medians = median_of(pixmap, radius, radius);
    let limit = threshold.min(255) as i32;
    let median_bytes = medians.as_bytes();

    for (out, &median) in pixmap.as_bytes_mut().iter_mut().zip(median_bytes) {
        if (*out as i32 - median as i32).abs() > limit {
            *out = median;
        }
    }
}

/// The median of each pixel's neighbourhood, as its own image. The window
/// reaches `across` columns either side and `down` rows above and below.
///
/// Shared by Median, Dust & Scratches and Paint Daubs, which differ only in
/// what they do with the answer.
pub(crate) fn median_of(source: &Pixmap, across: u32, down: u32) -> Pixmap {
    let width = source.width() as i32;
    let height = source.height() as i32;
    let reach = across as i32;
    let reach_y = down as i32;

    let mut out_map = source.clone();
    let stride = out_map.stride();

    out_map
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            let top = (y - reach_y).max(0);
            let bottom = (y + reach_y).min(height - 1);

            let mut hist = [[0u32; 256]; 4];
            let column = |hist: &mut [[u32; 256]; 4], x: i32, add: bool| {
                if x < 0 || x >= width {
                    return;
                }
                for yy in top..=bottom {
                    let line = source.row(yy as u32);
                    let i = x as usize * 4;
                    for c in 0..4 {
                        let bin = &mut hist[c][line[i + c] as usize];
                        if add {
                            *bin += 1;
                        } else {
                            *bin -= 1;
                        }
                    }
                }
            };

            for x in 0..=reach.min(width - 1) {
                column(&mut hist, x, true);
            }

            // Every column of the window holds the same number of rows, so the
            // count is the same for all four channels and is worked out once.
            for x in 0..width {
                let left = (x - reach).max(0);
                let right = (x + reach).min(width - 1);
                let total = (right - left + 1) as u32 * (bottom - top + 1) as u32;
                let target = total / 2;

                let i = x as usize * 4;
                for c in 0..4 {
                    let mut seen = 0u32;
                    let mut value = 255usize;
                    for (bin, &n) in hist[c].iter().enumerate() {
                        seen += n;
                        if seen > target {
                            value = bin;
                            break;
                        }
                    }
                    out[i + c] = value as u8;
                }

                column(&mut hist, x - reach, false);
                column(&mut hist, x + reach + 1, true);
            }
        });

    out_map
}

/// Blur flat areas while leaving edges alone — Filter ▸ Blur ▸ Surface Blur.
///
/// A neighbour only counts if it is within `threshold` of the pixel being
/// worked on, so a region blurs within itself but never across a boundary
/// into something a different colour. That is the whole difference between
/// this and a box blur, and it is why it cannot be separated into two passes
/// the way an ordinary blur can: which neighbours count depends on the centre
/// pixel, so the horizontal pass would not know what the vertical one wanted.
///
/// It is done with a **sliding histogram** rather than by visiting every
/// neighbour of every pixel. Written the obvious way this costs `(2r+1)²`
/// reads per pixel — at CS6's largest radius that is forty thousand, which on
/// a photograph is minutes of frozen application, and is how this first
/// shipped. Instead each row keeps a histogram of the window's contents and
/// slides it one column at a time, so a pixel costs `O(r)` to move the window
/// plus `O(threshold)` to add up the bins in range.
///
/// On a 2000×1500 image at threshold 15, that is the difference between 2.4s
/// and 0.04s at radius 5, and between about thirteen minutes and 0.35s at
/// radius 100.
///
/// The histogram is per channel, which means the channels also decide
/// independently whether a neighbour is near enough to count. That is what
/// Photoshop does, and it is the price of the histogram — a joint test across
/// R, G and B cannot be answered from three separate tallies.
pub fn surface_blur(pixmap: &mut Pixmap, radius: u32, threshold: u32) {
    if radius == 0 || pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let reach = radius as i32;
    let limit = threshold.max(1) as i32;

    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            let top = (y - reach).max(0);
            let bottom = (y + reach).min(height - 1);

            // One tally of values per channel over the window, which starts
            // at the left edge and is slid rightwards a column at a time.
            let mut hist = [[0u32; 256]; 4];
            let add_column = |hist: &mut [[u32; 256]; 4], x: i32| {
                if x < 0 || x >= width {
                    return;
                }
                for yy in top..=bottom {
                    let row = src.row(yy as u32);
                    let i = x as usize * 4;
                    for c in 0..4 {
                        hist[c][row[i + c] as usize] += 1;
                    }
                }
            };
            let remove_column = |hist: &mut [[u32; 256]; 4], x: i32| {
                if x < 0 || x >= width {
                    return;
                }
                for yy in top..=bottom {
                    let row = src.row(yy as u32);
                    let i = x as usize * 4;
                    for c in 0..4 {
                        hist[c][row[i + c] as usize] -= 1;
                    }
                }
            };

            for x in 0..=reach.min(width - 1) {
                add_column(&mut hist, x);
            }

            let centre_row = src.row(y as u32);
            for x in 0..width {
                let i = x as usize * 4;
                for c in 0..4 {
                    let centre = centre_row[i + c] as i32;
                    let low = (centre - limit).max(0) as usize;
                    let high = (centre + limit).min(255) as usize;

                    // Only the bins near enough to the centre value count.
                    // The centre pixel is always one of them, so `count` is
                    // never zero and there is no division to guard.
                    let mut total = 0u32;
                    let mut count = 0u32;
                    for (value, &n) in hist[c][low..=high].iter().enumerate() {
                        total += n * (low + value) as u32;
                        count += n;
                    }
                    out[i + c] = (total / count.max(1)) as u8;
                }

                // Slide the window one to the right for the next pixel.
                remove_column(&mut hist, x - reach);
                add_column(&mut hist, x + reach + 1);
            }
        });
}

#[cfg(test)]
mod blur_tests {
    use super::*;

    fn split(width: u32, height: u32) -> Pixmap {
        let mut px = Pixmap::new(width, height);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let v = if (x as u32) < width / 2 { 0 } else { 255 };
                px.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        px
    }

    #[test]
    fn average_flattens_everything_to_one_colour() {
        let mut px = split(8, 4);
        average(&mut px);
        let first = px.get(0, 0);
        for y in 0..4 {
            for x in 0..8 {
                assert_eq!(px.get(x, y), first, "average left more than one colour");
            }
        }
        // Half black and half white average to the middle, not to either end.
        assert!(first.r > 100 && first.r < 155, "got {}", first.r);
    }

    #[test]
    fn motion_blur_smears_along_its_angle_and_not_across_it() {
        // A horizontal smear crosses a vertical edge, so the edge softens...
        let mut across = split(16, 8);
        motion_blur(&mut across, 0.0, 8.0);
        let softened = (0..16)
            .filter(|&x| {
                let v = across.get(x, 4).r;
                v > 20 && v < 235
            })
            .count();
        assert!(
            softened > 2,
            "a horizontal smear did not blur a vertical edge"
        );

        // ...while a vertical smear runs along it and leaves it hard.
        let mut along = split(16, 8);
        motion_blur(&mut along, 90.0, 8.0);
        assert_eq!(along.get(3, 4).r, 0, "the dark side should stay dark");
        assert_eq!(along.get(12, 4).r, 255, "the light side should stay light");
    }

    #[test]
    fn surface_blur_keeps_the_edge_a_box_blur_would_lose() {
        // A threshold below the black-to-white step means no neighbour across
        // the edge ever counts, so the edge survives — which is the whole
        // point of this filter over an ordinary blur.
        let mut kept = split(16, 8);
        surface_blur(&mut kept, 3, 40);
        assert_eq!(kept.get(7, 4).r, 0, "the edge bled despite the threshold");
        assert_eq!(kept.get(8, 4).r, 255, "the edge bled despite the threshold");

        // Raised past the step, it behaves like a blur again.
        let mut blurred = split(16, 8);
        surface_blur(&mut blurred, 3, 255);
        let v = blurred.get(7, 4).r;
        assert!(
            v > 0 && v < 255,
            "a wide-open threshold should blur, got {}",
            v
        );
    }

    #[test]
    fn a_disc_blur_is_round_not_square() {
        // The point of the disc: a single bright pixel spreads into a circle,
        // not the square a box blur would leave. A small radius, because one
        // pixel spread over a large disc averages down below a single level
        // and there would be nothing left to measure.
        let mut px = Pixmap::filled(41, 41, Rgba8::BLACK);
        px.set(20, 20, Rgba8::WHITE);
        disc_blur(&mut px, 4);

        // Three out along a row: inside a radius of four, so it catches the
        // bright pixel.
        let side = px.get(23, 20).r;
        // Three out along both axes at once, which is 4.24 away — outside the
        // disc, though well inside the square a box blur would use.
        let corner = px.get(23, 23).r;

        assert!(
            side > 0,
            "the disc did not spread the pixel sideways at all"
        );
        assert_eq!(
            corner, 0,
            "the blur reached into the corners, so it is a square"
        );
    }

    #[test]
    fn a_disc_blur_conserves_what_it_spreads() {
        // A summed-area table is easy to get off by one, and the symptom is a
        // slight overall lightening or darkening rather than anything you
        // would see in the shape. Total brightness must survive.
        let mut px = Pixmap::new(64, 48);
        for y in 0..48i32 {
            for x in 0..64i32 {
                let v = ((x * 13 + y * 29) % 251) as u8;
                px.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        let before: u64 = px.as_bytes().chunks_exact(4).map(|p| p[0] as u64).sum();
        disc_blur(&mut px, 6);
        let after: u64 = px.as_bytes().chunks_exact(4).map(|p| p[0] as u64).sum();

        let drift = (before as f64 - after as f64).abs() / before as f64;
        assert!(
            drift < 0.02,
            "the disc blur shifted total brightness by {:.1}%",
            drift * 100.0
        );
    }

    #[test]
    fn a_median_removes_a_speck_a_mean_would_only_spread() {
        // The whole reason this filter exists. One wrong pixel among a few
        // hundred cannot move the middle value at all, where an average would
        // smear it over its neighbourhood.
        let mut px = Pixmap::filled(21, 21, Rgba8::new(120, 120, 120, 255));
        px.set(10, 10, Rgba8::WHITE);

        let mut blurred = px.clone();
        box_blur(&mut blurred, 3);
        assert_ne!(
            blurred.get(11, 10).r,
            120,
            "a box blur should have spread the speck to its neighbour"
        );

        median_filter(&mut px, 3);
        assert_eq!(px.get(10, 10).r, 120, "the median did not remove the speck");
        assert_eq!(px.get(11, 10).r, 120, "the median spread the speck instead");
    }

    #[test]
    fn the_median_agrees_with_sorting_the_neighbourhood() {
        // The sliding window is the risk here, exactly as it is in Surface
        // Blur: a column dropped a step early still gives a plausible median,
        // just the wrong one, and only in some columns.
        fn directly(src: &Pixmap, radius: i32) -> Pixmap {
            let mut out = src.clone();
            let w = src.width() as i32;
            let h = src.height() as i32;
            for y in 0..h {
                for x in 0..w {
                    for c in 0..4 {
                        let mut window = Vec::new();
                        for yy in (y - radius).max(0)..=(y + radius).min(h - 1) {
                            for xx in (x - radius).max(0)..=(x + radius).min(w - 1) {
                                let p = src.get(xx, yy);
                                window.push([p.r, p.g, p.b, p.a][c]);
                            }
                        }
                        window.sort_unstable();
                        let mut p = out.get(x, y);
                        let middle = window[window.len() / 2];
                        match c {
                            0 => p.r = middle,
                            1 => p.g = middle,
                            2 => p.b = middle,
                            _ => p.a = middle,
                        }
                        out.set(x, y, p);
                    }
                }
            }
            out
        }

        // Not square, and not a multiple of any radius used.
        let mut src = Pixmap::new(29, 19);
        for y in 0..19i32 {
            for x in 0..29i32 {
                let v = ((x * 17 + y * 43) % 251) as u8;
                src.set(x, y, Rgba8::new(v, 255 - v, v / 3, 190 + (v % 66)));
            }
        }

        for radius in [1u32, 2, 5, 25] {
            let mut fast = src.clone();
            median_filter(&mut fast, radius);
            assert_eq!(
                fast.as_bytes(),
                directly(&src, radius as i32).as_bytes(),
                "the sliding window disagrees with sorting the neighbourhood at radius {}",
                radius
            );
        }
    }

    #[test]
    fn dust_and_scratches_keeps_what_is_within_its_threshold() {
        // The threshold is what separates it from Median: at zero it is the
        // median exactly, and raising it hands more of the picture back.
        let mut speckled = Pixmap::filled(21, 21, Rgba8::new(120, 120, 120, 255));
        speckled.set(10, 10, Rgba8::WHITE);
        // A quieter blemish, well within a threshold of 60.
        speckled.set(5, 5, Rgba8::new(150, 150, 150, 255));

        let mut wide = speckled.clone();
        dust_and_scratches(&mut wide, 3, 60);
        assert_eq!(
            wide.get(10, 10).r,
            120,
            "the speck survived a wide threshold"
        );
        assert_eq!(
            wide.get(5, 5).r,
            150,
            "a difference inside the threshold was removed anyway"
        );

        let mut none = speckled.clone();
        dust_and_scratches(&mut none, 3, 0);
        let mut median = speckled.clone();
        median_filter(&mut median, 3);
        assert_eq!(
            none.as_bytes(),
            median.as_bytes(),
            "at threshold zero this should be exactly Median"
        );
    }

    #[test]
    fn the_sliding_histogram_agrees_with_visiting_every_neighbour() {
        // Surface Blur slides a tally of the window's contents sideways
        // rather than re-reading every neighbour, which is what makes a large
        // radius affordable at all. A window slid wrong — a column dropped one
        // step early, or one added twice at an edge — still produces a
        // plausible blur, just not the right one, and only in some columns.
        // So it is checked against the definition it is an optimisation of.
        fn directly(src: &Pixmap, radius: i32, threshold: i32) -> Pixmap {
            let mut out = src.clone();
            let w = src.width() as i32;
            let h = src.height() as i32;
            for y in 0..h {
                for x in 0..w {
                    let centre = src.get(x, y);
                    let centre = [centre.r, centre.g, centre.b, centre.a];
                    let mut totals = [0u32; 4];
                    let mut counts = [0u32; 4];
                    for yy in (y - radius).max(0)..=(y + radius).min(h - 1) {
                        for xx in (x - radius).max(0)..=(x + radius).min(w - 1) {
                            let p = src.get(xx, yy);
                            for (c, value) in [p.r, p.g, p.b, p.a].into_iter().enumerate() {
                                if (value as i32 - centre[c] as i32).abs() <= threshold {
                                    totals[c] += value as u32;
                                    counts[c] += 1;
                                }
                            }
                        }
                    }
                    let v = |c: usize| (totals[c] / counts[c].max(1)) as u8;
                    out.set(x, y, Rgba8::new(v(0), v(1), v(2), v(3)));
                }
            }
            out
        }

        // Not square, and not a multiple of the radius, so an off-by-one at
        // either edge shows up.
        let mut src = Pixmap::new(37, 23);
        for y in 0..23i32 {
            for x in 0..37i32 {
                let v = ((x * 11 + y * 29) % 251) as u8;
                src.set(x, y, Rgba8::new(v, 255 - v, v / 2, 200 + (v % 56)));
            }
        }

        for (radius, threshold) in [(1u32, 10u32), (3, 30), (8, 60), (40, 20)] {
            let mut fast = src.clone();
            surface_blur(&mut fast, radius, threshold);
            let slow = directly(&src, radius as i32, threshold as i32);
            assert_eq!(
                fast.as_bytes(),
                slow.as_bytes(),
                "the sliding window disagrees with visiting every neighbour at radius {} \
                 threshold {}",
                radius,
                threshold
            );
        }
    }
}
