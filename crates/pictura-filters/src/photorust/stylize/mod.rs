//! Filter ▸ Stylize.
//!
//! These do to a picture what a printmaker's technique does: they keep its
//! shapes and throw away its smoothness. Diffuse, Emboss, Extrude, Find Edges,
//! Solarize, Tiles, Trace Contour and Wind are built — the whole of CS6's
//! submenu — and Glowing Edges, which CS6 keeps in the Filter Gallery.

use crate::photorust::pixmap::{Pixmap, Rgba8};
pub(crate) use crate::{ContourEdge, DiffuseMode, ExtrudeType, TileFill, WindMethod};
use rayon::prelude::*;

impl DiffuseMode {}

/// How many passes Anisotropic makes, and so how far a pixel there reaches:
/// each pass reads one step out, and the next pass reads what that left.
/// [`crate::filters::Filter::reach`] has to agree with this, or a preview
/// cropped to a region comes out wrong along its edges.
pub const ANISOTROPIC_REACH: u32 = 4;

/// The eight neighbours of a pixel, in no order that matters.
pub(crate) const AROUND: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Filter ▸ Stylize ▸ Diffuse: shuffle each pixel with one of its neighbours.
///
/// The randomness is drawn from the pixel's own coordinates rather than from a
/// generator, so that a preview, the commit behind it and an undo/redo replay
/// all produce the same picture. CS6 re-rolls on every apply, which is the one
/// difference — but running it twice here still goes on diffusing, because the
/// second pass reads what the first one left.
pub fn diffuse(pixmap: &mut Pixmap, mode: DiffuseMode) {
    if pixmap.is_empty() {
        return;
    }
    if mode == DiffuseMode::Anisotropic {
        // Diffusion is iterative. One pass over the neighbours barely moves a
        // picture — on fine grain it only halves it, because half the
        // neighbours of a speckle are the same speckle — and the smeared look
        // CS6 gives is several steps of it.
        for _ in 0..ANISOTROPIC_REACH {
            let source = pixmap.clone();
            let (width, height) = (source.width() as i32, source.height() as i32);
            let stride = pixmap.stride();
            pixmap
                .as_bytes_mut()
                .par_chunks_exact_mut(stride)
                .enumerate()
                .for_each(|(row, out)| {
                    for x in 0..width {
                        let colour = along_the_edge(&source, x, row as i32, width, height);
                        let i = x as usize * 4;
                        out[i] = colour.r;
                        out[i + 1] = colour.g;
                        out[i + 2] = colour.b;
                        out[i + 3] = colour.a;
                    }
                });
        }
        return;
    }

    // Every output pixel reads its neighbours as they *were*: writing in
    // place would let a pixel already shuffled this pass be shuffled again by
    // the one next to it, which smears the result along each row.
    let source = pixmap.clone();
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let here = source.get(x, y);
                // Clamped rather than wrapped or left transparent: a
                // neighbour off the edge of the layer would otherwise punch
                // holes along it.
                let (dx, dy) = AROUND[(hash(x as u32, y as u32) % 8) as usize];
                let there = source.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1));
                let colour = match mode {
                    DiffuseMode::Normal => there,
                    // Compared by brightness and then taken whole, rather
                    // than channel by channel: the darker of two colours per
                    // channel is a third colour that was never in the
                    // picture, and it shows as a shift in hue along every
                    // edge.
                    DiffuseMode::DarkenOnly => {
                        if luma(there) < luma(here) {
                            there
                        } else {
                            here
                        }
                    }
                    DiffuseMode::LightenOnly => {
                        if luma(there) > luma(here) {
                            there
                        } else {
                            here
                        }
                    }
                    // Handled above, in its own iterated pass.
                    DiffuseMode::Anisotropic => here,
                };
                let i = x as usize * 4;
                out[i] = colour.r;
                out[i + 1] = colour.g;
                out[i + 2] = colour.b;
                out[i + 3] = colour.a;
            }
        });
}

