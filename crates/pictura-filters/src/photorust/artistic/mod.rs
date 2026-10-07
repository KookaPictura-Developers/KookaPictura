//! Filter ▸ Artistic.
//!
//! CS6 keeps this whole family in the Filter Gallery rather than in the Filter
//! menu. The Gallery is not built (docs/ROADMAP.md), so the filters live under
//! a Filter ▸ Artistic submenu instead, which is where the Gallery's own
//! category list would have put them. Colored Pencil, Cutout, Dry Brush, Film
//! Grain, Fresco, Neon Glow, Paint Daubs, Palette Knife, Plastic Wrap, Poster
//! Edges, Rough Pastels, Smudge Stick, Sponge, Underpainting and Watercolor —
//! the whole of CS6's Artistic group — are built.

use crate::photorust::pixmap::{Pixmap, Rgba8};
pub(crate) use crate::BrushType as DaubBrush;
use rayon::prelude::*;

mod daubs_to_smudge;
mod knife_glow_grain;
mod sponge_to_watercolor;
pub(crate) use daubs_to_smudge::*;
pub(crate) use knife_glow_grain::*;
pub(crate) use sponge_to_watercolor::*;

/// CS6's ranges for Colored Pencil, which its three sliders run over.
pub const PENCIL_WIDTH: std::ops::RangeInclusive<u32> = 1..=24;

pub const PENCIL_PRESSURE: std::ops::RangeInclusive<u32> = 0..=15;

pub const PENCIL_PAPER: std::ops::RangeInclusive<u32> = 0..=50;

/// CS6's ranges for Cutout, which its three sliders run over.
pub const CUTOUT_LEVELS: std::ops::RangeInclusive<u32> = 2..=8;

pub const CUTOUT_SIMPLICITY: std::ops::RangeInclusive<u32> = 0..=10;

pub const CUTOUT_FIDELITY: std::ops::RangeInclusive<u32> = 1..=3;

/// CS6's ranges for Dry Brush, which its three sliders run over.
pub const BRUSH_SIZE: std::ops::RangeInclusive<u32> = 0..=10;

pub const BRUSH_DETAIL: std::ops::RangeInclusive<u32> = 0..=10;

pub const BRUSH_TEXTURE: std::ops::RangeInclusive<u32> = 1..=3;

/// CS6's ranges for Palette Knife, which its three sliders run over.
pub const KNIFE_SIZE: std::ops::RangeInclusive<u32> = 1..=50;

pub const KNIFE_DETAIL: std::ops::RangeInclusive<u32> = 1..=3;

pub const KNIFE_SOFTNESS: std::ops::RangeInclusive<u32> = 0..=10;

/// CS6's ranges for Plastic Wrap, which its three sliders run over.
pub const WRAP_HIGHLIGHT: std::ops::RangeInclusive<u32> = 0..=20;

pub const WRAP_DETAIL: std::ops::RangeInclusive<u32> = 1..=15;

pub const WRAP_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// CS6's ranges for Poster Edges, which its three sliders run over.
pub const POSTER_THICKNESS: std::ops::RangeInclusive<u32> = 0..=10;

pub const POSTER_INTENSITY: std::ops::RangeInclusive<u32> = 0..=10;

pub const POSTER_LEVELS: std::ops::RangeInclusive<u32> = 0..=6;

/// CS6's ranges for Rough Pastels' two stroke sliders. Its texture controls
/// are [`super::texture`]'s.
pub const PASTEL_LENGTH: std::ops::RangeInclusive<u32> = 0..=40;

pub const PASTEL_DETAIL: std::ops::RangeInclusive<u32> = 1..=20;

/// CS6's ranges for Smudge Stick, which its three sliders run over.
pub const SMUDGE_LENGTH: std::ops::RangeInclusive<u32> = 0..=10;

pub const SMUDGE_HIGHLIGHT: std::ops::RangeInclusive<u32> = 0..=20;

pub const SMUDGE_INTENSITY: std::ops::RangeInclusive<u32> = 0..=10;

/// CS6's ranges for Sponge, which its three sliders run over.
pub const SPONGE_SIZE: std::ops::RangeInclusive<u32> = 0..=10;

pub const SPONGE_DEFINITION: std::ops::RangeInclusive<u32> = 0..=25;

pub const SPONGE_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// CS6's ranges for Underpainting's two sliders. Its texture controls are
/// [`super::texture`]'s.
pub const UNDERPAINT_SIZE: std::ops::RangeInclusive<u32> = 0..=40;

pub const UNDERPAINT_COVERAGE: std::ops::RangeInclusive<u32> = 0..=40;

/// CS6's ranges for Watercolor, which its three sliders run over.
pub const WATER_DETAIL: std::ops::RangeInclusive<u32> = 1..=14;

pub const WATER_SHADOW: std::ops::RangeInclusive<u32> = 0..=10;

pub const WATER_TEXTURE: std::ops::RangeInclusive<u32> = 1..=3;

/// CS6's ranges for Paint Daubs. The third control is a list of brushes rather
/// than a slider.
pub const DAUB_SIZE: std::ops::RangeInclusive<u32> = 1..=50;

pub const DAUB_SHARPNESS: std::ops::RangeInclusive<u32> = 0..=40;

impl DaubBrush {}

/// CS6's ranges for Neon Glow. The third control is a colour swatch rather
/// than a slider, and the two the picture is rendered between are the
/// document's own, so neither has a range here.
pub const NEON_SIZE: std::ops::RangeInclusive<i32> = -24..=24;

