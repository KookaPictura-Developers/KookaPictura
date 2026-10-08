//! Palette Knife, Neon Glow, Film Grain (split from `artistic` for file size).

#[allow(unused_imports)]
use super::*;

/// How many Kuwahara passes Palette Knife lays. photocraft lays one; a second
/// grows the patches into the broader blobs CS6's reference shows.
pub(crate) const KNIFE_PASSES: usize = 2;

/// Filter ▸ Artistic ▸ Palette Knife: the picture spread with a knife.
///
/// Ported from photocraft's Filter Gallery (Brandon Thomas, MIT OR
/// Apache-2.0). Kuwahara passes lay the picture down in flat patches with hard
/// edges between them, half of each channel is then rounded onto a coarse
/// palette so neighbouring patches of like colour fuse, and Softness blurs the
/// joins.
///
/// * **Stroke Size** is the Kuwahara reach, a third of the slider in pixels.
/// * **Stroke Detail** is the palette: `2 + 3·detail` levels per channel.
/// * **Softness** is a Gaussian of half the slider in pixels; 0 leaves the
///   joins hard.
///
/// Alpha is left alone: spreading the picture does not change the layer's
/// shape.
pub fn palette_knife(pixmap: &mut Pixmap, size: u32, detail: u32, softness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*KNIFE_SIZE.start(), *KNIFE_SIZE.end());
    let detail = detail.clamp(*KNIFE_DETAIL.start(), *KNIFE_DETAIL.end());
    let softness = softness.clamp(*KNIFE_SOFTNESS.start(), *KNIFE_SOFTNESS.end());

    for _ in 0..KNIFE_PASSES {
        kuwahara(pixmap, (size / 3).max(1) as usize);
    }
    let steps = (1 + detail * 3) as f32;
    let palette: [u8; 256] = std::array::from_fn(|v| {
        let v = v as f32 / 255.0;
        let rounded = (v * steps).round() / steps;
        ((rounded * 0.5 + v * 0.5) * 255.0).round() as u8
    });
    for px in pixmap.as_bytes_mut().chunks_exact_mut(4) {
        for c in 0..3 {
            px[c] = palette[px[c] as usize];
        }
    }
    crate::photorust::convolve::gaussian_blur_accelerated(pixmap, softness as f32 * 0.5);
}

/// Kuwahara smoothing: each pixel takes the mean colour of whichever of the
/// four `(2h+1)²` boxes with it at a corner has the least brightness variance,
/// `h = ⌈reach/2⌉`. Summed-area tables make the cost independent of `reach`.
pub(crate) fn kuwahara(pixmap: &mut Pixmap, reach: usize) {
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let half = reach.div_ceil(2).max(1) as isize;
    // Five planes per pixel — r, g, b, brightness, brightness² — summed over
    // the rectangle above and left of it, with a zero row and column in front.
    let (sw, sh) = (w + 1, h + 1);
    let mut sums = vec![[0f64; 5]; sw * sh];
    let bytes = pixmap.as_bytes();
    for y in 0..h {
        let mut run = [0f64; 5];
        for x in 0..w {
            let p = &bytes[(y * w + x) * 4..];
            let (r, g, b) = (p[0] as f64, p[1] as f64, p[2] as f64);
            let l = 0.299 * r + 0.587 * g + 0.114 * b;
            for (acc, v) in run.iter_mut().zip([r, g, b, l, l * l]) {
                *acc += v;
            }
            let above = sums[y * sw + x + 1];
            sums[(y + 1) * sw + x + 1] = std::array::from_fn(|k| above[k] + run[k]);
        }
    }
    // Mean of the box centred on (cx, cy), clipped to the picture.
    let mean = |cx: isize, cy: isize| -> [f64; 5] {
        let x0 = (cx - half).clamp(0, w as isize) as usize;
        let x1 = (cx + half + 1).clamp(0, w as isize) as usize;
        let y0 = (cy - half).clamp(0, h as isize) as usize;
        let y1 = (cy + half + 1).clamp(0, h as isize) as usize;
        let n = ((x1 - x0) * (y1 - y0)).max(1) as f64;
        let s = |x: usize, y: usize| sums[y * sw + x];
        let (a, b, c, d) = (s(x1, y1), s(x0, y1), s(x1, y0), s(x0, y0));
        std::array::from_fn(|k| (a[k] - b[k] - c[k] + d[k]) / n)
    };
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let (x, y) = (x as isize, y as isize);
                let mut best = (f64::MAX, [0f64; 5]);
                for (dx, dy) in [(-half, -half), (half, -half), (-half, half), (half, half)] {
                    let q = mean(x + dx, y + dy);
                    let spread = q[4] - q[3] * q[3];
                    if spread < best.0 {
                        best = (spread, q);
                    }
                }
                for c in 0..3 {
                    px[c] = best.1[c].round().clamp(0.0, 255.0) as u8;
                }
            }
        });
}

