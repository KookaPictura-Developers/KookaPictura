//! Ink Outlines, Spatter, Sprayed Strokes, Sumi-e (split from `brush_strokes` for file size).

#[allow(unused_imports)]
use super::*;

/// Filter ▸ Brush Strokes ▸ Ink Outlines: the picture redrawn in fine narrow
/// diagonal pen lines.
///
/// Adobe's own account of it is the shape of this: *"repaints lighter and
/// darker areas using strokes that move in opposite, diagonal directions"*,
/// with fine narrow diagonal lines drawn over the original detail.
///
/// 1. **The repaint.** The picture is laid in strokes twice, up to the right
///    and down to the right, and each pixel takes the one its tone calls for:
///    the light parts one way, the dark parts the other. That is the paint
///    under the pen, and it is why the sea comes back in diagonal bands.
/// 2. **The pen.** The detail the repaint left over is drawn along those same
///    diagonals as narrow lines: dark where the picture falls away, by
///    **Dark Intensity**, and white where it rises, by **Light Intensity**.
/// 3. **The fill.** What is deeply in shadow fills solid, so a dark subject
///    reads as a black shape with a drawn edge.
///
/// **Stroke Length** is how long the strokes and the lines are.
///
/// Alpha is left alone.
///
/// No GPU path, for Angled Strokes' reasons.
pub fn ink_outlines(pixmap: &mut Pixmap, length: u32, dark: u32, light: u32) {
    if pixmap.is_empty() {
        return;
    }
    let length = length.clamp(*INK_LENGTH.start(), *INK_LENGTH.end()) as f32;
    let dark = dark.clamp(*INK_DARK.start(), *INK_DARK.end()) as f32;
    let light = light.clamp(*INK_LIGHT.start(), *INK_LIGHT.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let stroke = INK_LINE + length * INK_LINE_PER_STEP;

    let luma = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    let tone: Vec<f32> = pixmap.as_bytes().par_chunks_exact(4).map(luma).collect();
    let mut judged = tone.clone();
    blur_field(&mut judged, w, h, ANGLED_JUDGE);
    // How dark it is hereabouts, over a wide reach.
    let mut broad = tone.clone();
    blur_field(&mut broad, w, h, INK_BROAD_SCALE);

    // 1: the repaint, in strokes running opposite ways.
    let rising = lay_strokes(pixmap, INK_LIGHT_ANGLE, stroke, 81);
    let falling = lay_strokes(pixmap, INK_DARK_ANGLE, stroke, 82);

    // 2: the detail the repaint left over, drawn along each diagonal.
    let mut soft = tone.clone();
    blur_field(&mut soft, w, h, INK_DETAIL_SCALE);
    let detail: Vec<f32> = tone.iter().zip(&soft).map(|(t, s)| t - s).collect();
    let up = streak_field(&detail, w, h, INK_LIGHT_ANGLE, stroke);
    let down = streak_field(&detail, w, h, INK_DARK_ANGLE, stroke);

    // The outline proper: where the picture turns, not where it is merely
    // uneven. The fine detail above draws the hatching inside a shape; this
    // draws the line round it, which is what the filter is named for.
    // Found on a lightly softened tone and left unsoftened afterwards: a
    // blurred edge map draws a smear where CS6 draws a line. This is the
    // filter's name, so it is drawn at nearly full strength whatever the
    // sliders say — they set how much *else* is drawn.
    let mut edges = tone.clone();
    blur_field(&mut edges, w, h, INK_OUTLINE_SOFT);
    let outline = sobel(&edges, w, h);

    let ink_gain = dark * INK_PER_STEP;
    let outline_gain = (INK_OUTLINE_FLOOR + dark * INK_OUTLINE_PER_STEP).min(1.0);
    let rim_gain = (INK_RIM_FLOOR + light * INK_RIM_PER_STEP).min(1.0);
    let chalk_gain = light * INK_CHALK_PER_STEP;
    let black_from = INK_BLACK_FROM + dark * INK_BLACK_PER_STEP;
    let contrast = 1.0 + (dark + light) / 100.0 * INK_CONTRAST;
    // The shape the ink fills, and the ring of ground just outside it. CS6
    // traces a white line right round a filled shape, and nothing tells where
    // that line goes as plainly as the shape's own edge: judged on tone
    // alone, a lit flank inside the shape looks exactly like the ground
    // beside it.
    // The shape the ink fills. Judged on the sharp tone, not on a blur of
    // it: a blurred tone gives a soft, spreading edge where CS6 cuts a crisp
    // silhouette. The holes the lit parts of a dark subject leave are stopped
    // up afterwards, by spreading the mask and pulling it back, which leaves
    // the outside edge exactly where it was.
    let dark_mask = |t: f32, from: f32| {
        let s = ((t / 255.0 - from) / INK_EDGE_SOFT * 0.5 + 0.5).clamp(0.0, 1.0);
        1.0 - s * s * (3.0 - 2.0 * s)
    };
    // Dark here *and* dark hereabouts. The sharp tone puts the edge exactly
    // where the subject's edge is; the broad tone says whether this is a dark
    // mass at all, which a thin dark band of sea is not.
    let shape: Vec<f32> = judged
        .par_iter()
        .zip(broad.par_iter())
        .map(|(t, b)| dark_mask(*t, black_from).min(dark_mask(*b, black_from + INK_BROAD_ALLOW)))
        .collect();
    let shape = close_gaps(&shape, w, h, INK_RIM_CLOSE as f32);
    // Whatever is enclosed by the shape belongs to it, however brightly it is
    // lit: a gloss in the middle of a black flank is not a hole in the horse.
    // Nothing local can tell that — the gloss is broad and bright on every
    // measure — so it is settled by reaching in from the frame instead.
    let shape = fill_holes(&shape, w, h);
    // How much of a dark mass a place is part of, however brightly it is lit
    // itself. A gloss on a black flank is still black flank, and CS6 keeps it
    // as a thin light mark on the black rather than as a white blot.
    let mut wider = judged.clone();
    blur_field(&mut wider, w, h, INK_WITHIN_SCALE);
    let within: Vec<f32> = wider
        .par_iter()
        .map(|b| dark_mask(*b, black_from + INK_BROAD_ALLOW))
        .collect();
    let within = &within;

    // The rim: the ring just inside the shape's edge, drawn white. Outside
    // it would sit on pale water and never be seen.
    let mut inner = shape.clone();
    inner.par_iter_mut().for_each(|v| *v = -*v);
    let eroded = widest_nearby_field(&inner, w, h, INK_RIM_WIDTH);
    let ring: Vec<f32> = shape
        .par_iter()
        .zip(eroded.par_iter())
        .map(|(s, e)| (s - (-e)).max(0.0))
        .collect();

    let (judged, broad, up, down, outline) = (&judged, &broad, &up, &down, &outline);
    let (shape, ring) = (&shape, &ring);

    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(rising.as_bytes().par_chunks_exact(stride))
        .zip(falling.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, ((out, r), f))| {
            for (x, ((px, r), f)) in out
                .chunks_exact_mut(4)
                .zip(r.chunks_exact(4))
                .zip(f.chunks_exact(4))
                .enumerate()
            {
                let i = y * w + x;
                // Light takes the rising stroke, dark the falling one.
                let t = ((judged[i] / 255.0 - 0.5) / ANGLED_SWITCH * 0.5 + 0.5).clamp(0.0, 1.0);
                let lit = t * t * (3.0 - 2.0 * t);

                let mark = |v: f32| {
                    let t = ((v - INK_MARK_FROM) / (INK_MARK_FULL - INK_MARK_FROM)).clamp(0.0, 1.0);
                    t * t * (3.0 - 2.0 * t)
                };
                // The pen follows the stroke the paint was laid with: the
                // dark line down to the right, the light line up to the left.
                let edge = ((outline[i] - INK_OUTLINE_FROM)
                    / (INK_OUTLINE_FULL - INK_OUTLINE_FROM))
                    .clamp(0.0, 1.0);
                let drawn_edge = edge * edge * (3.0 - 2.0 * edge);
                // Which side of the boundary this pixel is on decides which
                // pen draws it: the dark side is inked, the light side is
                // drawn in white. That is CS6's crisp white rim round a black
                // shape, and the black line round a bright one.
                let side = judged[i] - broad[i];
                let dark_side = if side < 0.0 { drawn_edge } else { 0.0 };
                let light_side = if side > 0.0 { drawn_edge } else { 0.0 };
                let inked = ((mark(-down[i]) * ink_gain).max(dark_side * outline_gain)).min(1.0);
                // White on the ring outside a filled shape — the rim — and
                // wherever the pen drew a light mark on light ground.
                let rim = ring[i] * rim_gain;
                // Held back inside a dark mass, both of them: the lit edge of
                // a gloss on a black flank is a strong edge like any other,
                // and drawn white it puts a white blot in the middle of the
                // silhouette.
                let outside = 1.0 - within[i];
                let chalked = ((mark(up[i]) * chalk_gain * outside)
                    .max(light_side * rim_gain * outside)
                    .max(rim))
                .min(1.0);
                let filled = 1.0 - shape[i];

                let damped = 1.0 - INK_INSIDE_DAMP * within[i];
                for c in 0..3 {
                    let paint = r[c] as f32 * lit + f[c] as f32 * (1.0 - lit);
                    let paint = ((paint - INK_PIVOT) * contrast + INK_PIVOT).clamp(0.0, 255.0);
                    let paint = paint * damped;
                    let v = paint * filled * (1.0 - inked);
                    // The rim is blocked inside a filled shape: it belongs on
                    // the water beside the horse, not on the horse's own lit
                    // flanks, which CS6 leaves solid black.
                    px[c] = (v + (255.0 - v) * chalked).round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: drawing over the picture does not change the
                // layer's shape.
            }
        });
}

/// CS6's ranges for Spatter, which its two sliders run over.
pub const SPATTER_RADIUS: std::ops::RangeInclusive<u32> = 0..=25;

pub const SPATTER_SMOOTHNESS: std::ops::RangeInclusive<u32> = 1..=15;

/// How far the spray throws a pixel, in pixels per step of Spray Radius.
///
/// Well under a pixel a step: the slider's top of 25 is meant to be drastic,
/// not unrecognisable, and a throw as wide as the slider number scrambles a
/// wave into fog long before it gets there.
pub(crate) const SPATTER_THROW: f32 = 0.45;

/// How large the clustering window is, in pixels either side: a floor and a
/// step of Smoothness. Small windows leave the grit sharp; large ones gather
/// the scattered pixels into rounded droplets.
///
/// It has to be a good fraction of the throw or the picture turns to mush:
/// the median is what puts a hard edge back round each droplet, and a window
/// much smaller than the spray never finds a dominant colour to settle on.
/// It barely widens with Smoothness. It is tempting to make Smoothness size
/// this window, but CS6's high settings come out *finer* than its low ones,
/// not blobbier — a wide median eats the bridle and the eye, which CS6 still
/// shows at 12. Smoothness is spent on [`SPATTER_COHERENCE`] instead.
pub(crate) const SPATTER_CLUSTER: f32 = 1.0;

pub(crate) const SPATTER_CLUSTER_PER_STEP: f32 = 0.08;

/// How far the throw is softened so that neighbouring pixels are thrown
/// together rather than each its own way, in pixels: a floor and a step of
/// Smoothness.
///
/// This is Smoothness. Near the bottom the two fields are nearly white noise
/// and every pixel goes its own way, which is the sharp, jagged, gritty end
/// of the slider; near the top the throw varies slowly across the picture and
/// whole clumps move together, which is the soft, organic, droplet end.
pub(crate) const SPATTER_COHERENCE: f32 = 0.4;

pub(crate) const SPATTER_COHERENCE_PER_STEP: f32 = 0.15;

/// How far the picture is softened before its edges are measured, and how
/// much a full edge holds the spray back. Some hold keeps a subject's
/// silhouette readable; too much and the outline never breaks up, which is
/// the one thing the filter is for.
pub(crate) const SPATTER_EDGE_SCALE: f32 = 2.0;

pub(crate) const SPATTER_EDGE_FROM: f32 = 10.0;

pub(crate) const SPATTER_EDGE_FULL: f32 = 60.0;

pub(crate) const SPATTER_EDGE_HOLD: f32 = 0.3;

/// Filter ▸ Brush Strokes ▸ Spatter: the picture as an airbrush would spatter
/// it.
///
/// Three stages, which are the filter as Adobe describes it:
///
/// 1. **The spray.** Each pixel takes its colour from another one thrown off
///    it, up to **Spray Radius** in each direction. The two fields that say
///    which way are softened first, by **Smoothness**, so that neighbours are
///    thrown together: at the bottom of the slider they are near white noise
///    and the picture comes apart into sharp grit, at the top they vary
///    slowly and whole clumps of colour move as one, which is what tears an
///    outline into the tongues and shards the filter is named for.
/// 2. **The clustering.** The sprayed pixels are settled by a median over a
///    small window, which puts a hard edge back round each droplet. A *whole*
///    pixel, not a median per channel — see [`median_pixel`].
/// 3. **Holding the edges.** The throw is damped where the picture has a
///    strong boundary, so the spray stylises an outline instead of eating it.
///
/// At Spray Radius 0 the picture is left exactly as it was.
///
/// Alpha is left alone.
///
/// No GPU path: the clustering is a median, sequential along each row.
pub fn spatter(pixmap: &mut Pixmap, radius: u32, smoothness: u32) {
    if pixmap.is_empty() {
        return;
    }
    let radius = radius.clamp(*SPATTER_RADIUS.start(), *SPATTER_RADIUS.end()) as f32;
    if radius == 0.0 {
        // No spray, so nothing to gather: Smoothness alone is not a filter.
        // Running the clustering anyway would quietly median the picture at
        // the bottom of the slider, where CS6 leaves it alone.
        return;
    }
    let smoothness =
        smoothness.clamp(*SPATTER_SMOOTHNESS.start(), *SPATTER_SMOOTHNESS.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let luma = |p: &[u8]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
    let mut tone: Vec<f32> = pixmap.as_bytes().par_chunks_exact(4).map(luma).collect();
    blur_field(&mut tone, w, h, SPATTER_EDGE_SCALE);
    let edges = sobel(&tone, w, h);

    // Which way each pixel is thrown. Not independent per pixel: a pixel that
    // goes its own way leaves a one-pixel fuzz, where CS6 throws *clumps* —
    // whole tongues of colour several pixels wide torn off an edge. Softening
    // the two fields makes neighbours agree over a few pixels, which is what
    // gives the spray its shards, and the softening widens with Smoothness.
    let coherence = SPATTER_COHERENCE + smoothness * SPATTER_COHERENCE_PER_STEP;
    let mut aside: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| noise((i % w) as i32, (i / w) as i32) * 2.0 - 1.0)
        .collect();
    let mut down: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| noise((i % w) as i32, (i / w + h + 977) as i32) * 2.0 - 1.0)
        .collect();
    for field in [&mut aside, &mut down] {
        blur_field(field, w, h, coherence);
        // The blur flattens the field towards nothing; this puts its spread
        // back, so Smoothness changes the size of the clumps and not how far
        // the spray reaches.
        unit_spread(field);
    }

    let throw = radius * SPATTER_THROW;
    let source = pixmap.clone();
    let (edges, src) = (&edges, &source);
    let (aside, down) = (&aside, &down);
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let i = y * w + x;
                let edge = ((edges[i] - SPATTER_EDGE_FROM)
                    / (SPATTER_EDGE_FULL - SPATTER_EDGE_FROM))
                    .clamp(0.0, 1.0);
                let reach = throw * (1.0 - SPATTER_EDGE_HOLD * edge);
                let away = |v: f32| v.clamp(-2.0, 2.0) * reach;
                let sx = (x as f32 + away(aside[i]))
                    .round()
                    .clamp(0.0, (w - 1) as f32) as usize;
                let sy = (y as f32 + away(down[i]))
                    .round()
                    .clamp(0.0, (h - 1) as f32) as usize;
                let from = (sy * w + sx) * 4;
                px[..3].copy_from_slice(&src.as_bytes()[from..from + 3]);
                // Alpha stands: spattering the picture does not change the
                // layer's shape.
            }
        });

    let cluster = (SPATTER_CLUSTER + smoothness * SPATTER_CLUSTER_PER_STEP).round() as i32;
    if cluster > 0 {
        let gathered = median_pixel(pixmap, cluster);
        for (out, from) in pixmap
            .as_bytes_mut()
            .chunks_exact_mut(4)
            .zip(gathered.as_bytes().chunks_exact(4))
        {
            out[..3].copy_from_slice(&from[..3]);
        }
    }
}