/// One step of diffusion that flows along an edge rather than across it.
///
/// Each neighbour is weighted by how much it differs from this pixel, so
/// neighbours on the same side of an edge are averaged in and ones on the far
/// side are barely counted. That is the whole difference between this and a
/// blur: a blur would take the edge with it.
pub(crate) fn along_the_edge(source: &Pixmap, x: i32, y: i32, width: i32, height: i32) -> Rgba8 {
    let here = source.get(x, y);
    // How big a colour difference counts as an edge, in levels. Small enough
    // that a real edge survives, large enough that the grain within a flat
    // region is smoothed away.
    const EDGE: f32 = 28.0;

    let mut total = 1.0f32;
    let mut sum = (here.r as f32, here.g as f32, here.b as f32, here.a as f32);
    for (dx, dy) in AROUND {
        let there = source.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1));
        let difference = ((there.r as f32 - here.r as f32).powi(2)
            + (there.g as f32 - here.g as f32).powi(2)
            + (there.b as f32 - here.b as f32).powi(2))
        .sqrt();
        let weight = (-(difference / EDGE).powi(2)).exp();
        sum.0 += there.r as f32 * weight;
        sum.1 += there.g as f32 * weight;
        sum.2 += there.b as f32 * weight;
        sum.3 += there.a as f32 * weight;
        total += weight;
    }
    Rgba8::new(
        (sum.0 / total).round().clamp(0.0, 255.0) as u8,
        (sum.1 / total).round().clamp(0.0, 255.0) as u8,
        (sum.2 / total).round().clamp(0.0, 255.0) as u8,
        (sum.3 / total).round().clamp(0.0, 255.0) as u8,
    )
}

/// Filter ▸ Stylize ▸ Emboss: the picture as though stamped into metal.
///
/// Flat grey everywhere the picture was flat, with a light and a dark edge
/// wherever it changed — which is the *difference* between what is on one
/// side of a pixel and what is on the other, measured along the angle the
/// light comes from. `angle` is where that light is, in degrees; `height` is
/// how far apart the two sides are, in pixels, which is how thick the relief
/// looks; `amount` (CS6's 1–500%) is how hard it is pressed.
///
/// Differenced channel by channel rather than on brightness, which is what
/// leaves the coloured fringes along an edge between two colours of the same
/// tone. CS6 does the same, and an emboss that worked on brightness alone
/// would come back a flat grey relief with the colour thrown away.
pub fn emboss(pixmap: &mut Pixmap, angle: f32, height: f32, amount: f32) {
    if pixmap.is_empty() {
        return;
    }
    let height = height.clamp(1.0, 100.0);
    let strength = amount.clamp(1.0, 500.0) / 100.0;
    let radians = angle.to_radians();
    // The house convention, shared with Motion Blur: y runs down the picture,
    // so the sine is negated and an angle reads the way it does on the dial.
    let (dx, dy) = (radians.cos(), -radians.sin());

    // Height is the thickness of the relief, not just how far apart the two
    // samples are. Reading a sharp edge at two points a long way apart would
    // give two thin lines with nothing between them — a doubled ghost rather
    // than a bevel — so the picture is softened first, by half the distance
    // the samples are about to span.
    let mut source = pixmap.clone();
    let blur = ((height - 1.0) / 2.0).round() as u32;
    if blur > 0 {
        crate::photorust::convolve::box_blur(&mut source, blur);
    }
    let reach = height / 2.0;

    let width = pixmap.width() as i32;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as f32 + 0.5;
            for x in 0..width {
                let at = x as f32 + 0.5;
                // Brightness is read as height, so the two samples are the
                // ground on either side of this pixel along the light's line.
                let toward = crate::photorust::pixmap::bilinear(
                    &source,
                    at + dx * reach - 0.5,
                    y + dy * reach - 0.5,
                );
                let away = crate::photorust::pixmap::bilinear(
                    &source,
                    at - dx * reach - 0.5,
                    y - dy * reach - 0.5,
                );
                let i = x as usize * 4;
                for (channel, (near, far)) in
                    [(toward.r, away.r), (toward.g, away.g), (toward.b, away.b)]
                        .into_iter()
                        .enumerate()
                {
                    // Ground that *falls away* towards the lamp is tilted to
                    // face it, and so is the lit side — which is why the far
                    // sample is the positive one. Get this the other way
                    // about and the whole picture reads as stamped in from
                    // the back, lit from the opposite corner to the one the
                    // angle names.
                    //
                    // Mid grey where nothing changes at all, which is what
                    // makes an embossed picture read as unlit metal rather
                    // than as a darkened photograph.
                    let relief = 128.0 + (far as f32 - near as f32) * strength;
                    out[i + channel] = relief.clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: stamping a layer does not change its shape.
            }
        });
}