/// Where on the Glow Brightness slider the lamp's light exactly reaches white.
///
/// Below it the picture lights up; above it the light carries the brightest
/// part *past* white and out the other side, and what was the lightest thing in
/// the frame comes back at the foreground colour. That fold is not a mistake to
/// be clamped away — it is the reason the same filter gives a green flower on a
/// blue ground at one end of the slider and a blue flower on a green ground at
/// the other, which is what CS6 does and what the reference shows.
pub(crate) const NEON_NEUTRAL: f32 = 25.0;

/// How much of the picture's own light the tube picks up and carries, as the
/// power the spread light is raised to.
///
/// A square rather than a straight line, and this is the number that decides
/// whether the filter looks like CS6's. The tube lights *the objects in the
/// picture* — Adobe's own wording — not the frame it stands in, so a dark
/// ground has to come back the colour it was rendered in and nothing else. Taken
/// straight, a ground at a quarter of the range still picks up a quarter of the
/// light and the whole picture goes off in the glow colour, which is a blue
/// photograph rather than a blue glow. Squared, that quarter becomes a
/// sixteenth and the ground stays where it belongs while the flowers light up
/// as hard as before.
pub(crate) const NEON_PICKUP: i32 = 2;

/// Filter ▸ Artistic ▸ Neon Glow: the picture lit by a tube of one colour.
///
/// Three things go in and CS6 asks the dialog for only one of them. **Glow
/// Color** is the swatch. The other two are the **document's foreground and
/// background colours**, which is what the picture is rendered between — so a
/// black-and-white pair gives the grey-and-neon look the filter is known for,
/// and a coloured foreground tints everything the tube does not reach. The
/// dialog does not ask because CS6 does not; the bridge fills them in, as it
/// does for Pointillize and Fibers.
///
/// What happens is one idea carried through:
///
/// 1. **The picture's own light is spread** by Glow Size, so a bright thing
///    lights what is near it. A *negative* size — CS6's slider runs to -24 —
///    lights the shadows instead, and the frame glows inwards from its dark
///    parts.
/// 2. **The lamp is turned up** by Glow Brightness, and the light is added to
///    what the picture already had.
/// 3. **What that comes to is folded back at white**. See [`NEON_NEUTRAL`]:
///    below the middle of the slider the picture lights up in the ordinary way,
///    and above it the brightest parts are driven past white and return as the
///    foreground colour. This is the whole of why the filter's two extremes
///    look like negatives of one another.
/// 4. **The result is rendered between the two swatches**, and carried towards
///    the glow colour by how lit it is.
///
/// Alpha is left alone: lighting the picture does not change the layer's shape.
///
/// No GPU path: per pixel over a blur, and the same round-trip argument as the
/// rest of the filter stack. The blur itself goes through the backend.
pub fn neon_glow(
    pixmap: &mut Pixmap,
    size: i32,
    brightness: u32,
    glow: Rgba8,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*NEON_SIZE.start(), *NEON_SIZE.end());
    let brightness = brightness.clamp(*NEON_BRIGHTNESS.start(), *NEON_BRIGHTNESS.end());
    let strength = brightness as f32 / NEON_NEUTRAL;

    // The lamp: the picture's own light, spread by Glow Size. Wound below
    // zero it is the shadows that light up instead.
    let mut lamp = lightness(pixmap, size < 0);
    crate::photorust::convolve::gaussian_blur_accelerated(&mut lamp, size.unsigned_abs() as f32);

    let w = pixmap.width() as i32;
    let stride = pixmap.stride();
    let lamp = &lamp;
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                let i = x as usize * 4;
                let own = luma(Rgba8::new(out[i], out[i + 1], out[i + 2], 255)) / 255.0;
                let lit = lamp.get(x, y).r as f32 / 255.0;
                // How much of the lamp's light lands here. See NEON_PICKUP:
                // what was already dark picks up almost none of it.
                let light = lit.powi(NEON_PICKUP) * strength;
                // What the picture had, plus what the lamp adds, folded back
                // where it runs past white.
                let exposed = own + light;
                let mix = (1.0 - (1.0 - exposed).abs()).clamp(0.0, 1.0);
                // The tube's colour goes where its light *stayed*. Both terms
                // are needed: the light alone would colour what the fold has
                // already driven past white and back to the foreground, which
                // is the one part of the picture that must come back
                // untinted.
                let tint = (mix * light).clamp(0.0, 1.0);
                for (c, (dark, pale)) in [
                    (foreground.r, background.r),
                    (foreground.g, background.g),
                    (foreground.b, background.b),
                ]
                .into_iter()
                .enumerate()
                {
                    let between = dark as f32 + (pale as f32 - dark as f32) * mix;
                    let tube = [glow.r, glow.g, glow.b][c] as f32;
                    let value = between + (tube - between) * tint;
                    out[i + c] = value.clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: lighting the picture does not change the
                // layer's shape.
            }
        });
}

