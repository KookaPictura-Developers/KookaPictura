//! Palette reduction and dithering for Save for Web.

use std::collections::HashMap;

use super::{ColorReduction, Dither, Indexed, PaletteOptions};

/// The 216 web-safe colours: every combination of 0x00, 0x33, … 0xFF.
pub fn web_safe_palette() -> Vec<[u8; 3]> {
    let steps = [0u8, 0x33, 0x66, 0x99, 0xcc, 0xff];
    let mut out = Vec::with_capacity(216);
    for r in steps {
        for g in steps {
            for b in steps {
                out.push([r, g, b]);
            }
        }
    }
    out
}

/// Reduce RGBA8888 to an indexed image as `options` describe.
pub fn quantize(rgba: &[u8], width: u32, height: u32, options: PaletteOptions) -> Indexed {
    let n = (width as usize) * (height as usize);
    let colors = usize::from(options.colors.clamp(2, 256));
    // Lay every pixel over the matte; with transparency, the mostly clear
    // ones become the transparent entry instead.
    let mut transparent = vec![false; n];
    let mut rgb = Vec::with_capacity(n);
    for (i, px) in rgba.as_chunks::<4>().0.iter().take(n).enumerate() {
        if options.transparency && px[3] < 128 {
            transparent[i] = true;
        }
        let a = f32::from(px[3]) / 255.0;
        let over = |c: u8, m: u8| f32::from(c) * a + f32::from(m) * (1.0 - a);
        rgb.push([
            over(px[0], options.matte[0]),
            over(px[1], options.matte[1]),
            over(px[2], options.matte[2]),
        ]);
    }
    let has_clear = transparent.iter().any(|&t| t);
    let budget = if has_clear { colors - 1 } else { colors }.max(1);
    let mut palette: Vec<[u8; 3]> = match options.reduction {
        ColorReduction::Restrictive => web_safe_palette(),
        ColorReduction::BlackAndWhite => vec![[0, 0, 0], [255, 255, 255]],
        ColorReduction::Grayscale => {
            let steps = budget.max(2);
            (0..steps)
                .map(|i| {
                    let v = (i * 255 / (steps - 1)) as u8;
                    [v, v, v]
                })
                .collect()
        }
        method => median_cut(&rgb, &transparent, budget, method),
    };
    // The fixed palettes ignore the table size, as CS6's do; only the 256
    // entries a table holds bound them.
    palette.truncate(256 - usize::from(has_clear));
    let indices = map_pixels(&rgb, &transparent, width as usize, &palette, options);
    let mut out: Vec<[u8; 4]> = palette.iter().map(|c| [c[0], c[1], c[2], 255]).collect();
    if has_clear {
        out.push([options.matte[0], options.matte[1], options.matte[2], 0]);
    }
    Indexed {
        width,
        height,
        palette: out,
        indices,
    }
}

