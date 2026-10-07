//! Graphic Pen, Halftone Pattern, Note Paper, Photocopy, Plaster (split from `sketch` for file size).

#[allow(unused_imports)]
use super::*;

/// Filter ▸ Sketch ▸ Graphic Pen: the picture drawn in fine pen strokes lying
/// along **Stroke Direction**, in the two swatches.
///
/// The picture's brightness is read as how much of the sheet takes ink — dark
/// means most of it, light almost none — and a field of noise decides *where*,
/// so a midtone comes back as a hatch of separate strokes rather than as an
/// even tone. The strokes are drawn by noise that is coherent along the
/// direction and uncorrelated across it, which is what makes them lie the way
/// the direction says while falling at random rather than on a printed screen.
/// What is inked takes the foreground colour and the rest the background:
/// there is no middle tone, which is what makes this read as a pen drawing
/// rather than as a tinted photograph.
///
/// **Stroke Length** is how long one stroke is. Short, and the marks are dots
/// and the drawing is dithering that keeps the photograph's detail; long, and
/// they are dashes and the texture smooths away. **Light/Dark Balance** slides
/// the whole thing towards ink or towards paper, and wound well up the darkest
/// areas reach solid black as the strokes run together.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: the marks are laid by where on the
/// canvas a pixel is, and the work per pixel is one comparison.
pub fn graphic_pen(
    pixmap: &mut Pixmap,
    stroke_length: u32,
    balance: u32,
    direction: StrokeDirection,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let length = stroke_length.clamp(*PEN_LENGTH.start(), *PEN_LENGTH.end()) as f32;
    let balance = balance.clamp(*PEN_BALANCE.start(), *PEN_BALANCE.end()) as f32 / 100.0;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The strokes run along this direction. The field below stays with itself
    // along it for about Stroke Length and is uncorrelated across it, so a
    // threshold taken through the field comes back as marks of that length
    // lying that way.
    let angle = direction.angle().to_radians();
    let (dx, dy) = (angle.cos(), -angle.sin());
    let streak = (PEN_STREAK + length * PEN_STREAK_PER_STEP).max(1.0);
    let mut hatch = crate::photorust::artistic::streaked_noise(w, h, streak, 161, (dx, dy));
    unit_spread(&mut hatch);

    let shift = (balance - 0.5) * PEN_BALANCE_SPAN;
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
    let hatch = &hatch;

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let tone =
                    (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0;
                // How much of the sheet this tone wants inked: its darkness,
                // slid towards ink or paper by Balance and wound up by the
                // gain so that a dark area reaches solid.
                let coverage = ((1.0 - tone + shift) * PEN_GAIN).clamp(0.0, 1.0);
                // Where the marks fall. Where the field sits under the
                // coverage the pixel is inked; because the field is streaked
                // along the direction those pixels come in strokes, and
                // because it is noise they fall at random rather than on a
                // printed screen.
                let threshold = 0.5 + hatch[i] * PEN_SPREAD;
                let color = if coverage > threshold { &ink } else { &paper };
                for (c, v) in color.iter().enumerate() {
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: drawing over the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Halftone Pattern, which its two sliders run over.
pub const HALFTONE_SIZE: std::ops::RangeInclusive<u32> = 1..=12;

pub const HALFTONE_CONTRAST: std::ops::RangeInclusive<u32> = 0..=50;

impl HalftonePattern {}

/// The finest cell the Line and Circle screens can be ruled into, in pixels.
///
/// CS6's Size runs down to 1, but a band or a ring one or two pixels across
/// has no room for an edge: it is on or off for the whole cell, so the screen
/// would come back as a plain threshold with no pattern in it. Three is the
/// finest that still rules. The Dot screen needs no floor — its cell is a
/// whole square of the chessboard, and at Size 1 that is one pixel, which is
/// the finest chessboard there is and the pattern CS6 draws there.
pub(crate) const HALFTONE_MIN_CELL: f32 = 3.0;

/// How many steps of Contrast double the steepness of the curve the tone is
/// read through.
///
/// Contrast is not the *edge* of a mark, it is how hard the picture is pushed
/// against the screen. Wound right down the screen only shades the picture and
/// the greys survive; wound up it cuts them away until nothing is left but ink
/// and paper. Five steps to a doubling is what puts CS6's 0–50 over that whole
/// run, and it is the number the picture is most sensitive to. At its default
/// of 5 the curve is exactly twice as steep as the screen, which is the
/// setting at which a mid grey — and only a mid grey — comes back as the full
/// black-and-white chessboard, a lighter tone as a chessboard of white against
/// a light grey, a darker one as black against a dark grey. Steeper than that
/// and whole bands of tone collapse onto the same flat chessboard, which is
/// the posterised sheet CS6 does not draw. It was fitted against CS6's own
/// output on `samples/horse-3.jpg` at Size 1, Contrast 5.
pub(crate) const HALFTONE_CONTRAST_DOUBLING: f32 = 5.0;

/// Filter ▸ Sketch ▸ Halftone Pattern: the picture ruled into a screen of
/// dots, rings or lines, in the two swatches.
///
/// The screen is a **threshold laid over the picture**: every pixel carries a
/// tone the screen asks it to beat — little where a mark falls, much where the
/// paper is meant to show — and how far the picture's own tone clears that
/// threshold is how much ink the pixel takes. Clear it by a long way and the
/// pixel is solid foreground, miss by a long way and it is bare background,
/// and in between it is a blend of the two.
///
/// That in-between is the filter's whole character and the thing it is easy to
/// get wrong. CS6 at a low Contrast does *not* come back as two colours: the
/// screen shades the photograph, the horse keeps its modelling and the sky
/// keeps its greys, and what the pattern adds is a texture over the top.
/// Contrast is what cuts the greys away — by the top of the slider nothing is
/// left but ink and paper, which is the two-tone halftone. A screen that
/// thresholds hard whatever Contrast says throws away nine tenths of the
/// picture and looks nothing like the original.
///
/// The Dot screen is a **chessboard of whole squares, square to the picture**
/// — Photoshop's own transparency grid is the thing it looks like. Nothing
/// grows inside a square: the two squares of the board ask opposite things of
/// the tone, so what changes is their *shade*, from pale squares on white
/// through the black-and-white chessboard a mid grey fuses into, on to black
/// squares with dark grey between them. It is worth being plain about what it
/// is **not**: there is no round dot swelling with the tone, and no screen
/// turned to 45°. That is the printer's halftone, and in CS6 it is a
/// different filter — Pixelate ▸ Color Halftone, in `filters::pixelate`.
///
/// **Pattern Type** is the shape of the screen — the chessboard of squares,
/// concentric rings about the middle of the picture, or parallel horizontal
/// lines. **Size** is how big a square, a ring or a band is, in pixels, so
/// Size 1 gives the finest chessboard there is, of single pixels.
/// **Contrast** is how hard the picture is pushed against the screen, from a
/// shading that leaves the greys to a cut that leaves none.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: the screen is laid by where on the
/// canvas a pixel is, and the work per pixel is one pattern sample.
pub fn halftone_pattern(
    pixmap: &mut Pixmap,
    size: u32,
    contrast: u32,
    pattern: HalftonePattern,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*HALFTONE_SIZE.start(), *HALFTONE_SIZE.end()) as f32;
    let contrast = contrast.clamp(*HALFTONE_CONTRAST.start(), *HALFTONE_CONTRAST.end()) as f32;
    // How steeply the tone is read against the screen. At the bottom of the
    // slider the curve is a straight line — the screen shades the picture and
    // the greys come through — and every few steps doubles it, so the top of
    // the slider is a cut with nothing between ink and paper.
    let gain = (contrast / HALFTONE_CONTRAST_DOUBLING).exp2();
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let cell = match pattern {
        // A square of the chessboard, in pixels, straight off the slider —
        // Size 4 is a four-pixel square, and Size 1 a single pixel.
        HalftonePattern::Dot => size,
        HalftonePattern::Line | HalftonePattern::Circle => size.max(HALFTONE_MIN_CELL),
    };
    // Half a pixel's share of the tone. A fine screen holds so few pixels per
    // cell that a mark takes a pixel whole or not at all, and the tones it can
    // actually strike are a short ladder. Reading the tone against the middle
    // of each rung rather than its bottom puts them where they belong:
    // without it the palest grey there is already inks half the sheet, and
    // with the ends left uncompressed black never closes up. A Line's ladder
    // and a Circle's run across the cell, so they have as many rungs as the
    // cell is pixels wide. The Dot screen's has two rungs whatever the Size,
    // because the board is two squares standing for two tones — so its
    // thresholds come out at a quarter and three quarters, and a square is a
    // square of the chessboard from the moment the tone reaches it.
    let rung = match pattern {
        HalftonePattern::Dot => 0.25,
        HalftonePattern::Line | HalftonePattern::Circle => 0.5 / cell,
    };
    let (mid_x, mid_y) = (w as f32 / 2.0, h as f32 / 2.0);

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

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let tone =
                    (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0;
                let darkness = 1.0 - tone;
                // The tone this pixel has to beat to take ink: little where a
                // mark falls, much where the paper is meant to show. Where
                // the threshold stands is *how much of the cell* takes ink at
                // or before it, so that a tone laid over a whole cell inks
                // that share of it — the squares and the gaps interlock at a
                // mid grey instead of leaving marks on pale ground. The
                // screen is laid by where on the canvas the pixel is, so the
                // pattern is anchored to the picture rather than to the tone.
                let screen = match pattern {
                    HalftonePattern::Dot => {
                        // A chessboard of squares, square to the picture: one
                        // square of the pair asks little of the tone and
                        // takes ink early, its neighbour asks everything and
                        // holds out. So a light grey comes back as pale
                        // squares on white and a mid grey as the
                        // black-and-white chessboard the eye fuses into that
                        // grey. This is the pattern CS6 draws; a screen with
                        // a dot growing in it — the printer's kind — is the
                        // wrong filter, that is Pixelate ▸ Color Halftone.
                        //
                        // A cosine either way rules the board: half a turn to
                        // a square, so the squares are `size` across and the
                        // board comes back round every two of them. A cosine
                        // and not a step, because CS6's squares are **soft**:
                        // seen close up they are rounded, brightest across
                        // the middle and fading into the join, and a board of
                        // flat tiles reads as pixel art instead. What keeps
                        // them squares rather than round dots is `rung` — the
                        // two squares stand for two tones, so a square is a
                        // square of the chessboard as soon as the tone
                        // reaches it, and only the rim is left in between.
                        use std::f32::consts::PI;
                        let wave = (PI * x as f32 / cell).cos() * (PI * y as f32 / cell).cos();
                        0.5 - 0.5 * wave
                    }
                    HalftonePattern::Line => {
                        let fy = ((y as f32 + 0.5) / cell).fract() - 0.5;
                        fy.abs() * 2.0
                    }
                    HalftonePattern::Circle => {
                        let dx = x as f32 - mid_x;
                        let dy = y as f32 - mid_y;
                        let along = ((dx * dx + dy * dy).sqrt() / cell).fract();
                        (along - 0.5).abs() * 2.0
                    }
                };
                // Stood in the middle of the rung it speaks for (see `rung`),
                // so a fine screen's few steps land where they belong.
                let screen = rung + screen.clamp(0.0, 1.0) * (1.0 - 2.0 * rung);
                // How far the picture clears the screen, read through the
                // contrast curve: a grey that only just clears it stays a
                // grey until Contrast is wound up enough to cut it away.
                let coverage = ((darkness - screen) * gain + 0.5).clamp(0.0, 1.0);
                for (c, (dark, pale)) in ink.iter().zip(paper.iter()).enumerate() {
                    px[c] = (pale + (dark - pale) * coverage).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: ruling the picture does not change the layer's
                // shape.
            }
        });
}

/// CS6's ranges for Note Paper, which its three sliders run over.
pub const NOTE_BALANCE: std::ops::RangeInclusive<u32> = 0..=50;

pub const NOTE_GRAININESS: std::ops::RangeInclusive<u32> = 0..=20;

pub const NOTE_RELIEF: std::ops::RangeInclusive<u32> = 0..=25;

/// How far the picture is softened before it is cut into paper and ink, in
/// pixels. Enough that the cut follows the shapes rather than every speck of
/// the photograph, not so much that CS6's ragged mane comes out as blobs.
pub(crate) const NOTE_SMOOTH: f32 = 1.0;

/// How wide the cut between paper and ink is, as a share of the tone range.
/// A hard step would alias; this is about a pixel of anti-aliasing.
pub(crate) const NOTE_CUT_SOFT: f32 = 0.04;

/// How far the ink is taken from the background towards the foreground.
///
/// Note Paper's ink is not ink: it is the holes in the top sheet, and what
/// shows through them is a mid-light grey, not black. Measured off CS6 on
/// `samples/horse-3.jpg` with black and white swatches — the horse comes out
/// at about 172, which is a third of the way to black.
pub(crate) const NOTE_INK: f32 = 0.33;

/// How softly the edge of a hole is rounded before it is lit, in pixels. This
/// is the width of the shadow line along the top of each hole.
pub(crate) const NOTE_WALL: f32 = 0.9;

/// How deep a hole is, in the same units as the grain.
pub(crate) const NOTE_DEPTH: f32 = 1.0;

/// How tall the paper's fibres stand, per step of Graininess.
pub(crate) const NOTE_GRAIN_PER_STEP: f32 = 0.007;

/// How big a fibre is, in pixels. CS6's are a couple of pixels across and a
/// little longer than they are tall, so the noise is blurred and then drawn
/// out along the row.
pub(crate) const NOTE_FIBRE: f32 = 0.8;

/// How steeply the surface is lit, per step of Relief.
pub(crate) const NOTE_RELIEF_PER_STEP: f32 = 0.18;

/// How much a slope darkens whichever way it faces, as a power of its
/// tilt. A sheet of rough paper is greyer than a flat one — every fibre
/// shades its neighbours — and CS6's paper at a heavy grain sits well below
/// white, with only the flat tops of the fibres catching it. Lambert alone
/// lights the half of the fibres facing the lamp brighter than flat, which
/// leaves the paper white with dark specks instead.
pub(crate) const NOTE_OCCLUSION: f32 = 3.0;

/// Where the light comes from: above and a little to the left, low over the
/// sheet. CS6 has no Light control on this filter, and its holes are shadowed
/// along the top edge and, more faintly, the left.
pub(crate) const NOTE_LIGHT: (f32, f32, f32) = (-0.3, -1.0, 1.2);

/// Filter ▸ Sketch ▸ Note Paper: the picture cut out of a sheet of handmade
/// paper, the dark parts showing through as holes onto a sheet beneath.
///
/// The picture is softened a little and **cut in two** at the tone **Image
/// Balance** asks for: whatever is darker than it becomes a hole, the rest
/// stays paper. Low values leave only the deepest shadows as holes, high
/// ones punch out everything but the highlights. The holes are a light grey
/// — a third of the way from the background to the foreground — and the
/// paper is the background itself.
///
/// The whole sheet is then read as a **surface** — the paper standing a step
/// above the holes, with a fibrous grain laid over both whose height is
/// **Graininess** — and lit from above. **Relief** is how steeply that
/// surface stands: at 0 the sheet is flat and the grain is invisible; as it
/// rises the top edge of every hole falls into shadow and the fibres cast
/// their own. A surface that is bumpy all over catches less light on average
/// than a flat one, so a heavy grain greys the paper as it does in CS6.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: a small blur, a threshold, a
/// noise field and one lit normal per pixel — the upload and read-back would
/// cost more than the arithmetic. The grain is laid by where on the canvas a
/// pixel is, so a preview crop cannot be filtered on its own either.
pub fn note_paper(
    pixmap: &mut Pixmap,
    balance: u32,
    graininess: u32,
    relief: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*NOTE_BALANCE.start(), *NOTE_BALANCE.end()) as f32;
    let graininess = graininess.clamp(*NOTE_GRAININESS.start(), *NOTE_GRAININESS.end()) as f32;
    let relief = relief.clamp(*NOTE_RELIEF.start(), *NOTE_RELIEF.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // Paper or hole: the softened brightness against the cut. Image Balance
    // runs the cut from black at 0 to white at 50.
    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    blur_field(&mut tone, w, h, NOTE_SMOOTH);
    let cut = balance / *NOTE_BALANCE.end() as f32;
    let paper: Vec<f32> = tone
        .par_iter()
        .map(|&t| ((t - cut) / NOTE_CUT_SOFT + 0.5).clamp(0.0, 1.0))
        .collect();

    // The surface: the paper a step above the holes, its edges rounded, and
    // the fibres over the top of both.
    let mut surface: Vec<f32> = paper.iter().map(|&p| p * NOTE_DEPTH).collect();
    blur_field(&mut surface, w, h, NOTE_WALL);
    if graininess > 0.0 {
        let mut fibre: Vec<f32> = (0..w * h)
            .into_par_iter()
            .map(|i| crate::photorust::artistic::noise((i % w) as i32, (i / w) as i32 + 7919) - 0.5)
            .collect();
        blur_field(&mut fibre, w, h, NOTE_FIBRE);
        // Drawn out along the row, so a fibre is longer than it is tall.
        let mut drawn: Vec<f32> = (0..w * h)
            .into_par_iter()
            .map(|i| {
                let x = i % w;
                let left = if x > 0 { fibre[i - 1] } else { fibre[i] };
                let right = if x + 1 < w { fibre[i + 1] } else { fibre[i] };
                (left + fibre[i] + right) / 3.0
            })
            .collect();
        // Blurring white noise flattens it; `unit_spread` puts it back to a
        // known height so Graininess means the same thing at every blur.
        unit_spread(&mut drawn);
        let height = graininess * NOTE_GRAIN_PER_STEP;
        surface
            .par_iter_mut()
            .zip(drawn.par_iter())
            .for_each(|(s, f)| *s += f * height);
    }

    let steep = relief * NOTE_RELIEF_PER_STEP;
    let (lx, ly, lz) = NOTE_LIGHT;
    let length = (lx * lx + ly * ly + lz * lz).sqrt();
    let (lx, ly, lz) = (lx / length, ly / length, lz / length);
    let hole = [
        background.r as f32 + (foreground.r as f32 - background.r as f32) * NOTE_INK,
        background.g as f32 + (foreground.g as f32 - background.g as f32) * NOTE_INK,
        background.b as f32 + (foreground.b as f32 - background.b as f32) * NOTE_INK,
    ];
    let sheet = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let (surface, paper) = (&surface, &paper);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            let up = y.saturating_sub(1);
            let down = (y + 1).min(h - 1);
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let left = x.saturating_sub(1);
                let right = (x + 1).min(w - 1);
                let gx = (surface[y * w + right] - surface[y * w + left]) * 0.5;
                let gy = (surface[down * w + x] - surface[up * w + x]) * 0.5;
                // Lambert against the surface's normal, over what a flat
                // sheet would catch, so flat paper is exactly its colour.
                let (nx, ny, nz) = (-gx * steep, -gy * steep, 1.0);
                let tilt = (nx * nx + ny * ny + nz * nz).sqrt();
                let lit = ((nx * lx + ny * ly + nz * lz) / tilt / lz).max(0.0)
                    / tilt.powf(NOTE_OCCLUSION);
                let p = paper[i];
                for c in 0..3 {
                    let base = hole[c] + (sheet[c] - hole[c]) * p;
                    // Past flat, the light has nowhere darker to come from:
                    // a highlight lifts towards white rather than scaling a
                    // colour that is already the paper.
                    let v = if lit <= 1.0 {
                        base * lit
                    } else {
                        base + (255.0 - base) * (lit - 1.0)
                    };
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: cutting the sheet does not change the layer's
                // shape.
            }
        });
}