/// CS6's ranges for Sprayed Strokes, which its two sliders run over.
pub const SPRAYED_LENGTH: std::ops::RangeInclusive<u32> = 0..=20;

pub const SPRAYED_RADIUS: std::ops::RangeInclusive<u32> = 0..=25;

impl StrokeDirection {
    /// The angle the strokes run along, in degrees anticlockwise from the
    /// horizontal — the convention [`lay_strokes`] and [`streak_field`] use.
    pub(crate) fn angle(self) -> f32 {
        match self {
            StrokeDirection::RightDiagonal => 45.0,
            StrokeDirection::Horizontal => 0.0,
            StrokeDirection::LeftDiagonal => 135.0,
            StrokeDirection::Vertical => 90.0,
        }
    }
}

/// How far the spray throws a pixel along the stroke, in pixels per step of
/// Spray Radius. It is thrown *along* the axis and not across it: that is
/// what makes Vertical comb the picture into long fingers rather than simply
/// roughening it.
pub(crate) const SPRAYED_SCATTER: f32 = 0.4;

/// How wide a tooth of the comb is, as a blur across the noise that throws
/// it. Much narrower than Angled Strokes' brush — CS6's spray separates into
/// fine threads, not into slabs.
pub(crate) const SPRAYED_TOOTH: f32 = 0.5;