/// Filter ▸ Stylize ▸ Find Edges: draw the picture's edges as dark lines on
/// white.
///
/// Every channel is run through a Sobel gradient and the result inverted, so
/// flat ground — where nothing changes and the gradient is zero — comes back
/// white, and a step between two tones comes back as a dark line. The gradient
/// is *not* normalised: it is the raw Sobel, clamped at 255, which is what
/// makes the lines bold — a step of even a quarter of the range goes to black
/// and the picture's own texture comes through as grey. Dividing it down to a
/// full-contrast step instead leaves a photograph almost white.
///
/// Working channel by channel rather than on brightness is what leaves the
/// coloured fringes along an edge between two colours: only the channels that
/// actually change darken, so the line takes the colour of the darker side. On
/// brightness alone the same edge would come back grey.
///
/// There is no GPU path. It is a 3×3 neighbourhood op and so would fit the
/// shader shape, but like the rest of the Blur and Stylize families it uploads
/// its input and reads the result straight back, which the measurements in
/// docs/gpu-migration.md say rarely pays for the trip.
pub fn find_edges(pixmap: &mut Pixmap) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // Premultiplied, as the rest of the convolution family is. On a layer with
    // soft edges, straight-alpha neighbours would otherwise read the hidden
    // colour of transparent pixels as a step and draw a halo round the
    // subject. The result is an intensity rather than an average, so it is
    // written back straight; the alpha it came with stands.
    let mut source = pixmap.clone();
    source.premultiply();
    let source = &source;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                // Clamp-to-edge, so the border does not read as a cliff.
                let sample = |dx: i32, dy: i32| {
                    source.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1))
                };
                let (tl, t, tr) = (sample(-1, -1), sample(0, -1), sample(1, -1));
                let (l, r) = (sample(-1, 0), sample(1, 0));
                let (bl, b, br) = (sample(-1, 1), sample(0, 1), sample(1, 1));

                let i = x as usize * 4;
                for c in 0..3 {
                    let pick = |p: Rgba8| match c {
                        0 => p.r as f32,
                        1 => p.g as f32,
                        _ => p.b as f32,
                    };
                    let gx =
                        -pick(tl) - 2.0 * pick(l) - pick(bl) + pick(tr) + 2.0 * pick(r) + pick(br);
                    let gy =
                        -pick(tl) - 2.0 * pick(t) - pick(tr) + pick(bl) + 2.0 * pick(b) + pick(br);
                    // The raw Sobel, clamped, then inverted: full contrast
                    // lands on black, nothing changing on white.
                    let magnitude = (gx * gx + gy * gy).sqrt().min(255.0);
                    out[i + c] = (255.0 - magnitude) as u8;
                }
                // Alpha stands: finding edges does not change the layer's shape.
            }
        });
}

impl ExtrudeType {}

/// Everything CS6's Extrude dialog collects.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ExtrudeOptions {
    pub kind: ExtrudeType,
    /// The grid's square, in pixels (CS6's 2–255).
    pub size: u32,
    /// How far the towers stand (1–255), and whether that comes from the
    /// tile's own brightness rather than from a roll of the dice.
    pub depth: f32,
    pub level_based: bool,
    /// Face the blocks with one flat colour instead of their piece of the
    /// picture. Read only for blocks.
    pub solid_front: bool,
    /// Leave out the part-tiles along the right and bottom edges, where the
    /// grid does not divide the picture evenly.
    pub mask_incomplete: bool,
}

impl Default for ExtrudeOptions {
    fn default() -> Self {
        Self {
            kind: ExtrudeType::Blocks,
            size: 30,
            depth: 30.0,
            level_based: false,
            solid_front: false,
            mask_incomplete: false,
        }
    }
}

/// How far the furthest tower is thrown outwards, as a fraction of its
/// distance from the middle of the frame, at full Depth.
pub(crate) const THROW: f32 = 0.85;

/// How each face is lit, in the order [top, right, bottom, left]. A lamp up
/// and to the left, which is where CS6's is.
pub(crate) const FACES: [f32; 4] = [1.30, 0.78, 0.60, 1.12];

