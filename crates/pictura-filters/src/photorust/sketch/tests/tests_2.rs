#[allow(unused_imports)]
use super::*;

/// The squares are soft-edged, not tiles: they are rounded, and fade into
/// one another rather than meeting at a step — without which a screen
/// reads as pixel art rather than as a screen. Contrast hardens the rim
/// along with everything else, so it is off the top of the slider that it
/// shows.
#[test]
fn halftone_pattern_dot_squares_are_soft_edged() {
    let mut pm = screen_sheet();
    halftone_pattern(&mut pm, 6, 7, HalftonePattern::Dot, BLACK, WHITE);
    let soft = screen_greys(&pm);
    assert!(
        soft > 64 * 64 / 8,
        "the squares met at a step, not a fade: {soft} of {} pixels",
        64 * 64
    );
    // Walking from the middle of one square to the middle of the next
    // crosses the join once and in order, through several shades.
    let row: Vec<u8> = (0..=6).map(|x| pm.get(x, 0).r).collect();
    assert!(
        row.windows(2).all(|p| p[0] <= p[1]),
        "the fade does not run from one square to the next: {row:?}"
    );
    let between = row.iter().filter(|&&v| (20..=235).contains(&v)).count();
    assert!(between >= 2, "the join is a step, not a fade: {row:?}");
}

/// At half tone the black squares and the white ones are the same size,
/// which is the chessboard the eye fuses back into that grey. At Size 1 a
/// square is a single pixel, so the chessboard is the pixel grid itself:
/// every pixel is the opposite of the four beside it and the same as the
/// four at its corners, which is the screen CS6 draws there.
#[test]
fn halftone_pattern_dot_is_a_chessboard_at_half_tone() {
    let mut pm = screen_sheet();
    halftone_pattern(&mut pm, 1, 50, HalftonePattern::Dot, BLACK, WHITE);
    for y in 1..63 {
        for x in 1..63 {
            let here = inked(&pm, x, y);
            assert_ne!(
                here,
                inked(&pm, x + 1, y),
                "({x}, {y}) matches its neighbour"
            );
            assert_ne!(
                here,
                inked(&pm, x, y + 1),
                "({x}, {y}) matches the one below"
            );
            assert_eq!(
                here,
                inked(&pm, x + 1, y + 1),
                "({x}, {y}) breaks the diagonal"
            );
        }
    }
    // Half the sheet, square for square with the paper, at a coarser Size
    // too — where a square of the board is eight pixels across. Weighed
    // rather than counted, because the join between two squares is a fade
    // (see `halftone_pattern_dot_squares_are_soft_edged`) and stands for
    // the part of a square it covers. Off the top of Contrast, too: the
    // join asks for exactly a mid grey, so at the top of the slider a
    // sheet of exactly that tone is a knife edge and lands wherever the
    // last bit of rounding sends it.
    let mut pm = screen_sheet();
    halftone_pattern(&mut pm, 8, 7, HalftonePattern::Dot, BLACK, WHITE);
    let ink: f32 = pm
        .as_bytes()
        .chunks_exact(4)
        .map(|p| 1.0 - p[0] as f32 / 255.0)
        .sum();
    let sheet = (64 * 64) as f32;
    assert!(
        (0.45..=0.55).contains(&(ink / sheet)),
        "the screen is not half ink: {:.3}",
        ink / sheet
    );
}

/// Bare paper stays bare and solid ink closes up: the ends of the tone
/// range are not lost to the coarse ladder a fine screen can strike.
#[test]
fn halftone_pattern_dot_reaches_both_ends() {
    for size in [1, 4, 12] {
        let mut white = Pixmap::filled(32, 32, WHITE);
        halftone_pattern(&mut white, size, 50, HalftonePattern::Dot, BLACK, WHITE);
        assert!(
            (0..32)
                .flat_map(|y| (0..32).map(move |x| (x, y)))
                .all(|(x, y)| !inked(&white, x, y)),
            "paper took ink at size {size}"
        );
        let mut black = Pixmap::filled(32, 32, BLACK);
        halftone_pattern(&mut black, size, 50, HalftonePattern::Dot, BLACK, WHITE);
        assert!(
            (0..32)
                .flat_map(|y| (0..32).map(move |x| (x, y)))
                .all(|(x, y)| inked(&black, x, y)),
            "the solid did not close up at size {size}"
        );
    }
}