/// CS6's ranges for Photocopy, which its two sliders run over.
pub const COPY_DETAIL: std::ops::RangeInclusive<u32> = 1..=24;

pub const COPY_DARKNESS: std::ops::RangeInclusive<u32> = 1..=50;

/// How wide the neighbourhood a pixel is compared with is, in pixels of
/// blur: a floor, and how much each step of Detail adds.
///
/// This is the whole of Detail, and it is not what the name suggests. A pixel
/// takes toner when it is darker than the picture *round it*, so a narrow
/// neighbourhood only catches the dark side of every edge — CS6 at Detail 4
/// draws the horse as an outline with its body left white — and a wide one
/// catches whole masses that are darker than their surroundings, which is why
/// at Detail 23 the body comes back solid black.
pub(crate) const COPY_REACH: f32 = 0.6;

pub(crate) const COPY_REACH_PER_STEP: f32 = 1.2;

/// How much toner a level of difference lays, per step of Darkness.
///
/// At the bottom of the slider the difference is shaded in, so the lines are
/// soft and grey and fade into the paper; by the top it is cut, and the copy
/// is toner or paper with nothing between.
pub(crate) const COPY_GAIN_PER_STEP: f32 = 1.1;

/// How much darker than the neighbourhood a pixel has to be before it takes
/// any toner, in levels. A copier does not pick up the faint texture of a
/// flat grey; without this every speck of JPEG noise in a sky comes back as
/// a grey smudge.
pub(crate) const COPY_FLOOR: f32 = 1.5;

