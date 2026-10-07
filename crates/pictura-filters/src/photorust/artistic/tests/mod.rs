use super::*;

/// A picture with detail in one end and a featureless grey in the other:
/// the pencil should draw on the one and leave the other as paper.
///
/// The flat half is the colour of the paper (Paper Brightness 25), because
/// the filter draws the whole picture — a flat *coloured* field takes
/// pencil like anything else, and only a field with nothing in it at all,
/// which is what a backing the colour of the sheet is, comes back bare.
pub(super) fn half_detailed() -> Pixmap {
    let mut pm = Pixmap::filled(160, 64, Rgba8::new(128, 128, 128, 255));
    for y in 0..64 {
        for x in 0..64 {
            // Fine stripes — veins for the pencil to follow.
            if x % 3 == 0 {
                pm.set(x, y, Rgba8::new(200, 60, 90, 255));
            }
        }
    }
    pm
}

/// How much colour the pencil left over a band of the picture.
///
/// Measured as colourfulness rather than as distance from the paper: the
/// paper is grey, and anything the pencil drew carries the picture's own
/// colour, so the two are told apart by whether the channels differ — and
/// a wash that happens to average out near the paper's own level is not
/// mistaken for bare paper.
pub(super) fn ink(pm: &Pixmap, xs: std::ops::Range<i32>) -> u32 {
    let columns = xs.len() as u32;
    xs.map(|x| {
        (0..64)
            .map(|y| {
                let p = pm.get(x, y);
                (p.r as i32 - p.g as i32).unsigned_abs()
                    + (p.g as i32 - p.b as i32).unsigned_abs()
                    + (p.r as i32 - p.b as i32).unsigned_abs()
            })
            .sum::<u32>()
    })
    .sum::<u32>()
        / columns
}

/// A green ground with a pink disc on it, which is the shape of the
/// picture Cutout is usually pointed at: two areas, each with a little
/// grain in it for the flattening to rub out.
pub(super) fn disc_on_a_ground() -> Pixmap {
    let mut pm = Pixmap::filled(96, 96, Rgba8::new(40, 90, 30, 255));
    for y in 0..96 {
        for x in 0..96 {
            let (dx, dy) = (x as f32 - 48.0, y as f32 - 48.0);
            let px = if dx * dx + dy * dy < 30.0 * 30.0 {
                Rgba8::new(210, 100, 150, 255)
            } else {
                Rgba8::new(40, 90, 30, 255)
            };
            // Grain, kept well inside a band so that it cannot cut a piece
            // of its own — it is there to be averaged away.
            let grain = ((x * 7 + y * 13) % 5) as u8 * 2;
            pm.set(
                x,
                y,
                Rgba8::new(px.r + grain, px.g + grain, px.b + grain, 255),
            );
        }
    }
    pm
}

/// How many different colours a picture is made of.
pub(super) fn shades(pm: &Pixmap) -> std::collections::HashSet<[u8; 3]> {
    pm.as_bytes()
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2]])
        .collect()
}

/// Two noisy fields with a hard boundary between them: the thing a
/// painting filter has to get right is flattening the fields *without*
/// softening what divides them.
pub(super) fn two_noisy_fields() -> Pixmap {
    let mut pm = Pixmap::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            let base = if x < 32 { 60 } else { 200 };
            let n = ((x * 7 + y * 13) % 5) * 5 - 10;
            let v = (base + n).clamp(0, 255) as u8;
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    pm
}

/// How much a band of the picture jitters from pixel to pixel.
pub(super) fn restlessness(pm: &Pixmap, xs: std::ops::Range<i32>) -> u32 {
    xs.map(|x| {
        (0..63)
            .map(|y| (pm.get(x, y).r as i32 - pm.get(x, y + 1).r as i32).unsigned_abs())
            .sum::<u32>()
    })
    .sum()
}

pub(super) fn pastel(pm: &mut Pixmap, length: u32, detail: u32, relief: u32) {
    use crate::photorust::texture::{Light, Texture};
    rough_pastels(
        pm,
        length,
        detail,
        Texture::Canvas,
        100,
        relief,
        Light::Bottom,
        false,
    );
}

pub(super) fn underpaint(pm: &mut Pixmap, size: u32, coverage: u32, relief: u32) {
    use crate::photorust::texture::{Light, Texture};
    underpainting(
        pm,
        size,
        coverage,
        Texture::Burlap,
        100,
        relief,
        Light::Top,
        false,
    );
}

/// Black and white, which is what the swatches usually are, and a blue
/// tube.
pub(super) const TUBE: Rgba8 = Rgba8::new(0, 0, 255, 255);

pub(super) fn lit(pm: &mut Pixmap, size: i32, brightness: u32) {
    neon_glow(pm, size, brightness, TUBE, Rgba8::BLACK, Rgba8::WHITE);
}

/// The same with a white tube, for the tests that are about where a tone
/// lands rather than what colour the lamp is — a coloured tube pulls every
/// channel about and would be measuring two things at once.
pub(super) fn lit_plainly(pm: &mut Pixmap, size: i32, brightness: u32) {
    neon_glow(
        pm,
        size,
        brightness,
        Rgba8::WHITE,
        Rgba8::BLACK,
        Rgba8::WHITE,
    );
}

/// A ramp from black to white, for asking what a filter does to each end
/// of the tonal range separately.
pub(super) fn ramp() -> Pixmap {
    let mut pm = Pixmap::new(64, 64);
    for y in 0..64 {
        for x in 0..64 {
            let v = (x * 4) as u8;
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    pm
}

/// How far the picture strays from the ramp it was, in levels per pixel,
/// over a band of it.
pub(super) fn scatter(pm: &Pixmap, xs: std::ops::Range<i32>) -> f32 {
    let n = xs.len() as f32 * 64.0;
    xs.map(|x| {
        (0..64)
            .map(|y| (pm.get(x, y).r as i32 - (x * 4)).abs() as f32)
            .sum::<f32>()
    })
    .sum::<f32>()
        / n
}

mod tests_1;
mod tests_2;
