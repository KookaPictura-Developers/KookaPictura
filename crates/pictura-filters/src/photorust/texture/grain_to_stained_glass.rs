//! Grain, Mosaic Tiles, Patchwork, Texturizer, Stained Glass (split from `texture` for file size).

#[allow(unused_imports)]
use super::*;

/// Filter ▸ Texture ▸ Grain: the picture as if printed through grain of one
/// of ten kinds.
///
/// **Intensity** is how strong the grain is and **Contrast** stretches the
/// result about mid-grey, grain and all. **Grain Type** decides what the
/// grain is: colour noise pixel by pixel (Regular, Soft) or gathered into
/// clumps (Clumped, Contrasty, Enlarged), streaks (Horizontal, Vertical), or
/// specks and dither in the document's swatches — Sprinkles throws specks of
/// the background colour, Speckle specks of the foreground thickest in the
/// dark, and Stippled cuts the whole picture into the two.
///
/// Alpha is left alone.
///
/// No GPU path: a noise and a curve per pixel, a few milliseconds on the
/// CPU, and the clumps and streaks are one-off blurs of the noise. The upload
/// would cost more than the work.
pub fn grain(
    pixmap: &mut Pixmap,
    intensity: u32,
    contrast: u32,
    kind: GrainType,
    foreground: crate::photorust::pixmap::Rgba8,
    background: crate::photorust::pixmap::Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let intensity = intensity.clamp(*GRAIN_INTENSITY.start(), *GRAIN_INTENSITY.end()) as f32;
    let contrast = contrast.clamp(*GRAIN_CONTRAST.start(), *GRAIN_CONTRAST.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let spread = intensity * GRAIN_SPREAD_PER_STEP;
    let share = intensity / *GRAIN_INTENSITY.end() as f32;

    let colour_kind = match kind {
        GrainType::Regular => Some(&GRAIN_REGULAR),
        GrainType::Soft => Some(&GRAIN_SOFT),
        GrainType::Clumped => Some(&GRAIN_CLUMPED),
        GrainType::Contrasty => Some(&GRAIN_CONTRASTY),
        GrainType::Enlarged => Some(&GRAIN_ENLARGED),
        _ => None,
    };
    let push = colour_kind.map_or(1.0, |k| k.contrast);
    let boost = match kind {
        GrainType::Horizontal | GrainType::Vertical => GRAIN_STREAK_BOOST,
        _ => colour_kind.map_or(1.0, |k| k.boost),
    };
    let gain = boost * 2f32.powf((contrast - GRAIN_CONTRAST_FLAT) / GRAIN_CONTRAST_DOUBLING * push);
    let stretch = |v: f32| ((v - 128.0) * gain + 128.0).clamp(0.0, 255.0);
    let lift = |v: f32| v + (255.0 - v) * share * GRAIN_SPECK_LIFT;

    // The colour kinds' noise, one field per channel, clumped and scaled
    // back to a spread of one so the clumping does not also fade it.
    let fields: Vec<Vec<f32>> = match colour_kind {
        Some(k) => (0..3)
            .map(|c| {
                let mut field: Vec<f32> = (0..width * height)
                    .into_par_iter()
                    .map(|i| {
                        let (x, y) = (i % width, i / width);
                        GRAIN_SHARED * grain_noise(x, y, 10)
                            + (1.0 - GRAIN_SHARED) * grain_noise(x, y, 11 + c)
                    })
                    .collect();
                if k.clump > 0.0 {
                    crate::photorust::artistic::blur_field(&mut field, width, height, k.clump);
                }
                crate::photorust::brush_strokes::unit_spread(&mut field);
                field
            })
            .collect(),
        None => Vec::new(),
    };
    let streaks: Vec<f32> = match kind {
        GrainType::Horizontal | GrainType::Vertical => (0..width * height)
            .into_par_iter()
            .map(|i| {
                let (x, y) = (i % width, i / width);
                // Along the band, and which band.
                let (along, band) = if kind == GrainType::Horizontal {
                    (x, y)
                } else {
                    (y, x)
                };
                let row = value_noise(0.0, band as f32, GRAIN_BAND, 61) * 2.0 - 1.0;
                let fade = value_noise(along as f32, band as f32 * 37.0, GRAIN_BAND_FADE, 62);
                let fade = GRAIN_BAND_FLOOR + (1.0 - GRAIN_BAND_FLOOR) * fade;
                row * fade + grain_noise(x, y, 63) * GRAIN_BAND_GRIT
            })
            .collect(),
        _ => Vec::new(),
    };
    // Speckle's neighbourhood, so what is darker than its surroundings can
    // be inked.
    let around: Vec<f32> = if kind == GrainType::Speckle {
        let bytes = pixmap.as_bytes();
        let stride = pixmap.stride();
        let mut field: Vec<f32> = (0..width * height)
            .into_par_iter()
            .map(|i| {
                let p = &bytes[(i / width) * stride + (i % width) * 4..];
                0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
            })
            .collect();
        crate::photorust::artistic::blur_field(
            &mut field,
            width,
            height,
            GRAIN_SPECKLE_NEIGHBOURHOOD,
        );
        field
    } else {
        Vec::new()
    };
    let around = &around;
    let ink = [
        foreground.r as f32,
        foreground.g as f32,
        foreground.b as f32,
    ];
    let paper = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let (fields, streaks) = (&fields, &streaks);

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * width + x;
                let lum = 0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32;
                let out: [f32; 3] = match kind {
                    GrainType::Regular
                    | GrainType::Soft
                    | GrainType::Clumped
                    | GrainType::Contrasty
                    | GrainType::Enlarged => {
                        let strength = colour_kind.map_or(1.0, |k| k.strength) * spread;
                        std::array::from_fn(|c| stretch(px[c] as f32 + fields[c][i] * strength))
                    }
                    GrainType::Sprinkles => {
                        let hit = (grain_noise(x, y, 31) + 1.0) * 0.5 < share * GRAIN_SPRINKLE;
                        if hit {
                            paper
                        } else {
                            std::array::from_fn(|c| stretch(lift(px[c] as f32)))
                        }
                    }
                    GrainType::Speckle => {
                        let on_mesh = x % GRAIN_MESH == 0 || y % GRAIN_MESH == 0;
                        let cut = share
                            * (GRAIN_SPECKLE_CUT
                                + if on_mesh { GRAIN_SPECKLE_ON_MESH } else { 0.0 }
                                + grain_noise(x, y, 41) * GRAIN_SPECKLE_SHAKE);
                        let darker = (around[i] - lum).max(0.0) * GRAIN_SPECKLE_EDGE;
                        if lum - darker < cut {
                            ink
                        } else {
                            // Lighter and richer under the specks.
                            let lifted = lum + (255.0 - lum) * share * GRAIN_SPECKLE_LIFT;
                            let rich = 1.0 + share * GRAIN_SPECKLE_SATURATE;
                            let scale = lifted / lum.max(1.0);
                            std::array::from_fn(|c| {
                                let v = lifted + (px[c] as f32 - lum) * scale.min(3.0) * rich;
                                stretch(v)
                            })
                        }
                    }
                    GrainType::Stippled => {
                        let shaken = stretch(lum) + grain_noise(x, y, 51) * spread * GRAIN_STIPPLE;
                        if shaken < 128.0 {
                            ink
                        } else {
                            paper
                        }
                    }
                    GrainType::Horizontal | GrainType::Vertical => {
                        let s = streaks[i];
                        let s = if s < 0.0 { s } else { s * GRAIN_STREAK_LIGHT };
                        let shift = s * spread * GRAIN_STREAK_STRENGTH;
                        std::array::from_fn(|c| stretch(px[c] as f32 + shift))
                    }
                };
                for c in 0..3 {
                    px[c] = out[c].round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: grain does not change the layer's shape.
            }
        });
}