/// The Line screen rules bands across the picture: every pixel in a row
/// agrees with its neighbour along it, and the screen repeats down.
#[test]
fn halftone_pattern_line_rules_bands() {
    let mut pm = screen_sheet();
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Line, BLACK, WHITE);
    for y in 0..(64 - 6) {
        for x in 0..(64 - 6) {
            assert_eq!(
                pm.get(x, y),
                pm.get(x + 1, y),
                "the bands do not run across"
            );
            assert_eq!(
                pm.get(x, y),
                pm.get(x, y + 6),
                "not periodic in y at ({x}, {y})"
            );
        }
    }
}

/// The Circle screen is radial: points the same distance from the middle
/// of the picture sit on the same ring.
#[test]
fn halftone_pattern_circle_rules_rings() {
    let mut pm = Pixmap::filled(128, 128, Rgba8::new(128, 128, 128, 255));
    halftone_pattern(&mut pm, 8, 50, HalftonePattern::Circle, BLACK, WHITE);
    for r in [9, 17, 26, 41, 55] {
        let right = pm.get(64 + r, 64);
        assert_eq!(right, pm.get(64 - r, 64), "not radial at r={r}");
        assert_eq!(right, pm.get(64, 64 + r), "not radial at r={r}");
        assert_eq!(right, pm.get(64, 64 - r), "not radial at r={r}");
    }
}

/// Size is how far apart the cells stand: a wider cell crosses a scanline
/// fewer times.
#[test]
fn halftone_pattern_size_widens_the_cells() {
    let bands = |size| {
        let mut pm = screen_sheet();
        halftone_pattern(&mut pm, size, 50, HalftonePattern::Line, BLACK, WHITE);
        let column: Vec<bool> = (0..64).map(|y| inked(&pm, 32, y)).collect();
        column.windows(2).filter(|pair| pair[0] != pair[1]).count()
    };
    assert!(bands(12) < bands(2), "{} against {}", bands(12), bands(2));
}

/// It draws in the swatches, not in the picture's own colours.
#[test]
fn halftone_pattern_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 255, 255));
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Dot, fore, back);
    for p in pm.as_bytes().chunks_exact(4) {
        let is_fore = p[0] == fore.r && p[1] == fore.g && p[2] == fore.b;
        let is_back = p[0] == back.r && p[1] == back.g && p[2] == back.b;
        assert!(is_fore || is_back, "{:?} is neither swatch", p);
    }
}

#[test]
fn halftone_pattern_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Circle, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn halftone_pattern_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Dot, BLACK, WHITE);
}

/// With no relief the sheet is flat: every pixel is the paper or the
/// hole and nothing else, whatever the grain.
#[test]
fn note_paper_without_relief_is_two_tones() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(230, 230, 230, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(20, 20, 20, 255));
    note_paper(&mut pm, 25, 20, 0, BLACK, WHITE);
    let hole = (255.0 * (1.0 - NOTE_INK)).round() as u8;
    let at = |x: usize, y: usize| pm.as_bytes()[(y * 64 + x) * 4];
    assert_eq!(at(2, 2), 255, "the paper is the background");
    assert_eq!(at(32, 32), hole, "the hole is a third of the way to black");
}

/// Image Balance is where the cut falls: a mid grey is paper at the
/// bottom of the slider and a hole at the top.
#[test]
fn note_paper_balance_moves_the_cut() {
    let grey = Rgba8::new(128, 128, 128, 255);
    let mut low = Pixmap::filled(32, 32, grey);
    note_paper(&mut low, 5, 0, 0, BLACK, WHITE);
    let mut high = Pixmap::filled(32, 32, grey);
    note_paper(&mut high, 45, 0, 0, BLACK, WHITE);
    assert_eq!(low.as_bytes()[0], 255);
    assert!(high.as_bytes()[0] < 200);
}