pub const NEON_BRIGHTNESS: std::ops::RangeInclusive<u32> = 0..=50;

/// CS6's ranges for Film Grain, which its three sliders run over.
pub const FILM_GRAIN: std::ops::RangeInclusive<u32> = 0..=20;

pub const FILM_HIGHLIGHT: std::ops::RangeInclusive<u32> = 0..=20;

pub const FILM_INTENSITY: std::ops::RangeInclusive<u32> = 0..=10;

/// The Stroke Pressure at which the pencil lays colour at life size — CS6's
/// default, and the figure everything else is measured against.
pub(crate) const NEUTRAL_PRESSURE: f32 = 8.0;

/// The scale, in pixels, below which the picture counts as *detail* — the
/// veins, the stipple, the boundaries. Measured as what a blur of this radius
/// takes away. Fixed rather than scaled by the pencil: it describes the
/// picture, not the hand.
pub(crate) const DETAIL_SCALE: f32 = 2.0;

/// How far, in pencil widths, the hand looks around itself to find detail to
/// thicken a line with.
pub(crate) const SPREAD: f32 = 1.3;

/// The most, in pixels, that the two reaches below are allowed to grow to.
///
/// Both scale with the pencil, and both would otherwise run away at the top of
/// CS6's range: a 24-wide pencil would look a whole 48 pixels around itself
/// and draw 60 beyond what it found, which stops being "the hand thickens the
/// edge" and becomes a stain spreading out of it.
pub(crate) const MAX_LOOK: f32 = 8.0;

pub(crate) const MAX_FILL: f32 = 8.0;

/// How far, in pencil widths, the drawn edge is then spread. Detail sits in a
/// one-pixel seam along a boundary, and a hand draws a line there rather than
/// the seam itself; this is what makes it a stroke a person would make.
pub(crate) const FILL: f32 = 2.5;

/// The hatch's spacing, in pencil widths. Strokes a single pixel apart read as
/// noise rather than as strokes, so the hand's marks are a couple of widths
/// apart however fine the pencil is.
pub(crate) const HATCH_SPACING: f32 = 1.3;

/// The local detail, in levels, below which it adds nothing to the stroke and
/// above which it adds all of it. Between the two it fades in.
///
/// An out-of-focus background carries a level or so of detail — its own grain,
/// and nothing else, because a blur is by definition what has no fine detail
/// left in it — and so falls under the floor and takes only the flat base
/// stroke. A petal with veins in it carries several, and is drawn in full.
pub(crate) const NOTHING_TO_DRAW: f32 = 1.5;

pub(crate) const DRAWN_IN_FULL: f32 = 4.0;

/// How far the picture is smoothed before the pencil lays it, in pixels. A
/// pencil puts down washes: enough to lose the stipple, not enough to lose
/// the shapes.
pub(crate) const WASH: f32 = 0.8;

/// How much of a full stroke is laid where the picture has nothing in
/// particular to say — a flat wash of colour, or a patch the same level as the
/// paper.
///
/// CS6's Colored Pencil draws the whole picture, not its edges: "the solid
/// colour background shows through the diagonal strokes". A flat pink petal is
/// therefore covered in pink strokes with paper between them, while a flat grey
/// field — which is what the backing colour usually is — disappears into the
/// paper because there is no contrast left to see. This is the number that
/// makes a large flat shape draw at all; without it the hand only ever traces
/// edges and large shapes come out hollow. Detail then adds the rest on top.
///
/// It is small, and deliberately: it is the floor under a part of the picture
/// with *nothing in it*, and in CS6 such a part comes back as bare paper — an
/// out-of-focus background goes flat grey rather than keeping a ghost of its
/// own colour. A base large enough to be seen greys the whole page towards the
/// photograph instead of leaving the drawn shapes to carry it.
pub(crate) const STROKE_BASE: f32 = 0.12;

/// How much of the laid colour the hatch can take back where a stroke thins
/// out, at neutral pressure.
///
/// This is the single most important number for the filter's look, and it cuts
/// both ways. A pencil leaves paper showing between its strokes, so it cannot
/// be nothing; but the hatch is narrow — [`STROKE_SHARPNESS`] keeps the strokes
/// to about a quarter of the page — so whatever this takes back, it takes back
/// from three quarters of every drawn shape. Set near 1 it does not read as
/// paper between strokes at all: a fully drawn petal keeps barely a third of
/// its own pink and the picture comes out as a washed photograph, which is the
/// opposite of the mistake it looks like it is guarding against. CS6 keeps the
/// colour and lets the paper streak across it, so this is a third rather than
/// nearly all. Pressing harder closes the gaps the rest of the way — see where
/// it is used.
pub(crate) const HATCH_DEPTH: f32 = 0.35;

/// How narrow a stroke is across its own width. A ridge shaped by an exponent
/// of one is a triangle that is half on and half off; raising it thins the
/// stroke so the paper between strokes stays the larger part of the page,
/// which is what makes the hatch read as separate pencil lines rather than as
/// a solid fill. This is the second of the two numbers that decide the look.
pub(crate) const STROKE_SHARPNESS: f32 = 2.5;

/// How dark the hatch itself lies on otherwise bare paper, in levels. This is
/// what leaves the faint diagonal strokes across an empty corner of the page,
/// where there was nothing to draw but the hand went over it anyway.
pub(crate) const HATCH_INK: f32 = 12.0;

