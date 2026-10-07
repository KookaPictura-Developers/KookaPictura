use super::*;
use crate::photorust::pixmap::{Rect, Rgba8};

fn step() -> Pixmap {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(60, 60, 60, 255));
    pm.fill_rect(Rect::new(32, 0, 32, 64), Rgba8::new(180, 180, 180, 255));
    pm
}

/// High brightness chalks the edge, low brightness inks it, and a flat
/// field far from the edge is left as it was either way.
#[test]
fn brightness_decides_chalk_or_ink() {
    let mut chalk = step();
    accented_edges(&mut chalk, 2, 50, 3);
    let mut ink = step();
    accented_edges(&mut ink, 2, 0, 3);
    assert!(
        chalk.get(31, 32).r > 150,
        "no chalk: {}",
        chalk.get(31, 32).r
    );
    assert!(ink.get(32, 32).r < 80, "no ink: {}", ink.get(32, 32).r);
    assert_eq!(chalk.get(4, 32).r, 60);
    assert_eq!(ink.get(60, 32).r, 180);
}

/// A wider edge reaches further from the boundary.
#[test]
fn edge_width_widens_the_accent() {
    let reach = |width| {
        let mut pm = step();
        accented_edges(&mut pm, width, 50, 3);
        pm.get(24, 32).r
    };
    assert!(
        reach(14) > reach(1) + 40,
        "{} against {}",
        reach(14),
        reach(1)
    );
}

#[test]
fn leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    accented_edges(&mut pm, 2, 38, 5);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

/// Strokes are laid along the diagonal their tone runs on, so a thin line
/// on that diagonal survives and one across it is painted out: in the
/// dark, lines rising to the right survive; in the light, falling ones;
/// and at Direction Balance 100 the light rises too.
#[test]
fn angled_strokes_run_by_tone() {
    let kept = |line: u8, ground: u8, rising: bool, balance| {
        let mut pm = Pixmap::filled(80, 80, Rgba8::new(ground, ground, ground, 255));
        for d in -30..30 {
            let y = if rising { 40 - d } else { 40 + d };
            pm.set(40 + d, y, Rgba8::new(line, line, line, 255));
        }
        angled_strokes(&mut pm, balance, 15, 0);
        let y = if rising { 38 } else { 42 };
        (pm.get(42, y).r as i32 - ground as i32).unsigned_abs()
    };
    assert!(
        kept(70, 20, true, 50) > kept(70, 20, false, 50) + 15,
        "dark did not rise"
    );
    assert!(
        kept(170, 230, false, 50) > kept(170, 230, true, 50) + 15,
        "light did not fall"
    );
    assert!(
        kept(170, 230, true, 100) > kept(170, 230, false, 100) + 15,
        "balance did not turn it"
    );
}

#[test]
fn angled_strokes_leave_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    angled_strokes(&mut pm, 50, 15, 3);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn angled_strokes_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    angled_strokes(&mut pm, 50, 15, 3);
}

/// Hatching follows texture: a busy field is hatched far more than a
/// smooth one of the same tone, and more so with more Strength.
#[test]
fn crosshatch_hatches_texture_not_smooth_areas() {
    let busy_field = || {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let v = if (x * 7 + y * 13) % 5 < 2 { 150 } else { 110 };
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        pm
    };
    let hatched = |mut pm: Pixmap, strength| {
        crosshatch(&mut pm, 9, 6, strength);
        restless(&pm)
    };
    let smooth = hatched(Pixmap::filled(64, 64, Rgba8::new(126, 126, 126, 255)), 1);
    let busy = hatched(busy_field(), 1);
    assert!(
        busy > smooth * 3,
        "texture was not favoured: {busy} against {smooth}"
    );
    assert!(
        hatched(busy_field(), 3) > busy,
        "more strength did not hatch harder"
    );
}

/// How much a picture changes from one pixel to the next, across.
fn restless(pm: &Pixmap) -> u32 {
    (0..64)
        .flat_map(|y| (1..64).map(move |x| (x, y)))
        .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
        .sum()
}

#[test]
fn crosshatch_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    crosshatch(&mut pm, 9, 6, 2);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn crosshatch_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    crosshatch(&mut pm, 9, 6, 1);
}

/// Black Intensity and Balance together take more of the picture to
/// black; White Intensity carries the light towards white.
#[test]
fn dark_strokes_push_the_tones_apart() {
    let tone = |value, balance, black, white| {
        let mut pm = Pixmap::filled(32, 32, Rgba8::new(value, value, value, 255));
        dark_strokes(&mut pm, balance, black, white);
        pm.get(16, 16).r
    };
    assert!(
        tone(100, 0, 2, 2) > 60,
        "a mid tone went black at low settings"
    );
    assert!(
        tone(100, 10, 10, 10) < 10,
        "a mid tone survived the top settings"
    );
    assert!(tone(20, 0, 2, 2) < 10, "a deep shadow did not go black");
    assert!(
        tone(225, 5, 5, 10) > tone(225, 5, 5, 0) + 10,
        "white intensity did nothing"
    );
}