/// The holes are cut below the paper and lit from above, so the wall
/// along the top of a hole is in shadow and the one along its bottom
/// catches the light.
#[test]
fn note_paper_shadows_the_top_edge_of_a_hole() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(230, 230, 230, 255));
    pm.fill_rect(Rect::new(0, 24, 64, 16), Rgba8::new(20, 20, 20, 255));
    note_paper(&mut pm, 25, 0, 20, BLACK, WHITE);
    let column = |y: usize| pm.as_bytes()[(y * 64 + 32) * 4];
    let hole = (255.0 * (1.0 - NOTE_INK)).round() as u8;
    let top = (21..27).map(column).min().unwrap();
    let bottom = (37..43).map(column).max().unwrap();
    assert!(top < hole - 30, "top wall {top} is not in shadow");
    assert!(bottom > hole + 30, "bottom wall {bottom} is not lit");
    assert_eq!(column(32), hole, "the floor of the hole is flat");
}

/// Graininess lays a texture over flat paper; without it the paper is
/// flat whatever the relief.
#[test]
fn note_paper_graininess_textures_the_sheet() {
    let spread = |grain: u32| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(230, 230, 230, 255));
        note_paper(&mut pm, 25, grain, 15, BLACK, WHITE);
        let v: Vec<f32> = pm.as_bytes().chunks_exact(4).map(|p| p[0] as f32).collect();
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        (v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / v.len() as f32).sqrt()
    };
    assert_eq!(spread(0), 0.0);
    assert!(spread(20) > 5.0);
}

/// It draws in the swatches: the paper is the background and the holes
/// lean towards the foreground.
#[test]
fn note_paper_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(230, 230, 230, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(20, 20, 20, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 0, 255));
    note_paper(&mut pm, 25, 0, 0, fore, back);
    let at = |x: usize, y: usize| {
        let p = &pm.as_bytes()[(y * 64 + x) * 4..][..3];
        (p[0], p[1], p[2])
    };
    assert_eq!(at(2, 2), (255, 255, 0));
    let (r, g, b) = at(32, 32);
    assert!(
        r < 255 && g < 255 && b > 0,
        "the hole ({r}, {g}, {b}) is not towards the foreground"
    );
}

#[test]
fn note_paper_is_deterministic() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(150, 120, 90, 255));
    a.fill_rect(Rect::new(10, 10, 20, 20), Rgba8::new(30, 30, 30, 255));
    let mut b = a.clone();
    note_paper(&mut a, 25, 10, 11, BLACK, WHITE);
    note_paper(&mut b, 25, 10, 11, BLACK, WHITE);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

#[test]
fn note_paper_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    note_paper(&mut pm, 25, 10, 11, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn note_paper_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    note_paper(&mut pm, 25, 10, 11, BLACK, WHITE);
}

/// A copier reproduces change, not tone: a flat sheet comes back as bare
/// paper however dark it was.
#[test]
fn photocopy_leaves_flat_areas_as_paper() {
    for level in [10, 128, 240] {
        let mut pm = Pixmap::filled(32, 32, Rgba8::new(level, level, level, 255));
        photocopy(&mut pm, 7, 50, BLACK, WHITE);
        assert!(
            pm.as_bytes().chunks_exact(4).all(|p| p[0] == 255),
            "flat {level} took toner"
        );
    }
}

/// Toner goes on the dark side of an edge and nowhere else.
#[test]
fn photocopy_inks_the_dark_side_of_an_edge() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(220, 220, 220, 255));
    pm.fill_rect(Rect::new(32, 0, 32, 64), Rgba8::new(40, 40, 40, 255));
    photocopy(&mut pm, 4, 30, BLACK, WHITE);
    let at = |x: usize| pm.as_bytes()[(32 * 64 + x) * 4];
    assert_eq!(at(33), 0, "the dark side of the edge is not toner");
    assert_eq!(at(30), 255, "the light side of the edge took toner");
    assert_eq!(at(62), 255, "the flat dark far from the edge took toner");
}