/// CS6's ranges for Mosaic Tiles, which its three sliders run over.
pub const TILE_SIZE: std::ops::RangeInclusive<u32> = 2..=100;

pub const TILE_GROUT: std::ops::RangeInclusive<u32> = 1..=15;

pub const TILE_LIGHTEN: std::ops::RangeInclusive<u32> = 0..=10;

/// How far a tile's edge wanders off the grid, in pixels and as a share of
/// the tile, and how coarse the wander is, as a share of the tile but no
/// finer than [`TILE_JAG_GRAIN_MIN`]. CS6's grout lines follow a square grid
/// but are ragged and chunky, like hand-cut tiles: at Tile Size 47 an edge
/// steps in and out by a few pixels every eight or so, at 12 the tiles are
/// lumpy enough to read as pebbles.
pub(crate) const TILE_JAG: f32 = 2.4;

pub(crate) const TILE_JAG_PER_SIZE: f32 = 0.07;

pub(crate) const TILE_JAG_GRAIN: f32 = 0.2;

pub(crate) const TILE_JAG_GRAIN_MIN: f32 = 6.0;

/// The share of the grid's edges that never got grout, so the tiles either
/// side are one. CS6's grid is not complete: here and there two tiles run
/// together into a longer, odd-shaped one.
pub(crate) const TILE_MISSING: f32 = 0.2;

/// How far a tile's edge is rounded over into the grout, in pixels, but no
/// more than this share of the tile and no less than a pixel. CS6's tile
/// faces are flat right up to a narrow rim, whatever their size: rounded
/// further, small tiles are all bevel and read as bubble wrap.
pub(crate) const TILE_BEVEL: f32 = 2.0;

pub(crate) const TILE_BEVEL_PER_SIZE: f32 = 0.06;

pub(crate) const TILE_BEVEL_MIN: f32 = 0.7;