/// How long the noise stays with itself along the stroke, as a multiple of
/// Stroke Length with a floor in pixels. A pixel and its neighbour up the
/// stroke are thrown together; its neighbour across the stroke is not.
pub(crate) const SPRAYED_RUN: f32 = 1.0;

pub(crate) const SPRAYED_RUN_FLOOR: f32 = 5.0;

/// How far the colour is then settled along the stroke, as a fraction of
/// Stroke Length. A median, as Angled Strokes uses: it lays each stroke as a
/// flat slab with a crisp end where an average would blur both.
pub(crate) const SPRAYED_SMEAR: f32 = 0.8;

/// Filter ▸ Brush Strokes ▸ Sprayed Strokes: the picture repainted in
/// angled, scattered strokes of its own dominant colours.
///
/// 1. **The spray.** Each pixel takes its colour from one thrown along the
///    stroke axis, up to **Spray Radius**. The field that throws it runs with
///    the stroke and changes across it, so the picture combs into threads
///    along **Stroke Direction** instead of simply roughening.
/// 2. **The strokes.** The sprayed colour is settled by a median along the
///    same axis, **Stroke Length** long, which gathers the threads into flat
///    strokes of the local dominant colour with crisp ends.
///
/// **Stroke Direction** is one of CS6's four: right diagonal, horizontal,
/// left diagonal, vertical.
///
/// At Spray Radius 0 and Stroke Length 0 the picture is left as it was.
///
/// Alpha is left alone.
///
/// No GPU path, for Angled Strokes' reasons: the settling is a median along a
/// line, and nothing downstream stays on the device.
pub fn sprayed_strokes(pixmap: &mut Pixmap, length: u32, radius: u32, direction: StrokeDirection) {
    if pixmap.is_empty() {
        return;
    }
    let length = length.clamp(*SPRAYED_LENGTH.start(), *SPRAYED_LENGTH.end()) as f32;
    let radius = radius.clamp(*SPRAYED_RADIUS.start(), *SPRAYED_RADIUS.end()) as f32;
    if length == 0.0 && radius == 0.0 {
        // No spray and no stroke: nothing to repaint with.
        return;
    }
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    let radians = direction.angle().to_radians();
    let along = (radians.cos(), -radians.sin());

    // Coherent along the stroke, near white noise across it — the comb.
    let run = (length * SPRAYED_RUN).max(SPRAYED_RUN_FLOOR);
    let mut throw = crate::photorust::artistic::streaked_noise(w, h, run, 91, along);
    blur_field(&mut throw, w, h, SPRAYED_TOOTH);
    unit_spread(&mut throw);

    let source = pixmap.clone();
    let scatter = radius * SPRAYED_SCATTER;
    let (src, throw) = (&source, &throw);
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(w * 4)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(4).enumerate() {
                let offset = throw[y * w + x].clamp(-2.0, 2.0) * scatter;
                let sx = (x as f32 + along.0 * offset)
                    .round()
                    .clamp(0.0, (w - 1) as f32) as usize;
                let sy = (y as f32 + along.1 * offset)
                    .round()
                    .clamp(0.0, (h - 1) as f32) as usize;
                let from = (sy * w + sx) * 4;
                px[..3].copy_from_slice(&src.as_bytes()[from..from + 3]);
                // Alpha stands: repainting the picture does not change the
                // layer's shape.
            }
        });

    if length > 0.0 {
        let settled = median_along(pixmap, along, (length * SPRAYED_SMEAR).max(1.0));
        for (out, from) in pixmap
            .as_bytes_mut()
            .chunks_exact_mut(4)
            .zip(settled.as_bytes().chunks_exact(4))
        {
            out[..3].copy_from_slice(&from[..3]);
        }
    }
}