/// Median cut over the distinct colours (weighted by count) of the opaque
/// pixels; when they already fit the budget they are the palette.
fn median_cut(
    rgb: &[[f32; 3]],
    transparent: &[bool],
    budget: usize,
    method: ColorReduction,
) -> Vec<[u8; 3]> {
    let mut counts: HashMap<[u8; 3], u32> = HashMap::new();
    for (c, &clear) in rgb.iter().zip(transparent) {
        if !clear {
            *counts.entry(c.map(|v| v.round() as u8)).or_default() += 1;
        }
    }
    let mut colors: Vec<([u8; 3], u32)> = counts.into_iter().collect();
    colors.sort_unstable();
    if colors.is_empty() {
        return vec![[0, 0, 0]];
    }
    if colors.len() <= budget {
        return colors.into_iter().map(|(c, _)| c).collect();
    }
    // Channel weights: Perceptual and Selective judge spread as the eye does.
    let weight = match method {
        ColorReduction::Adaptive => [1.0, 1.0, 1.0],
        _ => [0.30, 0.59, 0.11],
    };
    let mut boxes: Vec<Vec<([u8; 3], u32)>> = vec![colors];
    while boxes.len() < budget {
        let score = |b: &Vec<([u8; 3], u32)>| -> (f32, usize) {
            if b.len() < 2 {
                return (-1.0, 0);
            }
            let (channel, range) = (0..3)
                .map(|ch| {
                    let lo = b.iter().map(|(c, _)| c[ch]).min().unwrap_or(0);
                    let hi = b.iter().map(|(c, _)| c[ch]).max().unwrap_or(0);
                    (ch, f32::from(hi - lo) * weight[ch])
                })
                .fold(
                    (0, -1.0),
                    |best, cur| if cur.1 > best.1 { cur } else { best },
                );
            let population: u32 = b.iter().map(|(_, n)| n).sum();
            let s = if method == ColorReduction::Selective {
                range * (population as f32).sqrt()
            } else {
                range
            };
            (s, channel)
        };
        let Some((pick, (best, channel))) = boxes
            .iter()
            .map(score)
            .enumerate()
            .max_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
        else {
            break;
        };
        if best <= 0.0 {
            break;
        }
        let mut b = boxes.swap_remove(pick);
        b.sort_unstable_by_key(|(c, _)| c[channel]);
        // Split at the population median.
        let total: u32 = b.iter().map(|(_, n)| n).sum();
        let mut seen = 0;
        let mut at = 1;
        for (i, (_, n)) in b.iter().enumerate() {
            seen += n;
            if seen * 2 >= total {
                at = (i + 1).clamp(1, b.len() - 1);
                break;
            }
        }
        let upper = b.split_off(at);
        boxes.push(b);
        boxes.push(upper);
    }
    let mut palette: Vec<[u8; 3]> = boxes
        .iter()
        .map(|b| {
            let total: f64 = b.iter().map(|(_, n)| f64::from(*n)).sum();
            let mean = |ch: usize| {
                (b.iter()
                    .map(|(c, n)| f64::from(c[ch]) * f64::from(*n))
                    .sum::<f64>()
                    / total)
                    .round() as u8
            };
            [mean(0), mean(1), mean(2)]
        })
        .collect();
    palette.sort_unstable();
    palette.dedup();
    palette
}

/// A fixed value in `0.0..1.0` per pixel, so Noise dither is repeatable.
fn noise(x: usize, y: usize) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9e37_79b9) ^ (y as u32).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.0
}

const BAYER: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