/// How high a tile stands over its grout, in the same units as the picture's
/// relief. A step of its own rather than a share of the picture's brightness:
/// scaled by that, every bright tile stood tall and shaded like a dome.
pub(crate) const TILE_HEIGHT: f32 = 0.55;

/// How much the picture's own brightness raises the surface, so the picture
/// reads as pressed into the tiles, as CS6's embossed horse does.
pub(crate) const TILE_PICTURE_RELIEF: f32 = 1.6;

/// How hard the tiles are lit, and how dark the shadow a tile's edge throws
/// over the grout below and to the right of it is.
pub(crate) const TILE_SHADE: f32 = 1.3;

pub(crate) const TILE_SHADOW: f32 = 0.45;

pub(crate) const TILE_SHADOW_REACH: usize = 3;

/// How wide the grout is, in pixels each side of the grid line, per step of
/// Grout Width and less this, but never under [`TILE_GROUT_MIN`]. CS6's
/// grout at 3 is a line under two pixels wide, rims included, and at 8 a
/// band about seven wide: it widens faster than the slider from a start
/// near nothing.
pub(crate) const TILE_GROUT_SHARE: f32 = 0.55;

pub(crate) const TILE_GROUT_OFFSET: f32 = 0.75;

pub(crate) const TILE_GROUT_MIN: f32 = 0.5;

/// The grout: a grey this light at Lighten Grout 0 and this much lighter per
/// step, laid over the picture this thickly at 0 and this much thicker per
/// step. CS6's grout lets some of the picture through — dark over the
/// horse, pale over the sky — so the picture reads as one thing cut into
/// tiles rather than as squares on a grey ground.
pub(crate) const TILE_GROUT_GREY: f32 = 90.0;

pub(crate) const TILE_GROUT_PER_STEP: f32 = 14.0;

pub(crate) const TILE_GROUT_COVER: f32 = 0.25;

pub(crate) const TILE_GROUT_COVER_PER_STEP: f32 = 0.05;

/// Filter ▸ Texture ▸ Mosaic Tiles: the picture laid in small tiles with
/// grout between them.
///
/// The tiles are a square grid **Tile Size** apart whose lines are jagged by
/// a coarse noise — see [`TILE_JAG`] — and a few of which are missing, so
/// the odd pair of tiles runs together — see [`TILE_MISSING`]. The grout between them is **Grout
/// Width** wide. The grout is grey, as light as **Lighten Grout** asks, with
/// a little of the picture showing through. The tiles are raised, rounded at
/// the edge and lit from the top left, with the picture's own brightness
/// pressed into their faces, and their edges throw shadow into the grout.
///
/// Alpha is left alone.
///
/// No GPU path, for Craquelure's reasons: a few noises and a slope per
/// pixel, tens of milliseconds on the CPU, and the result is wanted straight
/// back.
pub fn mosaic_tiles(pixmap: &mut Pixmap, size: u32, grout: u32, lighten: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*TILE_SIZE.start(), *TILE_SIZE.end()) as f32;
    let grout = grout.clamp(*TILE_GROUT.start(), *TILE_GROUT.end()) as f32;
    let lighten = lighten.clamp(*TILE_LIGHTEN.start(), *TILE_LIGHTEN.end()) as f32;
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let jag = TILE_JAG + size * TILE_JAG_PER_SIZE;
    let grain = (size * TILE_JAG_GRAIN).max(TILE_JAG_GRAIN_MIN);
    // The grout cannot eat the whole tile.
    let half_grout = (grout * TILE_GROUT_SHARE - TILE_GROUT_OFFSET)
        .max(TILE_GROUT_MIN)
        .min(size * 0.3);
    let bevel = TILE_BEVEL
        .min(size * TILE_BEVEL_PER_SIZE)
        .max(TILE_BEVEL_MIN);

    let bytes = pixmap.as_bytes();
    let stride = pixmap.stride();
    let lum: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let p = &bytes[(i / width) * stride + (i % width) * 4..];
            (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0
        })
        .collect();

    // How far each pixel is inside its tile: negative in the grout.
    let inside: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % width) as f32, (i / width) as f32);
            let u = x + (value_noise(x, y, grain, 101) - 0.5) * 2.0 * jag;
            let v = y + (value_noise(x, y, grain, 102) - 0.5) * 2.0 * jag;
            // The nearest line of the grid each way, and whether the stretch
            // of it here got any grout.
            let (col, row) = ((u / size).round(), (v / size).round());
            let (cell_x, cell_y) = ((u / size).floor() as i32, (v / size).floor() as i32);
            let mut across = (u - col * size).abs();
            if lattice(col as i32, cell_y, 103) < TILE_MISSING {
                across = f32::MAX;
            }
            let mut down = (v - row * size).abs();
            if lattice(cell_x, row as i32, 104) < TILE_MISSING {
                down = f32::MAX;
            }
            across.min(down) - half_grout
        })
        .collect();

    let smooth = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let mut surface: Vec<f32> = (0..width * height)
        .into_par_iter()
        .map(|i| smooth(inside[i] / bevel) * TILE_HEIGHT + lum[i] * TILE_PICTURE_RELIEF)
        .collect();
    crate::photorust::artistic::blur_field(&mut surface, width, height, 0.4);

    let (lx, ly) = Light::TopLeft.towards();
    let grey = TILE_GROUT_GREY + lighten * TILE_GROUT_PER_STEP;
    let cover = TILE_GROUT_COVER + lighten * TILE_GROUT_COVER_PER_STEP;
    let (surface, inside) = (&surface, &inside);
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
                let shade = -(dx * lx + dy * ly) * TILE_SHADE;
                // The shadow a tile's edge throws over lower ground.
                let mut shadow = 0.0f32;
                for k in 1..=TILE_SHADOW_REACH {
                    let sx = x as f32 + lx * k as f32;
                    let sy = y as f32 + ly * k as f32;
                    if sx < 0.0 || sy < 0.0 {
                        break;
                    }
                    let (sx, sy) = (sx.round() as usize, sy.round() as usize);
                    if sx >= width || sy >= height {
                        break;
                    }
                    shadow = shadow.max(surface[sy * width + sx] - surface[i]);
                }
                let shadow = 1.0 - TILE_SHADOW * shadow.clamp(0.0, 1.0);
                // Tile or grout, blended across the edge's first pixel.
                let tile = smooth(inside[i] + 0.5);
                for c in 0..3 {
                    let own = px[c] as f32;
                    let grout = own + (grey - own) * cover;
                    let base = grout + (own - grout) * tile;
                    let v = base * (1.0 + shade * SHADE_BY_COLOUR) + shade * SHADE_BY_LIGHT;
                    px[c] = (v * shadow).clamp(0.0, 255.0).round() as u8;
                }
                // Alpha stands: tiling the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Patchwork, which its two sliders run over.
