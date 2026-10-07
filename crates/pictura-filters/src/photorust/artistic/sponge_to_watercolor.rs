//! Sponge, Underpainting, Watercolor (split from `artistic` for file size).

#[allow(unused_imports)]
use super::*;

/// The darkest value along a line through each pixel, channel by channel.
pub(crate) fn darkest_along(source: &Pixmap, angle: f32, length: f32) -> Pixmap {
    let steps = length.round().max(1.0) as i32;
    let radians = angle.to_radians();
    let (dx, dy) = (radians.cos(), -radians.sin());
    let (width, height) = (source.width() as i32, source.height() as i32);
    let mut out = source.clone();
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for x in 0..width {
                let i = x as usize * 4;
                for step in -steps / 2..=steps / 2 {
                    let sx = (x as f32 + dx * step as f32).round() as i32;
                    let sy = (y as f32 + dy * step as f32).round() as i32;
                    if sx < 0 || sy < 0 || sx >= width || sy >= height {
                        continue;
                    }
                    let line = source.row(sy as u32);
                    let j = sx as usize * 4;
                    for c in 0..3 {
                        row[i + c] = row[i + c].min(line[j + c]);
                    }
                }
            }
        });
    out
}

/// How big the sponge's blotches are, in pixels of blur on the noise they are
/// cut from: a floor and a step of Brush Size.
pub(crate) const SPONGE_BLOTCH: f32 = 2.0;

pub(crate) const SPONGE_BLOTCH_PER_STEP: f32 = 0.6;

/// How hard the blotches are cut, as a gain on the noise before `tanh`: high
/// at Smoothness 1, where their edges are crisp, and lower as it rises.
pub(crate) const SPONGE_CUT: f32 = 3.0;

pub(crate) const SPONGE_CUT_PER_STEP: f32 = 0.12;

/// How many levels lighter or darker a blotch is per step of Definition.
pub(crate) const SPONGE_LEVELS: f32 = 1.4;

/// How much the picture under the blotches is settled: a median, whose reach
/// grows with Brush Size, then a blur that grows with Smoothness.
pub(crate) const SPONGE_SETTLE: f32 = 1.0;

pub(crate) const SPONGE_SETTLE_PER_STEP: f32 = 0.2;

pub(crate) const SPONGE_SOFTEN_PER_STEP: f32 = 0.12;

/// Filter ▸ Artistic ▸ Sponge: the picture dabbed on with a sponge.
///
/// The picture is settled into soft patches, and a pattern of blotches is
/// laid over it, each a little lighter or a little darker than what is under
/// it — the open and closed cells of the sponge. The blotches are smooth
/// noise cut hard, so they come out as rounded, irregular spots rather than
/// as grain.
///
/// **Brush Size** is how big the blotches are. **Definition** is how much
/// lighter or darker they are. **Smoothness** is how soft their edges are,
/// and how soft the picture under them.
///
/// Alpha is left alone.
///
/// No GPU path. The work is a median, which is sequential along each row, and
/// the blurs, which already go through the backend.
pub fn sponge(pixmap: &mut Pixmap, size: u32, definition: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*SPONGE_SIZE.start(), *SPONGE_SIZE.end()) as f32;
    let definition = definition.clamp(*SPONGE_DEFINITION.start(), *SPONGE_DEFINITION.end()) as f32;
    let smoothness = smoothness.clamp(*SPONGE_SMOOTHNESS.start(), *SPONGE_SMOOTHNESS.end()) as f32;

    let settle = (SPONGE_SETTLE + size * SPONGE_SETTLE_PER_STEP).round() as u32;
    let mut settled = crate::photorust::convolve::median_of(pixmap, settle, settle);
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut settled,
        smoothness * SPONGE_SOFTEN_PER_STEP,
    );

    let scale = SPONGE_BLOTCH + size * SPONGE_BLOTCH_PER_STEP;
    let blotches = blurred_specks(pixmap.width(), pixmap.height(), scale, 31);
    let spread = speck_gain(scale);
    let cut = (SPONGE_CUT - (smoothness - 1.0) * SPONGE_CUT_PER_STEP).max(0.5);
    let levels = definition * SPONGE_LEVELS;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(settled.as_bytes().par_chunks_exact(stride))
        .zip(blotches.as_bytes().par_chunks_exact(stride))
        .for_each(|((out, settled), blotches)| {
            for ((px, from), b) in out
                .chunks_exact_mut(4)
                .zip(settled.chunks_exact(4))
                .zip(blotches.chunks_exact(4))
            {
                let cell = ((b[0] as f32 - 127.5) * spread * cut).tanh() * levels;
                for c in 0..3 {
                    px[c] = (from[c] as f32 + cell).clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: dabbing the picture does not change the
                // layer's shape.
            }
        });
}