#[test]
fn dark_strokes_leave_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    dark_strokes(&mut pm, 5, 5, 5);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn dark_strokes_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    dark_strokes(&mut pm, 5, 5, 5);
}

/// Ink darkens the dark side of an edge and white lights the light side,
/// each only when its slider is up.
#[test]
fn ink_outlines_ink_the_dark_side_and_chalk_the_light() {
    let drawn = |dark, light| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(120, 120, 120, 255));
        pm.fill_rect(Rect::new(32, 0, 32, 64), Rgba8::new(190, 190, 190, 255));
        ink_outlines(&mut pm, 4, dark, light);
        // A band either side: the ink lands along the stroke, which runs
        // diagonally, so it need not fall on the pixel beside the edge.
        let darkest = (27..32).map(|x| pm.get(x, 32).r).min().unwrap();
        let lightest = (33..38).map(|x| pm.get(x, 32).r).max().unwrap();
        (darkest, lightest)
    };
    let (dark_side, light_side) = drawn(40, 0);
    assert!(dark_side < 90, "the dark side was not inked: {dark_side}");
    let (_, light_side_lit) = drawn(0, 40);
    assert!(
        light_side_lit > light_side,
        "the light side was not chalked"
    );
}

#[test]
fn ink_outlines_leave_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    ink_outlines(&mut pm, 4, 20, 10);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn ink_outlines_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    ink_outlines(&mut pm, 4, 20, 10);
}

/// The spray throws colour about, further as Spray Radius rises, and at
/// 0 it leaves the picture where it was.
#[test]
fn spatter_throws_further_as_the_radius_rises() {
    let bleed = |radius| {
        let mut pm = Pixmap::filled(80, 80, Rgba8::new(40, 40, 40, 255));
        pm.fill_rect(Rect::new(0, 0, 40, 80), Rgba8::new(220, 220, 220, 255));
        spatter(&mut pm, radius, 1);
        // How far the light half has thrown pixels into the dark half.
        (40..70)
            .map(|x| (0..80).filter(|&y| pm.get(x, y).r > 120).count() as u32)
            .sum::<u32>()
    };
    assert_eq!(bleed(0), 0);
    assert!(bleed(20) > bleed(5), "{} against {}", bleed(20), bleed(5));
}

/// Smoothness gathers the grit: the result changes less from pixel to
/// pixel as it rises.
#[test]
fn spatter_smoothness_gathers_the_grit() {
    let grit = |smoothness| {
        let mut pm = Pixmap::filled(80, 80, Rgba8::new(40, 40, 40, 255));
        pm.fill_rect(Rect::new(0, 0, 40, 80), Rgba8::new(220, 220, 220, 255));
        spatter(&mut pm, 12, smoothness);
        (1..80)
            .flat_map(|y| (1..80).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    // Comfortably under three quarters — it measures about 0.57 of it.
    // The bar is the direction and a clear margin, not an exact ratio:
    // how much grit a given picture has left is a property of the
    // picture, not something the filter promises.
    assert!(
        grit(15) * 4 < grit(1) * 3,
        "{} against {}",
        grit(15),
        grit(1)
    );
}

#[test]
fn spatter_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    spatter(&mut pm, 10, 5);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

/// At the bottom of Spray Radius there is no spray, and Smoothness on its
/// own must not quietly median the picture.
#[test]
fn spatter_at_radius_zero_leaves_the_picture_alone() {
    let mut pm = Pixmap::filled(40, 40, Rgba8::new(40, 40, 40, 255));
    pm.fill_rect(Rect::new(0, 0, 20, 40), Rgba8::new(220, 220, 220, 255));
    pm.fill_rect(Rect::new(30, 30, 1, 1), Rgba8::new(0, 255, 0, 255));
    let before = pm.clone();
    for smoothness in [1, 8, 15] {
        spatter(&mut pm, 0, smoothness);
        assert_eq!(pm.as_bytes(), before.as_bytes());
    }
}

#[test]
fn spatter_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    spatter(&mut pm, 10, 5);
}

/// A noisy square, and how much it changes from pixel to pixel across and
/// down. Smearing along an axis makes neighbours along that axis agree,
/// so the count in that direction falls.
fn sprayed_grain(direction: StrokeDirection) -> (u32, u32) {
    let mut pm = Pixmap::new(64, 64);
    for y in 0..64i32 {
        for x in 0..64i32 {
            let v = ((x * 37 + y * 101) % 256) as u8;
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    sprayed_strokes(&mut pm, 12, 10, direction);
    let across = (1..64)
        .flat_map(|y| (1..64).map(move |x| (x, y)))
        .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
        .sum::<u32>();
    let down = (1..64)
        .flat_map(|y| (1..64).map(move |x| (x, y)))
        .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x, y - 1).r as i32).unsigned_abs())
        .sum::<u32>();
    (across, down)
}

