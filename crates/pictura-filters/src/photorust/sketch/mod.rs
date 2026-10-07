//! Filter ▸ Sketch.
//!
//! CS6 keeps this family in the Filter Gallery, as it does Artistic and Brush
//! Strokes. The Gallery is not built (docs/ROADMAP.md), so the filters live
//! under a Filter ▸ Sketch submenu instead.
//!
//! What binds the family together is that **it paints in the two swatches**,
//! not in the picture's own colours: every one of these but Chrome and Water
//! Paper throws the hue away, works on brightness alone, and maps the answer
//! between the document's foreground and background. The colours are
//! properties of the document rather than of the dialog, so the bridge fills
//! them in — see `Engine::filter_for`.

use crate::photorust::artistic::blur_field;
use crate::photorust::brush_strokes::{unit_spread, StrokeDirection};
use crate::photorust::pixmap::{Pixmap, Rgba8};
use crate::photorust::texture::{apply_relief_weighted, Finish, Light, Texture};
pub(crate) use crate::HalftoneType as HalftonePattern;
use rayon::prelude::*;

mod pen_to_plaster;
mod reticulation_to_water_paper;
pub(crate) use pen_to_plaster::*;
pub(crate) use reticulation_to_water_paper::*;

/// CS6's ranges for Bas Relief, which its two sliders run over.
pub const RELIEF_DETAIL: std::ops::RangeInclusive<u32> = 1..=15;

pub const RELIEF_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// How far the brightness is softened before the light is run across it, in
/// pixels per step of Smoothness.
///
/// This is the whole of Smoothness: it is what decides whether the carving
/// comes out as crisp chiselled detail or as broad, rounded, half-melted
/// forms, and nothing else in the filter changes that.
pub(crate) const RELIEF_SMOOTH_PER_STEP: f32 = 0.38;

/// How far apart the two samples are taken, in pixels — the depth of the
/// carving. One pixel either side: a relief is shallow, and reading further
/// turns the edges into double lines.
pub(crate) const RELIEF_REACH: f32 = 1.0;

/// How hard the slope is driven into the two swatches, per step of Detail.
///
/// Detail is the gain, not the scale. At the bottom of the slider only the
/// strongest boundaries lift off the flat mid-tone, which is CS6's
/// "generalized shapes"; at the top every small change in the surface is
/// pushed to one swatch or the other.
pub(crate) const RELIEF_GAIN_PER_STEP: f32 = 0.85;