/// Detail widens the neighbourhood, so a wider band of the dark side is
/// darker than what is round it.
#[test]
fn photocopy_detail_fills_in_more_of_a_dark_mass() {
    let inked = |detail: u32| {
        let mut pm = Pixmap::filled(96, 16, Rgba8::new(220, 220, 220, 255));
        pm.fill_rect(Rect::new(32, 0, 64, 16), Rgba8::new(40, 40, 40, 255));
        photocopy(&mut pm, detail, 30, BLACK, WHITE);
        (32..96)
            .filter(|&x| pm.as_bytes()[(8 * 96 + x) * 4] < 128)
            .count()
    };
    assert!(inked(20) > inked(2) + 5, "{} vs {}", inked(20), inked(2));
}

/// Darkness is how hard the difference is driven: soft grey at the
/// bottom, a cut at the top.
#[test]
fn photocopy_darkness_deepens_the_toner() {
    let at = |darkness: u32| {
        let mut pm = Pixmap::filled(64, 16, Rgba8::new(160, 160, 160, 255));
        pm.fill_rect(Rect::new(32, 0, 32, 16), Rgba8::new(130, 130, 130, 255));
        photocopy(&mut pm, 4, darkness, BLACK, WHITE);
        pm.as_bytes()[(8 * 64 + 33) * 4]
    };
    assert!(at(1) > 200, "Darkness 1 gives {}", at(1));
    assert_eq!(at(50), 0);
}

#[test]
fn photocopy_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 0, 255));
    photocopy(&mut pm, 7, 50, fore, back);
    let mut toner = false;
    for p in pm.as_bytes().chunks_exact(4) {
        let is_fore = p[0] == fore.r && p[1] == fore.g && p[2] == fore.b;
        let is_back = p[0] == back.r && p[1] == back.g && p[2] == back.b;
        toner |= is_fore;
        assert!(
            is_fore || is_back || p[0] < 255,
            "{:?} is off the swatches' line",
            p
        );
    }
    assert!(toner, "no pixel took the foreground");
}

#[test]
fn photocopy_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    photocopy(&mut pm, 7, 8, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn photocopy_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    photocopy(&mut pm, 7, 8, BLACK, WHITE);
}

/// With nothing dark enough to pour, the sheet is CS6's ramp: the
/// background at the edge nearest the light, the foreground at the far
/// one, linear in between.
#[test]
fn plaster_shades_the_sheet_as_a_ramp_away_from_the_light() {
    let mut pm = Pixmap::filled(64, 33, Rgba8::new(240, 240, 240, 255));
    plaster(&mut pm, 20, 2, Light::Top, BLACK, WHITE);
    let at = |x: usize, y: usize| pm.as_bytes()[(y * 64 + x) * 4];
    assert_eq!(at(10, 0), 255);
    assert_eq!(at(10, 32), 0);
    assert!(
        (at(10, 16) as i32 - 128).abs() <= 1,
        "middle is {}",
        at(10, 16)
    );
    assert_eq!(
        at(0, 16),
        at(63, 16),
        "a Top ramp does not change across a row"
    );

    let mut pm = Pixmap::filled(64, 33, Rgba8::new(240, 240, 240, 255));
    plaster(&mut pm, 20, 2, Light::Left, BLACK, WHITE);
    let at = |x: usize, y: usize| pm.as_bytes()[(y * 64 + x) * 4];
    assert_eq!(at(0, 10), 255);
    assert_eq!(at(63, 10), 0);
}

