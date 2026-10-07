//! Filter ▸ Pixelate.
//!
//! These throw away the detail of an image and replace it with a pattern of
//! cells, dots or clumps. What they have in common — and what makes them a
//! separate family from the blurs — is that the pattern is anchored to the
//! *canvas* rather than to the picture: it is a screen laid over the image,
//! not something computed from each pixel's neighbourhood.

use crate::photorust::pixmap::{Pixmap, Rgba8};
pub(crate) use crate::MezzotintType;
use rayon::prelude::*;

/// The four screen angles Color Halftone takes, in degrees, in CS6's channel
/// order: cyan, magenta, yellow, black.
pub type ScreenAngles = [f32; 4];

/// CS6's default screen angles (C, M, Y, K), which the halftone tests use.
#[cfg(test)]
pub const DEFAULT_SCREEN_ANGLES: ScreenAngles = [108.0, 162.0, 90.0, 45.0];

/// How different two colours may be and still count as the same patch, summed
/// across the three channels.
///
/// This is the number that makes Facet a *facet* rather than a noise filter.
/// See [`facet`] for why it is needed at all.
pub(crate) const FACET_TOLERANCE: i32 = 8;

/// Filter ▸ Pixelate ▸ Facet.
///
/// Clumps neighbouring pixels of similar colour into flat patches, which is
/// what gives a photograph the look of having been laid down in brush strokes.
///
/// It is done in two passes, and the second one is not an embellishment — it
/// is the filter.
///
/// **First**, each pixel takes the colour of whichever of its nine neighbours
/// is most *typical* of them: the one whose colour is closest to all the
/// others put together. That is a medoid, chosen over a mean or a median
/// because it always returns a colour that was really there. An average
/// invents a new colour halfway between two, which softens the picture instead
/// of clumping it, and a per-channel median can invent one too by taking its
/// red from one neighbour and its green from another. This pass swallows
/// grain and speckle.
///
/// **Second**, patches are grown across the picture: a pixel within
/// [`FACET_TOLERANCE`] of the patch already running to its left or above joins
/// it and takes its colour, and otherwise begins a patch of its own.
///
/// The second pass exists because the first one cannot work on its own, and
/// the reason is worth stating: on a smoothly graded area the most typical of
/// nine values is the one already in the middle, so a medoid — or any other
/// operation that treats the nine symmetrically — hands a gradient straight
/// back untouched. Applied to a photograph it would tidy the grain and leave
/// every smooth petal and every sky exactly as it found them, which is not
/// what the filter is for. Only something that carries a decision from one
/// pixel to the next can lay down a flat patch where the picture is smooth,
/// and that is what growing regions does.
///
/// The cost of that is a direction: patches grow rightwards and downwards
/// from wherever they start. Looking at both the left and the upper neighbour
/// rather than only one keeps the boundaries angular instead of striping the
/// image horizontally.
///
/// Takes no parameters, as in CS6, and is meant to be applied more than once —
/// each pass grows the patches further, which is what Ctrl+F is for.
pub fn facet(pixmap: &mut Pixmap) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    // -- the medoid pass, which is per pixel and so runs in parallel --------
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                // The nine candidates, edge-clamped like the blurs next door.
                let mut window = [Rgba8::TRANSPARENT; 9];
                for (k, slot) in window.iter_mut().enumerate() {
                    let dx = (k % 3) as i32 - 1;
                    let dy = (k / 3) as i32 - 1;
                    *slot = src.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1));
                }

                let mut best = 0usize;
                let mut best_distance = i32::MAX;
                for (k, candidate) in window.iter().enumerate() {
                    let mut total = 0i32;
                    for other in &window {
                        total += difference(*candidate, *other);
                    }
                    if total < best_distance {
                        best_distance = total;
                        best = k;
                    }
                }

                let chosen = window[best];
                let i = x as usize * 4;
                out[i] = chosen.r;
                out[i + 1] = chosen.g;
                out[i + 2] = chosen.b;
                out[i + 3] = chosen.a;
            }
        });

    // -- growing the patches, which has to be in order ---------------------
    //
    // Each pixel is compared against the patches already settled to its left
    // and above, so the pass runs top-left to bottom-right and cannot be split
    // across threads. It is one comparison per pixel, so that costs little.
    let settled = pixmap.clone();
    let mut grown: Vec<Rgba8> = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let here = settled.get(x, y);
            let index = (y * width + x) as usize;

            let left = if x > 0 { Some(grown[index - 1]) } else { None };
            let above = if y > 0 {
                Some(grown[index - width as usize])
            } else {
                None
            };

            // Whichever neighbouring patch this pixel is nearest to, if it is
            // near enough to belong to either.
            let mut best = here;
            let mut best_distance = FACET_TOLERANCE;
            for candidate in [left, above].into_iter().flatten() {
                let distance = difference(here, candidate);
                if distance <= best_distance {
                    best_distance = distance;
                    best = candidate;
                }
            }
            grown.push(best);
        }
    }

    for (out, colour) in pixmap.as_bytes_mut().chunks_exact_mut(4).zip(grown) {
        out[0] = colour.r;
        out[1] = colour.g;
        out[2] = colour.b;
        out[3] = colour.a;
    }
}