/// How far the underpainting is settled, per step of Brush Size: a median
/// reach, which gives the soft patches, and a blur, which washes them
/// together.
pub(crate) const UNDERPAINT_PATCH_PER_STEP: f32 = 0.6;

pub(crate) const UNDERPAINT_WASH_PER_STEP: f32 = 0.8;

/// How far the texture's slope bends where the paint is read from, per step
/// of Texture Coverage, and how far the surface is softened first so a hard
/// texture bends it over a pixel or two rather than in a single jump.
pub(crate) const UNDERPAINT_PUSH_PER_STEP: f32 = 3.5;

pub(crate) const UNDERPAINT_BEVEL: f32 = 0.7;

/// How much Relief each step of Texture Coverage adds.
pub(crate) const UNDERPAINT_RELIEF_PER_STEP: f32 = 1.0;

/// How much less the surface shows in the light than in the dark. See
/// [`crate::photorust::texture::apply_relief_weighted`].
pub(crate) const UNDERPAINT_DARK_BIAS: f32 = 0.4;

/// How hard the surface glints: see
/// [`crate::photorust::texture::Finish::crisp`].
pub(crate) const UNDERPAINT_CRISP: f32 = 0.75;

/// The shadow in the surface's low parts, and how much of the surface fades
/// out in broad soft patches. See [`crate::photorust::texture::Finish`].
pub(crate) const UNDERPAINT_OCCLUSION: f32 = 70.0;

pub(crate) const UNDERPAINT_PATCHY: f32 = 0.7;

/// Filter ▸ Artistic ▸ Underpainting: the picture laid in broadly on a
/// textured surface, as the first coat of a painting is.
///
/// 1. **Brush Size** settles the picture into soft washes of colour.
/// 2. **Texture Coverage** is how much the surface shows through: the paint is
///    bent about by the texture's slope, so edges take on its pattern, and
///    the surface stands deeper.
/// 3. The surface is lit — see [`crate::photorust::texture::apply_relief`] for
///    Texture, Scaling, Relief, Light and Invert.
///
/// Alpha is left alone.
///
/// No GPU path. The settling is a median, which is sequential along each row,
/// and every later stage wants the one before back on the CPU.
#[allow(clippy::too_many_arguments)]
pub fn underpainting(
    pixmap: &mut Pixmap,
    size: u32,
    coverage: u32,
    texture: crate::photorust::texture::Texture,
    scaling: u32,
    relief: u32,
    light: crate::photorust::texture::Light,
    invert: bool,
) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*UNDERPAINT_SIZE.start(), *UNDERPAINT_SIZE.end()) as f32;
    let coverage = coverage.clamp(*UNDERPAINT_COVERAGE.start(), *UNDERPAINT_COVERAGE.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);

    let patch = (size * UNDERPAINT_PATCH_PER_STEP).round() as u32;
    let mut paint = if patch > 0 {
        crate::photorust::convolve::median_of(pixmap, patch, patch)
    } else {
        pixmap.clone()
    };
    for (p, o) in paint
        .as_bytes_mut()
        .chunks_exact_mut(4)
        .zip(pixmap.as_bytes().chunks_exact(4))
    {
        p[3] = o[3];
    }
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut paint,
        size * UNDERPAINT_WASH_PER_STEP,
    );

    let mut surface =
        crate::photorust::texture::height_map(texture, pixmap.width(), pixmap.height(), scaling);
    if invert {
        surface.par_iter_mut().for_each(|h| *h = 1.0 - *h);
    }
    blur_field(&mut surface, width, height, UNDERPAINT_BEVEL);
    let push = coverage * UNDERPAINT_PUSH_PER_STEP;
    let (lx, ly) = light.towards();
    let (paint, surface) = (&paint, &surface);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, px) in out.chunks_exact_mut(4).enumerate() {
                // Read the paint from where the surface's slope bends it, as
                // light through rippled glass: flat surface, no change; the
                // side of a ridge, a jump. That is what cuts an edge into the
                // texture's own jagged pattern.
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let dx = (surface[y * width + right] - surface[y * width + left]) / 2.0;
                let dy = (surface[down * width + x] - surface[up * width + x]) / 2.0;
                // Along the light only: the paint slides down the slopes the
                // light shows, so with the light above, edges break into the
                // horizontal dashes CS6 gives and not into a grid of cells.
                let along = (dx * lx + dy * ly)
                    * push
                    * crate::photorust::texture::patchiness(x, y, UNDERPAINT_PATCHY);
                // Sampled between pixels, so the bend is smooth rather than a
                // pixel's jump.
                let fx = (x as f32 + lx * along).clamp(0.0, (width - 1) as f32);
                let fy = (y as f32 + ly * along).clamp(0.0, (height - 1) as f32);
                let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
                let (x1, y1) = ((x0 + 1).min(width - 1), (y0 + 1).min(height - 1));
                let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
                let (r0, r1) = (paint.row(y0 as u32), paint.row(y1 as u32));
                for c in 0..3 {
                    let top = r0[x0 * 4 + c] as f32 * (1.0 - tx) + r0[x1 * 4 + c] as f32 * tx;
                    let bottom = r1[x0 * 4 + c] as f32 * (1.0 - tx) + r1[x1 * 4 + c] as f32 * tx;
                    px[c] = (top * (1.0 - ty) + bottom * ty).round() as u8;
                }
                // Alpha stands: laying the picture in does not change the
                // layer's shape.
            }
        });

    // Coverage is surface as well as push: CS6's texture stands out plainly
    // at Relief 4 once the coverage is up.
    let relief = relief + (coverage * UNDERPAINT_RELIEF_PER_STEP).round() as u32;
    crate::photorust::texture::apply_relief_weighted(
        pixmap,
        texture,
        scaling,
        relief,
        light,
        invert,
        crate::photorust::texture::Finish {
            dark_bias: UNDERPAINT_DARK_BIAS,
            bevel: UNDERPAINT_BEVEL,
            crisp: UNDERPAINT_CRISP,
            occlusion: UNDERPAINT_OCCLUSION,
            patchy: UNDERPAINT_PATCHY,
            gain: 1.0,
        },
    );
}