/// How light each pixel is, as a picture in its own right — `r`, `g` and `b`
/// all carry the same brightness. `inverted` gives its negative, which is what
/// asks "how *dark* is it here" without a second code path.
pub(crate) fn lightness(pixmap: &Pixmap, inverted: bool) -> Pixmap {
    let w = pixmap.width() as i32;
    let mut out = Pixmap::new(pixmap.width(), pixmap.height());
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                let l = luma(pixmap.get(x, y)) as u8;
                let i = x as usize * 4;
                out[i..i + 3].copy_from_slice(&[if inverted { 255 - l } else { l }; 3]);
                out[i + 3] = 255;
            }
        });
    out
}

/// How restless the picture is around each pixel, as a picture in its own
/// right — the mean of how far brightness strays from its own local average
/// over a window of `reach`.
///
/// A variance would be the textbook answer and this is its cheaper cousin, the
/// mean absolute deviation. It answers the only question being asked of it —
/// *which of these four areas is the calmest* — with the same ordering, and it
/// stays in the 0..255 the rest of the engine is built around instead of
/// needing a plane of floats per channel.
pub(crate) fn roughness(pixmap: &Pixmap, reach: u32) -> Pixmap {
    let lum = lightness(pixmap, false);
    let stride = lum.stride();
    let mut mean = lum.clone();
    crate::photorust::convolve::box_blur(&mut mean, reach);
    let mut strays = lum;
    strays
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(mean.as_bytes().par_chunks_exact(stride))
        .for_each(|(out, mean)| {
            for (out, mean) in out.chunks_exact_mut(4).zip(mean.chunks_exact(4)) {
                let stray = (out[0] as i32 - mean[0] as i32).unsigned_abs() as u8;
                out[0..3].copy_from_slice(&[stray; 3]);
            }
        });
    crate::photorust::convolve::box_blur(&mut strays, reach);
    strays
}