/// Filter ▸ Sketch ▸ Photocopy: the picture as a cheap photocopier sees it,
/// in the two swatches.
///
/// A copier does not reproduce tone, it reproduces *change*: a flat area,
/// dark or light, comes back as bare paper, and toner sticks where the
/// picture is darker than what is round it. So each pixel is compared with a
/// blurred copy of the picture, and how far it falls below that is how much
/// toner it takes. **Detail** is how wide that neighbourhood is — narrow
/// draws outlines along the dark side of every edge, wide fills in whole
/// masses that are darker than their surroundings. **Darkness** is how hard
/// the difference is driven, from soft grey shading to a hard cut into
/// toner and paper.
///
/// Toner is the foreground, paper the background. Alpha is left alone.
///
/// No GPU path. The blur is the only real work, and at the top of Darkness a
/// difference of five levels is already full toner, so it has to stay in
/// floating point — the GPU blur works in eight bits, and its rounding would
/// come back as contour lines across every smooth gradient.
pub fn photocopy(
    pixmap: &mut Pixmap,
    detail: u32,
    darkness: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let detail = detail.clamp(*COPY_DETAIL.start(), *COPY_DETAIL.end()) as f32;
    let darkness = darkness.clamp(*COPY_DARKNESS.start(), *COPY_DARKNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    let mut around = tone.clone();
    blur_field(&mut around, w, h, COPY_REACH + detail * COPY_REACH_PER_STEP);

    let gain = darkness * COPY_GAIN_PER_STEP / 255.0;
    let toner = [
        foreground.r as f32,
        foreground.g as f32,
        foreground.b as f32,
    ];
    let paper = [
        background.r as f32,
        background.g as f32,
        background.b as f32,
    ];
    let (tone, around) = (&tone, &around);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                // Only what is darker than its surroundings takes toner; a
                // pixel lighter than them is paper, however dark it is.
                let below = (around[i] - tone[i] - COPY_FLOOR).max(0.0);
                let ink = (below * gain).min(1.0);
                for c in 0..3 {
                    px[c] = (paper[c] + (toner[c] - paper[c]) * ink)
                        .round()
                        .clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: copying the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Plaster, which its two sliders run over.
pub const PLASTER_BALANCE: std::ops::RangeInclusive<u32> = 0..=50;

pub const PLASTER_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// How far the picture is softened before it is poured, in pixels: a floor,
/// and how much each step of Smoothness adds. This rounds the outline off —
/// but only the outline's *noise*: CS6 at Smoothness 12 still keeps the
/// mane's spikes and the spray's droplets, and melts only the pixel-level
/// raggedness in between. What makes the top of the slider read as liquid is
/// the wider rim, not a blurrier shape.
pub(crate) const PLASTER_SMOOTH: f32 = 0.6;

pub(crate) const PLASTER_SMOOTH_PER_STEP: f32 = 0.2;

/// How the pools are cut out of the plaster.
///
/// The threshold is taken hard, as a mask of whole pixels, and the mask is
/// then softened by [`PLASTER_EDGE`] and cut again at its middle over
/// [`PLASTER_EDGE_AA`]. Cutting the tone directly leaves the photograph's
/// pixel staircase along every edge; cutting a softened mask puts the edge
/// between pixels, where the shape really runs, and anti-aliases it. The
/// softening is kept small so that the specks CS6 keeps — the white dots on
/// the horse, the spray's droplets — survive it.
pub(crate) const PLASTER_EDGE: f32 = 1.0;

pub(crate) const PLASTER_EDGE_AA: f32 = 0.35;

/// How wide the bevel on every edge is, in pixels of blur, and how much each
/// step of Smoothness widens it — hardly at all: CS6's bevel is about as
/// wide at Smoothness 12 as at 2, and a bevel that grows with the slider
/// turns the top of it into broad, blown-out bands.
///
/// This is the filter's look. The pools stand proud of the plaster, and the
/// mask is blurred into a height field whose shoulders run out past the edge
/// on both sides: eight to twelve pixels of rounded bevel at the bottom of the
/// slider and not much more at the top. Lit, those shoulders are what
/// make CS6's output look poured and three-dimensional rather than outlined.
pub(crate) const PLASTER_BEVEL: f32 = 4.0;

pub(crate) const PLASTER_BEVEL_PER_STEP: f32 = 0.08;

/// How steeply the bevel stands. The blurred mask's slope is only about a
/// tenth of a unit per pixel; this turns it into shoulders tilted most of
/// the way to upright, which is what lets them catch the light hard.
pub(crate) const PLASTER_STEEP: f32 = 14.0;

/// How high the lamp stands over the sheet, as the upward part of the vector
/// towards it — CS6's Light sets only its compass direction.
pub(crate) const PLASTER_ELEVATION: f32 = 0.7;

/// How glossy the plaster is. The highlight is a Blinn–Phong lobe: `SHINE`
/// is its exponent — how tight it is — and `GLOSS` how far it takes a
/// pixel towards the background colour. A wet, glossy bead is what CS6
/// draws; a matte shoulder with diffuse light alone looks like Bas Relief.
pub(crate) const PLASTER_SHINE: f32 = 8.0;

pub(crate) const PLASTER_GLOSS: f32 = 1.4;

/// How hard a shoulder is lifted towards the background colour: `LIFT` by
/// how much more light it catches than flat ground, and `SHEEN` by how
/// steep it is whichever way it faces.
///
/// CS6's lit bevels are broad bands of near-white, six to eight pixels
/// across, not the thin line a specular lobe draws on its own; and they run
/// on round the sides of a shape as well as along the edge squarely facing
/// the lamp — with Light at Top, the horse's legs are bright down their
/// left sides too. The sheen is what carries them round; the shade on the
/// side away from the lamp is what still pulls that side down.
pub(crate) const PLASTER_LIFT: f32 = 2.5;

pub(crate) const PLASTER_SHEEN: f32 = 0.6;

/// How much of the gloss shows on the pools themselves. They are the
/// foreground colour and mostly stay it; CS6 lets a little of the highlight
/// across onto the lip of a pool, and all of it onto a small one.
pub(crate) const PLASTER_POOL_GLOSS: f32 = 0.35;

/// How deep the shade on a shoulder facing away from the lamp goes, towards
/// the foreground colour. Diffuse light alone would take the far side to
/// black; CS6's shade is a darker band on the plaster, not a hole in it.
pub(crate) const PLASTER_SHADE: f32 = 0.6;

/// The thin grey line inside every pool, a few pixels in from its edge: where
/// on the bevel's coordinate it falls (0 deep in a pool, ½ on the edge), how
/// wide it is, and how far it is taken towards the plaster's colour.
pub(crate) const PLASTER_ECHO: (f32, f32) = (0.25, 0.06);

pub(crate) const PLASTER_ECHO_STRENGTH: f32 = 0.3;

/// Filter ▸ Sketch ▸ Plaster: the picture poured in plaster and lit from one
/// side, in the two swatches.
///
/// Whatever is darker than the tone **Image Balance** asks for becomes a pool
/// of the foreground colour, standing proud of the rest, which is the
/// plaster. The mask of the pools is blurred into a height field, so every
/// edge is a wide, rounded **bevel** — see [`PLASTER_BEVEL`] — and the whole
/// surface is lit from **Light** with a glossy highlight: the shoulders that
/// face the lamp catch a bright wet sheen, the ones that face away fall into
/// shade, and together they make the shapes look poured and three-
/// dimensional. **Smoothness** rounds off the outline's pixel noise.
///
/// The plaster is not left flat. CS6 shades it as a single **ramp** across
/// the whole picture, from the background colour at the edge nearest the
/// light to the foreground at the far edge. Measured off CS6 with Light Top,
/// it runs from 253 at the top of the frame to 9 at the bottom, linear in
/// between.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: three blurs and a few taps per
/// pixel. The ramp is laid across the whole frame, so a preview crop cannot
/// be filtered on its own.
pub fn plaster(
    pixmap: &mut Pixmap,
    balance: u32,
    smoothness: u32,
    light: Light,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let balance = balance.clamp(*PLASTER_BALANCE.start(), *PLASTER_BALANCE.end()) as f32;
    let smoothness =
        smoothness.clamp(*PLASTER_SMOOTHNESS.start(), *PLASTER_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The plaster: the softened brightness against the cut, 1 where it is
    // lighter. Image Balance runs the cut from black to white.
    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    blur_field(
        &mut tone,
        w,
        h,
        PLASTER_SMOOTH + smoothness * PLASTER_SMOOTH_PER_STEP,
    );
    let cut = balance / *PLASTER_BALANCE.end() as f32;
    let plaster: Vec<f32> = tone
        .par_iter()
        .map(|&t| if t >= cut { 1.0 } else { 0.0 })
        .collect();

    // The edge, anti-aliased: the mask softened a little and cut at its
    // middle, so the outline runs between pixels rather than along them.
    let mut edge = plaster.clone();
    blur_field(&mut edge, w, h, PLASTER_EDGE);
    // The bevel's coordinate: 1 in open plaster, 0 deep in a pool, a half on
    // the edge, running smoothly across the bevel's whole width.
    let mut bevel = plaster;
    blur_field(
        &mut bevel,
        w,
        h,
        PLASTER_BEVEL + smoothness * PLASTER_BEVEL_PER_STEP,
    );
    // The height. The pools stand proud and their tops are flat right out to
    // the edge: the whole slope lies on the plaster side of it. That is
    // where CS6 draws the relief — a bright band on the plaster along the
    // edges facing the lamp, a shaded one along the edges facing away — and
    // it leaves the pools a flat, clean black. A slope straddling the edge
    // puts half the highlight on the pools and turns them into glossy blobs.
    // Eased at the top, so the edge is a rounded lip rather than a crease.
    let height: Vec<f32> = bevel
        .par_iter()
        .map(|&g| {
            let t = (2.0 * (1.0 - g)).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        })
        .collect();

    // The ramp: how far along the line away from the light a pixel is, from
    // 0 at the corner nearest the lamp to 1 at the one furthest from it.
    let (lx, ly) = light.towards();
    let (ax, ay) = (-lx, -ly);
    let corners = [
        0.0,
        ax * (w - 1) as f32,
        ay * (h - 1) as f32,
        ax * (w - 1) as f32 + ay * (h - 1) as f32,
    ];
    let near = corners.iter().copied().fold(f32::INFINITY, f32::min);
    let far = corners.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (far - near).max(1.0);

    // The lamp, and the half-way vector between it and a viewer straight
    // above, for the highlight. A flat surface catches some of the lobe too;
    // that much is taken off so flat plaster is exactly the ramp.
    let lamp = {
        let n = (lx * lx + ly * ly + PLASTER_ELEVATION * PLASTER_ELEVATION).sqrt();
        (lx / n, ly / n, PLASTER_ELEVATION / n)
    };
    let half = {
        let (hx, hy, hz) = (lamp.0, lamp.1, lamp.2 + 1.0);
        let n = (hx * hx + hy * hy + hz * hz).sqrt();
        (hx / n, hy / n, hz / n)
    };
    let flat_sheen = half.2.powf(PLASTER_SHINE);

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
    let (bevel, edge, height) = (&bevel, &edge, &height);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            let up = y.saturating_sub(1);
            let down = (y + 1).min(h - 1);
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let g = bevel[i];
                let e = edge[i];
                let p = ((0.5 - e) / PLASTER_EDGE_AA + 0.5).clamp(0.0, 1.0);
                // The surface's normal, leaning down the slope.
                let gx = (height[y * w + (x + 1).min(w - 1)] - height[y * w + x.saturating_sub(1)])
                    * 0.5;
                let gy = (height[down * w + x] - height[up * w + x]) * 0.5;
                let (nx, ny, nz) = (-gx * PLASTER_STEEP, -gy * PLASTER_STEEP, 1.0);
                let n = (nx * nx + ny * ny + nz * nz).sqrt();
                let (nx, ny, nz) = (nx / n, ny / n, nz / n);
                // Diffuse against what flat ground catches: 1 on the flat,
                // above it on a shoulder facing the lamp, below it on one
                // facing away.
                let diffuse = (nx * lamp.0 + ny * lamp.1 + nz * lamp.2).max(0.0) / lamp.2;
                let shade = ((1.0 - diffuse).max(0.0) * PLASTER_SHADE).min(1.0);
                let lift =
                    ((diffuse - 1.0).max(0.0) * PLASTER_LIFT + (1.0 - nz) * PLASTER_SHEEN).min(1.0);
                let sheen = (nx * half.0 + ny * half.1 + nz * half.2)
                    .max(0.0)
                    .powf(PLASTER_SHINE);
                let gloss = ((sheen - flat_sheen).max(0.0) / (1.0 - flat_sheen) * PLASTER_GLOSS)
                    .min(1.0)
                    * (1.0 - p * (1.0 - PLASTER_POOL_GLOSS));
                // The echo, inside the pool only, tailing off to nothing in
                // its middle so a pool far from any edge stays flat.
                let echo = (-((g - PLASTER_ECHO.0) / PLASTER_ECHO.1).powi(2)).exp()
                    * PLASTER_ECHO_STRENGTH
                    * (g * 4.0).min(1.0)
                    * p;

                let along = ((x as f32 * ax + y as f32 * ay) - near) / span;
                for c in 0..3 {
                    let ramp = paper[c] + (ink[c] - paper[c]) * along;
                    let mut v = ramp + (ink[c] - ramp) * p;
                    v += (ramp - v) * echo;
                    v += (ink[c] - v) * shade;
                    v += (paper[c] - v) * lift;
                    v += (paper[c] - v) * gloss;
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: pouring the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Reticulation, which its three sliders run over.
pub const RETIC_DENSITY: std::ops::RangeInclusive<u32> = 0..=50;

pub const RETIC_FOREGROUND: std::ops::RangeInclusive<u32> = 0..=50;

pub const RETIC_BACKGROUND: std::ops::RangeInclusive<u32> = 0..=50;

/// How big a clump of grain is, in pixels of blur, and how much finer each
/// step of Density makes it. CS6's clumps are worms two or three pixels
/// across, packed into a labyrinth: dark worms on white in the highlights,
/// grey worms parted by thin black gaps in the shadows.
///
/// The grain is white noise band-passed — blurred by this, less the same
/// noise blurred by twice this — which is what packs it into worms of one
/// size rather than soft blobs of every size. Plain blurred noise reads as a
/// grey fog next to CS6.
pub(crate) const RETIC_CLUMP: f32 = 1.15;

pub(crate) const RETIC_CLUMP_PER_STEP: f32 = 0.006;

/// How far the grain pushes the tone, as a share of the tone range: a floor,
/// and how much each step of Density adds.
pub(crate) const RETIC_GRAIN: f32 = 0.167;

pub(crate) const RETIC_GRAIN_PER_STEP: f32 = 0.0032;

/// How far Density lifts the darkest tones before the grain is added, per
/// step. Without it a shadow sits below the curve's black point, half of
/// the grain is clipped away, and the horse sets solid black; CS6's shadows
/// keep a lighter web through them that thickens with Density.
pub(crate) const RETIC_LIFT_PER_STEP: f32 = 0.004;

/// The tone curve the grained picture is read through. Foreground Level
/// moves its black point up and bends the midtones down — at 50 the sea sets
/// dark while the sky barely moves — and Background Level brings its white
/// point down, washing the highlights out.
pub(crate) const RETIC_BLACK: f32 = 0.051;

pub(crate) const RETIC_BLACK_PER_STEP: f32 = 0.0086;

pub(crate) const RETIC_BEND: f32 = 0.619;

pub(crate) const RETIC_BEND_PER_STEP: f32 = 0.0542;

pub(crate) const RETIC_WHITE: f32 = 1.127;

pub(crate) const RETIC_WHITE_PER_STEP: f32 = 0.0085;

/// How far towards each swatch the result reaches. CS6's grain never quite
/// sets solid: the darkest clump keeps a trace of the background colour and
/// the palest a trace of the foreground.
pub(crate) const RETIC_DEEPEST: f32 = 0.068;

pub(crate) const RETIC_PALEST: f32 = 0.938;