/// How the mezzotint threshold is drawn.
///
/// The threshold is uniform across the *whole* range, which makes this a
/// proportional dither: a channel a fifth of the way up is thrown high one
/// time in five. That sounds like it would leave the picture as even confetti
/// with the subject barely showing, and confining the wobble to a narrow band
/// around mid-grey looks like the obvious fix — everything above the band
/// goes solid, everything below stays dark, and only the midtones break up.
///
/// It is the wrong fix, and two things in CS6's own output say so. Its white
/// petals are *not* perfectly solid: they carry a scattering of coloured
/// specks, which a band cannot produce because anything above it is thrown
/// high every time. And its dark ground is not sparsely flecked but densely
/// carpeted, which a band cannot produce either. Both densities are what
/// falls out of the plain proportional draw.
///
/// What made the first attempt look like static was not this at all — it was
/// looking at the result on a canvas fitted to the window. Any mezzotint
/// shrunk with smoothing turns back into the grey it came from.
///
/// **One threshold serves all three channels.** Drawing a separate one per
/// channel is the obvious reading of "each plate is screened", and it is
/// wrong. With independent draws every pixel picks its three answers out of a
/// hat, so all eight corners of the colour cube turn up everywhere: a dark
/// green ground comes back carrying red and blue specks it has no red or blue
/// to justify, and the result is a chromatic riot rather than a print.
///
/// With one threshold the channels cross it **in order of their own
/// strength**, so a pixel can only land on the colours between black and
/// itself. The same dark green ground — 19% red, 28% green, 4% blue — then
/// gives 72% black, 15% yellow, 9% green and 4% white, and no red or blue at
/// all, which is what CS6 produces. A pale pink petal gives 59% white, 25%
/// red, 10% magenta and 6% black: white with red pepper, again as CS6 has it.
///
/// It is tempting to think one threshold could only ever give black and
/// white. That is true of a *grey* pixel, whose three channels cross together
/// — and it is exactly why the colour appears where the picture has colour
/// and nowhere else.
#[inline]
pub(crate) fn dither_threshold(column: i32, band: i32) -> f32 {
    jitter(column, band, 1)
}

/// How steeply the dot patterns' grain is spread over the threshold range;
/// see `mezzotint`. Measured off CS6's Coarse Dots over
/// `samples/horse-3.jpg`, registered against the source and binned by its
/// tone: the share thrown black runs 83%, 76%, 61%, 43%, 19%, 10%, 5% and
/// 3% from the darkest eighth of the range to the lightest. This gives 95%,
/// 81%, 63%, 43%, 26%, 16%, 5% and 0%.
pub(crate) const MEZZOTINT_GRAIN_CONTRAST: f32 = 1.35;

/// How the line and stroke patterns' thresholds are bent away from mid-grey;
/// see `MezzotintType::streaks`. Above 1 the thresholds bunch about the middle and
/// straggle out towards the ends, which is what CS6's tone curve does: its
/// Short Lines throws the tenth of the range either side of mid-grey 42% and
/// 64% high — steeper than a plain bell of noise — yet still flecks 6% of
/// the darkest tenth. The strokes fit best at the same value.
pub(crate) const MEZZOTINT_LINE_SHAPE: f32 = 1.3;

/// The shape of a line or stroke pattern's grain; see
/// [`MezzotintType::streaks`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct Streaks {
    /// How far the grain reaches along a row, in pixels.
    pub(crate) reach: f32,
    /// How much of the grain reaches four times as far again.
    pub(crate) tail: f32,
    /// How far the grain is blurred down the column, in pixels. Nothing for
    /// the lines, which are a pixel tall.
    pub(crate) height: f32,
    /// How far the thresholds spread either side of mid-grey.
    pub(crate) spread: f32,
}