/// What is darker than the cut sets as a pool of the foreground, and
/// Image Balance moves the cut.
#[test]
fn plaster_pours_the_dark_as_foreground() {
    let grey = |balance: u32| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(128, 128, 128, 255));
        plaster(
            &mut pm,
            balance,
            2,
            Light::Top,
            Rgba8::new(0, 0, 200, 255),
            WHITE,
        );
        let p = &pm.as_bytes()[(10 * 64 + 32) * 4..][..3];
        (p[0], p[1], p[2])
    };
    assert_eq!(
        grey(45),
        (0, 0, 200),
        "a mid grey is poured at a high balance"
    );
    assert_ne!(
        grey(5),
        (0, 0, 200),
        "a mid grey is poured at a low balance"
    );
}

/// The pools stand proud of the plaster on a bevel that lies on the
/// plaster's side of the edge. Lit from the top, the shoulder above a
/// pool faces the lamp and catches a bright band; the one below faces
/// away and falls into shade; the pool itself stays flat foreground.
#[test]
fn plaster_lights_the_shoulder_facing_the_lamp() {
    // Wide enough that the bevel's blur does not reach its middle.
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(240, 240, 240, 255));
    pm.fill_rect(Rect::new(0, 20, 64, 24), Rgba8::new(10, 10, 10, 255));
    let mut flat = Pixmap::filled(64, 64, Rgba8::new(240, 240, 240, 255));
    plaster(&mut pm, 20, 1, Light::Top, BLACK, WHITE);
    plaster(&mut flat, 20, 1, Light::Top, BLACK, WHITE);
    let at = |pm: &Pixmap, y: usize| pm.as_bytes()[(y * 64 + 32) * 4] as i32;
    let diff = |y: usize| at(&pm, y) - at(&flat, y);
    let lit = (8..20).map(diff).max().unwrap();
    // The ramp is already dark down there, so the shade is measured as a
    // share of it rather than in levels.
    let shade = (44..56)
        .map(|y| at(&pm, y) as f32 / at(&flat, y).max(1) as f32)
        .fold(f32::INFINITY, f32::min);
    assert!(
        lit > 60,
        "the shoulder facing the lamp is only {lit} above the ramp"
    );
    assert!(
        shade < 0.85,
        "the shoulder facing away keeps {shade} of the ramp"
    );
    assert_eq!(at(&pm, 32), 0, "the top of the pool is flat foreground");
}

/// The grain rides on the picture: a light sheet stays light and a dark
/// one dark, each with a grain through it — not two swatches thrown down
/// at random.
#[test]
fn reticulation_keeps_the_picture_s_tone() {
    let mut light = Pixmap::filled(96, 96, Rgba8::new(225, 225, 225, 255));
    let mut dark = Pixmap::filled(96, 96, Rgba8::new(35, 35, 35, 255));
    // Moderate levels, so the curve is not what is being tested.
    reticulation(&mut light, 12, 20, 20, BLACK, WHITE);
    reticulation(&mut dark, 12, 20, 20, BLACK, WHITE);
    assert!(mean_tone(&light) > mean_tone(&dark) + 100.0);
    assert!(spread(&light) > 10.0, "no grain in the light");
    assert!(spread(&dark) > 10.0, "no grain in the dark");
}

/// Density is how hard the grain shakes the tone.
#[test]
fn reticulation_density_strengthens_the_grain() {
    let run = |density: u32| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(128, 128, 128, 255));
        reticulation(&mut pm, density, 20, 20, BLACK, WHITE);
        spread(&pm)
    };
    assert!(run(50) > run(0) + 5.0, "{} vs {}", run(50), run(0));
}

/// Foreground Level darkens the midtones; Background Level lightens the
/// highlights.
#[test]
fn reticulation_levels_move_the_tone() {
    let run = |tone: u8, fore: u32, back: u32| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(tone, tone, tone, 255));
        reticulation(&mut pm, 12, fore, back, BLACK, WHITE);
        mean_tone(&pm)
    };
    assert!(
        run(140, 50, 5) < run(140, 10, 5) - 40.0,
        "midtones did not darken"
    );
    assert!(
        run(220, 20, 40) > run(220, 20, 0) + 15.0,
        "highlights did not lighten"
    );
}

