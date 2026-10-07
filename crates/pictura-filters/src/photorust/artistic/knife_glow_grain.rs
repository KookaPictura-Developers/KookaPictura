//! Palette Knife, Neon Glow, Film Grain (split from `artistic` for file size).

#[allow(unused_imports)]
use super::*;

/// Filter ▸ Artistic ▸ Palette Knife: the picture spread with a knife.
///
/// The picture is settled, cut into contiguous areas of like colour, and each
/// area filled with its own average rounded onto a coarse palette. Both halves
/// are needed and neither is enough: the cut is what makes the areas follow
/// what is in the picture, and the palette is what fuses them into a few flat
/// masses with the hard ragged edges between them that a knife leaves.
///
/// * **Stroke Size** is how big a mass of colour the knife works in. It sets
///   both the spacing of the cut ([`KNIFE_WIDTH`]) and how far the picture is
///   settled first ([`KNIFE_SETTLE`]), which is what makes a wide stroke lose
///   the small things entirely rather than merely enlarging them.
/// * **Stroke Detail** is how sensitive the knife is to the smaller colour
///   breaks inside a mass — the number of rungs on the palette, see
///   [`KNIFE_PALETTE`]. Wound down, a whole flower goes over to two or three
///   pinks.
/// * **Softness** is the blade's edge, which eases the joins between one mass
///   and the next. At 0 they are left hard, as the reference has them.
///
/// Alpha is left alone: spreading the picture does not change the layer's
/// shape.
///
/// No GPU path, and there will not be one: the cut is a sequential walk over
/// the joins between pixels in order, each step depending on every step before
/// it — the same argument as flood fill. See
/// [`segment`](super::segment).
pub fn palette_knife(pixmap: &mut Pixmap, size: u32, detail: u32, softness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*KNIFE_SIZE.start(), *KNIFE_SIZE.end());
    let detail = detail.clamp(*KNIFE_DETAIL.start(), *KNIFE_DETAIL.end());
    let softness = softness.clamp(*KNIFE_SOFTNESS.start(), *KNIFE_SOFTNESS.end());

    // Settle the picture before deciding what belongs with what — and paint
    // back from the settled picture too, not from the original. The band of
    // in-between colours this lays along every strong edge is not an artefact
    // to be tolerated; it is where the ring comes from. See KNIFE_SETTLE.
    crate::photorust::convolve::median_filter(
        pixmap,
        ((size as f32 * KNIFE_SIMPLIFY).round() as u32).max(1),
    );
    crate::photorust::convolve::gaussian_blur_accelerated(
        pixmap,
        KNIFE_SETTLE_FLOOR + size as f32 * KNIFE_SETTLE,
    );

    let rungs = (KNIFE_PALETTE as f32
        / (1.0 + (*KNIFE_DETAIL.end() - detail) as f32 * KNIFE_DETAIL_PER_STEP))
        .round()
        .max(2.0) as u32;
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;

    // What is being painted *from* stays as it was settled. Every coat is cut
    // from it and coloured from it, so a coat laid over another is a fresh
    // reading of the picture rather than a reading of the paint already down —
    // which is what a painter does, and is also the only way the second coat
    // can put back what the first one lost.
    let subject = pixmap.clone();
    let mut depth = vec![0f32; width * height];
    let broad = size as f32 * KNIFE_WIDTH;

    for coat in 0..COATS {
        let stroke = broad * KNIFE_FINER.powi(coat as i32);
        let (labels, count) =
            crate::photorust::segment::regions(&subject, stroke, KNIFE_HOLD, KNIFE_STROKE);

        let mut wet = subject.clone();
        crate::photorust::segment::flatten(
            &mut wet,
            &labels,
            count,
            rungs,
            stroke * KNIFE_RAGGED / KNIFE_WIDTH,
        );

        // The first coat covers the canvas; every one after it goes on only
        // where the one before missed.
        let worked = if coat == 0 {
            vec![1.0f32; width * height]
        } else {
            where_it_matters(&subject, pixmap, stroke)
        };

        let fresh = wet.as_bytes().to_vec();
        pixmap
            .as_bytes_mut()
            .par_chunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(y, line)| {
                for (x, chunk) in line.chunks_exact_mut(4).enumerate() {
                    let p = y * width + x;
                    let over = worked[p];
                    for c in 0..3 {
                        let under = chunk[c] as f32;
                        chunk[c] = (under + (fresh[p * 4 + c] as f32 - under) * over) as u8;
                    }
                }
            });

        // And the paint stacks up where it went on. A finer stroke carries
        // less paint than a broad one, so it stands proportionally less proud
        // — otherwise the accents shout over the coat they were laid on.
        let slab = slab_of_paint(&labels, count, width, height, stroke);
        let carried = stroke / broad;
        depth
            .par_iter_mut()
            .zip(slab.par_iter().zip(worked.par_iter()))
            .for_each(|(total, (&this, &over))| *total += this * over * carried);
    }

    // Take the stairs off the edges. Displacing where a pixel reads its colour
    // from is done in whole pixels, so a torn boundary comes back climbing in
    // single-pixel steps — ragged at arm's length and *pixelated* up close,
    // which is the note this was added on. Under a pixel of blur reads as a
    // torn edge rather than as a stepped one and costs nothing else: there is
    // nothing this small anywhere else in the picture by now.
    crate::photorust::convolve::gaussian_blur(pixmap, KNIFE_NO_STAIRS);

    if softness > 0 {
        crate::photorust::convolve::gaussian_blur_accelerated(
            pixmap,
            size as f32 * softness as f32 * KNIFE_SOFTNESS_SCALE,
        );
    }

    // Last, because it is the only pass that is about the paint rather than
    // about the picture. Softness thins the paint as well as easing the joins:
    // a stroke laid on thin has no edge to catch the light.
    let left = 1.0 - softness as f32 / *KNIFE_SOFTNESS.end() as f32;
    light_the_paint(pixmap, &depth, left * PAINT_THICK * broad);
    // Alpha stands throughout: spreading the picture does not change the
    // layer's shape. Every pass above leaves it alone.
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