/// How much darker the pencil goes along a boundary, where the hand presses
/// hardest. This is what outlines a shape.
pub(crate) const EDGE_DARKENING: f32 = 0.9;

/// How thick that outline is, in pencil widths — a wider pencil draws a
/// heavier line, which is the most visible thing Pencil Width does.
pub(crate) const OUTLINE: f32 = 0.5;

/// Filter ▸ Artistic ▸ Colored Pencil: the picture redrawn in pencil on paper.
///
/// The sheet is laid first, a flat grey at Paper Brightness, and the picture is
/// then drawn on it in coloured pencil. Every pixel gets a stroke — CS6 draws
/// the whole picture and lets the paper show through the gaps — and what the
/// picture's own content decides is how *much* stroke, in two steps:
///
/// * **How much fine detail sits here** — what a blur of [`DETAIL_SCALE`] takes
///   away. Not the plain gradient: a soft, out-of-focus background has a
///   perfectly good gradient running across it and no detail in it, while a
///   petal full of veins has plenty. Detail is what earns the darker,
///   cross-hatched strokes along a boundary, so it is the picture's edges that
///   come through most strongly — CS6's "important edges are retained and given
///   a rough crosshatch appearance".
/// * **The answer is averaged over the pencil's reach**, not taken pixel by
///   pixel. An average, not a maximum: one noisy pixel in an empty sky must
///   not earn a stroke for everything around it, and a petal full of veins
///   must earn one for the whole petal rather than for the veins alone. That
///   is what thickens an edge into a drawn line instead of a one-pixel seam.
///
/// What is then laid down is the picture's own colour, lightly washed, crossed
/// by the hatch and darkened along the boundaries where the hand presses
/// hardest. Pencil Width sets the hatch's spacing, how far the hand looks, and
/// how heavy the outlines are; Stroke Pressure is the gain on all of it, so at
/// 0 the page stays blank; Paper Brightness is the sheet.
///
/// The hatch is seeded from each pixel's coordinates rather than from a RNG,
/// so a preview, the commit behind it and an undo/redo replay all draw the
/// same strokes.
///
/// No GPU path. It is per-pixel and would fit the shader shape, but like the
/// rest of the filter stack it would upload its input and read the result
/// straight back, which the measurements in docs/gpu-migration.md say rarely
/// pays for the trip. The blurs inside it do go through the backend.
/// Kooka: the sheet is the document background colour at Paper Brightness,
/// per CS6's "the solid background color shows through the smoother areas";
/// a white background is photorust's own grey sheet.
pub fn colored_pencil(
    pixmap: &mut Pixmap,
    width: u32,
    pressure: u32,
    paper_brightness: u32,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let width_setting = width.clamp(*PENCIL_WIDTH.start(), *PENCIL_WIDTH.end());
    let pressure = pressure.clamp(*PENCIL_PRESSURE.start(), *PENCIL_PRESSURE.end());
    let paper_brightness = paper_brightness.clamp(*PENCIL_PAPER.start(), *PENCIL_PAPER.end());

    // CS6's slider runs 0..50 over the whole range from black paper to white.
    let sheet = paper_brightness as f32 / *PENCIL_PAPER.end() as f32;
    let paper = [background.r, background.g, background.b].map(|c| c as f32 * sheet);
    let gain = pressure as f32 / NEUTRAL_PRESSURE;
    // What the hand does scales with the pressure, so at 0 the page stays
    // blank however much there was to draw. Past life size it is the gaps that
    // close up rather than the strokes that grow: a stroke cannot carry more
    // than its colour, and what "pressing harder" looks like from there is the
    // paper between the strokes disappearing. Never quite all of it: at the top
    // of the range the gaps are narrow, but a drawing with no paper left in it
    // is a photograph, so the slope here is gentle enough that 15 still shows
    // its strokes.
    let press = gain.min(1.0);
    let gap = if gain > 0.0 {
        (HATCH_DEPTH + (1.0 - gain) * 0.2).clamp(0.0, 0.97)
    } else {
        0.97
    };

    // How much fine detail sits at each pixel...
    let detail = detail_map(pixmap);
    // ...and how much of it there is about, which is what thickens an edge
    // into a drawn line rather than a one-pixel seam...
    let mut drawn_map = detail.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut drawn_map,
        (width_setting as f32 * SPREAD).min(MAX_LOOK),
    );
    // ...hardened into how much extra stroke the detail earns, and then spread
    // by a pencil width so a wide hand draws a wide line.
    drawn_map
        .as_bytes_mut()
        .par_chunks_exact_mut(4)
        .for_each(|px| {
            let drawn = fade(px[0] as f32 * gain, NOTHING_TO_DRAW, DRAWN_IN_FULL);
            px[0..3].copy_from_slice(&[(drawn * 255.0) as u8; 3]);
        });
    let mut drawn_map = crate::photorust::convolve::dilate(
        &drawn_map,
        (((width_setting as f32 * FILL).min(MAX_FILL)).round() as i32).max(1),
    );
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut drawn_map,
        (width_setting as f32 * 0.3).min(MAX_FILL / 2.0),
    );
    // The same detail thickened to the pencil's own width, which is what lays
    // the outline along a boundary.
    let outline = crate::photorust::convolve::dilate(
        &detail,
        ((width_setting as f32 * OUTLINE).round() as i32).max(1),
    );
    // The colour the pencil lays: washed, so a petal is a wash of pink with
    // the hatch over it rather than a photograph of a petal.
    let mut wash = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut wash, WASH);
    let (drawn_map, outline, wash) = (&drawn_map, &outline, &wash);

    let w = pixmap.width() as i32;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                // How much extra the detail here earns over the flat base.
                let detail_drawn = drawn_map.get(x, y).r as f32 / 255.0;
                let stroke = STROKE_BASE + (1.0 - STROKE_BASE) * detail_drawn;

                let hatch = hatch(x, y, width_setting);
                // The hatch thins the stroke where it thins, and leaves a
                // faint stroke of its own on bare paper. A hard hand fills
                // the gaps between its strokes and a light one leaves the
                // paper showing between them, which is most of what Stroke
                // Pressure looks like.
                let coverage = press * stroke * (1.0 - gap * (1.0 - hatch));
                // The stroke's own faint lead, strongest down its middle and
                // nothing in the gap, which is what leaves the diagonal marks
                // across a patch of bare paper.
                let ink = ((hatch - 0.6).max(0.0) / 0.4) * HATCH_INK * gain;
                // The harder the hand presses, the darker the lead lies.
                let pressed = (outline.get(x, y).r as f32 / 255.0 * gain).min(1.0);
                let darken = 1.0 - EDGE_DARKENING * pressed;

                let colour = wash.get(x, y);
                let i = x as usize * 4;
                for (c, value) in [colour.r, colour.g, colour.b].into_iter().enumerate() {
                    let lead = value as f32 * darken;
                    out[i + c] = (paper[c] + (lead - paper[c]) * coverage - ink)
                        .clamp(0.0, 255.0)
                        .round() as u8;
                }
                // Alpha stands: drawing on paper does not change the layer's
                // shape.
            }
        });
}