fn map_pixels(
    rgb: &[[f32; 3]],
    transparent: &[bool],
    width: usize,
    palette: &[[u8; 3]],
    options: PaletteOptions,
) -> Vec<u8> {
    let clear = palette.len() as u8;
    let amount = f32::from(options.amount.min(100)) / 100.0;
    let mut nearest_cache: HashMap<[u8; 3], u8> = HashMap::new();
    let mut nearest = |c: [f32; 3]| -> u8 {
        let key = c.map(|v| v.round().clamp(0.0, 255.0) as u8);
        *nearest_cache.entry(key).or_insert_with(|| {
            let mut best = (u32::MAX, 0u8);
            for (i, p) in palette.iter().enumerate() {
                let d: u32 = (0..3)
                    .map(|ch| {
                        let e = i32::from(key[ch]) - i32::from(p[ch]);
                        (e * e) as u32
                    })
                    .sum();
                if d < best.0 {
                    best = (d, i as u8);
                }
            }
            best.1
        })
    };
    let mut work = rgb.to_vec();
    let mut out = vec![0u8; rgb.len()];
    // The ordered and noise offsets span a typical palette step.
    let spread = 64.0 * amount;
    for i in 0..rgb.len() {
        if transparent[i] {
            out[i] = clear;
            continue;
        }
        let (x, y) = (i % width, i / width);
        let mut c = work[i];
        match options.dither {
            Dither::Pattern => {
                let offset = (BAYER[y % 4][x % 4] + 0.5) / 16.0 - 0.5;
                c = c.map(|v| v + offset * spread);
            }
            Dither::Noise => {
                let offset = noise(x, y) - 0.5;
                c = c.map(|v| v + offset * spread);
            }
            _ => {}
        }
        let index = nearest(c);
        out[i] = index;
        if options.dither == Dither::Diffusion && amount > 0.0 {
            let p = palette[usize::from(index)];
            let err = [0, 1, 2].map(|ch| (c[ch] - f32::from(p[ch])) * amount);
            let height = rgb.len() / width;
            for (dx, dy, w) in [(1i64, 0i64, 7.0), (-1, 1, 3.0), (0, 1, 5.0), (1, 1, 1.0)] {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < 0 || nx >= width as i64 || ny >= height as i64 {
                    continue;
                }
                let j = ny as usize * width + nx as usize;
                for ch in 0..3 {
                    work[j][ch] += err[ch] * w / 16.0;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(reduction: ColorReduction, colors: u16, dither: Dither) -> PaletteOptions {
        PaletteOptions {
            reduction,
            colors,
            dither,
            amount: 100,
            transparency: true,
            matte: [255, 255, 255],
        }
    }

    /// A 64 x 1 red-to-blue ramp, the last 8 pixels clear.
    fn ramp() -> Vec<u8> {
        (0..64u32)
            .flat_map(|i| {
                [
                    (255 - i * 4) as u8,
                    0,
                    (i * 4) as u8,
                    if i >= 56 { 0 } else { 255 },
                ]
            })
            .collect()
    }

    #[test]
    fn few_colours_are_kept_exactly() {
        let rgba = [255u8, 0, 0, 255, 0, 255, 0, 255, 255, 0, 0, 255];
        let q = quantize(
            &rgba,
            3,
            1,
            opts(ColorReduction::Selective, 256, Dither::None),
        );
        assert_eq!(q.palette.len(), 2);
        let colour = |i: usize| q.palette[usize::from(q.indices[i])];
        assert_eq!(colour(0), [255, 0, 0, 255]);
        assert_eq!(colour(1), [0, 255, 0, 255]);
        assert_eq!(q.indices[0], q.indices[2]);
    }

    #[test]
    fn the_table_size_and_transparency_are_honoured() {
        for method in [
            ColorReduction::Perceptual,
            ColorReduction::Selective,
            ColorReduction::Adaptive,
        ] {
            let q = quantize(&ramp(), 64, 1, opts(method, 8, Dither::None));
            assert!(q.palette.len() <= 8, "{method:?}: {}", q.palette.len());
            let clear = q.transparent_index().expect("a transparent entry");
            assert_eq!(usize::from(clear), q.palette.len() - 1);
            assert!(q.indices[56..].iter().all(|&i| i == clear));
            assert!(q.indices[..56].iter().all(|&i| i != clear));
        }
        let opaque = PaletteOptions {
            transparency: false,
            ..opts(ColorReduction::Adaptive, 8, Dither::None)
        };
        assert!(quantize(&ramp(), 64, 1, opaque)
            .transparent_index()
            .is_none());
    }

    #[test]
    fn fixed_palettes_and_dithers() {
        let web = quantize(
            &ramp(),
            64,
            1,
            opts(ColorReduction::Restrictive, 256, Dither::None),
        );
        assert!(web
            .palette
            .iter()
            .all(|c| c[3] == 0 || c[..3].iter().all(|v| v % 0x33 == 0)));
        let bw = quantize(
            &ramp(),
            64,
            1,
            opts(ColorReduction::BlackAndWhite, 2, Dither::Diffusion),
        );
        assert_eq!(&bw.palette[..2], &[[0, 0, 0, 255], [255, 255, 255, 255]]);
        // Diffusing a mid grey into black and white mixes both.
        let grey: Vec<u8> = (0..64).flat_map(|_| [128u8, 128, 128, 255]).collect();
        let mixed = quantize(
            &grey,
            8,
            8,
            opts(ColorReduction::BlackAndWhite, 2, Dither::Diffusion),
        );
        let whites = mixed.indices.iter().filter(|&&i| i == 1).count();
        assert!((24..=40).contains(&whites), "{whites} of 64 white");
        let flat = quantize(
            &grey,
            8,
            8,
            opts(ColorReduction::BlackAndWhite, 2, Dither::None),
        );
        assert!(
            flat.indices.iter().all(|&i| i == flat.indices[0]),
            "undithered is flat"
        );
        for dither in [Dither::Pattern, Dither::Noise] {
            let a = quantize(&grey, 8, 8, opts(ColorReduction::BlackAndWhite, 2, dither));
            let b = quantize(&grey, 8, 8, opts(ColorReduction::BlackAndWhite, 2, dither));
            assert_eq!(a, b, "{dither:?} is repeatable");
            assert!(
                a.indices.contains(&0) && a.indices.contains(&1),
                "{dither:?} mixes"
            );
        }
        let grays = quantize(
            &ramp(),
            64,
            1,
            opts(ColorReduction::Grayscale, 4, Dither::None),
        );
        assert!(grays
            .palette
            .iter()
            .all(|c| c[3] == 0 || (c[0] == c[1] && c[1] == c[2])));
    }
}