/// Stroke Direction is the axis the paint is laid along: Vertical settles
/// the picture up and down, Horizontal settles it side to side.
#[test]
fn sprayed_strokes_run_along_the_chosen_direction() {
    let (across, down) = sprayed_grain(StrokeDirection::Vertical);
    assert!(
        down < across,
        "vertical: {} down against {} across",
        down,
        across
    );
    let (across, down) = sprayed_grain(StrokeDirection::Horizontal);
    assert!(
        across < down,
        "horizontal: {} across against {} down",
        across,
        down
    );
}

/// The two diagonals are not the same picture, and neither is either of
/// the axes — the dropdown does something for all four.
#[test]
fn sprayed_strokes_directions_differ_from_one_another() {
    let painted = |direction| {
        let mut pm = Pixmap::filled(48, 48, Rgba8::new(30, 60, 90, 255));
        pm.fill_rect(Rect::new(10, 10, 28, 28), Rgba8::new(230, 200, 40, 255));
        sprayed_strokes(&mut pm, 10, 12, direction);
        pm.as_bytes().to_vec()
    };
    let all = [
        painted(StrokeDirection::RightDiagonal),
        painted(StrokeDirection::Horizontal),
        painted(StrokeDirection::LeftDiagonal),
        painted(StrokeDirection::Vertical),
    ];
    for (i, one) in all.iter().enumerate() {
        for other in &all[i + 1..] {
            assert_ne!(one, other);
        }
    }
}