/// How much fine detail sits at each pixel, in levels, as a picture in its own
/// right: how far it is from what a blur of [`DETAIL_SCALE`] leaves.
pub(crate) fn detail_map(pixmap: &Pixmap) -> Pixmap {
    let mut smoothed = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(&mut smoothed, DETAIL_SCALE);
    let (sharp, soft) = (pixmap, &smoothed);
    let w = pixmap.width() as i32;

    let mut out = Pixmap::new(pixmap.width(), pixmap.height());
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..w {
                let lost = (luma(sharp.get(x, y)) - luma(soft.get(x, y)))
                    .abs()
                    .min(255.0) as u8;
                let i = x as usize * 4;
                out[i..i + 3].copy_from_slice(&[lost; 3]);
                out[i + 3] = 255;
            }
        });
    out
}

/// A smooth 0..1 ramp between two thresholds — nothing below `from`, all of it
/// above `to`, and no hard line anywhere for the eye to find.
pub(crate) fn fade(value: f32, from: f32, to: f32) -> f32 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn luma(c: Rgba8) -> f32 {
    0.299 * c.r as f32 + 0.587 * c.g as f32 + 0.114 * c.b as f32
}

/// How much pencil this pixel has on it, from 0 for bare paper to 1 for the
/// middle of a stroke.
///
/// Two hatches at right angles — the second at half strength, which is what
/// makes it read as a crosshatch rather than as two equal grids — each broken
/// into dashes along its length so that no line runs the width of the picture.
pub(crate) fn hatch(x: i32, y: i32, width: u32) -> f32 {
    let spacing = width.max(1) as f32 * HATCH_SPACING;
    // The two diagonals. `across` counts stripes, `along` runs down a stroke.
    let one = stroke((x + y) as f32, (x - y) as f32, spacing, 0);
    let other = stroke((x - y) as f32, (x + y) as f32, spacing, 1);
    one.max(other * 0.45)
}

/// One direction of the hatch.
pub(crate) fn stroke(across: f32, along: f32, spacing: f32, seed: u32) -> f32 {
    // Which stripe this is, and where in it we are.
    let stripe = (across / spacing).floor();
    // A dash is a few stripe-widths of stroke with a gap after it; jittering
    // the phase per stripe keeps the dashes from lining up into a grid.
    let jitter = noise(stripe as i32, seed as i32) * spacing;
    let t = ((across + jitter) / spacing).fract().abs();
    // A ridge across the stripe: strongest down its middle, nothing at its
    // edges. The exponent narrows it, so the stroke covers well under half
    // the stripe and the paper shows between one line and the next — without
    // it the hatch is a solid fill and the picture reads as a wash.
    let ridge = (1.0 - (2.0 * t - 1.0).abs()).powf(STROKE_SHARPNESS);

    let dash = noise((along / (spacing * 3.0)).floor() as i32, stripe as i32 + 31);
    ridge * (0.35 + 0.65 * dash)
}

/// Deterministic 0..1 noise from two whole numbers.
pub(crate) fn noise(a: i32, b: i32) -> f32 {
    let mut h = (a as u32).wrapping_mul(0x27d4_eb2d)
        ^ (b as u32).wrapping_mul(0x1656_67b1)
        ^ 0x9e37_79b9
        ^ crate::photorust::seed().wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545_f491);
    h ^= h >> 13;
    (h % 1024) as f32 / 1023.0
}

/// The smallest median the picture is flattened by, and how much further each
/// step of Edge Simplicity takes it, in pixels.
///
/// A median is the right tool for cutting paper: it rubs out anything smaller
/// than half its window outright rather than fading it, and leaves the
/// boundaries it keeps as sharp as it found them. A blur would do the opposite
/// of both. The floor is there because Edge Simplicity 0 still asks for shapes
/// rather than for the photograph's own grain.
pub(crate) const SIMPLIFY_FLOOR: u32 = 1;