/// CS6's ranges for Sumi-e, which its three sliders run over.
pub const SUMI_WIDTH: std::ops::RangeInclusive<u32> = 3..=15;

pub const SUMI_PRESSURE: std::ops::RangeInclusive<u32> = 0..=15;

pub const SUMI_CONTRAST: std::ops::RangeInclusive<u32> = 0..=40;

/// The paper and the ink.
///
/// Sumi is a warm black, never the flat `#000` a threshold gives, and the
/// paper it sits on is unbleached — a cool grey-white picture on pure white
/// is the clearest sign of an ink wash that was done with a Levels slider.
pub(crate) const SUMI_PAPER: [f32; 3] = [245.0, 242.0, 231.0];

pub(crate) const SUMI_INK: [f32; 3] = [28.0, 30.0, 36.0];

/// How far the picture is softened before it is read as tone, in pixels: a
/// floor and a step of Stroke Width. A brush carries tone, not detail.
pub(crate) const SUMI_WASH: f32 = 0.6;

pub(crate) const SUMI_WASH_PER_STEP: f32 = 0.22;

/// How far the picture is flattened into slabs of one tone, as a median
/// radius in pixels per step of Stroke Width.
pub(crate) const SUMI_FLATTEN_PER_STEP: f32 = 0.6;