pub const PATCH_SQUARE: std::ops::RangeInclusive<u32> = 0..=10;

pub const PATCH_RELIEF: std::ops::RangeInclusive<u32> = 0..=25;

/// How wide a square is, in pixels: Square Size, plus this. Measured off CS6
/// by fitting `samples/horse-3.jpg` to its preview: at Square Size 4 the
/// squares repeat every 8.9 pixels of the image, at 8 every 13.1.
pub(crate) const PATCH_SQUARE_FROM: u32 = 5;

/// The squares' faces are lifted by this gamma. Measured off CS6 block by
/// block over the whole of `samples/horse-3.jpg`: a source tone of 12 comes
/// out at 37, 62 at 104, 137 at 190 and 237 at 233.
pub(crate) const PATCH_LIFT: f32 = 0.65;

/// How much a square's brightness is pushed at random, as a share of full
/// white. Adobe's account of the filter says it varies the tiles' depth at
/// random "to replicate the highlights and shadows", but only a trace
/// shows: any more and the dark of the horse comes out as a checkerboard,
/// where CS6's runs in even rows.
pub(crate) const PATCH_VARY: f32 = 0.02;

/// The Relief the constants here are set for. Relief scales the heights by
/// its square root: CS6's shadows are near black by 16, but the rest of the
/// picture is hardly darker than at 8.
pub(crate) const PATCH_RELIEF_FULL: f32 = 8.0;

/// The squares are real blocks, and the picture is a render of them. Each
/// stands this many pixels high per unit of brightness over a black one —
/// so the steps between a pale square and a dark one throw shadows — plus
/// [`PATCH_RIM`] pixels that every square has, rounded over at its edges
/// into the joint with the next. The rounding runs this share of a square
/// in from each edge, no less than [`PATCH_ROUND_MIN`] pixels, and is a
/// rounded rectangle, so it rounds the corners too: that is what gives
/// CS6's shadow under each square its crescent shape, thin under the middle
/// and thick at the ends.
pub(crate) const PATCH_STEP: f32 = 10.0;

pub(crate) const PATCH_RIM: f32 = 1.2;

pub(crate) const PATCH_ROUND: f32 = 0.32;

pub(crate) const PATCH_ROUND_MIN: f32 = 1.5;

/// How much tighter the rounding is on a square's left and right sides than
/// on its top and bottom, and how much shallower. CS6's joints down a row
/// are faint lines, at about 88% of the face across its sky, beside the ones
/// between rows at under 60%. The two roundings multiply, so the corners
/// sink furthest — the ends of CS6's crescents.
pub(crate) const PATCH_ROUND_SIDES: f32 = 0.35;

pub(crate) const PATCH_SIDE_DEPTH: f32 = 0.65;

/// How much higher a square's lower edge stands than its upper one, in
/// pixels: the rows lie like shingles, each square's foot standing over the
/// head of the one below and throwing it into shadow. CS6's rows read as
/// stacked one on another, and averaged down a square its shading runs from
/// a shadow at its head, 69%, up to 103% a fifth of the way down, and down
/// again to its foot; a block with a level face lit from above has a bright
/// bevel along its head instead, and reads as a chocolate bar.
pub(crate) const PATCH_TILT: f32 = 2.0;