/// Filter ▸ Stylize ▸ Extrude: break the picture into towers standing out of
/// the frame.
///
/// The whole thing is one perspective from a viewer over the middle of the
/// picture: a tile pushed towards them moves *away from the centre* and grows,
/// which is what makes the towers lean outwards and why the ones in the middle
/// stand square on. Everything is drawn back to front, so a nearer tower hides
/// what is behind it — the cheapest way to get that right, and the reason this
/// is the one filter here that cannot run a row at a time across threads.
///
/// The towers are drawn *over* the picture rather than onto an empty frame, so
/// what shows between them is the original — as CS6 leaves it.
pub fn extrude(pixmap: &mut Pixmap, opt: ExtrudeOptions) {
    if pixmap.is_empty() {
        return;
    }
    let size = opt.size.clamp(2, 255);
    let (width, height) = (pixmap.width(), pixmap.height());
    let source = pixmap.clone();
    let middle = (width as f32 / 2.0, height as f32 / 2.0);
    let throw = opt.depth.clamp(1.0, 255.0) / 255.0 * THROW;

    // Every tile, with how far it stands. Sorted before anything is drawn:
    // painting them in grid order would let a tile at the back cover one in
    // front of it purely because it came later in the picture.
    let mut towers: Vec<(f32, crate::photorust::pixmap::Rect, Rgba8)> = Vec::new();
    let mut y = 0u32;
    while y < height {
        let mut x = 0u32;
        while x < width {
            let tile = crate::photorust::pixmap::Rect::new(
                x as i32,
                y as i32,
                size.min(width - x),
                size.min(height - y),
            );
            let whole = tile.width == size && tile.height == size;
            if whole || !opt.mask_incomplete {
                let average = average_of(&source, tile);
                let stands = if opt.level_based {
                    // Bright tiles stand tallest, which is what makes a
                    // level-based extrude read as a relief of the picture.
                    luma(average) / 255.0
                } else {
                    hash(x, y) as f32 / u32::MAX as f32
                };
                towers.push((stands, tile, average));
            }
            x += size;
        }
        y += size;
    }
    towers.sort_by(|a, b| a.0.total_cmp(&b.0));

    for (stands, tile, average) in towers {
        let grow = 1.0 + throw * stands;
        let out = |px: f32, py: f32| {
            (
                middle.0 + (px - middle.0) * grow,
                middle.1 + (py - middle.1) * grow,
            )
        };
        let (x0, y0) = (tile.x as f32, tile.y as f32);
        let (x1, y1) = (x0 + tile.width as f32, y0 + tile.height as f32);
        let base = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
        let front = base.map(|(px, py)| out(px, py));

        match opt.kind {
            ExtrudeType::Blocks => {
                // The four walls, then the face over them. A wall on the far
                // side of a tower lands inside its own front face and is
                // painted over by it, which is back-face culling for free.
                for edge in 0..4 {
                    let next = (edge + 1) % 4;
                    fill_convex(
                        pixmap,
                        &[base[edge], base[next], front[next], front[edge]],
                        shade(average, FACES[edge]),
                    );
                }
                if opt.solid_front {
                    fill_convex(pixmap, &front, average);
                } else {
                    face_of_picture(pixmap, &source, tile, front[0], front[2]);
                }
            }
            ExtrudeType::Pyramids => {
                // A spike from the tile's own square up to a point over its
                // middle. With no face to carry the picture, all that tells
                // them apart is the shading — which is why a pyramid in the
                // dead centre of the frame, thrown nowhere at all, still
                // reads as a pyramid.
                let apex = out((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                for edge in 0..4 {
                    let next = (edge + 1) % 4;
                    fill_convex(
                        pixmap,
                        &[base[edge], base[next], apex],
                        shade(average, FACES[edge]),
                    );
                }
            }
        }
    }
}

/// The mean colour of a tile, which is what its walls are painted in.
pub(crate) fn average_of(source: &Pixmap, tile: crate::photorust::pixmap::Rect) -> Rgba8 {
    let mut sum = [0u64; 4];
    let mut count = 0u64;
    for y in tile.y..tile.y + tile.height as i32 {
        for x in tile.x..tile.x + tile.width as i32 {
            let p = source.get(x, y);
            sum[0] += p.r as u64;
            sum[1] += p.g as u64;
            sum[2] += p.b as u64;
            sum[3] += p.a as u64;
            count += 1;
        }
    }
    if count == 0 {
        return Rgba8::TRANSPARENT;
    }
    Rgba8::new(
        (sum[0] / count) as u8,
        (sum[1] / count) as u8,
        (sum[2] / count) as u8,
        (sum[3] / count) as u8,
    )
}

/// Lighten or darken a wall, keeping its alpha.
pub(crate) fn shade(c: Rgba8, by: f32) -> Rgba8 {
    let level = |v: u8| (v as f32 * by).clamp(0.0, 255.0) as u8;
    Rgba8::new(level(c.r), level(c.g), level(c.b), c.a)
}

/// Paint a block's face with its own piece of the picture, stretched to the
/// size the perspective made of it.
pub(crate) fn face_of_picture(
    pixmap: &mut Pixmap,
    source: &Pixmap,
    tile: crate::photorust::pixmap::Rect,
    top_left: (f32, f32),
    bottom_right: (f32, f32),
) {
    let (fx0, fy0) = top_left;
    let (fx1, fy1) = bottom_right;
    let (span_x, span_y) = (fx1 - fx0, fy1 - fy0);
    if span_x <= 0.0 || span_y <= 0.0 {
        return;
    }
    let from = (fy0.floor().max(0.0) as i32).max(0);
    let to = (fy1.ceil() as i32).min(pixmap.height() as i32);
    let left = (fx0.floor().max(0.0) as i32).max(0);
    let right = (fx1.ceil() as i32).min(pixmap.width() as i32);
    for y in from..to {
        let v = (y as f32 + 0.5 - fy0) / span_y;
        if !(0.0..1.0).contains(&v) {
            continue;
        }
        for x in left..right {
            let u = (x as f32 + 0.5 - fx0) / span_x;
            if !(0.0..1.0).contains(&u) {
                continue;
            }
            let sample = crate::photorust::pixmap::bilinear(
                source,
                tile.x as f32 + u * tile.width as f32 - 0.5,
                tile.y as f32 + v * tile.height as f32 - 0.5,
            );
            pixmap.set(x, y, sample);
        }
    }
}

/// Fill a convex polygon — a wall, or one side of a spike.
///
/// Convex, so a row crosses the outline exactly twice and the span between
/// those two crossings is the inside. A general polygon filler would need to
/// sort the crossings and pair them off; nothing here is ever concave.
pub(crate) fn fill_convex(pixmap: &mut Pixmap, points: &[(f32, f32)], colour: Rgba8) {
    if points.len() < 3 {
        return;
    }
    let top = points.iter().fold(f32::MAX, |a, p| a.min(p.1));
    let bottom = points.iter().fold(f32::MIN, |a, p| a.max(p.1));
    let first = (top.floor() as i32).max(0);
    let last = (bottom.ceil() as i32).min(pixmap.height() as i32);

    for y in first..last {
        // Rows are sampled down the middle, as everything else here is.
        let scan = y as f32 + 0.5;
        let (mut left, mut right) = (f32::MAX, f32::MIN);
        for i in 0..points.len() {
            let (ax, ay) = points[i];
            let (bx, by) = points[(i + 1) % points.len()];
            // A horizontal edge crosses nothing; the two edges either side of
            // it answer for its row.
            if (ay <= scan) == (by <= scan) {
                continue;
            }
            let t = (scan - ay) / (by - ay);
            let x = ax + (bx - ax) * t;
            left = left.min(x);
            right = right.max(x);
        }
        if left > right {
            continue;
        }
        let from = (left.round() as i32).max(0);
        let to = (right.round() as i32).min(pixmap.width() as i32);
        for x in from..to {
            pixmap.set(x, y, colour);
        }
    }
}

impl TileFill {}

/// Everything CS6's Tiles dialog collects, plus the two swatch colours, which
/// it does not ask for because they belong to the document.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TileOptions {
    /// CS6's "Number Of Tiles", 1–99. It counts tiles across the *shorter*
    /// side, so the tiles come out square and 10 on a portrait photograph
    /// means ten columns rather than ten rows.
    pub count: u32,
    /// CS6's "Maximum Offset", 1–99, a percentage of the tile's own size —
    /// so the same figure shifts a big tile further than a small one.
    pub offset: u32,
    pub fill: TileFill,
    /// Filled in by the bridge from the document's swatches.
    pub foreground: Rgba8,
    pub background: Rgba8,
}

impl Default for TileOptions {
    fn default() -> Self {
        Self {
            count: 10,
            offset: 10,
            fill: TileFill::BackgroundColor,
            foreground: Rgba8::BLACK,
            background: Rgba8::WHITE,
        }
    }
}

/// Filter ▸ Stylize ▸ Tiles: cut the picture into squares and nudge each one
/// off where it was.
///
/// The ground is laid first — whichever of the four fills was asked for — and
/// then every tile is dropped onto it at its own small offset, so what shows
/// through is the gap each tile left behind. Tiles never overlap by more than
/// they are offset, and nothing is scaled: a tile is a straight copy of its
/// square, moved.
///
/// The offsets are drawn from the tile's position in the grid rather than from
/// a RNG, so re-running the filter during an undo/redo replay lands every tile
/// exactly where it was. That is also why [`crate::filters::Filter::reach`]
/// says `None` for it: the grid is laid on the layer's own corner, so a crop
/// would start the grid somewhere else and its tiles would not line up with
/// the ones either side.
///
/// No GPU path. Each tile is a block copy — memory movement, not arithmetic —
/// and a shader would still have to upload the picture and read it back
/// (docs/gpu-migration.md).
pub fn tiles(pixmap: &mut Pixmap, opt: TileOptions) {
    if pixmap.is_empty() {
        return;
    }
    let (width, height) = (pixmap.width(), pixmap.height());
    let count = opt.count.clamp(1, 99);
    // Square tiles, counted across the shorter side. At least one pixel, or a
    // tall thin selection with 99 tiles asked of it would divide to nothing.
    let size = (width.min(height) / count).max(1);
    // CS6's percentage is of the tile, and at least a pixel once it is asked
    // for at all — otherwise 1% of a small tile rounds to no movement and the
    // filter appears to do nothing.
    let reach = ((size as f32 * opt.offset.clamp(0, 99) as f32 / 100.0).round() as i32).max(1);

    let source = pixmap.clone();

    // The ground the tiles land on.
    match opt.fill {
        TileFill::BackgroundColor => pixmap.fill(opt.background),
        TileFill::ForegroundColor => pixmap.fill(opt.foreground),
        TileFill::UnalteredImage => {}
        TileFill::InverseImage => {
            for px in pixmap.as_bytes_mut().chunks_exact_mut(4) {
                px[0] = 255 - px[0];
                px[1] = 255 - px[1];
                px[2] = 255 - px[2];
                // Alpha stands: inverting is a colour, not a shape.
            }
        }
    }

    let columns = width.div_ceil(size);
    let rows = height.div_ceil(size);
    for row in 0..rows {
        for column in 0..columns {
            // Two offsets from one hash, taken from opposite ends of it so
            // that a tile's horizontal and vertical shifts are independent.
            let h = hash(column, row);
            let span = (reach * 2 + 1) as u32;
            let dx = (h % span) as i32 - reach;
            let dy = ((h >> 16) % span) as i32 - reach;

            let (left, top) = ((column * size) as i32, (row * size) as i32);
            for y in top..(top + size as i32).min(height as i32) {
                for x in left..(left + size as i32).min(width as i32) {
                    let (tx, ty) = (x + dx, y + dy);
                    if tx >= 0 && ty >= 0 && tx < width as i32 && ty < height as i32 {
                        pixmap.set(tx, ty, source.get(x, y));
                    }
                }
            }
        }
    }
}

impl ContourEdge {}

/// Filter ▸ Stylize ▸ Trace Contour: draw the line where each channel crosses
/// a brightness.
///
/// It is a contour map of the picture: pick a level, and wherever a channel
/// steps across it, mark the pixel on the side Edge names. Everything else
/// goes white.
///
/// The three channels are traced *separately* and marked to black on their
/// own, which is where the colours in the result come from: a boundary only
/// the red channel crosses leaves red at 0 and the other two at 255, so the
/// line is cyan. Black lines are where all three cross together. Tracing
/// brightness instead would give a single black line and lose the whole
/// character of the filter.
///
/// The frame's own edge is sampled clamped, so the border does not read as a
/// crossing and get outlined all the way round.
///
/// No GPU path. It is a 3×3 neighbourhood test, so it would fit the shader
/// shape, but like the rest of this family it would upload its input and read
/// the result straight back (docs/gpu-migration.md).
pub fn trace_contour(pixmap: &mut Pixmap, level: u8, edge: ContourEdge) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = pixmap.clone();
    let source = &source;
    let level = level as i32;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let sample = |dx: i32, dy: i32| {
                    source.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1))
                };
                let here = sample(0, 0);
                // The four neighbours: a crossing is a step between two
                // pixels that share a side, so the diagonals would only
                // thicken the line without finding anything new.
                let around = [sample(-1, 0), sample(1, 0), sample(0, -1), sample(0, 1)];

                let i = x as usize * 4;
                for c in 0..3 {
                    let pick = |p: Rgba8| match c {
                        0 => p.r as i32,
                        1 => p.g as i32,
                        _ => p.b as i32,
                    };
                    // On the side Edge names, with a neighbour on the other
                    // side of the level — which is exactly "the boundary
                    // passes between us, and I am the one who gets inked".
                    let mine = pick(here);
                    let marked = match edge {
                        ContourEdge::Upper => {
                            mine >= level && around.iter().any(|&n| pick(n) < level)
                        }
                        ContourEdge::Lower => {
                            mine < level && around.iter().any(|&n| pick(n) >= level)
                        }
                    };
                    out[i + c] = if marked { 0 } else { 255 };
                }
                // Alpha stands: the contour is drawn on the layer's own shape.
            }
        });
}

