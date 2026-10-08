use super::*;

use crate::photorust::pixmap::Rect;

pub(super) const BLACK: Rgba8 = Rgba8::BLACK;

pub(super) const WHITE: Rgba8 = Rgba8::WHITE;

/// A raised bar on dark ground: a ridge with two faces, one of which the
/// light strikes and the other of which it cannot reach. A single step
/// would not do — it has one slope, so it carves one line, and says
/// nothing about which face takes which swatch.
pub(super) const RIDGE: std::ops::Range<i32> = 28..36;

pub(super) fn step() -> Pixmap {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(40, 40, 40, 255));
    pm.fill_rect(
        Rect::new(RIDGE.start, 0, RIDGE.len() as u32, 64),
        Rgba8::new(220, 220, 220, 255),
    );
    pm
}

/// Three flat bands — dark, mid and light — so each stick has somewhere
/// it must reach and somewhere it must not.
pub(super) fn bands() -> Pixmap {
    let mut pm = Pixmap::filled(120, 64, Rgba8::new(128, 128, 128, 255));
    pm.fill_rect(Rect::new(0, 0, 40, 64), Rgba8::new(16, 16, 16, 255));
    pm.fill_rect(Rect::new(80, 0, 40, 64), Rgba8::new(252, 252, 252, 255));
    pm
}

/// The mean colour of a band's middle, clear of the smudge at its edges.
pub(super) fn band_mean(pm: &Pixmap, from: i32, to: i32) -> [f32; 3] {
    let mut total = [0.0f32; 3];
    let mut count = 0.0;
    for y in 8..56 {
        for x in from..to {
            let p = pm.get(x, y);
            total[0] += p.r as f32;
            total[1] += p.g as f32;
            total[2] += p.b as f32;
            count += 1.0;
        }
    }
    [total[0] / count, total[1] / count, total[2] / count]
}

/// A dark subject on a light ground, with a fine ripple over the ground:
/// something for the masses to fill, something for the outlines to trace,
/// and something faint that only a high Detail should pick up.
pub(super) fn sketchable() -> Pixmap {
    let mut pm = Pixmap::new(128, 128);
    for y in 0..128i32 {
        for x in 0..128i32 {
            // Fine and strong: fine enough that Detail 0's simplifying
            // blur erases it, strong enough that Detail 5 draws it.
            let ripple = (((x + y) % 4 < 2) as i32) * 40;
            let v = (214 + ripple).clamp(0, 255) as u8;
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    // Tone 0.219, the same as the subject the sliders were matched on.
    pm.fill_rect(Rect::new(32, 32, 64, 64), Rgba8::new(67, 52, 47, 255));
    pm
}

/// How much of the sheet the stick covered.
pub(super) fn covered(pm: &Pixmap) -> usize {
    pm.as_bytes()
        .chunks_exact(4)
        .filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) < 384)
        .count()
}

/// What share of a square was covered by the stick.
pub(super) fn covered_in(pm: &Pixmap, from: i32, to: i32) -> f32 {
    let mut covered = 0.0;
    let mut total = 0.0;
    for y in from..to {
        for x in from..to {
            total += 1.0;
            if pm.get(x, y).r < 128 {
                covered += 1.0;
            }
        }
    }
    covered / total
}

/// The mean brightness of a whole sheet.
pub(super) fn sheet_mean(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4).map(|p| p[0] as f32).sum::<f32>() / (b.len() / 4) as f32
}