pub(crate) const SIMPLIFY_PER_STEP: u32 = 1;

/// Filter ▸ Artistic ▸ Cutout: the picture rebuilt out of pieces of coloured
/// paper.
///
/// CS6 describes it as a picture "made from roughly cut-out pieces of coloured
/// paper", and that is what the implementation is: the picture is flattened
/// into areas, cut along their boundaries, and each piece is then painted one
/// flat colour.
///
/// * **The flattening** is a median whose reach grows with Edge Simplicity. It
///   is what decides how much detail survives to be cut around at all — the
///   veins in a petal, the grain of a leaf — because a median smaller than a
///   feature keeps it and one larger rubs it out.
/// * **The cut** puts each channel into one of `levels` bands and takes a piece
///   to be a run of pixels agreeing on all three. Bands per channel rather than
///   bands of brightness: a pink petal and a green leaf can be exactly as
///   bright as each other, and cutting on brightness alone would join them into
///   one piece and paint the pair some average mud. Number of Levels is
///   therefore how finely the picture is cut, and the size of the pieces falls
///   away as it rises.
/// * **The colour** of a piece is the mean of what the picture had underneath
///   it. This is the difference between Cutout and Posterize, which is the
///   thing it is most often mistaken for: Posterize snaps every pixel to a
///   fixed grid of values, so a photograph comes back in colours it never
///   contained, while cut paper is chosen to match what it stands for. The
///   result is a palette of the picture's own muted colours rather than of
///   primaries.
/// * **Edge Fidelity** is how closely a piece is allowed to follow the picture.
///   At 3 the cut is taken exactly where the bands fall; below that the map is
///   passed through a majority vote first, which rounds the corners off a piece
///   and drops the single-pixel fringe along its boundary — scissors rather
///   than a scalpel.
///
/// Alpha is left alone: cutting the picture up does not change the layer's
/// shape.
///
/// No GPU path, and this one is not close. Finding the pieces is a flood fill,
/// which is the standing example in CLAUDE.md §7 of what does not fit a shader:
/// it is inherently sequential, each step depending on where the last one got
/// to. The median in front of it is already accelerated on its own account.
pub fn cutout(pixmap: &mut Pixmap, levels: u32, simplicity: u32, fidelity: u32) {
    if pixmap.is_empty() {
        return;
    }
    let levels = levels.clamp(*CUTOUT_LEVELS.start(), *CUTOUT_LEVELS.end());
    let simplicity = simplicity.clamp(*CUTOUT_SIMPLICITY.start(), *CUTOUT_SIMPLICITY.end());
    let fidelity = fidelity.clamp(*CUTOUT_FIDELITY.start(), *CUTOUT_FIDELITY.end());

    // Flatten the picture into areas worth cutting around...
    let mut flat = pixmap.clone();
    let reach = SIMPLIFY_FLOOR + simplicity * SIMPLIFY_PER_STEP;
    crate::photorust::convolve::median_filter(&mut flat, reach);
    // ...band it, so that "the same colour" becomes a question with a yes or
    // no answer...
    let mut bands = band_map(&flat, levels);
    // ...and let the scissors round off what the bands left ragged. Fidelity 3
    // is the scalpel and skips the vote entirely.
    let vote = (*CUTOUT_FIDELITY.end() - fidelity) as i32;
    if vote > 0 {
        bands = tidy(&bands, vote);
    }

    paint_pieces(pixmap, &bands);
}

/// Which band each channel of each pixel falls in, as a picture in its own
/// right — `r`, `g` and `b` hold band numbers rather than colours.
pub(crate) fn band_map(source: &Pixmap, levels: u32) -> Pixmap {
    let mut out = Pixmap::new(source.width(), source.height());
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(source.as_bytes().par_chunks_exact(stride))
        .for_each(|(out, src)| {
            for (out, src) in out.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
                for c in 0..3 {
                    // 256 rather than 255, so that every band is the same
                    // width and only 255 itself would overflow the top one.
                    out[c] = ((src[c] as u32 * levels) / 256).min(levels - 1) as u8;
                }
                out[3] = 255;
            }
        });
    out
}

/// The band map with each pixel replaced by whichever banding is commonest
/// within `radius` of it — the majority vote behind Edge Fidelity.
///
/// A tie is settled in favour of the pixel's own banding, so a vote can round a
/// corner off or rub out a fringe but never moves a boundary that both sides
/// agree on.
pub(crate) fn tidy(bands: &Pixmap, radius: i32) -> Pixmap {
    let (w, h) = (bands.width() as i32, bands.height() as i32);
    let mut out = bands.clone();
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            // At most 5×5 candidates, so a linear scan beats any map.
            let mut seen: Vec<(u32, u32)> = Vec::with_capacity(25);
            for x in 0..w {
                seen.clear();
                let mine = packed(bands.get(x, y));
                for yy in (y - radius).max(0)..=(y + radius).min(h - 1) {
                    for xx in (x - radius).max(0)..=(x + radius).min(w - 1) {
                        let key = packed(bands.get(xx, yy));
                        match seen.iter_mut().find(|(k, _)| *k == key) {
                            Some((_, n)) => *n += 1,
                            None => seen.push((key, 1)),
                        }
                    }
                }
                let best = seen.iter().map(|&(_, n)| n).max().unwrap_or(0);
                let won = seen
                    .iter()
                    .find(|&&(k, n)| n == best && k == mine)
                    .or_else(|| seen.iter().find(|&&(_, n)| n == best))
                    .map(|&(k, _)| k)
                    .unwrap_or(mine);
                let i = x as usize * 4;
                out[i] = (won >> 16) as u8;
                out[i + 1] = (won >> 8) as u8;
                out[i + 2] = won as u8;
            }
        });
    out
}