/// How far the picture is washed into patches, in pixels of median reach:
/// the reach at Brush Detail 1, and how much each step of detail takes off.
pub(crate) const WATER_WASH: f32 = 3.6;

pub(crate) const WATER_WASH_PER_STEP: f32 = 0.2;

/// The smart blur that smooths the gradients inside a wash and keeps its
/// boundary: reach in pixels, and how many levels apart two tones may be and
/// still be the same wash.
pub(crate) const WATER_FLATTEN_REACH: u32 = 2;

pub(crate) const WATER_FLATTEN_LEVELS: u32 = 16;

/// How many pools of tone the picture is quantised into: at Brush Detail 1,
/// and how many more each step adds. Each pixel keeps its own colour and is
/// taken to its pool's brightness, so a pool is one flat wash of paint.
pub(crate) const WATER_POOLS: f32 = 8.0;

pub(crate) const WATER_POOLS_PER_STEP: f32 = 1.0;

/// How much of the pooled picture is laid over the smooth wash — the cutout
/// layer's opacity in the stack. All of it reads as a poster.
pub(crate) const WATER_POOL_MIX: f32 = 0.35;

/// The drawing under the paint: an edge finder over the pooled picture, and
/// how dark a line a unit of edge draws, multiplied in. The scale in pixels
/// it is found at keeps the line a pixel or two wide rather than a crisp
/// single-pixel trace.
pub(crate) const WATER_LINE_SCALE: f32 = 0.8;

pub(crate) const WATER_LINE: f32 = 0.0025;

/// Local contrast in the light, before the shadows: the scale in pixels a
/// pixel is compared against, and how far its difference is pushed. What
/// gives CS6's white spray its dark flecks and blue-grey pools.
pub(crate) const WATER_LOCAL_SCALE: f32 = 3.0;

pub(crate) const WATER_LOCAL: f32 = 7.0;

/// Where Shadow Intensity starts taking tones to black, as a fraction of
/// white: at 0, and how far each step raises it; and how soft the fall is.
/// At 5 it reaches the sea and the whole of it goes dark, as CS6's does.
pub(crate) const WATER_SHADOW_FROM: f32 = 0.12;

pub(crate) const WATER_SHADOW_PER_STEP: f32 = 0.085;

pub(crate) const WATER_SHADOW_SOFT: f32 = 0.1;

/// How much richer the colour is than the photograph's.
pub(crate) const WATER_SATURATION: f32 = 1.25;

/// How deep the colour is laid at any setting: watercolour is richer in the
/// dark than the photograph, as a gamma.
pub(crate) const WATER_DEPTH: f32 = 1.15;

/// The granulation of the pigment: levels at Texture 1 and per step, and the
/// size of a speck in pixels. Laid on before the wash, in brightness only, so
/// the washing turns it into a faint mottle inside the pools — laid on after,
/// it is noise and reads as film grain.
pub(crate) const WATER_GRAIN: f32 = 2.0;

pub(crate) const WATER_GRAIN_SCALE: f32 = 2.0;

/// What each step of Texture above 1 adds: a fine grit laid on at the end,
/// in levels, strongest in the dark and fading out in the light. Laid on
/// before the washing it is enlarged into blotches that cover the sky.
pub(crate) const WATER_GRIT_PER_STEP: f32 = 9.0;

pub(crate) const WATER_GRIT_SCALE: f32 = 0.7;