/// A smooth left-to-right ramp through the whole tonal range.
pub(super) fn ramp() -> Pixmap {
    let mut pm = Pixmap::new(256, 64);
    for y in 0..64i32 {
        for x in 0..256i32 {
            let v = x as u8;
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    pm
}

/// How many times a scanline turns around.
pub(super) fn turns(pm: &Pixmap, y: i32) -> usize {
    let row: Vec<i32> = (0..pm.width() as i32)
        .map(|x| pm.get(x, y).r as i32)
        .collect();
    let mut turns = 0;
    let mut rising: Option<bool> = None;
    for pair in row.windows(2) {
        if pair[1] == pair[0] {
            continue;
        }
        let now = pair[1] > pair[0];
        if rising == Some(!now) {
            turns += 1;
        }
        rising = Some(now);
    }
    turns
}

/// Conté Crayon over a flat sheet of one tone, with everything but the
/// two levels left at CS6's defaults.
pub(super) fn crayoned(tone: u8, fore: u32, back: u32) -> Pixmap {
    let mut pm = Pixmap::filled(128, 128, Rgba8::new(tone, tone, tone, 255));
    conte_crayon(
        &mut pm,
        fore,
        back,
        Texture::Canvas,
        100,
        0,
        Light::Top,
        false,
        BLACK,
        WHITE,
    );
    pm
}

/// What share of a sheet the crayon covered.
pub(super) fn crayon_cover(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4).filter(|p| p[0] < 128).count() as f32 / (b.len() / 4) as f32
}

/// What share of a sheet came back a grey rather than either swatch.
pub(super) fn crayon_greys(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4)
        .filter(|p| (40..=215).contains(&p[0]))
        .count() as f32
        / (b.len() / 4) as f32
}

/// The two levels pull against each other: more foreground carries the
/// crayon further up the tones, more background brings the paper further
/// down.
///
/// Read as a mean tone. Counting pixels past a cutoff says nothing here —
/// a midtone settles near that cutoff by definition, so the count answers
/// noise rather than the sliders.
pub(super) fn crayon_mean(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4).map(|p| p[0] as f32).sum::<f32>() / (b.len() / 4) as f32
}

/// A flat mid-grey sheet, which the screen has half its range to ink.
pub(super) fn pen_sheet() -> Pixmap {
    Pixmap::filled(160, 160, Rgba8::new(128, 128, 128, 255))
}

/// Whether a pixel came back as ink rather than as paper.
pub(super) fn inked(pm: &Pixmap, x: i32, y: i32) -> bool {
    pm.get(x, y).r < 128
}

/// What share of a sheet the pen covered.
pub(super) fn pen_cover(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4).filter(|p| p[0] < 128).count() as f32 / (b.len() / 4) as f32
}

/// How often two neighbouring pixels, a step apart, are both ink or both
/// paper. Lines running one way agree along them and alternate across.
pub(super) fn agreement(pm: &Pixmap, dx: i32, dy: i32) -> f32 {
    let (mut same, mut total) = (0.0f32, 0.0f32);
    for y in 0..160i32 {
        for x in 0..160i32 {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= 160 || ny >= 160 {
                continue;
            }
            total += 1.0;
            if inked(pm, x, y) == inked(pm, nx, ny) {
                same += 1.0;
            }
        }
    }
    same / total.max(1.0)
}

/// A flat mid-grey sheet, where the screen has half the range to ink.
pub(super) fn screen_sheet() -> Pixmap {
    Pixmap::filled(64, 64, Rgba8::new(128, 128, 128, 255))
}

/// How many pixels came back a grey rather than one of the two swatches.
pub(super) fn screen_greys(pm: &Pixmap) -> usize {
    pm.as_bytes()
        .chunks_exact(4)
        .filter(|p| (20..=235).contains(&p[0]))
        .count()
}

pub(super) fn mean_tone(pm: &Pixmap) -> f32 {
    let bytes = pm.as_bytes();
    bytes.chunks_exact(4).map(|p| p[0] as f32).sum::<f32>() / (bytes.len() / 4) as f32
}

pub(super) fn spread(pm: &Pixmap) -> f32 {
    let mean = mean_tone(pm);
    let bytes = pm.as_bytes();
    (bytes
        .chunks_exact(4)
        .map(|p| (p[0] as f32 - mean).powi(2))
        .sum::<f32>()
        / (bytes.len() / 4) as f32)
        .sqrt()
}

pub(super) fn inked_share(pm: &Pixmap) -> f32 {
    let bytes = pm.as_bytes();
    bytes.chunks_exact(4).filter(|p| p[0] < 128).count() as f32 / (bytes.len() / 4) as f32
}

mod tests_1;
mod tests_2;