/// Paint each pixel from whichever of its four brush loads the picture is
/// calmest over.
///
/// **Picked outright, not blended between.** Leaning on all four by how calm
/// each is looks like the gentler, better-behaved thing to do, and it undoes
/// the filter: the four loads either side of a boundary are equally calm, so
/// the lean is even and the edge averages into a smear, and on a flat surface
/// the blend of four overlapping means is just the picture again, smoothed.
/// What is left is the photograph. The whole of the brushwork — the flat facet,
/// the hard step from one to the next — is in the committing to one load and
/// discarding the other three, so that is what this does.
pub(crate) fn lay_the_paint(pixmap: &mut Pixmap, load: &Pixmap, calm: &Pixmap, reach: i32) {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                // The four areas with this pixel at a corner, named by where
                // their middles are. Clamped, so a pixel at the frame's edge
                // dips its brush inside the picture rather than off it.
                let i = x as usize * 4;
                let mine = luma(Rgba8::new(out[i], out[i + 1], out[i + 2], 255));
                let mut best = (u32::MAX, 0i32, 0i32);
                for (dx, dy) in [
                    (-reach, -reach),
                    (reach, -reach),
                    (-reach, reach),
                    (reach, reach),
                ] {
                    let (sx, sy) = ((x + dx).clamp(0, w - 1), (y + dy).clamp(0, h - 1));
                    let stray = calm.get(sx, sy).r as u32;
                    // Two loads can be exactly as calm as each other — one
                    // either side of a boundary usually are, and over a weave
                    // or a field of grass all four routinely are. Settle it on
                    // which came back with a colour nearer the pixel's own,
                    // which is the side of the boundary this pixel is on;
                    // otherwise the answer turns on the order they happen to
                    // be listed in and flickers from pixel to pixel.
                    let apart = (mine - luma(load.get(sx, sy))).abs() as u32;
                    let score = stray << 9 | apart.min(511);
                    if score < best.0 {
                        best = (score, sx, sy);
                    }
                }
                let colour = load.get(best.1, best.2);
                out[i] = colour.r;
                out[i + 1] = colour.g;
                out[i + 2] = colour.b;
                // Alpha stands: repainting the picture does not change the
                // layer's shape.
            }
        });
}