/// Where the light comes from: its direction across the picture, pointing
/// towards the light, and its height above the horizon in degrees. CS6's is
/// above and a little to the left, and low enough that a step of a couple
/// of pixels throws a shadow several pixels long.
pub(crate) const PATCH_LIGHT: (f32, f32) = (-0.2, -0.98);

pub(crate) const PATCH_ELEVATION: f32 = 28.0;

/// How much of a face's light is ambient, which neither slope nor shadow
/// takes away: the floor that shadows fall to.
pub(crate) const PATCH_AMBIENT: f32 = 0.38;

/// Shadows are traced from each pixel back towards the light in steps of
/// this many pixels, as far as this, and their edge is softened over this
/// many pixels of height, so a shadow deepens into the joint rather than
/// being cut out.
pub(crate) const PATCH_SHADOW_STEP: f32 = 0.5;

pub(crate) const PATCH_SHADOW_REACH: f32 = 14.0;

pub(crate) const PATCH_SHADOW_SOFT: f32 = 1.1;

/// Filter ▸ Texture ▸ Patchwork: the picture as squares of cloth or tile,
/// each one the average colour under it, raised by its brightness.
///
/// The picture is cut into squares **Square Size** and five pixels across,
/// and each is filled with the average of the picture under it. Then the
/// squares are built as blocks — each as high as it is bright, its edges
/// and corners rounded down into the joints — and the picture is a render
/// of those blocks lit from above and a little to the left: slopes facing
/// the light are brighter, slopes facing away darker, and every block
/// throws a shadow onto whatever stands lower beside it. **Relief** is how
/// high it all stands.
///
/// The height is worked out at any point, not per pixel, so slopes and
/// shadows are measured between pixels and their edges come out smooth
/// rather than staircased.
///
/// Alpha is left alone.
///
/// No GPU path: it would fit — a height and a short ray per pixel — but it
/// runs in tens of milliseconds on the CPU and its result is wanted straight
/// back.
pub fn patchwork(pixmap: &mut Pixmap, square: u32, relief: u32) {
    if pixmap.is_empty() {
        return;
    }
    let n = (square.clamp(*PATCH_SQUARE.start(), *PATCH_SQUARE.end()) + PATCH_SQUARE_FROM) as usize;
    // Deepening by the square root: CS6 at Relief 16 has darker joints than
    // at 8, not twice the shading everywhere.
    let relief = (relief.clamp(*PATCH_RELIEF.start(), *PATCH_RELIEF.end()) as f32
        / PATCH_RELIEF_FULL)
        .sqrt();
    let (width, height) = (pixmap.width() as usize, pixmap.height() as usize);
    let (cols, rows) = (width.div_ceil(n), height.div_ceil(n));
    let stride = pixmap.stride();

    // Each square's average colour, lifted, and its height.
    let bytes = pixmap.as_bytes();
    let squares: Vec<([f32; 3], f32)> = (0..cols * rows)
        .into_par_iter()
        .map(|k| {
            let (cx, cy) = (k % cols, k / cols);
            let mut total = [0.0f32; 3];
            let mut count = 0.0f32;
            for y in cy * n..((cy + 1) * n).min(height) {
                for x in cx * n..((cx + 1) * n).min(width) {
                    let p = &bytes[y * stride + x * 4..];
                    for c in 0..3 {
                        total[c] += p[c] as f32;
                    }
                    count += 1.0;
                }
            }
            let mean = total.map(|t| t / count.max(1.0));
            let lum = (0.299 * mean[0] + 0.587 * mean[1] + 0.114 * mean[2]) / 255.0;
            // Lifted by brightness rather than channel by channel, so the
            // browns keep their warmth instead of greying.
            let gain = if lum > 0.0 {
                lum.powf(PATCH_LIFT) / lum
            } else {
                1.0
            };
            // The variation is the tiles' depth, so it goes with Relief.
            let vary = (lattice(cx as i32, cy as i32, 111) - 0.5)
                * 2.0
                * PATCH_VARY
                * 255.0
                * relief.min(1.5);
            let face = mean.map(|v| (v * gain + vary).clamp(0.0, 255.0));
            (face, lum)
        })
        .collect();
    let squares = &squares;

    // The blocks' height at any point, in pixels.
    let size = n as f32;
    let round = (size * PATCH_ROUND).max(PATCH_ROUND_MIN).min(size * 0.5);
    let surface = move |x: f32, y: f32| -> f32 {
        let x = x.clamp(0.0, width as f32 - 0.001);
        let y = y.clamp(0.0, height as f32 - 0.001);
        let (cx, cy) = ((x / size) as usize, (y / size) as usize);
        let base = squares[cy * cols + cx].1 * PATCH_STEP;
        let rim = if n >= 3 {
            // A quarter circle over the rounding, in from an edge.
            let quarter = |edge: f32, r: f32| {
                let t = 1.0 - (edge / r).min(1.0);
                (1.0 - t * t).max(0.0).sqrt()
            };
            let (u, v) = (x - cx as f32 * size, y - cy as f32 * size);
            let down = quarter(v.min(size - v), round);
            let across = quarter(u.min(size - u), round * PATCH_ROUND_SIDES);
            down * (1.0 - PATCH_SIDE_DEPTH * (1.0 - across))
        } else {
            1.0
        };
        let tilt = if n >= 3 {
            (y / size).fract() * PATCH_TILT
        } else {
            0.0
        };
        (base + rim * PATCH_RIM + tilt) * relief
    };

    let (lx, ly) = {
        let (x, y) = PATCH_LIGHT;
        let len = (x * x + y * y).sqrt();
        (x / len, y / len)
    };
    let elevation = PATCH_ELEVATION.to_radians();
    let (flat, rise) = (elevation.cos(), elevation.tan());
    let light = [lx * flat, ly * flat, elevation.sin()];
    let diffuse = 1.0 - PATCH_AMBIENT;
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            for (x, px) in out.chunks_exact_mut(4).enumerate() {
                let colour = squares[(y / n) * cols + x / n].0;
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                // The slope, across the pixel, and how squarely it faces
                // the light against a flat face.
                let dx = surface(fx + 0.5, fy) - surface(fx - 0.5, fy);
                let dy = surface(fx, fy + 0.5) - surface(fx, fy - 0.5);
                let norm = (dx * dx + dy * dy + 1.0).sqrt();
                let facing = ((-dx * light[0] - dy * light[1] + light[2]) / norm).max(0.0);
                // The shadow: how far anything between here and the light
                // stands above the ray back to it.
                let here = surface(fx, fy);
                let mut over = 0.0f32;
                let mut t = PATCH_SHADOW_STEP;
                while t <= PATCH_SHADOW_REACH {
                    let h = surface(fx + lx * t, fy + ly * t);
                    over = over.max(h - here - t * rise);
                    t += PATCH_SHADOW_STEP;
                }
                let lit = 1.0 - (over / PATCH_SHADOW_SOFT).clamp(0.0, 1.0);
                let light_here = PATCH_AMBIENT + diffuse * facing * lit;
                // Against a square's own tilted face, so the face keeps the
                // square's colour.
                let slope = PATCH_TILT * relief / size;
                let light_flat = PATCH_AMBIENT
                    + diffuse * ((-slope * light[1] + light[2]) / (slope * slope + 1.0).sqrt());
                let k = light_here / light_flat;
                for c in 0..3 {
                    px[c] = (colour[c] * k).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: patching the picture does not change the
                // layer's shape.
            }
        });
}