/// One pixel's banding as a single number, so that "the same piece of paper"
/// is one comparison rather than three.
pub(crate) fn packed(c: Rgba8) -> u32 {
    (c.r as u32) << 16 | (c.g as u32) << 8 | c.b as u32
}

/// Cut `pixmap` along the boundaries in `bands` and paint each piece the mean
/// of what it covers.
///
/// A piece is a run of pixels reachable from one another through neighbours
/// with the same banding — four-connected, so that two areas touching only at a
/// corner are two pieces, which is what a pair of scissors would leave.
///
/// The fill walks whole rows at a time rather than single pixels. A piece is
/// routinely most of the picture — a sky, or the background this filter is
/// usually pointed at — and a stack holding one entry per pixel in it would be
/// both slower and, on a photograph, hundreds of megabytes.
pub(crate) fn paint_pieces(pixmap: &mut Pixmap, bands: &Pixmap) {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    let count = (w * h) as usize;
    const UNCUT: u32 = u32::MAX;

    let key = |x: i32, y: i32| packed(bands.get(x, y));
    let mut piece = vec![UNCUT; count];
    // Per piece: the running total of the colours under it, and how many
    // pixels that is.
    let mut totals: Vec<([u64; 3], u64)> = Vec::new();
    let mut runs: Vec<i32> = Vec::new();

    for seed in 0..count {
        if piece[seed] != UNCUT {
            continue;
        }
        let id = totals.len() as u32;
        totals.push(([0; 3], 0));
        let ([r, g, b], n) = &mut totals[id as usize];
        let mine = key(seed as i32 % w, seed as i32 / w);
        runs.push(seed as i32);

        while let Some(start) = runs.pop() {
            let (y, x0) = (start / w, start % w);
            // Another run may have reached this one between being noted and
            // being taken up.
            if piece[start as usize] != UNCUT {
                continue;
            }
            // How far the run reaches either way along its row.
            let mut lo = x0;
            while lo > 0 && key(lo - 1, y) == mine {
                lo -= 1;
            }
            let mut hi = x0;
            while hi + 1 < w && key(hi + 1, y) == mine {
                hi += 1;
            }
            for x in lo..=hi {
                let i = (y * w + x) as usize;
                piece[i] = id;
                let px = pixmap.get(x, y);
                *r += px.r as u64;
                *g += px.g as u64;
                *b += px.b as u64;
                *n += 1;
            }
            // The rows above and below, which the run may have opened up:
            // note the start of each stretch of them that belongs to this
            // piece and has not been taken yet.
            for row in [y - 1, y + 1] {
                if row < 0 || row >= h {
                    continue;
                }
                let ours = |x: i32| piece[(row * w + x) as usize] == UNCUT && key(x, row) == mine;
                let mut x = lo;
                while x <= hi {
                    if ours(x) {
                        runs.push(row * w + x);
                        while x <= hi && ours(x) {
                            x += 1;
                        }
                    } else {
                        x += 1;
                    }
                }
            }
        }
    }

    let colours: Vec<[u8; 3]> = totals
        .iter()
        .map(|&([r, g, b], n)| {
            let n = n.max(1);
            [(r / n) as u8, (g / n) as u8, (b / n) as u8]
        })
        .collect();

    let stride = pixmap.stride();
    let (colours, piece) = (&colours, &piece);
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let first = row * w as usize;
            for (x, out) in out.chunks_exact_mut(4).enumerate() {
                out[0..3].copy_from_slice(&colours[piece[first + x] as usize]);
                // Alpha stands: cutting the picture up does not change the
                // layer's shape.
            }
        });
}

/// How many colours the paint is mixed from at Brush Detail 0, and how many
/// more each step of the slider adds.
///
/// CS6 says Dry Brush "simplifies an image by reducing its range of colours to
/// areas of common colour", and this is that range. At the bottom of the
/// slider the picture is painted out of a handful of colours and comes back
/// poster-like; at the top the steps are finer than the eye can find and what
/// is left is the brushwork alone.
pub(crate) const PAINT_FLOOR: u32 = 4;

pub(crate) const PAINT_PER_STEP: u32 = 6;

/// How much relief each step of Texture raises the brushwork into.
///
/// Texture is **not** grain sprinkled over the picture. Sprinkling is what it
/// looks like it ought to be and the result is unmistakable — it reads as
/// sensor noise, not as paint — and setting it low enough to stop reading as
/// noise leaves a slider that does nothing at all. Both were tried.
///
/// What the slider does in CS6 is give the paint *body*: at 3 the same picture
/// comes back crunchier than at 1, with the strokes standing out from each
/// other and the surfaces rougher, which is what paint laid on thickly looks
/// like. So it is worked as relief — every facet the brush left is pushed
/// further from its neighbours, at the brush's own scale — rather than as
/// anything added on top.
pub(crate) const RELIEF_PER_STEP: f32 = 0.5;

