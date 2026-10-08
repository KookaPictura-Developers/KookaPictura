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