/// Filter ▸ Sketch ▸ Bas Relief: the picture carved in shallow relief and lit
/// from one side, in the two swatches.
///
/// The picture's brightness is read as a height field: what was light stands
/// proud, what was dark is cut away. **Smoothness** softens that surface
/// first — a low value leaves every fleck of the photograph as its own ridge,
/// a high one rounds the whole thing off. Then the surface is lit from
/// **Light**: each pixel is compared with its neighbour on the lit side, and
/// a slope facing the light is driven towards the background colour while one
/// facing away goes to the foreground. **Detail** is how hard it is driven.
///
/// A flat surface catches no light either way, so it comes out at the midpoint
/// of the two swatches — which is why most of the picture is an even tone with
/// the carving standing out of it.
///
/// Alpha is left alone.
///
/// No GPU path. It is two taps and a blend per pixel — the kind of work that
/// would fit, but the upload and read-back would cost more than the
/// arithmetic saves (docs/gpu-migration.md).
pub fn bas_relief(
    pixmap: &mut Pixmap,
    detail: u32,
    smoothness: u32,
    light: Light,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let detail = detail.clamp(*RELIEF_DETAIL.start(), *RELIEF_DETAIL.end()) as f32;
    let smoothness = smoothness.clamp(*RELIEF_SMOOTHNESS.start(), *RELIEF_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The surface: the picture's own brightness, softened by Smoothness.
    let mut surface: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    blur_field(&mut surface, w, h, smoothness * RELIEF_SMOOTH_PER_STEP);

    let (lx, ly) = light.towards();
    let (dx, dy) = (lx * RELIEF_REACH, ly * RELIEF_REACH);
    let gain = detail * RELIEF_GAIN_PER_STEP;
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
    let surface = &surface;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let at = |ox: f32, oy: f32| {
                    let sx = (x as f32 + ox).round().clamp(0.0, (w - 1) as f32) as usize;
                    let sy = (y as f32 + oy).round().clamp(0.0, (h - 1) as f32) as usize;
                    surface[sy * w + sx]
                };
                // A face is lit when it *falls* towards the light: walking
                // south from a bump's peak with the sun in the south, the
                // ground drops away and the slope you are on is the one
                // facing the sun. Reading the slope the other way up lights
                // every ridge from the opposite side to the one asked for.
                let slope = (at(-dx, -dy) - at(dx, dy)) / 255.0;
                let lit = (0.5 + slope * gain).clamp(0.0, 1.0);
                for (c, (dark, pale)) in ink.iter().zip(paper.iter()).enumerate() {
                    px[c] = (dark + (pale - dark) * lit).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: carving the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Chalk & Charcoal, which its three sliders run over.
pub const CHALK_AREA: std::ops::RangeInclusive<u32> = 0..=20;

pub const CHALK_PRESSURE: std::ops::RangeInclusive<u32> = 0..=5;

/// The mid grey the two media are drawn on.
///
/// A literal neutral grey, not the midpoint of the two swatches: with a blue
/// foreground and a white background the midpoint is pale blue, and CS6's
/// ground stays grey. Adobe calls it "a solid midtone gray foundation", and
/// that is exactly what it is — the paper, not a mixture of the sticks.
pub(crate) const CHALK_GROUND: f32 = 128.0;

/// How far the brightness is smudged before either stick is laid on it, in
/// pixels. Charcoal does not respect single pixels.
pub(crate) const CHALK_SMUDGE: f32 = 1.4;

/// Where the charcoal reaches to, as a fraction of white: a floor and a step
/// of Charcoal Area. Anything darker than this is taken by the foreground.
pub(crate) const CHARCOAL_FROM: f32 = 0.185;

pub(crate) const CHARCOAL_PER_STEP: f32 = 0.019;

/// Where the chalk reaches down to, as a fraction of white: a ceiling and a
/// step of Chalk Area. Anything lighter than this is taken by the background.
pub(crate) const CHALK_FROM: f32 = 0.955;

pub(crate) const CHALK_PER_STEP: f32 = 0.0125;

/// How soft the change from ground to stick is: the widest it gets, and how
/// far each step of Stroke Pressure narrows it.
///
/// This is Stroke Pressure. A light hand leaves the tone grading into the
/// paper; a heavy one lays the stick down flat, and the picture separates
/// into three colours with hard edges between them.
pub(crate) const CHALK_SOFT: f32 = 0.38;

pub(crate) const CHALK_SOFT_PER_STEP: f32 = 0.072;

/// How long the strokes are, in pixels, and how wide. Long coarse drags, not
/// fine hatching: a stick of charcoal is blunt.
pub(crate) const CHALK_STROKE: f32 = 26.0;

pub(crate) const CHALK_STROKE_WIDTH: f32 = 1.1;

/// How far the strokes swing the tone they are read against.
///
/// The swing is what breaks a boundary into separate marks rather than a
/// feathered outline, and it is also what puts texture *inside* an area.
///
/// **It does not change with Stroke Pressure**, and it is worth knowing why,
/// because scaling it looks like the obvious thing to do. The swing and the
/// threshold width together already give both ends of that slider: wound
/// down, the threshold is wide, so the swing only moves coverage a little
/// either way and the picture grades smoothly with no strokes to speak of;
/// wound up, the threshold is a hard line, so ground within a swing of it
/// breaks into separate marks while everything further off lies flat. Making
/// the swing fall as well flattens nothing extra at the top and turns the
/// bottom of the slider — which should be the *smoothest* setting there is —
/// into hatching laid over the whole picture.
pub(crate) const CHALK_SWING: f32 = 0.06;

/// Which way each stick is dragged, in degrees anticlockwise from the
/// horizontal. Opposite diagonals: it is what tells the two media apart where
/// they meet, and the crossing marks are plain in CS6 wherever the ground
/// shows between them.
pub(crate) const CHARCOAL_ANGLE: f32 = -45.0;

pub(crate) const CHALK_ANGLE: f32 = 45.0;

/// Filter ▸ Sketch ▸ Chalk & Charcoal: the picture redrawn in charcoal and
/// chalk over a mid-grey ground.
///
/// Three tones and no more. The dark of the picture is taken by charcoal in
/// the foreground colour, the light by chalk in the background colour, and
/// what neither reaches is left as bare grey paper — which is why the result
/// reads as a drawing rather than as a tinted photograph.
///
/// **Charcoal Area** is how far up the tones the charcoal climbs and **Chalk
/// Area** how far down the chalk comes; wound far enough they meet in the
/// middle and the grey disappears. **Stroke Pressure** is how hard the sticks
/// are pressed: lightly, and each grades into the paper; heavily, and they lie
/// flat with a hard edge.
///
/// Both are dragged along opposite diagonals, and the marks swing the tone
/// they are read against, so an edge breaks into separate strokes instead of
/// being feathered.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons: the streaked noise each stick is
/// dragged along would upload and read back for a few taps of arithmetic.
pub fn chalk_and_charcoal(
    pixmap: &mut Pixmap,
    charcoal_area: u32,
    chalk_area: u32,
    pressure: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let charcoal_area = charcoal_area.clamp(*CHALK_AREA.start(), *CHALK_AREA.end()) as f32;
    let chalk_area = chalk_area.clamp(*CHALK_AREA.start(), *CHALK_AREA.end()) as f32;
    let pressure = pressure.clamp(*CHALK_PRESSURE.start(), *CHALK_PRESSURE.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    blur_field(&mut tone, w, h, CHALK_SMUDGE);

    let marks = |angle: f32, salt: usize| {
        let radians: f32 = angle.to_radians();
        let mut field = crate::photorust::artistic::streaked_noise(
            w,
            h,
            CHALK_STROKE,
            salt,
            (radians.cos(), -radians.sin()),
        );
        blur_field(&mut field, w, h, CHALK_STROKE_WIDTH);
        unit_spread(&mut field);
        field
    };
    let charcoal_marks = marks(CHARCOAL_ANGLE, 131);
    let chalk_marks = marks(CHALK_ANGLE, 132);

    let charcoal_edge = CHARCOAL_FROM + charcoal_area * CHARCOAL_PER_STEP;
    let chalk_edge = CHALK_FROM - chalk_area * CHALK_PER_STEP;
    let soft = (CHALK_SOFT - pressure * CHALK_SOFT_PER_STEP).max(0.015);
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
    let (tone, charcoal_marks, chalk_marks) = (&tone, &charcoal_marks, &chalk_marks);

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let stick = |edge: f32, swing: f32, below: bool| {
                    let against = tone[i] + swing.clamp(-2.0, 2.0) * CHALK_SWING;
                    let over = if below {
                        edge - against
                    } else {
                        against - edge
                    };
                    let t = (over / soft * 0.5 + 0.5).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                };
                let mut dark = stick(charcoal_edge, charcoal_marks[i], true);
                let mut light = stick(chalk_edge, chalk_marks[i], false);
                // Wound far enough the two areas overlap. Neither stick gives
                // way to the other — they mix on the paper.
                let laid = dark + light;
                if laid > 1.0 {
                    dark /= laid;
                    light /= laid;
                }
                let bare = 1.0 - dark - light;
                for (c, (dark_c, pale_c)) in ink.iter().zip(paper.iter()).enumerate() {
                    let v = CHALK_GROUND * bare + dark_c * dark + pale_c * light;
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: redrawing the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Charcoal, which its three sliders run over.
pub const COAL_THICKNESS: std::ops::RangeInclusive<u32> = 1..=7;

pub const COAL_DETAIL: std::ops::RangeInclusive<u32> = 0..=5;

pub const COAL_BALANCE: std::ops::RangeInclusive<u32> = 0..=100;

/// How far the picture is simplified before anything is drawn, in pixels: the
/// most, at Detail 0, and how far each step of Detail claws back.
///
/// Detail runs backwards from the other sliders — it is how much of the
/// photograph's own fine structure survives to be drawn, so *less* of it
/// means *more* blur and broader, emptier shapes.
pub(crate) const COAL_SIMPLIFY: f32 = 2.6;

pub(crate) const COAL_SIMPLIFY_PER_STEP: f32 = 0.44;

/// Where the charcoal fills solid, as a fraction of white: a floor and the
/// span Light/Dark Balance runs over.
///
/// This is the balance. At the bottom only what is nearly black takes any
/// charcoal at all and the sheet is almost bare; at the top the stick reaches
/// most of the way up the tones and very little paper is left.
pub(crate) const COAL_MASS_FROM: f32 = 0.10;

pub(crate) const COAL_MASS_SPAN: f32 = 0.50;

/// How soft the edge of a filled mass is, as a fraction of white.
///
/// A ramp, not a threshold: charcoal is a tonal medium, and a drawing made
/// with it runs from bare paper through every grey to solid black. Cut hard,
/// the picture goes to two colours and the grain that should be reading as
/// shading is left as bare streaks scratched over a silhouette.
///
/// But only just wide enough to shade the subject. Widened past that it
/// reaches up into the midtones and lays a haze over the sea and sky as
/// well — and then there is no drawing left, only an evenly hatched
/// rectangle. The empty paper around the subject is half of what makes this
/// read as a sketch.
pub(crate) const COAL_MASS_SOFT: f32 = 0.22;

/// Which edges are drawn, in Sobel units of the simplified brightness: the
/// threshold at Detail 0, and how far each step of Detail lowers it. A low
/// Detail draws only where the picture really turns; a high one picks up
/// every ripple.
pub(crate) const COAL_EDGE_FROM: f32 = 180.0;

pub(crate) const COAL_EDGE_PER_STEP: f32 = 15.0;

pub(crate) const COAL_EDGE_SPAN: f32 = 30.0;

/// How far a drawn line is softened, and how dark it is allowed to get.
pub(crate) const COAL_EDGE_SOFT: f32 = 0.7;

pub(crate) const COAL_EDGE_WEIGHT: f32 = 0.45;

/// How far an edge is spread, in pixels per step of Charcoal Thickness —
/// CS6's "expands the stroke kernel", and the whole of that slider's effect
/// on the outlines.
pub(crate) const COAL_THICK_PER_STEP: f32 = 0.25;

/// The strokes the stick leaves: which way it is dragged, how long each mark
/// is, and how wide.
///
/// Short. Charcoal hatching is made of many small strokes packed together,
/// not of lines drawn from one side of the picture to the other, and the
/// length here is the length of one of them.
pub(crate) const COAL_ANGLE: f32 = 45.0;

pub(crate) const COAL_STROKE: f32 = 10.0;

pub(crate) const COAL_STROKE_WIDTH: f32 = 0.8;

/// How far the strokes swing the shading either side of what the tone owes.
///
/// Deep enough that the hatch reads as separate marks, shallow enough that
/// they stay grey rather than snapping to paper and ink.
pub(crate) const COAL_GRAIN_DEPTH: f32 = 0.24;

/// How far a thick stick shifts the whole hatch towards ink.
pub(crate) const COAL_THICK_INK: f32 = 0.12;

/// Filter ▸ Sketch ▸ Charcoal: the picture redrawn as a charcoal sketch on
/// bare paper, in the two swatches.
///
/// 1. **The masses.** What is darker than **Light/Dark Balance** fills with
///    charcoal in the foreground colour. Everything else is left as paper in
///    the background colour — there is no middle tone, which is what makes
///    this a sketch rather than a photograph.
/// 2. **The outlines.** The picture's edges are drawn as well, so a shape too
///    light to fill still gets a line round it. **Detail** decides how weak an
///    edge still counts, and **Charcoal Thickness** how far each line is
///    spread — bold heavy outlines at the top of the slider.
/// 3. **The tooth.** Both are dragged along one diagonal and broken up by the
///    grain of the paper, so the white of the sheet streaks through even the
///    densest mass. A thicker stick fills more of it in.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons.
pub fn charcoal(
    pixmap: &mut Pixmap,
    thickness: u32,
    detail: u32,
    balance: u32,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let thickness = thickness.clamp(*COAL_THICKNESS.start(), *COAL_THICKNESS.end()) as f32;
    let detail = detail.clamp(*COAL_DETAIL.start(), *COAL_DETAIL.end()) as f32;
    let balance = balance.clamp(*COAL_BALANCE.start(), *COAL_BALANCE.end()) as f32 / 100.0;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // 1: the picture simplified to what a stick of charcoal can say.
    let mut tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    let simplify = (COAL_SIMPLIFY - detail * COAL_SIMPLIFY_PER_STEP).max(0.3);
    blur_field(&mut tone, w, h, simplify);

    // 2: the outlines, spread by Charcoal Thickness.
    let mut edges = crate::photorust::brush_strokes::sobel(&tone, w, h);
    let edge_from = (COAL_EDGE_FROM - detail * COAL_EDGE_PER_STEP).max(3.0);
    edges.par_iter_mut().for_each(|e| {
        let t = ((*e - edge_from) / COAL_EDGE_SPAN).clamp(0.0, 1.0);
        *e = t * t * (3.0 - 2.0 * t);
    });
    let spread = (thickness * COAL_THICK_PER_STEP).round() as usize;
    crate::photorust::brush_strokes::widest_nearby(&mut edges, w, h, spread);
    // Softened, so a line lies down into the shading around it rather than
    // being stamped over the top of it in flat black.
    blur_field(&mut edges, w, h, COAL_EDGE_SOFT);
    edges.par_iter_mut().for_each(|e| *e *= COAL_EDGE_WEIGHT);

    // 3: the grain of the paper.
    let radians: f32 = COAL_ANGLE.to_radians();
    let mut grain = crate::photorust::artistic::streaked_noise(
        w,
        h,
        COAL_STROKE,
        141,
        (radians.cos(), -radians.sin()),
    );
    blur_field(&mut grain, w, h, COAL_STROKE_WIDTH);
    unit_spread(&mut grain);

    let mass_from = COAL_MASS_FROM + balance * COAL_MASS_SPAN;
    // A heavier stick lays a mark where a finer one would have left paper.
    let bias = thickness / *COAL_THICKNESS.end() as f32 * COAL_THICK_INK;
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
    let (tone, edges, grain) = (&tone, &edges, &grain);

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                // How much charcoal this pixel is owed, before the hatch:
                // a smooth ramp from bare paper to solid, plus the outlines.
                let t =
                    ((mass_from - tone[i] / 255.0) / COAL_MASS_SOFT * 0.5 + 0.5).clamp(0.0, 1.0);
                let mass = t * t * (3.0 - 2.0 * t);
                let owed = mass.max(edges[i]).clamp(0.0, 1.0);

                // The hatch: the strokes swing the shading up and down about
                // what is owed, so a shaded area comes back as many separate
                // marks, each carrying its own grey.
                //
                // The swing closes at both ends of the range. Bare paper has
                // nothing drawn on it and solid black has no gaps left in it;
                // it is only in between that a stick leaves strokes at all.
                // Without that, marks appear on the empty sheet and holes
                // open in the darks. And a *swing* rather than a threshold is
                // what keeps the drawing continuous in tone: compared against
                // a threshold instead, every pixel comes out either paper or
                // ink, and the result is one bit deep and staircased where
                // charcoal is soft and grey.
                // Flattened, so the stick still leaves strokes well down into
                // the darks and up into the lights. Squared off, the swing
                // dies away so fast that anything approaching solid comes
                // back as a flat silhouette with no stroke in it at all.
                let open = (4.0 * owed * (1.0 - owed)).clamp(0.0, 1.0).sqrt();
                let depth = COAL_GRAIN_DEPTH * open;
                let coal = (owed + bias * open + grain[i] * depth).clamp(0.0, 1.0);
                for (c, (dark, pale)) in ink.iter().zip(paper.iter()).enumerate() {
                    px[c] = (pale + (dark - pale) * coal).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: sketching the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Chrome, which its two sliders run over.
pub const CHROME_DETAIL: std::ops::RangeInclusive<u32> = 0..=10;

pub const CHROME_SMOOTHNESS: std::ops::RangeInclusive<u32> = 0..=10;

/// How far the surface is melted before it is polished, in pixels: a floor
/// and a step of Smoothness.
///
/// This is what makes the metal *liquid*. The waveform below draws a band
/// wherever the surface crosses a level, so the shape of those bands is the
/// shape of the surface's contours — and a photograph's contours are ragged
/// until they are smoothed into something that pours.
pub(crate) const CHROME_MELT: f32 = 1.2;

pub(crate) const CHROME_MELT_PER_STEP: f32 = 1.6;

/// How much further the broad surface is poured than the melted one, and how
/// much of the melted surface survives at the bottom of Detail.
pub(crate) const CHROME_BROAD: f32 = 3.0;

pub(crate) const CHROME_DETAIL_FLOOR: f32 = 0.2;

/// What share of the picture is left outside the stretch at each end, so a
/// handful of extreme pixels cannot set the range for all of it.
pub(crate) const CHROME_FLOOR: f32 = 0.005;

/// How far the tones are pulled towards filling the range.
///
/// Part of the way, not all. Stretched fully, a flat expanse like a clear sky
/// is spread across a whole trough of the waveform and comes back as one
/// enormous dark slab, where CS6 leaves it bright; not stretched at all, a
/// photograph in a narrow band of tones barely completes a cycle and the
/// metal has no reflections in it worth the name.
pub(crate) const CHROME_STRETCH: f32 = 0.5;

/// How many times the waveform runs from light to dark across the tonal
/// range: a floor and a step of Detail.
///
/// This is the whole trick. A polished surface does not shade smoothly from
/// dark to light — it mirrors whatever is around it, so the tone runs up to a
/// highlight, breaks, and starts again. Cycling the tone several times over
/// the range is what the eye reads as a horizon reflected in metal.
pub(crate) const CHROME_CYCLES: f32 = 3.5;

pub(crate) const CHROME_CYCLES_PER_STEP: f32 = 0.7;

/// How narrow the dark troughs are.
///
/// A plain cosine spends as much of its run dark as it does light, and the
/// picture comes out a quarter black — banded like corrugated card rather
/// than polished. Metal is bright nearly everywhere; the dark shows only as a
/// thin line where the surface turns right away from the light. Raising this
/// pinches the troughs towards those lines and leaves the rest of the sheet
/// in the light.
pub(crate) const CHROME_TROUGH: f32 = 3.5;

/// How far the sheet is tipped towards the light, as a fraction of a cycle
/// per level of slope.
///
/// A *signed* slope, read along one diagonal: it shifts where the waveform
/// sits rather than adding brightness, so one face of a fold catches the
/// light and the other loses it. Taken as a magnitude instead — which is what
/// an edge detector gives — every edge in the picture gets a bright halo on
/// both sides of it, and the result is a contour map rather than metal.
pub(crate) const CHROME_TIP: f32 = 0.10;

/// Filter ▸ Sketch ▸ Chrome: the picture as a sheet of polished metal.
///
/// 1. **The surface.** The brightness is read as a height field and melted by
///    **Smoothness**, so its contours pour instead of following every ragged
///    edge of the photograph. **Detail** puts back as much of the picture's
///    own fine structure as you ask for.
/// 2. **The polish.** That surface is run through a waveform rather than a
///    straight ramp: the tone climbs to a highlight, clips to white, drops
///    away to near-black and climbs again, several times over the range.
///    Each crossing draws a band, and because the bands follow the surface's
///    contours they read as a horizon reflected in chrome.
///
/// **Detail** also sets how many times the waveform runs, so winding it up
/// gives every ripple a reflection of its own.
///
/// Unlike the rest of this family it does not paint between the two
/// swatches: chrome is grey, and CS6's is grey whatever the swatches are set
/// to.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons.
pub fn chrome(pixmap: &mut Pixmap, detail: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let detail = detail.clamp(*CHROME_DETAIL.start(), *CHROME_DETAIL.end()) as f32;
    let smoothness = smoothness.clamp(*CHROME_SMOOTHNESS.start(), *CHROME_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // 1: the surface.
    let tone: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .collect();
    let mut melted = tone.clone();
    blur_field(
        &mut melted,
        w,
        h,
        CHROME_MELT + smoothness * CHROME_MELT_PER_STEP,
    );

    // The same surface poured out much further, for Detail to choose
    // against: at the bottom of the slider the chrome runs in a few broad
    // sheets, at the top it keeps every fold the melt left.
    //
    // Chosen *between* two melts rather than by adding the fine structure
    // back on top. Added back, the gain needed at the top of the slider also
    // multiplies the photograph's grain, and the waveform turns that grain
    // into dense speckle — the picture stops being metal and becomes noise.
    let mut broad = tone.clone();
    blur_field(
        &mut broad,
        w,
        h,
        (CHROME_MELT + smoothness * CHROME_MELT_PER_STEP) * CHROME_BROAD,
    );
    let keep =
        CHROME_DETAIL_FLOOR + detail / *CHROME_DETAIL.end() as f32 * (1.0 - CHROME_DETAIL_FLOOR);
    let surface: Vec<f32> = broad
        .par_iter()
        .zip(melted.par_iter())
        .map(|(b, m)| b + (m - b) * keep)
        .collect();

    // Stretched to fill the range. A photograph's tones sit in whatever
    // narrow band the exposure gave them, and the waveform below runs on a
    // *fraction* of a cycle across such a band — a couple of sheets of metal
    // and no reflection at all. CS6's cycles several times over exactly such
    // a range, which it can only do by measuring the range first. Read off
    // percentiles rather than the outright darkest and lightest pixel, so one
    // speck of blown highlight cannot flatten the whole picture.
    let surface = {
        let mut sorted: Vec<f32> = surface.clone();
        sorted.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let at = |f: f32| sorted[((sorted.len() - 1) as f32 * f) as usize];
        let (low, high) = (at(CHROME_FLOOR), at(1.0 - CHROME_FLOOR));
        let span = (high - low).max(1.0);
        surface
            .par_iter()
            .map(|v| {
                let pulled = ((v - low) / span).clamp(0.0, 1.0) * 255.0;
                v + (pulled - v) * CHROME_STRETCH
            })
            .collect::<Vec<f32>>()
    };

    // The slope along one diagonal, signed, for the tip towards the light.
    let mut lit = vec![0.0f32; w * h];
    lit.par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, slot) in row.iter_mut().enumerate() {
                let at = |ox: i32, oy: i32| {
                    let sx = (x as i32 + ox).clamp(0, w as i32 - 1) as usize;
                    let sy = (y as i32 + oy).clamp(0, h as i32 - 1) as usize;
                    surface[sy * w + sx]
                };
                *slot = (at(-1, -1) - at(1, 1)) / 255.0;
            }
        });

    // 2: the polish.
    let cycles = CHROME_CYCLES + detail * CHROME_CYCLES_PER_STEP;
    let (surface, lit) = (&surface, &lit);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let level = (surface[i] / 255.0).clamp(0.0, 1.0);
                // Up to a highlight, over the edge, and up again. The tip
                // slides the sheet along the waveform rather than brightening
                // it, so a fold lights on one face and darkens on the other.
                let along = level * cycles + lit[i].clamp(-1.0, 1.0) * CHROME_TIP;
                let wave = 0.5 - 0.5 * (along * std::f32::consts::TAU).cos();
                // Squared off, so it holds white and holds black instead of
                // rippling evenly between them.
                let polished = 1.0 - (1.0 - wave).powf(CHROME_TROUGH);
                let v = (polished * 255.0).round().clamp(0.0, 255.0) as u8;
                px[0] = v;
                px[1] = v;
                px[2] = v;
                // Alpha stands: polishing the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Conté Crayon's two level sliders. Its Texture, Scaling,
/// Relief, Light and Invert are the block in [`super::texture`],
/// shared with Texturizer and the rest.
pub const CONTE_LEVEL: std::ops::RangeInclusive<u32> = 1..=15;

/// How far up the tones a full Foreground Level carries the crayon, and how
/// far down a full Background Level brings the paper — the black point and
/// the white point of the mapping between the two swatches.
pub(crate) const CONTE_BLACK: f32 = 0.28;

pub(crate) const CONTE_WHITE: f32 = 0.45;

/// How hard the paper's weave is brought up: a power on the slope below one
/// lifts the faint threads towards the strong, and the hollows between them
/// are darkened as a groove is.
pub(crate) const CONTE_CRISP: f32 = 0.35;

pub(crate) const CONTE_OCCLUSION: f32 = 1.3;

/// How far the picture is softened before its tones are mapped, in pixels.
///
/// A crayon is blunt. Mapped off the photograph as it stands, every hair of
/// the subject comes back as a hard black line and the drawing is sharper
/// than the thing that supposedly drew it.
pub(crate) const CONTE_BLUNT: f32 = 1.1;

/// Filter ▸ Sketch ▸ Conté Crayon: the picture drawn in a waxy stick on
/// textured paper, in the two swatches.
///
/// The picture's tones are mapped between the two swatches: the dark carried
/// in the foreground colour, the light left as bare paper in the background
/// colour, and **everything between them drawn in the greys between the
/// two**. **Foreground Level** is how far up the tones go solid and
/// **Background Level** how far down the paper stays bare, so together they
/// set the contrast of the drawing. Over that, the paper's own tooth grains
/// the midtones.
///
/// **Texture**, **Scaling**, **Relief**, **Light** and **Invert** are the
/// block CS6 shares with Texturizer: the same paper, read as a height field
/// and lit from one side, laid over the finished drawing.
///
/// The paper reaches the drawing *only* through that lighting, which is why a
/// square weave comes out as horizontal striations under a light from the
/// top: a slope is lit by how far it falls away from the light, so the
/// threads running across the picture catch it and the ones running down it
/// do not. Reading the height field directly instead — on the reasoning that
/// a waxy stick catches the tops of the grain — lays the weave over the
/// picture as a square grid, the same whichever way the light is set, and no
/// Light setting can then look right.
///
/// Alpha is left alone.
///
/// No GPU path, for Bas Relief's reasons.
#[allow(clippy::too_many_arguments)]
pub fn conte_crayon(
    pixmap: &mut Pixmap,
    foreground_level: u32,
    background_level: u32,
    texture: Texture,
    scaling: u32,
    relief: u32,
    light: Light,
    invert: bool,
    foreground: Rgba8,
    background: Rgba8,
) {
    if pixmap.is_empty() {
        return;
    }
    let top = *CONTE_LEVEL.end() as f32 - 1.0;
    let fore = (foreground_level.clamp(*CONTE_LEVEL.start(), *CONTE_LEVEL.end()) - 1) as f32 / top;
    let back = (background_level.clamp(*CONTE_LEVEL.start(), *CONTE_LEVEL.end()) - 1) as f32 / top;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // The picture softened to what a blunt stick can say.
    let mut tones: Vec<f32> = pixmap
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0)
        .collect();
    blur_field(&mut tones, w, h, CONTE_BLUNT);
    let tones = &tones;

    let black_point = CONTE_BLACK * fore;
    let white_point = 1.0 - CONTE_WHITE * back;
    let span = (white_point - black_point).max(0.05);
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
                // Where this tone sits between the two swatches: 1 all
                // crayon, 0 all paper, and every grey in between.
                let laid = ((white_point - tones[y * w + x]) / span).clamp(0.0, 1.0);
                for (c, (dark, pale)) in ink.iter().zip(paper.iter()).enumerate() {
                    px[c] = (pale + (dark - pale) * laid).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: drawing over the picture does not change the
                // layer's shape.
            }
        });

    // And the paper itself, lit from one side.
    //
    // Harder than Texturizer lays the same surface. There the paper is a
    // ground the picture sits on and a whisper of it is enough; here the
    // paper *is* the drawing's grain, and CS6 shows the weave plainly at a
    // Relief of 4 where Texturizer at 4 is barely there. The slope is
    // sharpened and the hollows darkened to bring it up without touching what
    // the slider means.
    apply_relief_weighted(
        pixmap,
        texture,
        scaling,
        relief,
        light,
        invert,
        Finish {
            crisp: CONTE_CRISP,
            occlusion: CONTE_OCCLUSION,
            ..Finish::default()
        },
    );
}

/// CS6's ranges for Graphic Pen, which its two sliders run over.
pub const PEN_LENGTH: std::ops::RangeInclusive<u32> = 1..=15;

pub const PEN_BALANCE: std::ops::RangeInclusive<u32> = 0..=100;

/// How long one stroke is, in pixels: a floor and a step of Stroke Length.
///
/// This is the whole of Stroke Length. The strokes are drawn by a field of
/// noise that stays with itself along **Stroke Direction** for about this far
/// and is uncorrelated across it, so a threshold taken through it comes back
/// as separate marks of this length lying the way the direction says. At the
/// bottom a mark is a pixel — the drawing is dithering, and keeps every fleck
/// of the photograph; at the top it is a long dash, and the texture smooths
/// away.
pub(crate) const PEN_STREAK: f32 = 1.0;

pub(crate) const PEN_STREAK_PER_STEP: f32 = 1.6;

/// How far Light/Dark Balance slides the ink either way, and how hard the tone
/// is driven into it.
///
/// At the middle of the slider the coverage is the tone's own darkness, wound
/// up by the gain so that the darks of a photograph reach solid rather than
/// stopping at stripes. At the bottom nearly everything but the darkest tones
/// is left as bare paper and at the top even a white ground takes ink. CS6
/// opens this filter low, near the bottom of the slider, which is what makes
/// it start as a light sketch rather than as a solid block.
pub(crate) const PEN_BALANCE_SPAN: f32 = 0.9;

pub(crate) const PEN_GAIN: f32 = 1.5;

/// The spread of the threshold field, in units of the noise's own width.
///
/// The coverage is a fraction of the sheet to ink and the threshold field says
/// which pixels those are: where the field is below the coverage the pixel is
/// inked. Widened, the marks separate into distinct strokes with paper between
/// them; narrowed, they clump. This is what keeps a midtone a hatch of strokes
/// rather than an even grey.
pub(crate) const PEN_SPREAD: f32 = 0.22;

#[cfg(test)]
mod tests;