/// It draws in the swatches: a blue ink and a yellow paper mean every
/// pixel lies between the two.
#[test]
fn reticulation_paints_between_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(128, 128, 128, 255));
    reticulation(
        &mut pm,
        12,
        40,
        5,
        Rgba8::new(0, 0, 255, 255),
        Rgba8::new(255, 255, 0, 255),
    );
    for p in pm.as_bytes().chunks_exact(4) {
        assert_eq!(p[0], p[1], "{:?} is off the line between the swatches", p);
        assert_eq!(p[0] as u16 + p[2] as u16, 255, "{:?} is off the line", p);
    }
}

#[test]
fn reticulation_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
    a.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = a.clone();
    let mut b = a.clone();
    reticulation(&mut a, 12, 40, 5, BLACK, WHITE);
    reticulation(&mut b, 12, 40, 5, BLACK, WHITE);
    assert_eq!(a.as_bytes(), b.as_bytes());
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&a), alpha(&before));
}

#[test]
fn reticulation_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    reticulation(&mut pm, 12, 40, 5, BLACK, WHITE);
}

/// A stamp is two colours: away from an edge every pixel is one swatch
/// or the other.
#[test]
fn stamp_is_two_tones_away_from_its_edges() {
    let mut pm = Pixmap::filled(96, 96, Rgba8::new(230, 230, 230, 255));
    pm.fill_rect(Rect::new(0, 0, 48, 96), Rgba8::new(20, 20, 20, 255));
    stamp(&mut pm, 25, 1, BLACK, WHITE);
    let at = |x: usize| pm.as_bytes()[(48 * 96 + x) * 4];
    assert_eq!(at(10), 0);
    assert_eq!(at(86), 255);
}

/// Light/Dark Balance moves the cut: a mid grey is paper at the bottom
/// of the slider and ink at the top.
#[test]
fn stamp_balance_moves_the_cut() {
    let run = |balance: u32| {
        let mut pm = Pixmap::filled(32, 32, Rgba8::new(110, 110, 110, 255));
        stamp(&mut pm, balance, 5, BLACK, WHITE);
        pm.as_bytes()[(16 * 32 + 16) * 4]
    };
    assert_eq!(run(0), 255);
    assert_eq!(run(50), 0);
}

/// Smoothness melts detail away: a fine chequer of ink and paper is
/// picked out at the bottom of the slider and dissolves into one tone at
/// the top.
#[test]
fn stamp_smoothness_melts_fine_detail() {
    // Wide enough that the middle is clear of the borders even for the
    // widest melt, where the blur repeats the outermost stripe and
    // tips the tone to one side of the cut.
    let changes = |smoothness: u32| {
        let mut pm = Pixmap::filled(256, 16, Rgba8::new(230, 230, 230, 255));
        for i in 0..32 {
            pm.fill_rect(Rect::new(i * 8, 0, 4, 16), Rgba8::new(10, 10, 10, 255));
        }
        stamp(&mut pm, 25, smoothness, BLACK, WHITE);
        let row = &pm.as_bytes()[(8 * 256 + 96) * 4..(8 * 256 + 160) * 4];
        row.chunks_exact(4)
            .zip(row.chunks_exact(4).skip(1))
            .filter(|(a, b)| (a[0] > 127) != (b[0] > 127))
            .count()
    };
    assert!(
        changes(1) >= 12,
        "fine stripes lost at Smoothness 1: {}",
        changes(1)
    );
    assert_eq!(changes(50), 0, "fine stripes survived Smoothness 50");
}