/// Filter ▸ Artistic ▸ Watercolor: the picture washed in with a wet brush.
///
/// The classic stack, in order:
///
/// 1. **Dry brush** — Dry Brush's own dabs, their size set by **Brush
///    Detail**: blotches with ragged edges, where a median would leave smooth
///    rounded shapes.
/// 2. **Smart blur** — the gradients inside each shape smoothed away, its
///    boundary kept.
/// 3. **Cutout** — the brightness pulled part of the way to a handful of
///    pools, each pixel keeping its colour.
/// 4. **Find edges** — the pools' boundaries drawn in as thin dark lines and
///    multiplied over the paint.
/// 5. **Shadow Intensity** takes the darker tones to black: at 1 only the
///    deepest shadows, by 5 everything below the midtones. **Texture** is how
///    much the pigment granulates.
///
/// Alpha is left alone.
///
/// No GPU path. The dabs and the smart blur are sequential along each row.
pub fn watercolor(pixmap: &mut Pixmap, detail: u32, shadow: u32, texture: u32) {
    if pixmap.is_empty() {
        return;
    }
    let detail = detail.clamp(*WATER_DETAIL.start(), *WATER_DETAIL.end()) as f32;
    let shadow = shadow.clamp(*WATER_SHADOW.start(), *WATER_SHADOW.end()) as f32;
    let texture = texture.clamp(*WATER_TEXTURE.start(), *WATER_TEXTURE.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let luma = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;

    // Granulation, under everything else.
    let grain = blurred_specks(pixmap.width(), pixmap.height(), WATER_GRAIN_SCALE, 61);
    let grain_gain = WATER_GRAIN * speck_gain(WATER_GRAIN_SCALE);
    let grit = blurred_specks(pixmap.width(), pixmap.height(), WATER_GRIT_SCALE, 62);
    let grit_gain = (texture - 1.0) * WATER_GRIT_PER_STEP * speck_gain(WATER_GRIT_SCALE);
    let mut grained = pixmap.clone();
    grained
        .as_bytes_mut()
        .par_chunks_exact_mut(4)
        .zip(grain.as_bytes().par_chunks_exact(4))
        .for_each(|(px, g)| {
            let speck = (g[0] as f32 - 127.5) * grain_gain;
            for c in 0..3 {
                px[c] = (px[c] as f32 + speck).round().clamp(0.0, 255.0) as u8;
            }
        });

    // 1 and 2: dry brush, smart blur.
    let reach = (WATER_WASH - (detail - 1.0) * WATER_WASH_PER_STEP)
        .round()
        .max(1.0) as u32;
    // Dabs, not a median: a median leaves smooth, rounded shapes, where a
    // brush leaves blotches with ragged edges — Dry Brush's own dabs.
    let mut wash = grained;
    paint_in_dabs(&mut wash, reach, true);
    crate::photorust::convolve::surface_blur(&mut wash, WATER_FLATTEN_REACH, WATER_FLATTEN_LEVELS);

    // 3: cutout — pools of tone, then their edges rounded.
    let steps = (WATER_POOLS + (detail - 1.0) * WATER_POOLS_PER_STEP - 1.0).max(1.0);
    wash.as_bytes_mut().par_chunks_exact_mut(4).for_each(|px| {
        let level = luma(px);
        let pool = (level / 255.0 * steps).round() / steps * 255.0;
        // Less in the light: a pale wash is thin and has no hard pools, and a
        // clear sky banded into contours reads as a map.
        let mix = WATER_POOL_MIX * (1.0 - level / 255.0);
        let scale = 1.0 + (pool / level.max(1.0) - 1.0) * mix;
        for c in 0..3 {
            px[c] = (px[c] as f32 * scale).round().clamp(0.0, 255.0) as u8;
        }
    });

    // 4: find edges, on the pools' brightness.
    let mut tone: Vec<f32> = wash.as_bytes().par_chunks_exact(4).map(luma).collect();
    blur_field(&mut tone, width, height, WATER_LINE_SCALE);
    let mut around = tone.clone();
    blur_field(&mut around, width, height, WATER_LOCAL_SCALE);
    let (tone, around) = (&tone, &around);

    let from = WATER_SHADOW_FROM + shadow * WATER_SHADOW_PER_STEP;
    let depth: [f32; 256] = std::array::from_fn(|v| 255.0 * (v as f32 / 255.0).powf(WATER_DEPTH));

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(wash.as_bytes().par_chunks_exact(stride))
        .zip(grit.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, ((out, wash), grit))| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(height - 1));
            for (x, ((px, w), g)) in out
                .chunks_exact_mut(4)
                .zip(wash.chunks_exact(4))
                .zip(grit.chunks_exact(4))
                .enumerate()
            {
                let (left, right) = (x.saturating_sub(1), (x + 1).min(width - 1));
                let at = |xx: usize, yy: usize| tone[yy * width + xx];
                // Sobel.
                let gx = at(right, up) + 2.0 * at(right, y) + at(right, down)
                    - at(left, up)
                    - 2.0 * at(left, y)
                    - at(left, down);
                let gy = at(left, down) + 2.0 * at(x, down) + at(right, down)
                    - at(left, up)
                    - 2.0 * at(x, up)
                    - at(right, up);
                let line = 1.0 - ((gx * gx + gy * gy).sqrt() * WATER_LINE).min(1.0);

                let mut paint = [0.0f32; 3];
                for c in 0..3 {
                    paint[c] = depth[w[c] as usize] * line;
                }
                // Local contrast in the light: a speck of spray a little
                // darker than the white round it is pulled down further, so
                // it is caught by the shadows and dries as a dark fleck.
                let i = y * width + x;
                let pale = around[i] / 255.0;
                let lift = (tone[i] - around[i]) * WATER_LOCAL * pale * pale;
                let grey = 0.299 * paint[0] + 0.587 * paint[1] + 0.114 * paint[2];
                let lifted = (grey + lift).max(0.0);
                let ratio = lifted / grey.max(1.0);
                for v in paint.iter_mut() {
                    *v = (lifted + (*v * ratio - lifted) * WATER_SATURATION).clamp(0.0, 255.0);
                }
                let grey = lifted.min(255.0);
                let t = ((grey / 255.0 - (from - WATER_SHADOW_SOFT)) / (2.0 * WATER_SHADOW_SOFT))
                    .clamp(0.0, 1.0);
                let kept = t * t * (3.0 - 2.0 * t);
                let speck = (g[0] as f32 - 127.5) * grit_gain * (1.0 - grey / 255.0);
                for c in 0..3 {
                    px[c] = (paint[c] * kept + speck).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: washing the picture in does not change the
                // layer's shape.
            }
        });
}