/// The most, in levels, that the relief may move any one pixel.
///
/// Without it the slider grows a white rim around every dark background: the
/// step from a petal to the grass behind it is a hundred levels, and half of
/// that added back is a bloom along the whole outline. The steps between
/// facets, which are what the relief is *for*, are a few levels each and never
/// come near this. So the cap costs the effect nothing and takes the halo off
/// it — the same bargain as Unsharp Mask's threshold, from the other end.
pub(crate) const RELIEF_CAP: f32 = 14.0;

/// How deep the canvas's own tooth bites, in levels, and how many pixels across
/// one tooth of it is. The faint surface under the paint, there at every
/// setting: it is what the picture is painted *on*, not what Texture does.
pub(crate) const TOOTH_DEPTH: f32 = 1.2;

pub(crate) const TOOTH_SCALE: i32 = 3;

/// Filter ▸ Artistic ▸ Dry Brush: the picture repainted with a stiff, half-dry
/// brush — CS6 puts it "between oil and watercolour".
///
/// The brush is the whole filter. A dab of a dry brush picks up one load of
/// colour and puts it down flat, so a stroke lays a patch of a single colour
/// rather than a blend of everything under it — and a painter working up to a
/// boundary loads the brush from *one side* of it, never from across it. That
/// is what keeps a painting's edges crisp while its surfaces go flat, and it
/// is the behaviour to reproduce.
///
/// So each pixel looks at the four square areas that have it at a corner, asks
/// which of them the picture is calmest over, and takes that one's average
/// colour. On a flat surface all four answer much the same and the texture
/// averages away; against a boundary the two quadrants lying across it are in
/// uproar and lose to the two that do not, so the pixel is painted from its own
/// side and the edge stays where it was. Both questions are answered from box
/// blurs — the colour from one over the picture, the calmness from one over how
/// far the picture strays from its own local average — so the cost per pixel
/// does not grow with the brush.
///
/// **Brush Size** is how wide that load is. **Brush Detail** is how many
/// colours the paint is mixed from, which is CS6's own account of the slider —
/// see [`PAINT_FLOOR`]. **Texture** is how much body the paint is laid on with,
/// worked as relief on the brushwork the pass above left — see
/// [`RELIEF_PER_STEP`]. Under all of it is the canvas's own faint tooth, seeded
/// from each pixel's coordinates rather than from a RNG, so a preview, the
/// commit behind it and an undo/redo replay all show the same surface.
///
/// Alpha is left alone: repainting the picture does not change the layer's
/// shape.
///
/// No GPU path. It is per-neighbourhood and uniform, so unlike Cutout it would
/// fit a shader — but it is four box blurs and a choice between four lookups,
/// and like the rest of the filter stack it would upload its input and read the
/// result straight back, which docs/gpu-migration.md says rarely pays for the
/// trip.
pub fn dry_brush(pixmap: &mut Pixmap, size: u32, detail: u32, texture: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*BRUSH_SIZE.start(), *BRUSH_SIZE.end());
    let detail = detail.clamp(*BRUSH_DETAIL.start(), *BRUSH_DETAIL.end());
    let texture = texture.clamp(*BRUSH_TEXTURE.start(), *BRUSH_TEXTURE.end());

    // Brush Size is the half-width of one load, and also how far the four of
    // them sit from the pixel — so the brush considers `4 * size + 1` across
    // and the facets it leaves are about a load wide. Size 0 is no brush at
    // all rather than the smallest one: every load is then the pixel itself
    // and nothing is painted, which is what the bottom of a slider should
    // mean.
    let reach = size;
    paint_in_dabs(pixmap, reach, false);

    // Lay it on thickly, at the scale of the brush that put it there — the
    // relief is of the facets, so anything finer than one would raise the
    // picture's own grain instead of the paint.
    raise_the_paint(
        pixmap,
        reach.max(1) as f32,
        (texture - *BRUSH_TEXTURE.start()) as f32 * RELIEF_PER_STEP,
    );
    finish(pixmap, PAINT_FLOOR + detail * PAINT_PER_STEP, TOOTH_DEPTH);
}

/// Repaint the picture a dab at a time, each dab one flat load of colour taken
/// from whichever side of itself the picture is calmest over.
///
/// Shared by Dry Brush and Fresco, which CS6 gives the same three sliders and
/// which differ in what happens *after* the painting rather than in the
/// painting. `round` is the shape of the dab: square for a stiff flat brush,
/// circular for the rounded dabs a fresco is laid in.
pub(crate) fn paint_in_dabs(pixmap: &mut Pixmap, reach: u32, round: bool) {
    if reach == 0 {
        return;
    }
    let mut load = pixmap.clone();
    if round {
        crate::photorust::convolve::disc_blur(&mut load, reach);
    } else {
        crate::photorust::convolve::box_blur(&mut load, reach);
    }
    let calm = roughness(pixmap, reach);
    lay_the_paint(pixmap, &load, &calm, reach as i32);
}

/// How hard the plaster takes the pigment down, as the exponent of a curve over
/// the tonal range.
///
/// This is the whole difference between Fresco and Dry Brush, which CS6 gives
/// the same three sliders and the same coarse dabs. Pigment laid into wet
/// plaster sinks in and dries dark, and the filter is emphatic about it: the
/// grass behind these flowers goes from a middling green to very nearly black,
/// which is a square of the tone it had.
///
/// [`BURNISH`] is the other end of the same curve. The darks going down is only
/// half of what the reference shows — its petals come back *brighter* than the
/// photograph's, not merely deeper — so the range is carried a little past
/// white before the curve is taken. What was already light is lifted and
/// everything below it still sinks. A plain gamma gives the blacks and loses
/// the flowers.
pub(crate) const PLASTER: f32 = 2.0;