/// A thin dark line on a light ground is inked even where it is not
/// darker than the cut on its own — it is darker than what is round it.
#[test]
fn stamp_picks_up_a_thin_line_on_light_ground() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(220, 220, 220, 255));
    pm.fill_rect(Rect::new(0, 31, 64, 2), Rgba8::new(140, 140, 140, 255));
    let mut flat = Pixmap::filled(64, 64, Rgba8::new(140, 140, 140, 255));
    stamp(&mut pm, 25, 1, BLACK, WHITE);
    stamp(&mut flat, 25, 1, BLACK, WHITE);
    assert_eq!(
        flat.as_bytes()[0],
        255,
        "the line's own tone is above the cut"
    );
    assert_eq!(
        pm.as_bytes()[(32 * 64 + 32) * 4],
        0,
        "the line was not picked up"
    );
}

#[test]
fn stamp_paints_in_the_swatches_and_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(230, 230, 230, 77));
    pm.fill_rect(Rect::new(0, 0, 32, 64), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 0, 255));
    stamp(&mut pm, 25, 1, fore, back);
    let px = |x: usize| {
        let p = &pm.as_bytes()[(32 * 64 + x) * 4..][..3];
        (p[0], p[1], p[2])
    };
    assert_eq!(px(4), (0, 0, 200));
    assert_eq!(px(60), (255, 255, 0));
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn stamp_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    stamp(&mut pm, 25, 5, BLACK, WHITE);
}

/// Paper stays paper: the grain only takes ink away, so a light sheet is
/// clean background at every Contrast.
#[test]
fn torn_edges_leaves_the_paper_clean() {
    for contrast in [1, 12, 25] {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(235, 235, 235, 255));
        torn_edges(&mut pm, 25, 11, contrast, BLACK, WHITE);
        assert!(
            pm.as_bytes().chunks_exact(4).all(|p| p[0] == 255),
            "Contrast {contrast}"
        );
    }
}

/// Image Balance is the cut: a mid grey is paper at the bottom of the
/// slider and ink at the top.
#[test]
fn torn_edges_balance_moves_the_cut() {
    let run = |balance: u32| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(120, 120, 120, 255));
        torn_edges(&mut pm, balance, 11, 1, BLACK, WHITE);
        inked_share(&pm)
    };
    assert_eq!(run(0), 0.0);
    assert!(run(50) > 0.9, "only {} inked at 50", run(50));
}

/// Contrast is how hard the grain bites: solid ink at the bottom of the
/// slider, riddled with holes at the top.
#[test]
fn torn_edges_contrast_bites_holes_in_the_ink() {
    let run = |contrast: u32| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(10, 10, 10, 255));
        torn_edges(&mut pm, 25, 11, contrast, BLACK, WHITE);
        inked_share(&pm)
    };
    assert!(run(1) > 0.95, "{} inked at Contrast 1", run(1));
    assert!(run(25) < 0.75, "{} inked at Contrast 25", run(25));
}

/// The edge frays: across a boundary there are pixels of both kinds in a
/// band, not one clean step — and a low Smoothness frays it wider.
#[test]
fn torn_edges_frays_the_boundary() {
    let fringe = |smoothness: u32| {
        let mut pm = Pixmap::filled(128, 64, Rgba8::new(235, 235, 235, 255));
        pm.fill_rect(Rect::new(0, 0, 64, 64), Rgba8::new(10, 10, 10, 255));
        torn_edges(&mut pm, 25, smoothness, 12, BLACK, WHITE);
        // Columns whose average is well clear of both the flecked ink
        // inside the shape and the clean paper outside it.
        let mean = |x: usize| {
            (0..64)
                .map(|y| pm.as_bytes()[(y * 128 + x) * 4] as f32)
                .sum::<f32>()
                / 64.0
        };
        let inside = mean(8);
        (0..128)
            .filter(|&x| {
                let m = mean(x);
                m > inside + 20.0 && m < 235.0
            })
            .count()
    };
    assert!(fringe(15) >= 2, "no fringe at all");
    assert!(fringe(2) > fringe(15), "{} vs {}", fringe(2), fringe(15));
}