/// How big one stroke of the knife is, as slack given to the segmentation per
/// square pixel of Stroke Size.
///
/// **This filter works on regions, not on windows, and that is the whole of
/// it.** Everything else in this module looks at a fixed neighbourhood around
/// each pixel, so a petal and a blade of grass are the same size of thing to
/// it. Adobe's own description of Palette Knife is not that: it segments the
/// picture into contiguous areas of like colour, holds the boundaries between
/// them, and flattens what is inside each one. So a petal that happens to be
/// one colour across four hundred pixels comes back as *one* flat mass, while
/// the busy grass behind it comes back as a dozen — from a single uniform pass,
/// which is the thing about the reference that no window operation explains.
///
/// It is a spacing, in pixels per step of Stroke Size: see
/// [`segment::regions`](crate::photorust::segment::regions), where it sets how
/// far apart the regions start. Small — smaller than the masses that come back
/// — because the regions are not the masses. They are cut, and then the palette
/// below fuses them into masses; a boundary in the reference is the edge of a
/// run of regions that happened to round the same way, which is why it wanders
/// and steps about instead of curving the way one region's edge would.
pub(crate) const KNIFE_WIDTH: f32 = 1.0;

/// How tightly a region is held to its own patch of canvas.
///
/// This is `compactness` in [`segment::regions`](crate::photorust::segment::regions):
/// how much being near counts against being alike. Low, so the regions follow
/// what is in the picture rather than tiling it — what makes the masses coarse
/// is the palette, not the shape of the regions underneath.
pub(crate) const KNIFE_HOLD: f32 = 12.0;

/// How coarse the palette is, in levels per channel, and how much coarser a
/// step down in Stroke Detail makes it.
///
/// **The colours are compressed, and this is the part that took longest to
/// see.** The reference is not a photograph reduced to averages — it is a
/// photograph reduced to a *handful of colours*, half a dozen greens and four
/// or five pinks, each sitting flat over a large area with a hard, ragged edge
/// against the next. Regions alone never give that: they give a hundred
/// slightly different greens and boundaries no eye can find. Rounding what each
/// region came to onto a coarse palette is what fuses them.
///
/// Stroke Detail is how sensitive the knife is to the smaller colour breaks
/// inside a mass, which is exactly a count of rungs: wound up the palette is
/// fine and a petal keeps its shading, wound down it is coarse and the whole
/// flower goes over to two or three pinks.
pub(crate) const KNIFE_PALETTE: u32 = 16;

pub(crate) const KNIFE_DETAIL_PER_STEP: f32 = 0.6;