impl MezzotintType {
    /// How far the dot patterns' grain is blurred, in pixels, or `None` for
    /// Fine Dots and the line and stroke patterns.
    ///
    /// CS6's Medium, Grainy and Coarse Dots are not squares of 2, 3 and 4
    /// pixels, however the names read: its Coarse Dots over
    /// `samples/horse-3.jpg` is a scatter of irregular specks and worm-like
    /// blobs of every shape, with the horse's outline and muscles still
    /// drawn through them. That is a threshold that is itself grain — white
    /// noise blurred into clumps this big — so a dot is wherever the clump
    /// dips under the picture's tone, and takes the clump's shape; and each
    /// pixel is decided on its own tone, so detail survives. A square cell
    /// thrown one way or the other as a whole is pixel art instead.
    fn grain(self) -> Option<f32> {
        match self {
            MezzotintType::MediumDots => Some(0.55),
            MezzotintType::GrainyDots => Some(0.8),
            MezzotintType::CoarseDots => Some(1.05),
            _ => None,
        }
    }

    /// The grain of the line and stroke patterns, or `None` for the dots.
    ///
    /// CS6's lines are not dashes of a set length. Over `samples/horse-3.jpg`
    /// they run from a pixel or two to a few hundred, one pixel tall, and the
    /// horse's highlights stay drawn through them. That is a threshold made of
    /// noise smoothed along the row only: a streak is wherever it stays under
    /// the picture's tone, so its length is random and grows the further the
    /// tone is from mid-grey. A cell of fixed width thrown as a whole instead
    /// gives every dash the same length and turns the picture into blocks.
    ///
    /// The longer lines owe their long streaks more to the **spread** than to
    /// the reach. CS6 grows harsher from Short to Long: the tenth of the range
    /// just above black is thrown high 8% of the time by Short Lines, 2% by
    /// Medium and never by Long, so a light sky goes nearly solid and its few
    /// dark streaks run on.
    ///
    /// The strokes are the same thing, thicker and more ragged. Each row of
    /// CS6's agrees with the next about as well as a stroke two or three
    /// pixels tall would, and hardly at all with the one after — and not in
    /// fixed pairs, so it is a blur down the column rather than a taller cell.
    /// Along the row the pattern forgets itself at two rates, most of it
    /// within a pixel or two and the rest far later, which is the **tail**:
    /// it is what leaves the odd stroke running on past the others. Medium
    /// Strokes is softer than its neighbours either side — CS6 flecks a
    /// twentieth of the darkest tones with it — and that is in the
    /// measurement, not a slip.
    ///
    /// Every number was fitted to CS6's output, registered against the
    /// source, on its tone curve and on how quickly the pattern forgets
    /// itself along a row and down a column.
    fn streaks(self) -> Option<Streaks> {
        let streaks = |reach, tail, height, spread| {
            Some(Streaks {
                reach,
                tail,
                height,
                spread,
            })
        };
        match self {
            MezzotintType::ShortLines => streaks(3.0, 0.0, 0.0, 0.21),
            MezzotintType::MediumLines => streaks(6.0, 0.0, 0.0, 0.13),
            MezzotintType::LongLines => streaks(9.0, 0.0, 0.0, 0.09),
            MezzotintType::ShortStrokes => streaks(1.5, 0.5, 0.7, 0.14),
            MezzotintType::MediumStrokes => streaks(2.0, 0.7, 0.7, 0.22),
            MezzotintType::LongStrokes => streaks(5.0, 0.7, 0.7, 0.14),
            _ => None,
        }
    }
}