/// How many washes there are between bare paper and full black.
///
/// A painter loads the brush a few times, not two hundred: the whole look
/// rests on there being *few* tones, each one flat. A continuous ramp from
/// the photograph's own luminance is a grey photograph, however well it is
/// blurred.
pub(crate) const SUMI_LEVELS: f32 = 4.0;

/// How far the washes are biased towards bare paper, in fractions of a wash.
/// Without it every faint tone in the photograph picks up the lightest wash
/// and the whole sheet goes muddy, where a painting leaves it empty.
pub(crate) const SUMI_DROP: f32 = 0.3;

/// Where the paper is left bare, as a fraction of white: a floor and a step
/// of Contrast. Everything lighter than this is untouched paper — the empty
/// space that is most of any of these paintings.
pub(crate) const SUMI_PAPER_FROM: f32 = 0.62;

pub(crate) const SUMI_PAPER_PER_STEP: f32 = 0.006;

/// Where the ink goes solid, as a fraction of white: a floor and a step of
/// Contrast. Everything darker than this is simply black.
///
/// A painting has a bottom to its range as well as a top. Scaling the ink by
/// the photograph's own darkest tone instead leaves the deepest shadow at
/// three quarters of a wash — dark grey, never black — and the whole sheet
/// reads as a faded photocopy rather than as ink.
pub(crate) const SUMI_INK_FULL: f32 = 0.15;

pub(crate) const SUMI_INK_FULL_PER_STEP: f32 = 0.004;

/// How much ink the brush carries: a floor and a step of Stroke Pressure.
///
/// The floor is high enough that even a light touch blacks out the deepest
/// shadow — ink is black, and a load that scales the whole range leaves the
/// darkest wash at three quarters, which is dark grey. What Pressure buys is
/// how far the ink reaches *up* into the midtones.
pub(crate) const SUMI_LOAD: f32 = 0.95;

pub(crate) const SUMI_LOAD_PER_STEP: f32 = 0.045;

/// How hard the washes separate from one another, per step of Contrast.
pub(crate) const SUMI_HARD_PER_STEP: f32 = 0.03;

/// How far the ink bleeds into the paper, in pixels: a floor and a step of
/// Stroke Width.
pub(crate) const SUMI_BLEED: f32 = 0.7;

pub(crate) const SUMI_BLEED_PER_STEP: f32 = 0.14;

/// How far the ink creeps along the paper's fibres, in pixels, and how
/// coarse that creep is.
///
/// This is what keeps a wash's edge from being the shape of the photograph
/// underneath it. Ink on paper wanders — it follows the fibres, runs further
/// in one place than the next — and an edge that does not wander reads as a
/// selection that has been feathered.
pub(crate) const SUMI_CREEP: f32 = 2.2;

pub(crate) const SUMI_CREEP_GRAIN: f32 = 1.6;

/// How much darker the rim of a wet wash dries than its middle.
///
/// Water carries the pigment outward as it dries and strands it at the
/// boundary. It is the single most recognisable thing about ink on paper,
/// and nothing else in a tonal filter produces it.
pub(crate) const SUMI_POOL: f32 = 0.22;