/// How Texturizer lights its surface, beyond the controls.
///
/// **Texturizer's relief is far deeper than the other filters'.** Measured on
/// CS6's output over `samples/horse-3.jpg`, registered against the source,
/// its Brick at Relief 24 moves a pixel by 80 levels, one standard deviation,
/// and its Burlap and Sandstone at 16 by 70 and 40 — six to fifteen times
/// what the same Relief does in Rough Pastels. So each texture has its own
/// gain, fitted on those numbers; Canvas, which there was no reference for,
/// is a guess of the same order.
///
/// **And it lights every tone alike.** CS6 moves the horse's blacks as far
/// as the sky's whites — its brick throws white edges across the black
/// horse — so the share of the light that scales with the picture's own
/// brightness is mostly taken back out by the dark bias.
pub(crate) const TEXTURIZER_FINISH: Finish = Finish {
    dark_bias: 0.6,
    bevel: BEVEL,
    crisp: 1.0,
    occlusion: 0.0,
    patchy: 0.0,
    gain: 1.0,
};

pub(crate) const TEXTURIZER_BURLAP_GAIN: f32 = 13.0;

pub(crate) const TEXTURIZER_CANVAS_GAIN: f32 = 20.0;

pub(crate) const TEXTURIZER_SANDSTONE_GAIN: f32 = 15.0;

/// Brick is lit harder still, and crisply: CS6's shows a white lip along the
/// top of each course and a black joint under it, with the face between
/// keeping the picture's tone. The grit on the face slopes almost as steeply
/// as the joints do, so without the crispness — which lifts strong slopes
/// over faint ones — the depth that draws the joints turns every face to
/// speckle. The occlusion is what makes the joint black rather than merely
/// shaded.
pub(crate) const TEXTURIZER_BRICK_GAIN: f32 = 135.0;

pub(crate) const TEXTURIZER_BRICK_CRISP: f32 = 2.2;

pub(crate) const TEXTURIZER_BRICK_OCCLUSION: f32 = 60.0;