/// How far the picture is settled before the cut, in pixels per step of Stroke
/// Size.
///
/// Two jobs, and the second is the one that sets the size. A photograph's grain
/// is the enemy of a segmentation in a way it is not of a blur — two pixels of
/// the same petal a few levels apart will be pulled into different regions —
/// and a little blur fixes that at any radius.
///
/// The size comes from the dark ring around the flowers, which drove four wrong
/// mechanisms before this one. Blurring widens the step from pink to near-black
/// green into a band of the colours in between, and the palette then rounds the
/// middle of that band onto one rung: a flat plum ring with a hard edge on both
/// sides, following the outline all the way round. How wide the blur is, is how
/// wide the ring is. Nothing looks for the outline and nothing draws it.
///
/// Mostly floor, and barely rising with Stroke Size, because the ring in the
/// reference is about as wide at the top of the slider as at the middle — what
/// a wide stroke widens is the *masses*, which is the spacing's business. Made
/// proportional it swallows the flower: a blur wide enough to matter at Stroke
/// Size 50 pulls the background into the petals and the whole picture comes
/// back dull and fattened.
pub(crate) const KNIFE_SETTLE_FLOOR: f32 = 1.0;

pub(crate) const KNIFE_SETTLE: f32 = 0.03;

/// How much of the picture's small change is thrown away before the cut, in
/// pixels of median per step of Stroke Size.
///
/// This is where Stroke Size gets its *abstraction* from, and it has to be a
/// median rather than more blur for the reason above: at the top of the slider
/// the reference has lost the veins of a petal and the individual blades of
/// grass entirely, but the flower is still exactly the shape and the colour it
/// was. A median throws away whatever is narrower than its window and leaves
/// everything else where it stood; a blur wide enough to do the same would
/// drag the background into the flower.
pub(crate) const KNIFE_SIMPLIFY: f32 = 0.2;

/// How much longer than wide one stroke is.
///
/// A knife is a blade, and what a blade leaves is longer than it is wide —
/// every stroke in a real palette-knife painting is a slab dragged in one
/// direction, not a dab. Round masses come back reading as cobbles however
/// well their colours are judged, which is what this was added to fix.
///
/// Which way each one runs is worked out from the picture rather than chosen:
/// see [`segment::regions`](crate::photorust::segment::regions).
pub(crate) const KNIFE_STROKE: f32 = 2.2;

/// How many coats go on, and how much finer each is than the one under it.
///
/// **Oil laid on with a knife is built up, not laid down.** A painting of one
/// is a broad coat that covers the canvas and gets the masses right, and then
/// smaller strokes worked into the parts of it that had something to say —
/// the poppies over the field, the light on the water. One coat, however well
/// cut, gives a picture where a petal and a whole hillside are the same size of
/// thing, because they were both painted at the same size of stroke.
///
/// Two coats, because the third adds work and very little paint: by then what
/// the second missed is a pixel here and there. Where each coat goes is
/// [`where_it_matters`].
pub(crate) const COATS: usize = 2;

pub(crate) const KNIFE_FINER: f32 = 0.45;

/// Under a pixel, to take the stairs off a torn edge without softening the
/// tear. See where it is used for why a torn edge has stairs at all.
pub(crate) const KNIFE_NO_STAIRS: f32 = 0.7;

/// How far a boundary is allowed to wander off where it really is, in pixels
/// per step of Stroke Size.
///
/// Without this the masses come back with smooth, rounded, curving edges and
/// the result reads as a median filter rather than as a knife — which is
/// exactly the note this constant was added on. The reference's edges step and
/// kink; a knife is a straight blade dragged through wet paint and it leaves a
/// torn edge, not a drawn one. See
/// [`segment::flatten`](crate::photorust::segment::flatten) for how little it
/// costs: displacing where a pixel reads its colour from changes nothing inside
/// a flat mass and everything at its edge.
pub(crate) const KNIFE_RAGGED: f32 = 0.12;

/// How far each step of Softness carries, in pixels per step of Stroke Size.
///
/// Nothing at the bottom of the slider: the joins in the reference at Softness
/// 0 are *hard*, and ragged, and that is most of what makes it look scraped on
/// rather than painted. This only eases them.
pub(crate) const KNIFE_SOFTNESS_SCALE: f32 = 0.03;

/// How thickly the paint stands, how far its edge falls away, how much the
/// load varies from one stroke to the next, and where the light comes from.
///
/// **This is what makes it a palette knife and not a poster.** A real knife
/// painting is not flat colour: every stroke is a slab of paint standing a
/// millimetre or two off the canvas, so it has a lit edge on one side, a shadow
/// on the other, and a ridge of surplus paint where the blade lifted. Take the
/// relief away and the same picture reads as printed rather than laid on, which
/// is the note this pass was added on.
///
/// The light comes from the upper left because that is where it comes from in
/// every painting of one: it is the convention a viewer reads relief by, and
/// lighting from below makes the strokes read as dents instead.
///
/// The shoulder is a fraction of the stroke rather than a fixed distance —
/// a wide stroke carries more paint and its edge falls away further — and the
/// load is what stops the surface reading as machined: a knife picks up a
/// different amount of paint every time it goes back to the palette.
pub(crate) const PAINT_THICK: f32 = 0.16;