/// Which colours survive: nothing below the first, all of it above the
/// second.
///
/// A threshold, not a slope. Scaled straight off saturation, a pale blue sky
/// counts as a colour and picks up a wash of itself — which is how an empty
/// sheet ends up grey.
///
/// Most of these paintings are ink alone; the ones that are not put a few
/// deliberate colours on the same bare ground — a pink blossom, a yellow
/// bird. Keeping colour in proportion to how saturated the photograph
/// already was reproduces both: a beach comes out monochrome, a flower keeps
/// its petals.
pub(crate) const SUMI_COLOUR_FROM: f32 = 0.45;

pub(crate) const SUMI_COLOUR_FULL: f32 = 0.9;

/// The most of itself a colour may keep. There is always ink in the brush:
/// a wash that is purely the photograph's own green is a green photograph,
/// not a painting, and every colour in these pictures is muted by the ink
/// it is mixed with.
pub(crate) const SUMI_COLOUR_MOST: f32 = 0.6;

/// How strongly the paper's own fibre shows through, in levels.
pub(crate) const SUMI_FIBRE: f32 = 3.5;

/// Filter ▸ Brush Strokes ▸ Sumi-e: the picture repainted as a Japanese ink
/// wash — bare warm paper, a few flat washes of warm black, edges that bleed
/// and rims that pool.
///
/// **This is the painting, not CS6's filter.** CS6's Sumi-e lays a diagonal
/// hatch over the photograph and drives the darks down; it is named after the
/// tradition but does not look much like it. What is built here is the
/// tradition — asked for deliberately, and a knowing departure from the
/// parity the rest of this file keeps.
///
/// 1. **The wash.** The picture is softened and flattened into slabs, then
///    read as tone alone and cut into [`SUMI_LEVELS`] washes. Anything
///    lighter than **Contrast**'s threshold is left as bare paper.
/// 2. **The bleed.** Each wash's edge is made to creep along the paper's
///    fibres and then softened, so it wanders instead of tracing whatever was
///    in the photograph.
/// 3. **The pooling.** Where a wash ends, the ink dries darker — the rim that
///    water leaves as it retreats.
/// 4. **The paper.** What is left bare takes the paper's warm white and its
///    fibre. Colour survives only where the photograph was strongly
///    saturated, so most pictures come out in ink alone.
///
/// **Stroke Width** is how wide the brush is and so how far the ink spreads,
/// **Stroke Pressure** how much it carries, and **Contrast** how sharply the
/// washes separate and how much paper is left bare.
///
/// Alpha is left alone.
///
/// No GPU path: the flattening is a median, sequential along each row.
pub fn sumi_e(pixmap: &mut Pixmap, width: u32, pressure: u32, contrast: u32) {
    if pixmap.is_empty() {
        return;
    }
    let width = width.clamp(*SUMI_WIDTH.start(), *SUMI_WIDTH.end()) as f32;
    let pressure = pressure.clamp(*SUMI_PRESSURE.start(), *SUMI_PRESSURE.end()) as f32;
    let contrast = contrast.clamp(*SUMI_CONTRAST.start(), *SUMI_CONTRAST.end()) as f32;
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);

    // 1: the wash. Tone, not detail — and only a few tones of it.
    let mut washed = pixmap.clone();
    crate::photorust::convolve::gaussian_blur_accelerated(
        &mut washed,
        SUMI_WASH + width * SUMI_WASH_PER_STEP,
    );
    let flatten = (width * SUMI_FLATTEN_PER_STEP).round() as i32;
    if flatten > 0 {
        washed = median_pixel(&washed, flatten);
    }

    let paper_from = (SUMI_PAPER_FROM + contrast * SUMI_PAPER_PER_STEP).min(0.95);
    let ink_full = (SUMI_INK_FULL + contrast * SUMI_INK_FULL_PER_STEP).min(paper_from - 0.08);
    let hard = 1.0 + contrast * SUMI_HARD_PER_STEP;
    let load = SUMI_LOAD + pressure * SUMI_LOAD_PER_STEP;
    let mut ink: Vec<f32> = washed
        .as_bytes()
        .par_chunks_exact(4)
        .map(|p| {
            let tone = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0;
            let d = ((paper_from - tone) / (paper_from - ink_full)).clamp(0.0, 1.0);
            let d = (d.powf(1.0 / hard) * load).clamp(0.0, 1.0);
            // Cut into a painter's handful of washes.
            ((d * SUMI_LEVELS - SUMI_DROP).round().max(0.0) / SUMI_LEVELS).min(1.0)
        })
        .collect();

    // 2: the bleed. The wash creeps along the fibres, then softens.
    let grain = |salt: usize| {
        let mut field: Vec<f32> = (0..w * h)
            .into_par_iter()
            .map(|i| noise((i % w) as i32, (i / w + salt) as i32) * 2.0 - 1.0)
            .collect();
        blur_field(&mut field, w, h, SUMI_CREEP_GRAIN);
        unit_spread(&mut field);
        field
    };
    let (aside, down) = (grain(0), grain(h + 613));
    let crept: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let (x, y) = ((i % w) as f32, (i / w) as f32);
            let away = |v: f32| v.clamp(-2.0, 2.0) * SUMI_CREEP;
            let sx = (x + away(aside[i])).round().clamp(0.0, (w - 1) as f32) as usize;
            let sy = (y + away(down[i])).round().clamp(0.0, (h - 1) as f32) as usize;
            ink[sy * w + sx]
        })
        .collect();
    ink = crept;
    let bleed = SUMI_BLEED + width * SUMI_BLEED_PER_STEP;
    blur_field(&mut ink, w, h, bleed);

    // 3: the pooling. A wash dries darker where it ends.
    let rim = sobel(&ink, w, h);

    // 4: the paper.
    let (ink, rim) = (&ink, &rim);
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .zip(washed.as_bytes().par_chunks_exact(stride))
        .enumerate()
        .for_each(|(y, (out, tone))| {
            for (x, (px, t)) in out
                .chunks_exact_mut(4)
                .zip(tone.chunks_exact(4))
                .enumerate()
            {
                let i = y * w + x;
                // How much of the picture's own colour the brush kept.
                let (mut most, mut least) = (0.0f32, 255.0f32);
                for c in 0..3 {
                    most = most.max(t[c] as f32);
                    least = least.min(t[c] as f32);
                }
                let saturation = if most > 1.0 {
                    (most - least) / most
                } else {
                    0.0
                };
                let k = ((saturation - SUMI_COLOUR_FROM) / (SUMI_COLOUR_FULL - SUMI_COLOUR_FROM))
                    .clamp(0.0, 1.0);
                let keep = k * k * (3.0 - 2.0 * k) * SUMI_COLOUR_MOST;

                // How much pigment there is comes from tone alone, never
                // from colour. Giving a saturated area a body of its own
                // lays a flat field of itself over the paper — a green
                // background stops being empty space and becomes a green
                // wall. Colour tints the pigment; it does not summon any.
                let body = (ink[i] + rim[i] * SUMI_POOL).clamp(0.0, 1.0);

                let fibre = (noise(x as i32, (y + 7919) as i32) * 2.0 - 1.0) * SUMI_FIBRE;
                for c in 0..3 {
                    let pigment = SUMI_INK[c] * (1.0 - keep) + t[c] as f32 * 0.85 * keep;
                    let v = SUMI_PAPER[c] * (1.0 - body) + pigment * body + fibre;
                    px[c] = v.round().clamp(0.0, 255.0) as u8;
                }
                // Alpha stands: painting the picture does not change the
                // layer's shape.
            }
        });
}