impl WindMethod {
    /// How far a streak can run, how many edges get one (out of 100), and how
    /// much of the streak's strength is left at its far end.
    fn shape(self) -> (u32, u32, f32) {
        match self {
            WindMethod::Wind => (12, 55, 0.0),
            WindMethod::Blast => (32, 85, 0.55),
            WindMethod::Stagger => (16, 70, 0.25),
        }
    }
}

/// How different two neighbouring pixels have to be, in levels of brightness,
/// before the wind can catch the edge between them. Low enough that a
/// photograph streaks all over, high enough that film grain does not.
pub(crate) const WIND_EDGE: f32 = 10.0;

/// Filter ▸ Stylize ▸ Wind: blow the picture sideways off its edges.
///
/// Every row is worked on its own — the wind is horizontal, so nothing crosses
/// between rows — and within a row the filter looks for a step *down* in
/// brightness away from the wind. Where it finds one, the bright pixel at the
/// step is dragged downwind over a few pixels, fading as it goes, which is the
/// streak. Flat ground has no step to catch and so comes through untouched,
/// which is why the effect reads as edges torn sideways rather than as a blur.
///
/// Because only that one polarity of step counts, a shape streaks off the side
/// the wind blows it towards and the other side stays clean — which is what
/// CS6 does, and the thing that makes the Direction setting visible at all.
///
/// `from_right` is CS6's Direction: the wind *comes from* that side, so "From
/// the Right" drags the picture towards the left.
///
/// The three methods are the same machine at different settings — see
/// [`WindMethod::shape`] — except that Stagger also lets a streak wander a row
/// up or down as it runs. It reads its neighbouring rows but never writes to
/// them, so the rows still parallelise.
///
/// The streak lengths are drawn from each pixel's coordinates rather than from
/// a RNG, so a preview, the commit behind it and an undo/redo replay all blow
/// the same way. CS6 re-rolls on every apply; that is the one difference, and
/// the same one Diffuse already carries.
///
/// No GPU path. A streak writes over the pixels ahead of it and the next
/// streak writes over that, so the work is sequential along a row rather than
/// per-pixel — the shape a shader is worst at.
pub fn wind(pixmap: &mut Pixmap, method: WindMethod, from_right: bool) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = pixmap.clone();
    let source = &source;
    // Which way the picture travels: away from where the wind comes from.
    let step = if from_right { -1 } else { 1 };
    let (longest, density, tail) = method.shape();

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let here = source.get(x, y);
                // The pixel the wind reached first. A step between the two is
                // an edge facing into it, and so something for it to catch:
                // what gets dragged is that upwind pixel, over this one and
                // the ones beyond. Dragging *this* pixel forward instead
                // would smear the far side of the edge over itself and show
                // nothing at all.
                let behind = (x - step).clamp(0, width - 1);
                let upwind = source.get(behind, y);
                // Only where the upwind pixel is the *brighter* of the two.
                // That is what keeps the streaks to one side of a shape: a
                // bright subject on a dark ground is caught where it gives
                // way to the ground and blown out over it, while its other
                // side — where the ground gives way to the subject — is left
                // alone. Catching both would streak every shape from both
                // sides at once, which is not what the wind does.
                if luma(upwind) - luma(here) < WIND_EDGE {
                    continue;
                }

                let h = hash(x as u32, y as u32);
                if h % 100 >= density {
                    continue;
                }
                let length = 1 + (h >> 8) % longest;

                for i in 0..length as i32 {
                    let tx = x + step * i;
                    if tx < 0 || tx >= width {
                        break;
                    }
                    // Stagger's wander: the streak reads a row above or below
                    // as it runs, so its far end no longer lines up with the
                    // edge it came from.
                    let sy = match method {
                        WindMethod::Stagger => {
                            let drift = (hash(x as u32, (y + i) as u32) % 3) as i32 - 1;
                            (y + drift).clamp(0, height - 1)
                        }
                        _ => y,
                    };
                    let colour = source.get(behind, sy);

                    // Full strength at the edge, falling to `tail` at the far
                    // end — a streak that stopped dead would read as a bar.
                    let t = i as f32 / length as f32;
                    let weight = 1.0 - t * (1.0 - tail);
                    let i = tx as usize * 4;
                    for (c, value) in [colour.r, colour.g, colour.b].into_iter().enumerate() {
                        let was = out[i + c] as f32;
                        out[i + c] = (was + (value as f32 - was) * weight).round() as u8;
                    }
                    // Alpha stands: the wind moves colour about, and a streak
                    // that carried alpha with it would tear holes in the layer.
                }
            }
        });
}