pub(crate) const PAINT_SHOULDER: f32 = 0.3;

pub(crate) const PAINT_LOAD: f32 = 0.5;

pub(crate) const PAINT_LIGHT: (f32, f32) = (-0.7, -0.7);

/// How deep the ridges the blade drags through a stroke are, and how far apart.
///
/// The pitch is a fraction of the stroke's width, so a wide stroke gets a few
/// broad ridges rather than a wide stroke's worth of fine ones. They run
/// *along* the stroke, which is why the orientation of each mass has to be
/// worked out before they can be laid: ridges running the wrong way across a
/// stroke read as corrugation, not as paint.
pub(crate) const PAINT_DRAG: f32 = 0.15;

pub(crate) const PAINT_DRAG_PITCH: f32 = 0.5;

/// How different a coat has to have left the picture before another one is
/// worked into it, in levels, and how different before it is worked in fully.
///
/// Low enough that anything the broad coat plainly lost gets gone back over,
/// high enough that the whole canvas does not — a second coat laid everywhere
/// buries the first and there was no point laying it.
pub(crate) const PAINT_MISS_LOW: f32 = 10.0;

pub(crate) const PAINT_MISS_HIGH: f32 = 17.0;

/// Three box passes make a good enough Gaussian, and this one is over a height
/// field that nothing but the lighting will ever see.
pub(crate) fn soften(field: &mut [f32], width: usize, height: usize, radius: usize) {
    if radius == 0 {
        return;
    }
    let mut scratch = vec![0.0f32; field.len()];
    for _ in 0..3 {
        // Across, then down. A box blur is separable for the same reason a
        // Gaussian is.
        scratch.copy_from_slice(field);
        field
            .par_chunks_exact_mut(width)
            .enumerate()
            .for_each(|(y, line)| {
                let row = &scratch[y * width..(y + 1) * width];
                for (x, slot) in line.iter_mut().enumerate() {
                    let from = x.saturating_sub(radius);
                    let to = (x + radius + 1).min(width);
                    *slot = row[from..to].iter().sum::<f32>() / (to - from) as f32;
                }
            });
        scratch.copy_from_slice(field);
        field
            .par_chunks_exact_mut(width)
            .enumerate()
            .for_each(|(y, line)| {
                let from = y.saturating_sub(radius);
                let to = (y + radius + 1).min(height);
                for (x, slot) in line.iter_mut().enumerate() {
                    let mut total = 0.0;
                    for row in from..to {
                        total += scratch[row * width + x];
                    }
                    *slot = total / (to - from) as f32;
                }
            });
    }
}

/// Smooth noise along one number, for the ridges the blade drags.
pub(crate) fn furrow(t: f32, salt: u32) -> f32 {
    let cell = t.floor();
    let step = t - cell;
    let ease = step * step * (3.0 - 2.0 * step);
    let at = |c: f32| noise(c as i32, salt as i32) * 2.0 - 1.0;
    at(cell) + (at(cell + 1.0) - at(cell)) * ease
}