/// Filter ▸ Pixelate ▸ Mezzotint.
///
/// Every channel is thrown to one end or the other — nothing in between — so
/// the picture comes back in nothing but the eight corners of the colour cube:
/// black, white, and the fully saturated primaries and secondaries. That is
/// why a photograph turns into red, green and yellow confetti rather than a
/// gritty version of itself.
///
/// The decision is a **random dither**: a channel three quarters of the way up
/// has three chances in four of being thrown to the top, so an area keeps its
/// tone on average even though no single pixel does. Compare the halftone next
/// door, which keeps tone by growing a dot; this keeps it by weighting a coin.
/// See [`dither_threshold`] for why it is drawn across the whole range rather
/// than a band in the middle, which is the plausible-looking alternative.
///
/// All three channels are decided by the same draw, which is what keeps the
/// palette to the colours a region actually contains. See
/// [`dither_threshold`].
pub fn mezzotint(pixmap: &mut Pixmap, kind: MezzotintType) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    // The dot patterns' threshold: white noise blurred into clumps, then
    // spread over the range by a logistic curve. Spread evenly — a slope of
    // 1.7 is close to blurred noise's own cumulative distribution — the
    // dither would keep every area's tone exactly; CS6's grain is harsher
    // than that, so the curve is shallower and the thresholds bunch about
    // the middle. See [`MEZZOTINT_GRAIN_CONTRAST`].
    let grain: Option<Vec<f32>> = kind.grain().map(|sigma| {
        let (w, h) = (width as usize, height as usize);
        let mut field: Vec<f32> = (0..w * h)
            .into_par_iter()
            .map(|i| jitter((i % w) as i32, (i / w) as i32, 3) - 0.5)
            .collect();
        crate::photorust::artistic::blur_field(&mut field, w, h, sigma);
        crate::photorust::brush_strokes::unit_spread(&mut field);
        field
            .par_iter_mut()
            .for_each(|v| *v = 1.0 / (1.0 + (-MEZZOTINT_GRAIN_CONTRAST * *v).exp()));
        field
    });
    let field = grain.or_else(|| {
        kind.streaks()
            .map(|streaks| streak_field(width, height, streaks))
    });
    let field = field.as_deref();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let i = x as usize * 4;
                let own = src.get(x, y);
                // Each pixel is decided on its own tone, so the picture's
                // detail is drawn through the pattern rather than lost under
                // it.
                let threshold = match field {
                    Some(field) => field[row * width as usize + x as usize],
                    None => dither_threshold(x, y),
                };
                for (c, level) in [own.r, own.g, own.b].into_iter().enumerate() {
                    out[i + c] = if level as f32 / 255.0 > threshold {
                        255
                    } else {
                        0
                    };
                }
                // Transparency is left where it was: this is a printing
                // process, not an eraser.
                out[i + 3] = own.a;
            }
        });
}

/// The line and stroke patterns' threshold: white noise smoothed along each
/// row, blurred a little down the column for the strokes, then spread about
/// mid-grey. See [`MezzotintType::streaks`].
///
/// Not a GPU candidate: each row is a pair of sequential recurrences, and the
/// whole filter is a cheap single pass over the image.
pub(crate) fn streak_field(width: i32, height: i32, streaks: Streaks) -> Vec<f32> {
    let mut field = row_smoothed(width, height, streaks.reach, 5);
    if streaks.tail > 0.0 {
        let tail = row_smoothed(width, height, streaks.reach * 4.0, 6);
        let (near, far) = ((1.0 - streaks.tail).sqrt(), streaks.tail.sqrt());
        field
            .par_iter_mut()
            .zip(tail.par_iter())
            .for_each(|(v, t)| *v = near * *v + far * t);
    }
    if streaks.height > 0.0 {
        blur_columns(&mut field, width as usize, height as usize, streaks.height);
    }
    crate::photorust::brush_strokes::unit_spread(&mut field);
    field
        .par_iter_mut()
        .for_each(|v| *v = 0.5 + streaks.spread * v.signum() * v.abs().powf(MEZZOTINT_LINE_SHAPE));
    field
}

/// White noise smoothed along each row by an exponential run forwards and
/// back, spread to unit size.
///
/// The exponential rather than a Gaussian because CS6's streaks have a long
/// tail — the odd one runs on far past the rest — and a Gaussian cuts its
/// grain off sharply.
pub(crate) fn row_smoothed(width: i32, height: i32, reach: f32, salt: u32) -> Vec<f32> {
    let w = width as usize;
    // Noise is drawn past both ends, so a streak reaching the edge is the
    // same as one anywhere else rather than dying away into it.
    let pad = (reach * 6.0).ceil() as i32;
    let keep = (-1.0 / reach).exp();
    let mut field = vec![0.0f32; w * height as usize];
    field
        .par_chunks_exact_mut(w)
        .enumerate()
        .for_each(|(y, out)| {
            let mut run: Vec<f32> = Vec::with_capacity(w + 2 * pad as usize);
            let mut state = 0.0;
            for x in -pad..width + pad {
                state = keep * state + (1.0 - keep) * (jitter(x, y as i32, salt) - 0.5);
                run.push(state);
            }
            let mut state = 0.0;
            for v in run.iter_mut().rev() {
                state = keep * state + (1.0 - keep) * *v;
                *v = state;
            }
            out.copy_from_slice(&run[pad as usize..pad as usize + w]);
        });
    crate::photorust::brush_strokes::unit_spread(&mut field);
    field
}