/// For each pixel, the colour of the neighbour whose brightness is the median
/// of those in a square window `reach` either side.
///
/// A whole pixel, not a median per channel: taking each channel's own median
/// mixes colours that were never next to each other and turns the spray to
/// mush. Choosing one of the sprayed pixels keeps the picture's own palette,
/// which is what gives each droplet a hard edge.
pub(crate) fn median_pixel(source: &Pixmap, reach: i32) -> Pixmap {
    let (w, h) = (source.width() as i32, source.height() as i32);
    let mut out = source.clone();
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            let y = y as i32;
            let (top, bottom) = ((y - reach).max(0), (y + reach).min(h - 1));
            let mut samples: Vec<(u32, usize)> =
                Vec::with_capacity(((2 * reach + 1) * (2 * reach + 1)) as usize);
            for x in 0..w {
                samples.clear();
                let (left, right) = ((x - reach).max(0), (x + reach).min(w - 1));
                for sy in top..=bottom {
                    for sx in left..=right {
                        let i = (sy * w + sx) as usize * 4;
                        let p = &source.as_bytes()[i..i + 3];
                        let luma = 299 * p[0] as u32 + 587 * p[1] as u32 + 114 * p[2] as u32;
                        samples.push((luma, i));
                    }
                }
                let mid = samples.len() / 2;
                let (_, i) = *samples.select_nth_unstable_by_key(mid, |s| s.0).1;
                let o = x as usize * 4;
                row[o..o + 3].copy_from_slice(&source.as_bytes()[i..i + 3]);
            }
        });
    out
}

/// Average a field along a straight line at `angle`, `length` long.
pub(crate) fn streak_field(field: &[f32], w: usize, h: usize, angle: f32, length: f32) -> Vec<f32> {
    let radians = angle.to_radians();
    let (dx, dy) = (radians.cos(), -radians.sin());
    let steps = (length.round() as i32).max(1) / 2;
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, slot) in row.iter_mut().enumerate() {
                let (mut total, mut count) = (0.0f32, 0.0f32);
                for step in -steps..=steps {
                    let sx = (x as f32 + dx * step as f32).round() as i32;
                    let sy = (y as f32 + dy * step as f32).round() as i32;
                    if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                        continue;
                    }
                    total += field[sy as usize * w + sx as usize];
                    count += 1.0;
                }
                *slot = total / count.max(1.0);
            }
        });
    out
}