#[test]
fn torn_edges_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
    a.fill_rect(Rect::new(4, 4, 20, 20), Rgba8::new(20, 20, 20, 200));
    let before = a.clone();
    let mut b = a.clone();
    torn_edges(&mut a, 25, 11, 17, BLACK, WHITE);
    torn_edges(&mut b, 25, 11, 17, BLACK, WHITE);
    assert_eq!(a.as_bytes(), b.as_bytes());
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&a), alpha(&before));
}

#[test]
fn torn_edges_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    torn_edges(&mut pm, 25, 11, 17, BLACK, WHITE);
}

#[test]
fn plaster_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    plaster(&mut pm, 20, 2, Light::Top, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn plaster_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    plaster(&mut pm, 20, 2, Light::Top, BLACK, WHITE);
}

/// A one-pixel sheet has no ramp to lay; it must not divide by zero.
#[test]
fn plaster_over_a_single_pixel_is_finite() {
    let mut pm = Pixmap::filled(1, 1, Rgba8::new(240, 240, 240, 255));
    plaster(&mut pm, 20, 2, Light::TopLeft, BLACK, WHITE);
    assert_eq!(pm.as_bytes()[0], 255);
}

/// The colour runs along the fibres: a dark bar bleeds out into the
/// paper either side of it, down and across, and a flat sheet with
/// nothing to bleed stays the colour it was, give or take the weave.
#[test]
fn water_paper_bleeds_colour_out_of_a_shape() {
    let bar = || {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(200, 200, 200, 255));
        pm.fill_rect(Rect::new(44, 0, 8, 96), Rgba8::new(20, 20, 20, 255));
        pm
    };
    // Brightness and Contrast at their flat middles, so only the bleed
    // is being tested.
    let mut short = bar();
    water_paper(&mut short, 3, 50, 50);
    let mut long = bar();
    water_paper(&mut long, 50, 50, 50);
    let beside = |pm: &Pixmap| (0..96).map(|y| pm.get(40, y).r as f32).sum::<f32>() / 96.0;
    assert!(
        beside(&long) < beside(&short) - 10.0,
        "longer fibres did not carry the ink further: {} against {}",
        beside(&long),
        beside(&short)
    );
}

/// Brightness lifts and sinks the picture; Contrast pulls its ends apart.
#[test]
fn water_paper_brightness_and_contrast_move_the_tone() {
    let run = |tone: u8, brightness: u32, contrast: u32| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(tone, tone, tone, 255));
        water_paper(&mut pm, 15, brightness, contrast);
        mean_tone(&pm)
    };
    assert!(
        run(128, 90, 50) > run(128, 50, 50) + 30.0,
        "brightness did not lift"
    );
    assert!(
        run(128, 10, 50) < run(128, 50, 50) - 30.0,
        "brightness did not sink"
    );
    let span = |contrast| run(200, 50, contrast) - run(60, 50, contrast);
    assert!(span(90) > span(20) + 60.0, "{} vs {}", span(90), span(20));
}

/// Unlike the rest of the family it keeps the picture's hue: a red sheet
/// stays red.
#[test]
fn water_paper_keeps_the_picture_s_colours() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(200, 40, 40, 255));
    water_paper(&mut pm, 15, 60, 80);
    let p = pm.get(32, 32);
    assert!(
        p.r as i32 > p.g as i32 + 80 && p.r as i32 > p.b as i32 + 80,
        "{:?}",
        p
    );
}

/// The weave shows in the ink, not on bare paper.
#[test]
fn water_paper_weave_shows_in_the_dark() {
    let run = |tone: u8| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(tone, tone, tone, 255));
        water_paper(&mut pm, 15, 50, 50);
        spread(&pm)
    };
    assert!(run(40) > run(230) + 2.0, "{} vs {}", run(40), run(230));
}

#[test]
fn water_paper_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
    a.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = a.clone();
    let mut b = a.clone();
    water_paper(&mut a, 15, 60, 80);
    water_paper(&mut b, 15, 60, 80);
    assert_eq!(a.as_bytes(), b.as_bytes());
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&a), alpha(&before));
}

#[test]
fn water_paper_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    water_paper(&mut pm, 15, 60, 80);
}