/// A Gaussian blur of a floating-point field down its columns only, with the
/// edge rows standing in for what is past them.
pub(crate) fn blur_columns(field: &mut [f32], width: usize, height: usize, sigma: f32) {
    let taps = (sigma * 3.0).ceil() as i32;
    let kernel = crate::photorust::convolve::gaussian_kernel_1d(sigma, taps);
    let source = field.to_vec();
    field
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, out)| {
            out.fill(0.0);
            for (i, weight) in kernel.iter().enumerate() {
                let from = (y as i32 + i as i32 - taps).clamp(0, height as i32 - 1) as usize;
                for (v, s) in out
                    .iter_mut()
                    .zip(&source[from * width..(from + 1) * width])
                {
                    *v += weight * s;
                }
            }
        });
}

/// How far apart two colours are, summed across the three channels.
#[inline]
pub(crate) fn difference(a: Rgba8, b: Rgba8) -> i32 {
    (a.r as i32 - b.r as i32).abs()
        + (a.g as i32 - b.g as i32).abs()
        + (a.b as i32 - b.b as i32).abs()
}

/// How far Fragment throws each of its four copies, in pixels. CS6's Fragment
/// takes no settings, so this is the whole of it.
pub(crate) const FRAGMENT_OFFSET: i32 = 4;

/// Filter ▸ Pixelate ▸ Fragment.
///
/// Four copies of the picture, shifted away from one another and averaged —
/// the look of a photograph taken through a shaking lens. Takes no parameters,
/// as in CS6.
///
/// The copies go to the four corners of a square rather than to the four
/// compass points. A diamond of offsets averages out to something very close
/// to a plain blur; a square keeps the doubled edges that make it read as four
/// exposures rather than one soft one.
pub fn fragment(pixmap: &mut Pixmap) {
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // Premultiplied, so a copy that lands half off a soft edge does not drag
    // the colour of invisible pixels into visible ones.
    pixmap.premultiply();
    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let mut total = [0u32; 4];
                for dy in [-FRAGMENT_OFFSET, FRAGMENT_OFFSET] {
                    for dx in [-FRAGMENT_OFFSET, FRAGMENT_OFFSET] {
                        let p =
                            src.get((x + dx).clamp(0, width - 1), (y + dy).clamp(0, height - 1));
                        total[0] += p.r as u32;
                        total[1] += p.g as u32;
                        total[2] += p.b as u32;
                        total[3] += p.a as u32;
                    }
                }
                let i = x as usize * 4;
                for c in 0..4 {
                    out[i + c] = (total[c] / 4) as u8;
                }
            }
        });

    pixmap.unpremultiply();
}

/// Filter ▸ Pixelate ▸ Pointillize.
///
/// The picture is rebuilt out of scattered round dabs, each carrying the
/// average colour of what it covers, on a ground of the **background colour**
/// — that is what Photoshop's description means by "uses the background color
/// as a canvas area between the dots", and it is the part that makes the
/// result look painted rather than merely speckled. The gaps are not left as
/// they were and they are not black; they are bare canvas.
///
/// The dabs sit on a jittered lattice like Crystallize's seeds, but here they
/// do not tile the plane: each is a disc a little smaller than its cell, and
/// the size varies from one to the next. Equal discs on a regular grid read as
/// a pattern; unequal ones scattered read as brush work, and the gaps between
/// them are what the ground shows through.
pub fn pointillize(pixmap: &mut Pixmap, cell_size: u32, background: Rgba8) {
    let cell = cell_size.max(1) as f32;
    if pixmap.is_empty() || cell_size <= 1 {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // What each dab is loaded with. Only a three-by-three average, not the
    // whole cell: averaging a dab's own area is the obvious thing to do and it
    // flattens the picture, because neighbouring dabs then carry nearly the
    // same colour and the result is a smooth field of dots rather than a
    // painting. Taking the colour from near the middle keeps the variation
    // that makes it look mixed on a palette; the small average is only there
    // so a single stuck pixel cannot decide a whole dab.
    let mut averaged = pixmap.clone();
    crate::photorust::convolve::box_blur(&mut averaged, 1);

    let blurred = &averaged;
    let source = pixmap.clone();
    let src = &source;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as f32 + 0.5;
            for x in 0..width {
                let px = x as f32 + 0.5;
                let base_i = (px / cell).floor() as i32;
                let base_j = (y / cell).floor() as i32;

                // Nearest first, so that where two dabs overlap the one whose
                // middle is closer is the one on top — the later stroke.
                let mut best = f32::MAX;
                let mut colour: Option<Rgba8> = None;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let (i, j) = (base_i + di, base_j + dj);
                        let cx = (i as f32 + jitter(i, j, 1)) * cell;
                        let cy = (j as f32 + jitter(i, j, 2)) * cell;
                        // Between a little over half a cell and four fifths of
                        // one. Big enough that the dabs crowd together and
                        // touch — about a quarter of the ground still shows
                        // through, which is what CS6 leaves — and varied
                        // enough that they do not read as a pattern.
                        let radius = cell * (0.55 + jitter(i, j, 3) * 0.25);

                        let distance = (cx - px) * (cx - px) + (cy - y) * (cy - y);
                        if distance <= radius * radius && distance < best {
                            best = distance;
                            colour = Some(blurred.get(
                                (cx as i32).clamp(0, width - 1),
                                (cy as i32).clamp(0, height - 1),
                            ));
                        }
                    }
                }

                let i = x as usize * 4;
                let paint = colour.unwrap_or(background);
                out[i] = paint.r;
                out[i + 1] = paint.g;
                out[i + 2] = paint.b;
                // Bare canvas is as opaque as the ground colour says; a dab
                // carries whatever transparency it picked up.
                out[i + 3] = match colour {
                    Some(dab) => dab.a,
                    None => {
                        // ...but nothing is painted outside the layer either.
                        if src.get(x, row as i32).a == 0 {
                            0
                        } else {
                            background.a
                        }
                    }
                };
            }
        });
}