/// Stand every facet further from its neighbours — the body of the paint.
///
/// What is added back is what a blur of the brush's own scale takes away, which
/// on a picture the brush has just been over is the facets and the steps
/// between them and nothing else: there is no finer detail left for it to find.
/// So this raises the brushwork rather than the photograph, which is the
/// difference between paint laid on thickly and a sharpened snapshot.
pub(crate) fn raise_the_paint(pixmap: &mut Pixmap, scale: f32, amount: f32) {
    if amount <= 0.0 || pixmap.is_empty() {
        return;
    }
    let mut flattened = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut flattened, scale);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(flattened.as_bytes().par_chunks_exact(stride))
        .for_each(|(out, flat)| {
            for (out, flat) in out.chunks_exact_mut(4).zip(flat.chunks_exact(4)) {
                for c in 0..3 {
                    let lift =
                        ((out[c] as f32 - flat[c] as f32) * amount).clamp(-RELIEF_CAP, RELIEF_CAP);
                    out[c] = (out[c] as f32 + lift).clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}

/// How coarse the grain goes at the top of the Grain slider — the heaviest
/// clump of silver it can lay, in levels.
///
/// Over half the tonal range, which looks far too much written down and is not:
/// [`speck`] bunches its draws towards the middle, so the *typical* deposit is
/// nearer a third of this and only the rare clump gets anywhere near it. That
/// is the shape of the thing being reproduced — a fast film is mostly faint
/// with the odd heavy grain — and the number to compare against CS6 is what a
/// mid setting looks like, not what the arithmetic allows at the end.
pub(crate) const GRAIN_MAX: f32 = 140.0;

/// How gradually a value becomes a highlight once it is over the line.
///
/// The line itself runs the whole way: at Highlight Area 0 it sits at white, so
/// nothing in the picture is over it and the filter is its grain and nothing
/// else — which is CS6's default, and what its own screenshot of the default
/// shows — and at 20 it sits at black and everything is.
pub(crate) const HIGHLIGHT_SPREAD: f32 = 48.0;

/// How far towards white the lit part of the picture is carried at Intensity
/// 10, as a fraction of what is left between it and white.
pub(crate) const HIGHLIGHT_PULL: f32 = 0.5;

/// How much of its grain the lit part gives up, and how far the grain it keeps
/// drifts from grey into colour.
///
/// Both come straight out of CS6's one-line account of the filter — "a
/// smoother, more saturated pattern is added to the image's lighter areas" —
/// and they are what stops Film Grain being Add Noise with extra steps. Smooth
/// and coloured in the light, even and grey in the shadows and midtones.
pub(crate) const HIGHLIGHT_SMOOTHING: f32 = 0.7;

pub(crate) const HIGHLIGHT_COLOUR: f32 = 0.6;

/// Filter ▸ Artistic ▸ Film Grain: the picture as a fast film would have taken
/// it.
///
/// CS6 describes it in one line — "applies an even pattern to the shadow tones
/// and midtones; a smoother, more saturated pattern is added to the image's
/// lighter areas" — and both halves of that sentence are load-bearing. An even
/// pattern everywhere is Filter ▸ Noise ▸ Add Noise, which this is not.
///
/// * **Grain** is how coarse the silver is, in levels. It is laid as one offset
///   across all three channels rather than three separate ones, because that is
///   what grain in a film emulsion is: clumps of silver that are there or not
///   there, which lighten and darken without tinting.
/// * **Highlight Area** is how far down the range counts as "the lighter
///   areas". At 0 the line sits at white and nothing is above it, which is why
///   CS6's default settings come back as the picture with grain on it and
///   nothing else. Wound up, the line drops into the midtones and takes more
///   and more of the picture with it.
///
///   **Each channel is asked separately**, not the pixel's brightness, and this
///   is the single decision that makes the filter look like CS6's rather than
///   like a wash. A dark green — the grass behind these flowers is (40, 90, 30)
///   — has one channel over the line and two under it, so green alone is
///   carried up and what comes back is a *vivid* green, not a paler one.
///   Testing the brightness instead lifts all three together, which is the
///   definition of washing a colour out. It is also what "a more saturated
///   pattern in the lighter areas" means: the saturation is not added, it falls
///   out of asking each channel where it stands.
/// * **Intensity** is how hard that lighter part is then carried towards white.
///   It has nothing to work on until Highlight Area gives it something, which
///   is the pair's whole relationship and is worth knowing before wondering why
///   a slider at 10 is doing nothing.
///
/// Where the two overlap, the grain also goes *smoother* and *more coloured* —
/// a highlight on film is a thinner, finer deposit, and what grain is left in
/// it sits in the dye layers rather than in the silver.
///
/// The grain is seeded from each pixel's coordinates rather than from a RNG, so
/// a preview, the commit behind it and an undo/redo replay all show the same
/// film.
///
/// Alpha is left alone: exposing the picture differently does not change the
/// layer's shape.
///
/// No GPU path. It is the most shader-shaped operation in this module — per
/// pixel, no neighbourhood at all — but it is also one pass over the picture
/// doing a dozen operations per pixel, so the upload and the read straight back
/// would cost more than the arithmetic they carried. docs/gpu-migration.md has
/// the measurements.
pub fn film_grain(pixmap: &mut Pixmap, grain: u32, highlight_area: u32, intensity: u32) {
    if pixmap.is_empty() {
        return;
    }
    let grain = grain.clamp(*FILM_GRAIN.start(), *FILM_GRAIN.end());
    let highlight_area = highlight_area.clamp(*FILM_HIGHLIGHT.start(), *FILM_HIGHLIGHT.end());
    let intensity = intensity.clamp(*FILM_INTENSITY.start(), *FILM_INTENSITY.end());

    let coarseness = grain as f32 / *FILM_GRAIN.end() as f32 * GRAIN_MAX;
    // Where the highlights start. At white when the area is shut, so nothing
    // is one; at black when it is fully open, so everything is.
    let line = 255.0 * (1.0 - highlight_area as f32 / *FILM_HIGHLIGHT.end() as f32);
    let pull = intensity as f32 / *FILM_INTENSITY.end() as f32 * HIGHLIGHT_PULL;

    let w = pixmap.width() as i32;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                let i = x as usize * 4;
                let silver = speck(x, y, 0);
                for c in 0..3 {
                    // Where this channel — not this pixel — stands against the
                    // line.
                    let lit = fade(out[i + c] as f32, line, line + HIGHLIGHT_SPREAD);
                    // Thinner deposit in the light, so less of it and finer.
                    let amount = coarseness * (1.0 - HIGHLIGHT_SMOOTHING * lit);
                    // Grey in the shadows and midtones, drifting into the dye
                    // layers as the picture lightens.
                    let deposit = silver + lit * HIGHLIGHT_COLOUR * (speck(x, y, c + 1) - silver);
                    let value = out[i + c] as f32 + deposit * amount;
                    let exposed = value + (255.0 - value) * lit * pull;
                    out[i + c] = exposed.clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: exposing the picture differently does not
                // change the layer's shape.
            }
        });
}

/// One clump of silver, from -1 to 1.
///
/// Two draws averaged rather than one, which bunches the result towards the
/// middle. Grain that is uniformly anything between its extremes reads as
/// television static; a film's is mostly faint with the odd heavy clump, and
/// two draws is the cheapest thing that looks like that.
pub(crate) fn speck(x: i32, y: i32, layer: usize) -> f32 {
    let salt = layer as i32 * 977;
    noise(x + salt, y - salt) + noise(y * 3 + salt, x * 5 + salt) - 1.0
}

/// The canvas's tooth at a pixel, from -1 in a dip to 1 on a rise.
///
/// Value noise on a grid of [`TOOTH_SCALE`], carried smoothly between the
/// corners rather than held flat across each cell. The interpolation is the
/// whole point: noise taken straight off a grid is squares, and noise taken per
/// pixel is static, while a weave is neither — it has a size, and it runs into
/// itself.
pub(crate) fn weave(x: i32, y: i32) -> f32 {
    let (cx, cy) = (x.div_euclid(TOOTH_SCALE), y.div_euclid(TOOTH_SCALE));
    let across = |d: i32| {
        let t = d as f32 / TOOTH_SCALE as f32;
        t * t * (3.0 - 2.0 * t)
    };
    let (sx, sy) = (
        across(x.rem_euclid(TOOTH_SCALE)),
        across(y.rem_euclid(TOOTH_SCALE)),
    );
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let top = lerp(noise(cx, cy), noise(cx + 1, cy), sx);
    let bottom = lerp(noise(cx, cy + 1), noise(cx + 1, cy + 1), sx);
    lerp(top, bottom, sy) * 2.0 - 1.0
}

/// Mix the paint from `levels` colours per channel, and lay the canvas's tooth
/// over what it made.
///
/// The tooth goes on after the mixing rather than before, because it is the
/// surface the picture was painted on and not one of the colours it was
/// painted with — quantising it away and then wondering where the texture went
/// is the obvious way to get this wrong.
pub(crate) fn finish(pixmap: &mut Pixmap, levels: u32, tooth: f32) {
    let w = pixmap.width() as i32;
    let steps = levels.max(2);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                let grain = if tooth > 0.0 {
                    weave(x, y) * tooth
                } else {
                    0.0
                };
                let i = x as usize * 4;
                for c in 0..3 {
                    // 256 rather than 255 so that every step is the same
                    // width, then back out over the range so that black stays
                    // black and white stays white.
                    let step = (out[i + c] as u32 * steps / 256).min(steps - 1);
                    let mixed = (step * 255) as f32 / (steps - 1) as f32;
                    out[i + c] = (mixed + grain).clamp(0.0, 255.0).round() as u8;
                }
            }
        });
}