/// For each pixel, the colour of the sample whose brightness is the median of
/// those along a line through it, `length` long.
pub(crate) fn median_along(source: &Pixmap, along: (f32, f32), length: f32) -> Pixmap {
    let (w, h) = (source.width() as i32, source.height() as i32);
    let steps = (length.round() as i32).max(1) / 2;
    let offsets: Vec<(i32, i32)> = (-steps..=steps)
        .map(|s| {
            (
                (along.0 * s as f32).round() as i32,
                (along.1 * s as f32).round() as i32,
            )
        })
        .collect();
    let mut out = source.clone();
    let stride = out.stride();
    out.as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, row)| {
            let mut samples: Vec<(u32, usize)> = Vec::with_capacity(offsets.len());
            for x in 0..w {
                samples.clear();
                for &(dx, dy) in &offsets {
                    let (sx, sy) = (x + dx, y as i32 + dy);
                    if sx < 0 || sy < 0 || sx >= w || sy >= h {
                        continue;
                    }
                    let i = (sy * w + sx) as usize * 4;
                    let p = &source.as_bytes()[i..i + 3];
                    let luma = 299 * p[0] as u32 + 587 * p[1] as u32 + 114 * p[2] as u32;
                    samples.push((luma, i));
                }
                let mid = samples.len() / 2;
                let (_, i) = *samples.select_nth_unstable_by_key(mid, |s| s.0).1;
                let o = x as usize * 4;
                row[o..o + 3].copy_from_slice(&source.as_bytes()[i..i + 3]);
            }
        });
    out
}

/// Sobel magnitude of a field, clamped at the edges.
pub(crate) fn sobel(field: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let (up, down) = (y.saturating_sub(1), (y + 1).min(h - 1));
            for (x, slot) in row.iter_mut().enumerate() {
                let (left, right) = (x.saturating_sub(1), (x + 1).min(w - 1));
                let at = |xx: usize, yy: usize| field[yy * w + xx];
                let gx = at(right, up) + 2.0 * at(right, y) + at(right, down)
                    - at(left, up)
                    - 2.0 * at(left, y)
                    - at(left, down);
                let gy = at(left, down) + 2.0 * at(x, down) + at(right, down)
                    - at(left, up)
                    - 2.0 * at(x, up)
                    - at(right, up);
                *slot = (gx * gx + gy * gy).sqrt();
            }
        });
    out
}

/// Fill whatever the mask encloses: everything outside it that cannot be
/// reached from the edge of the frame is inside it.
pub(crate) fn fill_holes(field: &[f32], w: usize, h: usize) -> Vec<f32> {
    let inside = |v: f32| v > 0.5;
    let mut ground = vec![false; w * h];
    let mut queue: Vec<usize> = Vec::new();
    let open = |i: usize, ground: &mut Vec<bool>, queue: &mut Vec<usize>| {
        if !ground[i] && !inside(field[i]) {
            ground[i] = true;
            queue.push(i);
        }
    };
    for x in 0..w {
        open(x, &mut ground, &mut queue);
        open((h - 1) * w + x, &mut ground, &mut queue);
    }
    for y in 0..h {
        open(y * w, &mut ground, &mut queue);
        open(y * w + w - 1, &mut ground, &mut queue);
    }
    while let Some(i) = queue.pop() {
        let (x, y) = (i % w, i / w);
        if x > 0 {
            open(i - 1, &mut ground, &mut queue);
        }
        if x + 1 < w {
            open(i + 1, &mut ground, &mut queue);
        }
        if y > 0 {
            open(i - w, &mut ground, &mut queue);
        }
        if y + 1 < h {
            open(i + w, &mut ground, &mut queue);
        }
    }
    field
        .par_iter()
        .zip(ground.par_iter())
        .map(|(v, out)| if *out { *v } else { v.max(1.0) })
        .collect()
}

/// Stop up the gaps in a mask that are narrower than `reach`, leaving its
/// outside edge where it was.
///
/// By blurring and cutting rather than by spreading and pulling back over a
/// square window: the square leaves square corners, and they show as
/// rectangular blocks wherever a shape has a hole in it. What is left is
/// taken together with the mask it started from, so the edge stays crisp.
pub(crate) fn close_gaps(field: &[f32], w: usize, h: usize, reach: f32) -> Vec<f32> {
    let mut spread = field.to_vec();
    blur_field(&mut spread, w, h, reach * 0.5);
    spread
        .par_iter_mut()
        .zip(field.par_iter())
        .for_each(|(v, f)| {
            let filled = if *v > CLOSE_CUT { 1.0 } else { 0.0 };
            *v = f.max(filled);
        });
    spread
}

/// How much of a neighbourhood must be inside the mask for a gap in it to
/// count as stopped up.
pub(crate) const CLOSE_CUT: f32 = 0.5;

/// [`widest_nearby`] over a plain field.
pub(crate) fn widest_nearby_field(field: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut out = field.to_vec();
    widest_nearby(&mut out, w, h, radius);
    out
}

/// Replace each value with the largest within `radius` of it, across then
/// down.
pub(crate) fn widest_nearby(field: &mut [f32], w: usize, h: usize, radius: usize) {
    if radius == 0 {
        return;
    }
    let scratch = field.to_vec();
    field
        .par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let line = &scratch[y * w..(y + 1) * w];
            for (x, slot) in row.iter_mut().enumerate() {
                let (from, to) = (x.saturating_sub(radius), (x + radius + 1).min(w));
                // Seeded at negative infinity, not zero: a field with negative
                // values in it — an inverted mask, say — would otherwise come
                // back as all zeroes.
                *slot = line[from..to]
                    .iter()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max);
            }
        });
    let scratch = field.to_vec();
    field
        .par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, row)| {
            let (from, to) = (y.saturating_sub(radius), (y + radius + 1).min(h));
            for (x, slot) in row.iter_mut().enumerate() {
                *slot = (from..to)
                    .map(|yy| scratch[yy * w + x])
                    .fold(f32::NEG_INFINITY, f32::max);
            }
        });
}