/// Filter ▸ Pixelate ▸ Mosaic.
///
/// The picture is ruled into squares and each is filled with the average of
/// what was under it — the plainest of this family, and the one the others are
/// variations on. Crystallize is this with the lattice points nudged off their
/// grid; this is what is left when they are not.
///
/// `cell_size` is the side of a square in pixels, as CS6's slider is. The grid
/// is anchored to the layer's own origin rather than to the middle, so the
/// tiles do not shift about when the filter is re-applied at a different size.
pub fn mosaic(pixmap: &mut Pixmap, cell_size: u32) {
    let cell = cell_size.max(1) as i32;
    if pixmap.is_empty() || cell <= 1 {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let cols = (width + cell - 1) / cell;
    let rows = (height + cell - 1) / cell;

    // Premultiplied, so that a square straddling a soft edge does not average
    // the colour of invisible pixels into the visible ones.
    pixmap.premultiply();

    // One average per square. Rows of squares read their own band of the
    // picture and nothing else, so they can be worked out side by side.
    let source = pixmap.clone();
    let src = &source;
    let mut tiles = vec![[0u8; 4]; (cols * rows) as usize];
    tiles
        .par_chunks_exact_mut(cols as usize)
        .enumerate()
        .for_each(|(row, band)| {
            let top = row as i32 * cell;
            let bottom = (top + cell).min(height);
            for (col, tile) in band.iter_mut().enumerate() {
                let left = col as i32 * cell;
                let right = (left + cell).min(width);

                let mut total = [0u32; 4];
                let mut count = 0u32;
                for y in top..bottom {
                    let line = src.row(y as u32);
                    for x in left..right {
                        let i = x as usize * 4;
                        for c in 0..4 {
                            total[c] += line[i + c] as u32;
                        }
                        count += 1;
                    }
                }
                // The squares at the right and bottom edges are cut short, so
                // they average what is there rather than what would have been.
                let n = count.max(1);
                for c in 0..4 {
                    // Kooka: rounded to the nearest, the exact block mean the
                    // Mosaic oracle requires (photorust truncates).
                    tile[c] = ((total[c] + n / 2) / n) as u8;
                }
            }
        });

    let tiles = &tiles;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(y, out)| {
            let band = (y as i32 / cell) * cols;
            for x in 0..width as usize {
                let tile = tiles[(band + x as i32 / cell) as usize];
                let i = x * 4;
                out[i..i + 4].copy_from_slice(&tile);
            }
        });

    pixmap.unpremultiply();
}