/// CS6's ranges for Glowing Edges, which its three sliders run over.
pub const GLOW_WIDTH: std::ops::RangeInclusive<u32> = 1..=14;

pub const GLOW_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=20;

pub const GLOW_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// How much Gaussian each step of Smoothness is worth, in pixels of radius.
pub(crate) const GLOW_SMOOTHING: f32 = 0.3;

/// Glowing Edges: the picture's edges lit up on a black ground.
///
/// It is Find Edges the other way about — the raw gradient kept rather than
/// inverted, so what changes glows and what is flat goes black — with the
/// three controls CS6 gives it:
///
/// * **Smoothness** blurs the picture before the gradient is taken, so grain
///   and fine texture stop registering as edges and only real boundaries
///   light up.
/// * **Edge Width** thickens the line afterwards, by letting each pixel take
///   the brightest gradient within half that width of it.
/// * **Edge Brightness** is the gain on the result, and the reason the lines
///   blow out to white at the top of its range.
///
/// The gradient is taken channel by channel, which is where the colour comes
/// from: a pink petal against dark ground steps furthest in red, so its outline
/// glows magenta, while a boundary all three channels cross comes back white.
/// On brightness alone every edge would be the same grey.
///
/// Note that CS6 keeps this one in the Filter Gallery rather than in the
/// Filter ▸ Stylize submenu. The Gallery is not built (docs/ROADMAP.md), so the
/// submenu is where it is reachable from.
///
/// No GPU path, for the reason the rest of this family has none: it would
/// upload its input and read the result straight back
/// (docs/gpu-migration.md). The blur inside it does go through the backend.
pub fn glowing_edges(pixmap: &mut Pixmap, width: u32, brightness: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let smoothness = smoothness.clamp(*GLOW_SMOOTHNESS.start(), *GLOW_SMOOTHNESS.end());
    let width = width.clamp(*GLOW_WIDTH.start(), *GLOW_WIDTH.end());
    let brightness = brightness.clamp(*GLOW_BRIGHTNESS.start(), *GLOW_BRIGHTNESS.end());

    // Smoothness first: the gradient is taken on the blurred copy, so what
    // the blur removed never becomes an edge in the first place.
    let mut smoothed = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut smoothed,
        smoothness as f32 * GLOW_SMOOTHING,
    );
    // Premultiplied, as the rest of the convolution family is: on a layer
    // with soft edges, straight-alpha neighbours would read the hidden colour
    // of transparent pixels as a step and ring the subject with a glow it
    // does not have.
    smoothed.premultiply();

    let mut edges = gradient(&smoothed);
    if width / 2 > 0 {
        edges = crate::photorust::convolve::dilate(&edges, (width / 2) as i32);
    }

    // Edge Brightness is a plain gain, and CS6's 6 is about life-size.
    let gain = brightness as f32 / 5.0;
    let stride = pixmap.stride();
    let edges = &edges;
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..edges.width() as i32 {
                let lit = edges.get(x, y);
                let i = x as usize * 4;
                for (c, value) in [lit.r, lit.g, lit.b].into_iter().enumerate() {
                    out[i + c] = (value as f32 * gain).min(255.0) as u8;
                }
                // Alpha stands: lighting the edges does not change the
                // layer's shape.
            }
        });
}