pub(crate) const BURNISH: f32 = 1.1;

/// Filter ▸ Artistic ▸ Fresco: the picture laid into wet plaster.
///
/// CS6 paints it "in a coarse style using short, rounded, and hastily applied
/// dabs", and gives it Dry Brush's three sliders because it is Dry Brush's
/// brush — the same load of flat colour taken from whichever side of a boundary
/// the picture is calm over, which is what keeps a painting's edges while its
/// surfaces go flat. Two things differ, and the second is the one that matters:
///
/// * **The dab is round**, not square. A fresco is laid in with the tip.
/// * **The plaster takes the pigment down.** See [`PLASTER`]: the darks go very
///   dark, the lights stay where they are, and the picture comes back with the
///   weight the reference has. Without it this is Dry Brush with a different
///   name on the menu.
///
/// **Brush Size**, **Brush Detail** and **Texture** mean exactly what they mean
/// in [`dry_brush`], down to the ranges, because in CS6 they are the same three
/// sliders.
///
/// Alpha is left alone, and there is no GPU path, for the same reasons as
/// [`dry_brush`].
pub fn fresco(pixmap: &mut Pixmap, size: u32, detail: u32, texture: u32) {
    if pixmap.is_empty() {
        return;
    }
    let size = size.clamp(*BRUSH_SIZE.start(), *BRUSH_SIZE.end());
    let detail = detail.clamp(*BRUSH_DETAIL.start(), *BRUSH_DETAIL.end());
    let texture = texture.clamp(*BRUSH_TEXTURE.start(), *BRUSH_TEXTURE.end());

    // A coarser hand than Dry Brush's at the same setting — "coarse" and
    // "hastily applied" are CS6's own words for it, and the reference's dabs
    // are plainly larger than the same number gives there. Size 0 is still no
    // brush at all.
    let reach = size * 2;
    paint_in_dabs(pixmap, reach, true);
    sink_into_the_plaster(pixmap);
    raise_the_paint(
        pixmap,
        reach.max(1) as f32,
        (texture - *BRUSH_TEXTURE.start()) as f32 * RELIEF_PER_STEP,
    );
    finish(pixmap, PAINT_FLOOR + detail * PAINT_PER_STEP, TOOTH_DEPTH);
}

/// Take the picture's darks down the way wet plaster takes pigment down.
///
/// Per channel rather than on brightness, so that what a colour loses is its
/// weakest channel first — which is what deepens a colour instead of merely
/// dimming it, and why the reference's greens go black while its pinks only go
/// redder.
pub(crate) fn sink_into_the_plaster(pixmap: &mut Pixmap) {
    // 256 entries is cheaper than a `powf` per channel per pixel, and exact:
    // there are only 256 answers.
    let sunk: [u8; 256] = std::array::from_fn(|v| {
        let carried = (v as f32 / 255.0 * BURNISH).min(1.0);
        (255.0 * carried.powf(PLASTER)).round() as u8
    });
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(4)
        .for_each(|px| {
            for c in 0..3 {
                px[c] = sunk[px[c] as usize];
            }
            // Alpha stands: laying the picture into plaster does not change
            // the layer's shape.
        });
}

/// How far a daub reaches, in pixels either side per step of Brush Size, and
/// how much longer than tall the two Wide brushes lay it.
///
/// The daub is a **median**, not an average, and that is what CS6's Simple
/// brush looks like: a median throws away anything narrower than half its
/// window — the stamens, the veins, the points of the petals — and leaves
/// everything broader exactly where it was, edge and all. So a petal comes
/// back as a rounded, creamy lobe with a crisp outline, which an average
/// (blurred edges) or an edge-preserving blur (the picture untouched, since
/// Surface Blur keeps every fine thing that differs enough) does not give.
pub(crate) const DAUB_REACH: f32 = 0.45;

pub(crate) const WIDE_STRETCH: f32 = 2.0;

/// Sharpness on the painting brushes: an unsharp mask over the daubs, its
/// scale in pixels and its strength per step. This is what draws the thin
/// dark line and light rim round each shape in CS6's Simple.
pub(crate) const EDGE_SCALE: f32 = 2.0;

pub(crate) const EDGE_PER_STEP: f32 = 0.2;

/// Sharpness on the Rough brushes: the same mask taken on brightness, wider
/// and much harder, and only in one direction. Dark Rough's halo on the dark
/// side of every boundary is what becomes CS6's heavy black outline.
pub(crate) const ROUGH_SCALE: f32 = 3.0;

pub(crate) const ROUGH_PER_STEP: f32 = 0.5;

/// The texture of the Rough brushes, in levels and in pixels of speck.
///
/// Two coats. The *tooth* goes on under the daubs (except Sparkle's), so the median turns it
/// into blotches the daubs' own shape and the mask above then works on them.
/// The *grit* goes on last, colour by colour, and is what gives CS6's rough
/// petals their scatter of lilac, white and deeper pink.
pub(crate) const DAUB_TOOTH: f32 = 15.0;

pub(crate) const DAUB_TOOTH_SCALE: f32 = 2.5;

pub(crate) const DAUB_GRIT: f32 = 14.0;

pub(crate) const DAUB_GRIT_SCALE: f32 = 2.0;

/// How much of the daub Wide Blurry softens it by, as a blur of this fraction
/// of its reach.
pub const WIDE_BLUR: f32 = 0.3;

#[cfg(test)]
mod tests;