/// A deterministic hash in 0..1, for jittering the crystal lattice.
///
/// Seeded from the cell's coordinates rather than from a running random
/// source, for the reason every generated pattern in this engine is: undo and
/// redo replay the filter, and a crystal pattern that came out differently
/// each time could not be undone.
pub(crate) fn jitter(i: i32, j: i32, salt: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(0x9E3779B1)
        ^ (j as u32).wrapping_mul(0x85EBCA77)
        ^ salt.wrapping_mul(0xC2B2AE35)
        ^ crate::photorust::seed().wrapping_mul(0x27d4_eb2d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2545F491);
    h ^= h >> 13;
    (h % 10007) as f32 / 10007.0
}

/// Filter ▸ Pixelate ▸ Crystallize.
///
/// The picture is broken into irregular polygonal cells, each filled with the
/// average of what was underneath it. The cells are the regions nearest to a
/// scatter of seed points — a Voronoi diagram — and the seeds are a square
/// lattice at `cell_size` spacing with each one nudged somewhere inside its
/// own square. That nudge is the whole difference between this and Mosaic: an
/// unjittered lattice gives back plain square tiles.
///
/// `cell_size` is the lattice spacing in pixels, as CS6's slider is.
pub fn crystallize(pixmap: &mut Pixmap, cell_size: u32) {
    let cell = cell_size.max(1) as f32;
    if pixmap.is_empty() || cell_size <= 1 {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // Enough seeds to cover the canvas with one to spare on each side, so a
    // pixel at the edge still has neighbours to be nearer to.
    let cols = (width as f32 / cell).ceil() as i32 + 2;
    let rows = (height as f32 / cell).ceil() as i32 + 2;
    let seed_at = |i: i32, j: i32| -> (f32, f32) {
        (
            (i as f32 + jitter(i, j, 1)) * cell,
            (j as f32 + jitter(i, j, 2)) * cell,
        )
    };
    let index_of = |i: i32, j: i32| -> usize {
        ((j + 1).clamp(0, rows - 1) * cols + (i + 1).clamp(0, cols - 1)) as usize
    };

    // Which cell each pixel belongs to. Worked out once and kept, rather than
    // twice: the answer is needed both to add a pixel into its cell's average
    // and again to paint that average back over it.
    let mut owner = vec![0u32; (width * height) as usize];
    owner
        .par_chunks_exact_mut(width as usize)
        .enumerate()
        .for_each(|(row, line)| {
            let y = row as f32 + 0.5;
            for (x, slot) in line.iter_mut().enumerate() {
                let px = x as f32 + 0.5;
                let base_i = (px / cell).floor() as i32;
                let base_j = (y / cell).floor() as i32;

                // A seed never leaves its own square, so the nearest one is
                // always in the ring of squares around this pixel's.
                let mut best = f32::MAX;
                let mut best_index = 0usize;
                for dj in -1..=1 {
                    for di in -1..=1 {
                        let (i, j) = (base_i + di, base_j + dj);
                        let (sx, sy) = seed_at(i, j);
                        let distance = (sx - px) * (sx - px) + (sy - y) * (sy - y);
                        if distance < best {
                            best = distance;
                            best_index = index_of(i, j);
                        }
                    }
                }
                *slot = best_index as u32;
            }
        });

    // What each cell averages out to. Accumulated in one sweep rather than in
    // parallel: a per-thread tally of every cell would cost more memory than
    // the image at the smallest cell size.
    let count = (cols * rows) as usize;
    let mut totals = vec![[0u64; 4]; count];
    let mut counts = vec![0u32; count];
    for (i, chunk) in pixmap.as_bytes().chunks_exact(4).enumerate() {
        let cell = owner[i] as usize;
        for c in 0..4 {
            totals[cell][c] += chunk[c] as u64;
        }
        counts[cell] += 1;
    }

    let totals = &totals;
    let counts = &counts;
    let owner = &owner;
    let stride = pixmap.stride();
    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            for x in 0..width as usize {
                let cell = owner[row * width as usize + x] as usize;
                let n = counts[cell].max(1) as u64;
                let i = x * 4;
                for c in 0..4 {
                    out[i + c] = (totals[cell][c] / n) as u8;
                }
            }
        });
}