/// A longer stroke settles the picture further along its axis.
#[test]
fn sprayed_strokes_lengthen_with_stroke_length() {
    let along = |length| {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64i32 {
            for x in 0..64i32 {
                let v = ((x * 37 + y * 101) % 256) as u8;
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        sprayed_strokes(&mut pm, length, 8, StrokeDirection::Horizontal);
        (1..64)
            .flat_map(|y| (1..64).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    assert!(along(20) < along(4), "{} against {}", along(20), along(4));
}

/// At the bottom of both sliders there is neither spray nor stroke, and
/// the picture is left exactly as it was.
#[test]
fn sprayed_strokes_at_zero_leave_the_picture_alone() {
    let mut pm = Pixmap::filled(40, 40, Rgba8::new(40, 90, 140, 255));
    pm.fill_rect(Rect::new(5, 5, 12, 20), Rgba8::new(220, 210, 60, 255));
    let before = pm.clone();
    sprayed_strokes(&mut pm, 0, 0, StrokeDirection::RightDiagonal);
    assert_eq!(pm.as_bytes(), before.as_bytes());
}

#[test]
fn sprayed_strokes_leave_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    sprayed_strokes(&mut pm, 12, 10, StrokeDirection::Vertical);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn sprayed_strokes_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    sprayed_strokes(&mut pm, 12, 10, StrokeDirection::Vertical);
}

/// A picture to paint: a dark subject on a lighter, broken ground.
fn sumi_subject() -> Pixmap {
    let mut pm = Pixmap::new(64, 64);
    for y in 0..64i32 {
        for x in 0..64i32 {
            // Broken, but never dark: this stands for the sky and the
            // lit water, which the brush is meant to leave as paper.
            let wave = (((x * 3 + y * 7) as f32 * 0.4).sin() * 18.0) as i32;
            let at = |base: i32| (base + wave).clamp(0, 255) as u8;
            pm.set(x, y, Rgba8::new(at(150), at(180), at(230), 255));
        }
    }
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(28, 24, 22, 255));
    pm
}

fn sumi_mean(pm: &Pixmap) -> f32 {
    let b = pm.as_bytes();
    b.chunks_exact(4)
        .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
        .sum::<f32>()
        / (b.len() / 4) as f32
}

/// How near a pixel is to the paper, and to full ink.
fn is_paper(p: Rgba8) -> bool {
    (p.r as i32 - SUMI_PAPER[0] as i32).abs() <= 6
        && (p.g as i32 - SUMI_PAPER[1] as i32).abs() <= 6
        && (p.b as i32 - SUMI_PAPER[2] as i32).abs() <= 6
}
fn is_ink(p: Rgba8) -> bool {
    (p.r as i32 - SUMI_INK[0] as i32).abs() <= 12
        && (p.g as i32 - SUMI_INK[1] as i32).abs() <= 12
        && (p.b as i32 - SUMI_INK[2] as i32).abs() <= 12
}

/// The light ground is left as bare paper, and the dark subject goes to
/// full ink.
///
/// Both halves matter. A wash that never reaches the paper leaves the
/// sheet grey all over — the empty space is most of any of these
/// paintings — and one that never reaches full ink leaves the subject a
/// dark grey, which reads as a faded photograph rather than as sumi.
#[test]
fn sumi_e_leaves_bare_paper_and_reaches_full_ink() {
    let mut pm = sumi_subject();
    sumi_e(&mut pm, 7, 3, 12);
    // Well clear of the subject, and well inside it.
    for (x, y) in [(3, 3), (60, 4), (4, 60)] {
        assert!(
            is_paper(pm.get(x, y)),
            "({}, {}) is {:?}, not paper",
            x,
            y,
            pm.get(x, y)
        );
    }
    for (x, y) in [(32, 32), (26, 38)] {
        assert!(
            is_ink(pm.get(x, y)),
            "({}, {}) is {:?}, not ink",
            x,
            y,
            pm.get(x, y)
        );
    }
}

/// The paper is warm, not the white the picture happened to contain.
#[test]
fn sumi_e_paints_on_warm_paper() {
    let mut pm = sumi_subject();
    sumi_e(&mut pm, 7, 3, 12);
    let paper = pm.get(3, 3);
    assert!(paper.r > paper.b, "{:?} is not a warm white", paper);
}

/// Stroke Pressure is how much ink the brush carries, so more of it puts
/// more of the picture under a wash.
#[test]
fn sumi_e_pressure_lays_more_ink() {
    let inked = |pressure| {
        let mut pm = sumi_subject();
        sumi_e(&mut pm, 7, pressure, 12);
        sumi_mean(&pm)
    };
    assert!(inked(15) < inked(0), "{} against {}", inked(15), inked(0));
}

/// Contrast carries the ink further up into the midtones, so less of the
/// sheet is left bare.
#[test]
fn sumi_e_contrast_spreads_the_ink() {
    let bare = |contrast| {
        let mut pm = sumi_subject();
        sumi_e(&mut pm, 7, 3, contrast);
        pm.as_bytes()
            .chunks_exact(4)
            .filter(|p| is_paper(Rgba8::new(p[0], p[1], p[2], p[3])))
            .count()
    };
    assert!(bare(40) < bare(0), "{} against {}", bare(40), bare(0));
}

/// A wide brush on wet paper spreads, so the picture comes back softer.
#[test]
fn sumi_e_width_softens_the_picture() {
    let grain = |width| {
        let mut pm = sumi_subject();
        sumi_e(&mut pm, width, 3, 12);
        (1..64)
            .flat_map(|y| (1..64).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    assert!(grain(15) < grain(3), "{} against {}", grain(15), grain(3));
}

/// Only a strong colour survives, and even then the ink mutes it.
///
/// Most of these paintings are ink alone, and the ones that are not put a
/// few deliberate colours on the same bare ground. Carried straight off
/// saturation instead, every faintly tinted thing in a photograph — a
/// pale blue sky above all — lays down a wash of itself, and the sheet is
/// never empty.
#[test]
fn sumi_e_keeps_only_a_strong_colour() {
    // Same tone in both, so only the saturation differs.
    let painted = |colour: Rgba8| {
        let mut pm = Pixmap::filled(48, 48, Rgba8::new(240, 240, 238, 255));
        pm.fill_rect(Rect::new(12, 12, 24, 24), colour);
        sumi_e(&mut pm, 7, 3, 12);
        pm.get(24, 24)
    };
    let vivid = painted(Rgba8::new(190, 20, 90, 255));
    let faint = painted(Rgba8::new(120, 104, 100, 255));
    assert!(
        vivid.r as i32 - vivid.g as i32 > 12,
        "{:?} lost a strong colour entirely",
        vivid
    );
    // Muted by the ink rather than reproduced.
    assert!(vivid.r < 190, "{:?} is the photograph's own colour", vivid);
    let spread = |p: Rgba8| p.r.max(p.g).max(p.b) as i32 - p.r.min(p.g).min(p.b) as i32;
    assert!(
        spread(faint) <= 8,
        "{:?} kept a colour it should not have",
        faint
    );
}

#[test]
fn sumi_e_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    sumi_e(&mut pm, 7, 3, 12);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn sumi_e_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    sumi_e(&mut pm, 7, 3, 12);
}

#[test]
fn over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    accented_edges(&mut pm, 2, 38, 5);
}