/// The raw Sobel magnitude of each channel, as a picture in its own right.
///
/// Shared ground with [`find_edges`], which inverts this and draws it on
/// white; here it is what glows.
pub(crate) fn gradient(source: &Pixmap) -> Pixmap {
    let width = source.width() as i32;
    let height = source.height() as i32;
    let mut out = Pixmap::new(source.width(), source.height());

    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                // Clamp-to-edge, so the border does not read as a cliff and
                // light up all the way round the frame.
                let sample = |dx: i32, dy: i32| {
                    source.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1))
                };
                let (tl, t, tr) = (sample(-1, -1), sample(0, -1), sample(1, -1));
                let (l, r) = (sample(-1, 0), sample(1, 0));
                let (bl, b, br) = (sample(-1, 1), sample(0, 1), sample(1, 1));

                let i = x as usize * 4;
                for c in 0..3 {
                    let pick = |p: Rgba8| match c {
                        0 => p.r as f32,
                        1 => p.g as f32,
                        _ => p.b as f32,
                    };
                    let gx =
                        -pick(tl) - 2.0 * pick(l) - pick(bl) + pick(tr) + 2.0 * pick(r) + pick(br);
                    let gy =
                        -pick(tl) - 2.0 * pick(t) - pick(tr) + pick(bl) + 2.0 * pick(b) + pick(br);
                    out[i + c] = (gx * gx + gy * gy).sqrt().min(255.0) as u8;
                }
                out[i + 3] = 255;
            }
        });
    out
}

/// Perceived brightness, for the two modes that ask which of two colours is
/// the darker.
#[inline]
pub(crate) fn luma(c: Rgba8) -> f32 {
    0.299 * c.r as f32 + 0.587 * c.g as f32 + 0.114 * c.b as f32
}

/// Which neighbour a pixel reaches for. Seeded from where the pixel is, so
/// the answer is the same every time the filter runs over the same picture.
#[inline]
pub(crate) fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x27d4_eb2d)
        ^ y.wrapping_mul(0x1656_67b1)
        ^ crate::photorust::seed().wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_f491);
    h ^= h >> 13;
    h
}

#[cfg(test)]
mod tests;