/// Filter ▸ Pixelate ▸ Color Halftone.
///
/// Reproduces what a printing press does: the picture is separated into cyan,
/// magenta, yellow and black, and each is redrawn as a grid of dots whose size
/// follows how much of that ink the area wants. The four grids are set at
/// different angles so their dots fall between one another rather than on top.
///
/// `max_radius` is the size of a dot at full ink, in pixels, and sets the
/// coarseness of the whole thing — the grid spacing follows from it.
pub fn color_halftone(pixmap: &mut Pixmap, max_radius: f32, angles: ScreenAngles) {
    let max_radius = max_radius.max(1.0);
    if pixmap.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;

    // Spacing between dots. A dot of this radius on this grid covers rather
    // more than its own cell, so the darkest areas close up into solid ink
    // instead of stopping at a lattice of touching circles.
    let spacing = max_radius * 2.0;

    // What each dot is worth is the ink over the whole cell it stands for, not
    // the one pixel at its middle — a single pixel would let a speck of noise
    // decide the size of a dot covering a hundred of them. A box blur the
    // width of a cell is that average, already computed for every position.
    let mut averaged = pixmap.clone();
    crate::photorust::convolve::box_blur(&mut averaged, max_radius.round().max(1.0) as u32);

    // Each screen's rotation, worked out once.
    let screens: Vec<(f32, f32)> = angles
        .iter()
        .map(|degrees| {
            let (sin, cos) = degrees.to_radians().sin_cos();
            (sin, cos)
        })
        .collect();

    let source = pixmap.clone();
    let blurred = &averaged;
    let original = &source;
    let stride = pixmap.stride();

    pixmap
        .as_bytes_mut()
        .par_chunks_exact_mut(stride)
        .enumerate()
        .for_each(|(row, out)| {
            let y = row as i32;
            for x in 0..width {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let mut covered = [false; 4];

                for (channel, &(sin, cos)) in screens.iter().enumerate() {
                    // Into the screen's own frame, where its dots sit on a
                    // plain square lattice.
                    let u = px * cos + py * sin;
                    let v = -px * sin + py * cos;
                    let cell_u = (u / spacing).round();
                    let cell_v = (v / spacing).round();

                    // A dot can reach past its own cell, so the neighbours
                    // have to be asked too or the overlaps that make dark
                    // areas solid would be clipped away.
                    'dots: for du in -1..=1 {
                        for dv in -1..=1 {
                            let centre_u = (cell_u + du as f32) * spacing;
                            let centre_v = (cell_v + dv as f32) * spacing;
                            // Back out of the screen's frame to find which
                            // part of the picture this dot stands for.
                            let cx = centre_u * cos - centre_v * sin;
                            let cy = centre_u * sin + centre_v * cos;

                            let ink = ink_at(blurred, cx, cy, channel, width, height);
                            if ink <= 0.0 {
                                continue;
                            }
                            // Area proportional to the ink, so that half the
                            // ink covers half the paper. Radius is therefore
                            // the square root of it.
                            let radius = spacing * (ink / std::f32::consts::PI).sqrt();
                            let dx = px - cx;
                            let dy = py - cy;
                            if dx * dx + dy * dy <= radius * radius {
                                covered[channel] = true;
                                break 'dots;
                            }
                        }
                    }
                }

                // Ink on paper: each of the three colours takes its own
                // channel out of the white, and black takes all of it.
                let black = if covered[3] { 0.0 } else { 1.0 };
                let value = |c: bool| if c { 0.0 } else { 255.0 * black };
                let i = x as usize * 4;
                out[i] = value(covered[0]) as u8;
                out[i + 1] = value(covered[1]) as u8;
                out[i + 2] = value(covered[2]) as u8;
                // Whatever was see-through stays see-through: a screen is
                // printed on the picture, not on the space around it.
                out[i + 3] = original.get(x, y).a;
            }
        });
}

/// How much of one ink an area wants, from 0 to 1.
///
/// A plain separation — no press profile, no colour management — since what
/// matters here is the pattern of dots. The one judgement in it is how much
/// black to use: the naive answer, black wherever all three colours are
/// wanted, lays down so much of it that a dark area closes up into a solid
/// sheet and the screen disappears. A press does not do that either. This
/// uses a **skeleton black**, squared so that it stays out of the midtones
/// and only comes up in the deepest shadows, leaving the colour to the other
/// three plates where the picture still has some.
pub(crate) fn ink_at(src: &Pixmap, x: f32, y: f32, channel: usize, width: i32, height: i32) -> f32 {
    let px = src.get(
        (x as i32).clamp(0, width - 1),
        (y as i32).clamp(0, height - 1),
    );
    let cyan = 1.0 - px.r as f32 / 255.0;
    let magenta = 1.0 - px.g as f32 / 255.0;
    let yellow = 1.0 - px.b as f32 / 255.0;

    let common = cyan.min(magenta).min(yellow);
    let black = common * common;
    match channel {
        0 => cyan - black,
        1 => magenta - black,
        2 => yellow - black,
        _ => black,
    }
    .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests;