/// Filter ▸ Texture ▸ Texturizer: the picture printed on one of the four
/// surfaces, and nothing else done to it.
///
/// It is the texture block Rough Pastels, Underpainting and Conté Crayon
/// carry, on its own: the same controls in the same order — **Texture**,
/// **Scaling**, **Relief**, **Light** and **Invert** — over the same ranges,
/// and the same four surfaces, so a brick here is the brick there. It is lit
/// much more deeply than they are; see [`TEXTURIZER_FINISH`].
///
/// Alpha is left alone.
///
/// No GPU path, for [`apply_relief`]'s reasons.
pub fn texturizer(
    pixmap: &mut Pixmap,
    texture: Texture,
    scaling: u32,
    relief: u32,
    light: Light,
    invert: bool,
) {
    let finish = match texture {
        Texture::Brick => Finish {
            gain: TEXTURIZER_BRICK_GAIN,
            crisp: TEXTURIZER_BRICK_CRISP,
            occlusion: TEXTURIZER_BRICK_OCCLUSION,
            ..TEXTURIZER_FINISH
        },
        Texture::Burlap => Finish {
            gain: TEXTURIZER_BURLAP_GAIN,
            ..TEXTURIZER_FINISH
        },
        Texture::Canvas => Finish {
            gain: TEXTURIZER_CANVAS_GAIN,
            ..TEXTURIZER_FINISH
        },
        Texture::Sandstone => Finish {
            gain: TEXTURIZER_SANDSTONE_GAIN,
            ..TEXTURIZER_FINISH
        },
    };
    apply_relief_weighted(pixmap, texture, scaling, relief, light, invert, finish);
}

/// CS6's ranges for Stained Glass, which its three sliders run over.
pub const GLASS_CELL: std::ops::RangeInclusive<u32> = 2..=50;

pub const GLASS_BORDER: std::ops::RangeInclusive<u32> = 1..=20;

pub const GLASS_LIGHT: std::ops::RangeInclusive<u32> = 0..=10;

/// How far apart the panes' seeds are, in pixels per step of Cell Size.
/// CS6's panes are bigger than the slider reads: across a row its sky
/// crosses a lead every 20 pixels or so at Cell Size 10, and every 49 at 26.
pub(crate) const GLASS_SPACING: f32 = 2.2;

/// How far each seed is nudged off its lattice point, as a share of the
/// lattice step. CS6's panes are irregular but even — five and six sided,
/// none much bigger than the rest — which is a hexagonal lattice shaken a
/// little rather than points thrown down anyhow; a free scatter leaves slivers
/// beside panes three times their size.
pub(crate) const GLASS_JITTER: f32 = 0.6;

/// How wide the lead is, in pixels per step of Border Thickness. CS6's lead
/// is about two and a half pixels at 4 and seven at 10.
pub(crate) const GLASS_LEAD: f32 = 0.62;

/// Light Intensity's glow: how far it reaches from the middle of the image,
/// as a share of the image's size, and how much of the way to white it takes
/// the glass there at full intensity.
///
/// It grows with the **square** of the slider. Measured on CS6's output over
/// `samples/horse-3.jpg`, the glass in the middle goes 85% of the way to
/// white at 7 and hardly a tenth at 2 — a straight line through the first
/// would light the second three times too brightly. At 7 it is still half as
/// strong a fifth of the image out, and has faded to a sixth a third out.
pub(crate) const GLASS_GLOW_REACH: f32 = 0.26;

pub(crate) const GLASS_GLOW: f32 = 1.75;