/// Stand the paint off the canvas and light it.
///
/// Every mass becomes a slab: thickest in the middle, falling away to nothing
/// at its edge, standing a little higher or lower than its neighbours depending
/// on how much paint the knife had on it, and furrowed along its own length by
/// the blade. Then the whole surface is lit from the upper left, which is the
/// only step that touches the colours — everything above it is building a
/// height field that is never itself seen.
///
/// The orientation of each mass comes from its own second moments. A knife
/// stroke is longer than it is wide and the drag runs the long way; measuring
/// it is cheaper and steadier than guessing at the picture's own direction,
/// because the masses are already the shape the strokes are.
pub(crate) fn slab_of_paint(
    labels: &[u32],
    count: usize,
    width: usize,
    height: usize,
    stroke: f32,
) -> Vec<f32> {
    if count == 0 {
        return vec![0.0; width * height];
    }

    // Where each mass lies and which way it runs, from its own moments.
    let mut tally = vec![0f64; count];
    let mut sums = vec![[0f64; 5]; count];
    for (p, &label) in labels.iter().enumerate() {
        let k = label as usize;
        let (x, y) = ((p % width) as f64, (p / width) as f64);
        tally[k] += 1.0;
        sums[k][0] += x;
        sums[k][1] += y;
        sums[k][2] += x * x;
        sums[k][3] += x * y;
        sums[k][4] += y * y;
    }
    let lie: Vec<(f32, f32, f32)> = (0..count)
        .map(|k| {
            let n = tally[k].max(1.0);
            let (mx, my) = (sums[k][0] / n, sums[k][1] / n);
            let xx = sums[k][2] / n - mx * mx;
            let xy = sums[k][3] / n - mx * my;
            let yy = sums[k][4] / n - my * my;
            // The long axis of the mass. A round mass has no long axis and the
            // angle it gives is arbitrary, which is harmless: the furrows have
            // to run *some* way and on a round mass no way is wrong.
            let angle = 0.5 * (2.0 * xy).atan2(xx - yy);
            let load = noise(k as i32, 0x5bd1) - 0.5;
            (angle.cos() as f32, angle.sin() as f32, load)
        })
        .collect();

    // A slab per mass: full thickness inside, falling away at the edge. Built
    // as an interior mask and then softened, which costs one blur and needs no
    // distance transform.
    let mut paint = vec![0f32; width * height];
    paint
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, line)| {
            for (x, slot) in line.iter_mut().enumerate() {
                let p = y * width + x;
                let mine = labels[p];
                let same = (x == 0 || labels[p - 1] == mine)
                    && (x + 1 == width || labels[p + 1] == mine)
                    && (y == 0 || labels[p - width] == mine)
                    && (y + 1 == height || labels[p + width] == mine);
                *slot = if same { 1.0 } else { 0.0 };
            }
        });
    soften(
        &mut paint,
        width,
        height,
        ((stroke * PAINT_SHOULDER).round() as usize).max(1),
    );

    // Then the load each stroke was carrying, and the furrows the blade left
    // along it. Both are held down at the stroke's edge by the slab itself —
    // there is no paint out there to stand proud or to be dragged.
    let pitch = (stroke * PAINT_DRAG_PITCH).max(1.0);
    paint
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, line)| {
            for (x, slot) in line.iter_mut().enumerate() {
                let k = labels[y * width + x] as usize;
                let (across, along, load) = lie[k];
                // Measured across the stroke, so the furrows run along it.
                let over = (x as f32 * -along + y as f32 * across) / pitch;
                let drag = furrow(over, k as u32 & 0xffff);
                *slot *= 1.0 + load * PAINT_LOAD;
                *slot += drag * PAINT_DRAG * *slot;
            }
        });
    soften(&mut paint, width, height, 1);
    paint
}

/// Where the coat that has just gone on missed the picture, and so where
/// another one is worth laying.
///
/// This is how a painter works and it is the point of laying the paint in more
/// than one coat: the first is a broad one that covers the canvas and gets the
/// big shapes down, and it is *wrong* in the places where the picture had
/// something small to say. Those are the places that get worked into, and the
/// broad coat is left standing everywhere else. Laying the second coat
/// everywhere instead gives back the fine cut on its own, with the coarse one
/// buried and nothing gained from having laid it.
pub(crate) fn where_it_matters(subject: &Pixmap, canvas: &Pixmap, spread: f32) -> Vec<f32> {
    let width = subject.width() as usize;
    let height = subject.height() as usize;
    let (was, is) = (subject.as_bytes(), canvas.as_bytes());
    let mut missed = vec![0f32; width * height];
    missed
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, line)| {
            for (x, slot) in line.iter_mut().enumerate() {
                let i = (y * width + x) * 4;
                *slot = (0..3)
                    .map(|c| (was[i + c] as i32 - is[i + c] as i32).abs())
                    .max()
                    .unwrap_or(0) as f32;
            }
        });
    // Over an area rather than a pixel: a stroke is worked into or it is not,
    // and half of one cannot be.
    soften(
        &mut missed,
        width,
        height,
        ((spread * 0.25).round() as usize).max(1),
    );
    for v in missed.iter_mut() {
        *v = ((*v - PAINT_MISS_LOW) / (PAINT_MISS_HIGH - PAINT_MISS_LOW)).clamp(0.0, 1.0);
    }
    missed
}

/// Light the paint. The only step in the whole filter that turns a height back
/// into a colour; everything before it was building the height.
pub(crate) fn light_the_paint(pixmap: &mut Pixmap, depth: &[f32], lift: f32) {
    if lift <= 0.0 {
        return;
    }
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let (lx, ly) = PAINT_LIGHT;
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(width * 4)
        .enumerate()
        .for_each(|(y, line)| {
            for (x, chunk) in line.chunks_exact_mut(4).enumerate() {
                let at = |dx: usize, dy: usize| depth[dy * width + dx];
                let dx = at((x + 1).min(width - 1), y) - at(x.saturating_sub(1), y);
                let dy = at(x, (y + 1).min(height - 1)) - at(x, y.saturating_sub(1));
                let shade = (1.0 + (dx * lx + dy * ly) * lift).clamp(0.7, 1.35);
                for c in 0..3 {
                    chunk[c] = (chunk[c] as f32 * shade).clamp(0.0, 255.0) as u8;
                }
            }
        });
}