/// Filter ▸ Texture ▸ Stained Glass: the picture remade as panes of flat
/// colour held in lead.
///
/// The panes are the cells of a Voronoi diagram — every pixel belongs to its
/// nearest seed — laid on a jittered hexagonal lattice whose spacing
/// follows **Cell Size**; see [`GLASS_JITTER`]. Each pane is filled with the
/// average of the picture under it. The lead between them is **Border
/// Thickness** wide and in the **foreground colour**, as CS6's is, and it
/// runs round the edge of the image as well, half as wide there, since the
/// pane on the far side is missing. **Light Intensity** is a glow centred on
/// the image, as if the window were lit from behind at its middle: it takes
/// the glass towards white and leaves the lead alone.
///
/// Alpha is left alone.
///
/// No GPU path, for Crystallize's reasons: each pane's average needs every
/// pixel in it gathered first, and the whole filter is a few tens of
/// milliseconds on the CPU.
pub fn stained_glass(
    pixmap: &mut Pixmap,
    cell_size: u32,
    border: u32,
    light: u32,
    lead: crate::photorust::pixmap::Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let step = GLASS_SPACING * cell_size.clamp(*GLASS_CELL.start(), *GLASS_CELL.end()) as f32;
    let row_step = step * 3f32.sqrt() / 2.0;
    let half_lead =
        0.5 * GLASS_LEAD * border.clamp(*GLASS_BORDER.start(), *GLASS_BORDER.end()) as f32;

    // Seeds from two steps before the canvas to two after, so a pixel at the
    // edge has neighbours on every side to be nearer to.
    let cols = (width as f32 / step).ceil() as i32 + 5;
    let rows = (height as f32 / row_step).ceil() as i32 + 5;
    let seed_at = |i: i32, j: i32| -> (f32, f32) {
        // Every other row is shifted half a step, which is what makes the
        // lattice hexagonal.
        let shift = 0.5 * (j & 1) as f32;
        (
            (i as f32 + shift + GLASS_JITTER * (lattice(i, j, 41) - 0.5)) * step,
            (j as f32 + GLASS_JITTER * (lattice(i, j, 42) - 0.5)) * row_step,
        )
    };
    let index_of = |i: i32, j: i32| -> usize { ((j + 2) * cols + (i + 2)) as usize };

    // Which pane each pixel is in, and how far it is from the pane's edge —
    // the nearer of the image's edge and the line halfway to a neighbouring
    // seed. Worked out once and kept: both the averages and the painting
    // need it.
    let (w, h) = (width as usize, height as usize);
    let mut owner = vec![0u32; w * h];
    let mut inset = vec![0f32; w * h];
    owner
        .par_chunks_exact_mut(w)
        .zip(inset.par_chunks_exact_mut(w))
        .enumerate()
        .for_each(|(row, (owners, insets))| {
            let py = row as f32 + 0.5;
            let j0 = (py / row_step).floor() as i32;
            for x in 0..w {
                let px = x as f32 + 0.5;
                let i0 = (px / step).floor() as i32;
                let mut near = [(0f32, 0f32); 25];
                let mut n = 0;
                let (mut best, mut best_at, mut best_index) = (f32::MAX, 0, 0usize);
                for j in (j0 - 2).max(-2)..=(j0 + 2).min(rows - 3) {
                    for i in (i0 - 2).max(-2)..=(i0 + 2).min(cols - 3) {
                        let (sx, sy) = seed_at(i, j);
                        let d = (sx - px) * (sx - px) + (sy - py) * (sy - py);
                        if d < best {
                            best = d;
                            best_at = n;
                            best_index = index_of(i, j);
                        }
                        near[n] = (sx, sy);
                        n += 1;
                    }
                }
                let (bx, by) = near[best_at];
                // The distance to the bisector between the nearest seed and
                // each other one, which is the distance to that side of the
                // pane.
                let mut edge = px.min(width as f32 - px).min(py).min(height as f32 - py);
                for (k, &(sx, sy)) in near[..n].iter().enumerate() {
                    if k == best_at {
                        continue;
                    }
                    let d = (sx - px) * (sx - px) + (sy - py) * (sy - py);
                    let apart = ((sx - bx) * (sx - bx) + (sy - by) * (sy - by)).sqrt();
                    if apart > 0.0 {
                        edge = edge.min((d - best) / (2.0 * apart));
                    }
                }
                owners[x] = best_index as u32;
                insets[x] = edge;
            }
        });

    // What each pane averages to, in one sweep: a per-thread tally of every
    // pane would cost more than the image at the smallest cell size.
    let count = (cols * rows) as usize;
    let mut totals = vec![[0u64; 3]; count];
    let mut counts = vec![0u32; count];
    for (i, px) in pixmap.as_bytes().chunks_exact(4).enumerate() {
        let pane = owner[i] as usize;
        for c in 0..3 {
            totals[pane][c] += px[c] as u64;
        }
        counts[pane] += 1;
    }
    let panes: Vec<[f32; 3]> = totals
        .iter()
        .zip(&counts)
        .map(|(t, &n)| {
            let n = n.max(1) as f32;
            [t[0] as f32 / n, t[1] as f32 / n, t[2] as f32 / n]
        })
        .collect();

    let reach = GLASS_GLOW_REACH * (width as f32 * height as f32).sqrt();
    let intensity = light.min(*GLASS_LIGHT.end()) as f32 / *GLASS_LIGHT.end() as f32;
    let peak = (GLASS_GLOW * intensity * intensity).min(1.0);
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    let lead = [lead.r as f32, lead.g as f32, lead.b as f32];
    let (owner, inset, panes) = (&owner, &inset, &panes);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let dy = row as f32 + 0.5 - cy;
            for (x, px) in out.chunks_exact_mut(4).take(w).enumerate() {
                let i = row * w + x;
                // A pixel's worth of soft edge, so the lead is not stepped.
                let glass = (inset[i] - half_lead + 0.5).clamp(0.0, 1.0);
                let dx = x as f32 + 0.5 - cx;
                let glow = peak * (-(dx * dx + dy * dy) / (reach * reach)).exp();
                let pane = panes[owner[i] as usize];
                for c in 0..3 {
                    let lit = pane[c] + (255.0 - pane[c]) * glow;
                    px[c] = (lead[c] + (lit - lead[c]) * glass)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
            }
        });
}
