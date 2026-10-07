#[allow(unused_imports)]
use super::*;

/// A surface with no slope catches no light, so it comes out midway
/// between the two swatches whatever its own brightness was.
///
/// This is most of the picture, and it is what makes the filter read as
/// carved stone rather than as a photograph with edges drawn on.
#[test]
fn bas_relief_leaves_flat_ground_midway_between_the_swatches() {
    let mut pm = step();
    bas_relief(&mut pm, 13, 3, Light::Left, BLACK, WHITE);
    // Well clear of the edge on both sides, and both were flat.
    for (x, y) in [(8, 8), (8, 50), (56, 8), (56, 50)] {
        let p = pm.get(x, y);
        assert!(
            (p.r as i32 - 128).abs() <= 3,
            "({}, {}) is {:?}, not the midpoint",
            x,
            y,
            p
        );
    }
}

/// The two faces of a ridge take opposite swatches, and swapping the
/// light over swaps them — which is the whole of the Light list.
#[test]
fn bas_relief_lights_the_two_faces_from_the_chosen_side() {
    let lit = |light| {
        let mut pm = step();
        bas_relief(&mut pm, 13, 1, light, BLACK, WHITE);
        // The ridge's two faces.
        (
            pm.get(RIDGE.start, 32).r as i32,
            pm.get(RIDGE.end - 1, 32).r as i32,
        )
    };
    let (near, far) = lit(Light::Left);
    assert!(
        (near - far).abs() > 60,
        "the ridge was not carved: {} against {}",
        near,
        far
    );
    let (flipped_near, flipped_far) = lit(Light::Right);
    assert!(
        (near - far).signum() != (flipped_near - flipped_far).signum(),
        "lighting from the other side did not turn the carving over: \
         {} against {} became {} against {}",
        near,
        far,
        flipped_near,
        flipped_far
    );
}

/// Detail is the gain: more of it drives more of the picture off the flat
/// mid-tone and into the swatches.
#[test]
fn bas_relief_detail_deepens_the_carving() {
    let carved = |detail| {
        let mut pm = step();
        bas_relief(&mut pm, detail, 3, Light::Left, BLACK, WHITE);
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| (p[0] as i32 - 128).unsigned_abs())
            .sum::<u32>()
    };
    assert!(
        carved(15) > carved(1),
        "{} against {}",
        carved(15),
        carved(1)
    );
}

/// Smoothness rounds the surface off before it is lit, so the carving
/// spreads out instead of standing on one crisp line.
///
/// Measured as the width of the carved band, not its depth: at any useful
/// Detail a hard edge drives both faces clean into the swatches whatever
/// the Smoothness, so the peak is pinned at the ends of the range and
/// says nothing. What rounding off does is make the band wider.
#[test]
fn bas_relief_smoothness_rounds_the_carving_off() {
    let width = |smoothness| {
        let mut pm = step();
        bas_relief(&mut pm, 13, smoothness, Light::Left, BLACK, WHITE);
        (0..64)
            .filter(|&x| (pm.get(x, 32).r as i32 - 128).abs() > 10)
            .count()
    };
    assert!(width(15) > width(1), "{} against {}", width(15), width(1));
}

/// It paints in the swatches, not in the picture's own colours — the one
/// thing every filter in this family has in common.
#[test]
fn bas_relief_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(32, 0, 32, 64), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 255, 255));
    bas_relief(&mut pm, 13, 3, Light::Bottom, fore, back);
    // Nothing green survives: every pixel lies on the line between the
    // two swatches, which for these two means r == g and b at or above
    // both.
    for p in pm.as_bytes().chunks_exact(4) {
        assert_eq!(
            p[0], p[1],
            "{:?} is not on the line between the swatches",
            p
        );
        assert!(
            p[2] >= p[0],
            "{:?} is not on the line between the swatches",
            p
        );
    }
}

#[test]
fn bas_relief_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    bas_relief(&mut pm, 13, 3, Light::Bottom, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn bas_relief_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    bas_relief(&mut pm, 13, 3, Light::Bottom, BLACK, WHITE);
}

/// The dark goes to the foreground, the light to the background, and the
/// middle is left as bare paper.
#[test]
fn chalk_and_charcoal_gives_the_dark_to_one_stick_and_the_light_to_the_other() {
    let mut pm = bands();
    chalk_and_charcoal(&mut pm, 6, 6, 5, BLACK, WHITE);
    let dark = band_mean(&pm, 8, 32);
    let mid = band_mean(&pm, 52, 68);
    let light = band_mean(&pm, 88, 112);
    assert!(
        dark[0] < 40.0,
        "the charcoal did not take the dark: {:?}",
        dark
    );
    assert!(
        light[0] > 215.0,
        "the chalk did not take the light: {:?}",
        light
    );
    assert!(
        (mid[0] - CHALK_GROUND).abs() < 20.0,
        "the middle is not bare paper: {:?}",
        mid
    );
}

/// The ground is a neutral grey, not the midpoint of the two swatches.
///
/// Every other filter in this family works between the two colours, so
/// splitting the difference is the natural thing to write — and with a
/// blue foreground against white it gives a pale blue paper where CS6's
/// stays grey. Adobe is explicit that the drawing sits on "a solid
/// midtone gray foundation".
#[test]
fn chalk_and_charcoal_draws_on_grey_paper_whatever_the_swatches() {
    let blue = Rgba8::new(0, 80, 200, 255);
    let mut pm = bands();
    chalk_and_charcoal(&mut pm, 6, 6, 5, blue, WHITE);
    let mid = band_mean(&pm, 52, 68);
    for (c, v) in mid.iter().enumerate() {
        assert!(
            (v - CHALK_GROUND).abs() < 20.0,
            "channel {} of the paper is {}, not grey: {:?}",
            c,
            v,
            mid
        );
    }
}

/// Each Area slider is how far its stick climbs into the midtones.
///
/// Probed at a tone the stick can actually reach. Neither reaches a flat
/// 50% grey even wound to the top — CS6's do not either — so testing both
/// against the middle of the range only shows that nothing happened.
#[test]
fn chalk_and_charcoal_areas_reach_further_into_the_middle() {
    let drawn = |tone: u8, charcoal, chalk| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(tone, tone, tone, 255));
        chalk_and_charcoal(&mut pm, charcoal, chalk, 5, BLACK, WHITE);
        band_mean(&pm, 8, 56)[0]
    };
    // A shade below the middle, which the charcoal climbs to but only
    // once the slider is wound well up.
    assert!(
        drawn(115, 20, 6) < drawn(115, 6, 6),
        "the charcoal did not climb: {} against {}",
        drawn(115, 20, 6),
        drawn(115, 6, 6)
    );
    // And a shade above it for the chalk coming down.
    assert!(
        drawn(191, 6, 20) > drawn(191, 6, 6),
        "the chalk did not come down: {} against {}",
        drawn(191, 6, 20),
        drawn(191, 6, 6)
    );
}

/// Stroke Pressure is how hard the stick is pressed: lightly, the tone
/// grades into the paper; heavily, it lies flat and the picture separates
/// into three colours with hard edges between them.
#[test]
fn chalk_and_charcoal_pressure_flattens_the_sticks() {
    let between = |pressure| {
        let mut pm = bands();
        chalk_and_charcoal(&mut pm, 10, 10, pressure, BLACK, WHITE);
        // How many pixels are neither paper nor near one of the swatches.
        pm.as_bytes()
            .chunks_exact(4)
            .filter(|p| {
                let v = p[0] as f32;
                v > 30.0 && v < 225.0 && (v - CHALK_GROUND).abs() > 25.0
            })
            .count()
    };
    assert!(
        between(5) < between(0),
        "{} against {}",
        between(5),
        between(0)
    );
}

/// Near a threshold, a light touch grades smoothly and a firm one breaks
/// the ground into separate marks.
///
/// This is the direction of Stroke Pressure, and it is the opposite of
/// what it looks like from the top of the slider alone. Wound up, the
/// threshold is a hard line and ground within a swing of it dithers into
/// strokes — so the *firm* setting is the one with visible marks on it.
/// Wound down, the threshold is wide enough that the same swing only
/// nudges the coverage, and the picture grades like a photograph with
/// almost no strokes at all. The bottom of this slider is the smoothest
/// setting there is, not the roughest.
#[test]
fn chalk_and_charcoal_pressure_breaks_the_ground_into_marks() {
    // A shade below where the chalk starts at Area 6, so both settings
    // have the same ground to work on.
    let marks = |pressure| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(204, 204, 204, 255));
        chalk_and_charcoal(&mut pm, 6, 6, pressure, BLACK, WHITE);
        (1..96)
            .flat_map(|y| (1..96).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    assert!(
        marks(5) > marks(0) * 5 / 4,
        "a firm hand did not mark the ground more than a light one: {} against {}",
        marks(5),
        marks(0)
    );
}

#[test]
fn chalk_and_charcoal_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    chalk_and_charcoal(&mut pm, 6, 6, 1, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn chalk_and_charcoal_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    chalk_and_charcoal(&mut pm, 6, 6, 1, BLACK, WHITE);
}

/// The dark subject fills with charcoal and the light ground is left as
/// bare paper — there is no middle tone at all.
///
/// That is what separates a sketch from a photograph, and it is the one
/// thing every slider here has to keep true.
///
/// Measured as how much of each was covered, not as the mean tone of it:
/// a filled mass is *streaked*, because the stick skips over the tooth of
/// the paper, so its average sits well up towards the middle of the range
/// even where the fill is working perfectly well. See
/// [`charcoal_lets_the_paper_show_through_a_filled_mass`], which insists
/// on exactly that.
#[test]
fn charcoal_fills_the_dark_and_leaves_the_light_bare() {
    let mut pm = sketchable();
    charcoal(&mut pm, 3, 2, 45, BLACK, WHITE);
    let inside = covered_in(&pm, 48, 80);
    let outside = covered_in(&pm, 4, 26);
    assert!(
        inside > 0.4,
        "the subject did not fill: {:.2} covered",
        inside
    );
    assert!(
        outside < 0.05,
        "the ground did not stay bare: {:.2} covered",
        outside
    );
}

/// Light/Dark Balance is how far up the tones the charcoal reaches.
///
/// Read as a mean tone, not as a count of pixels over some cutoff. The
/// stick shades rather than fills, so a midtone under a firm balance
/// settles at a genuine mid grey — a count of "dark" pixels lands right
/// on its own threshold there and answers noise.
///
/// Checked against the tones of the photograph these were matched on: the
/// subject at 0.219 and the sea at 0.640. Wind the span up until that sea
/// goes solid and the picture stops being a drawing at the top of the
/// slider.
#[test]
fn charcoal_balance_decides_how_far_the_stick_reaches() {
    let sheet = |tone: u8, balance| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(tone, tone, tone, 255));
        charcoal(&mut pm, 3, 2, balance, BLACK, WHITE);
        sheet_mean(&pm)
    };
    // 0.219 — the subject. Much lighter at the bottom of the slider than
    // at 45, and well shaded by 45.
    let low = sheet(56, 1);
    let mid = sheet(56, 45);
    assert!(
        low > mid * 1.5,
        "the bottom of the slider shaded nearly as much as the middle: \
         {} against {}",
        low,
        mid
    );
    assert!(mid < 130.0, "the subject was not shaded at 45: {}", mid);
    // 0.640 — a midtone. Shaded at the top of the slider, never solid.
    let high = sheet(163, 100);
    assert!(
        high > 90.0,
        "a midtone went solid at the top of the slider: {}",
        high
    );
    assert!(high < 235.0, "the top of the slider did nothing: {}", high);
}

/// Detail is how weak an edge still gets drawn.
#[test]
fn charcoal_detail_picks_up_the_finer_marks() {
    let drawn = |detail| {
        let mut pm = sketchable();
        charcoal(&mut pm, 3, detail, 45, BLACK, WHITE);
        covered(&pm)
    };
    assert!(drawn(5) > drawn(0), "{} against {}", drawn(5), drawn(0));
}

/// Charcoal Thickness spreads every line it draws, and fills more of the
/// paper's tooth back in.
#[test]
fn charcoal_thickness_lays_a_bolder_line() {
    let bold = |thickness| {
        let mut pm = sketchable();
        charcoal(&mut pm, thickness, 2, 45, BLACK, WHITE);
        covered(&pm)
    };
    assert!(bold(7) > bold(1), "{} against {}", bold(7), bold(1));
}

/// A shaded mass carries the stroke, rather than coming out flat.
///
/// Measured as the spread of tone inside it, not as how much bare paper
/// is left there. A dense mass is dense — the white inside CS6's subject
/// is the subject's own markings, not the sheet showing through — so
/// counting pale pixels asks a question that has no answer once the
/// shading is working. What must not happen is a flat silhouette.
#[test]
fn charcoal_leaves_the_stroke_in_a_shaded_mass() {
    let mut pm = sketchable();
    charcoal(&mut pm, 3, 2, 45, BLACK, WHITE);
    // Inside the subject, well clear of its outline.
    let mut values = Vec::new();
    for y in 44..84 {
        for x in 44..84 {
            values.push(pm.get(x, y).r as f32);
        }
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let spread =
        (values.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / values.len() as f32).sqrt();
    assert!(
        spread > 10.0,
        "the mass came out flat: tone varies by only {:.1} across it",
        spread
    );
}

/// The marks are short strokes, not lines drawn clear across the picture.
///
/// This is the thing that keeps going wrong, and no other test here sees
/// it: shade an even tone and every measure of *how much* charcoal went
/// down comes out identical whether the stick laid a hundred short marks
/// or one unbroken line through the lot of them. Only the length of a run
/// along the drag tells them apart.
#[test]
fn charcoal_lays_short_marks_rather_than_long_lines() {
    // An even midtone, so the whole sheet is shaded the same and any run
    // found is the stroke's own doing.
    let mut pm = Pixmap::filled(200, 200, Rgba8::new(110, 110, 110, 255));
    charcoal(&mut pm, 1, 2, 50, BLACK, WHITE);
    // Walk down the drag, which runs at 45 degrees.
    let mut longest = 0;
    for start in 0..200i32 {
        let mut run = 0;
        for step in 0..(200 - start) {
            let (x, y) = (start + step, 199 - step);
            if pm.get(x, y).r < 128 {
                run += 1;
                longest = longest.max(run);
            } else {
                run = 0;
            }
        }
    }
    assert!(
        longest < 40,
        "the stick drew a line {} pixels long — these should be strokes",
        longest
    );
}

/// A shaded area comes back in continuous tone, not one bit deep.
///
/// Charcoal is soft and grey: a stroke carries its own weight and fades
/// off at its edges. Compare the shading against a threshold instead of
/// swinging it — which is the natural way to get *separate* marks — and
/// every pixel lands on either paper or ink, the greys vanish, and the
/// result is a staircased one-bit picture. Nothing else here sees that:
/// how much charcoal went down, how long the marks are and where they
/// fall all come out the same either way.
#[test]
fn charcoal_shades_in_greys_rather_than_one_bit() {
    let mut pm = Pixmap::filled(200, 200, Rgba8::new(110, 110, 110, 255));
    charcoal(&mut pm, 1, 2, 50, BLACK, WHITE);
    let greys = pm
        .as_bytes()
        .chunks_exact(4)
        .filter(|p| (40..=215).contains(&p[0]))
        .count();
    let total = pm.as_bytes().len() / 4;
    assert!(
        greys * 4 > total,
        "only {} of {} pixels carry a grey — the shading went one bit deep",
        greys,
        total
    );
}

/// It paints in the swatches, not in the picture's own colours.
#[test]
fn charcoal_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 255, 255));
    charcoal(&mut pm, 3, 2, 45, fore, back);
    for p in pm.as_bytes().chunks_exact(4) {
        assert_eq!(
            p[0], p[1],
            "{:?} is not on the line between the swatches",
            p
        );
        assert!(
            p[2] >= p[0],
            "{:?} is not on the line between the swatches",
            p
        );
    }
}

#[test]
fn charcoal_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    charcoal(&mut pm, 3, 2, 45, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn charcoal_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    charcoal(&mut pm, 3, 2, 45, BLACK, WHITE);
}

/// A straight ramp of tone comes back oscillating: up to a highlight,
/// over the edge, and up again, several times across the range.
///
/// This is the whole filter. Any ordinary tone mapping — brighten,
/// darken, threshold, invert — takes a ramp to something that still only
/// turns around once or not at all, and would pass every other test here:
/// the output would still be grey, still respond to the sliders, still
/// leave alpha alone. What makes it read as polished metal rather than as
/// a photograph is that the tone *cycles*.
#[test]
fn chrome_cycles_the_tone_rather_than_mapping_it_straight() {
    let mut pm = ramp();
    chrome(&mut pm, 5, 1);
    let turns = turns(&pm, 32);
    assert!(
        turns >= 4,
        "a straight ramp came back turning only {} times — the tone was \
         mapped, not cycled",
        turns
    );
}

/// Detail is how many times it cycles, so more of it puts more bands
/// across the same range.
#[test]
fn chrome_detail_adds_bands() {
    let banded = |detail| {
        let mut pm = ramp();
        chrome(&mut pm, detail, 1);
        turns(&pm, 32)
    };
    assert!(
        banded(10) > banded(0),
        "{} against {}",
        banded(10),
        banded(0)
    );
}

/// Smoothness melts the surface, so the picture comes back with less
/// small structure in it.
#[test]
fn chrome_smoothness_melts_the_surface() {
    let rough = |smoothness| {
        let mut pm = Pixmap::new(128, 128);
        for y in 0..128i32 {
            for x in 0..128i32 {
                let v = (((x * 5 + y * 11) % 64) * 4) as u8;
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        chrome(&mut pm, 5, smoothness);
        (1..128)
            .flat_map(|y| (1..128).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    assert!(rough(10) < rough(0), "{} against {}", rough(10), rough(0));
}

/// Chrome is grey, whatever colour went in.
///
/// The rest of this family paints between the two swatches; this one
/// does not take them at all, because polished metal has no colour of its
/// own to take.
#[test]
fn chrome_comes_back_grey() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    chrome(&mut pm, 5, 1);
    for p in pm.as_bytes().chunks_exact(4) {
        assert_eq!(p[0], p[1], "{:?} is not grey", p);
        assert_eq!(p[1], p[2], "{:?} is not grey", p);
    }
}

#[test]
fn chrome_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    chrome(&mut pm, 5, 1);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn chrome_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    chrome(&mut pm, 5, 1);
}

/// The dark is carried in the foreground colour and the light left as
/// bare paper in the background colour.
#[test]
fn conte_crayon_carries_the_dark_and_leaves_the_light() {
    assert!(
        crayon_cover(&crayoned(30, 11, 7)) > 0.9,
        "the crayon did not carry the dark"
    );
    assert!(
        crayon_cover(&crayoned(240, 11, 7)) < 0.1,
        "the crayon did not leave the light"
    );
}

#[test]
fn conte_crayon_levels_pull_against_each_other() {
    // A midtone, where both still have something to decide.
    let mid = 150;
    let level = crayon_mean(&crayoned(mid, 8, 8));
    assert!(
        crayon_mean(&crayoned(mid, 15, 8)) < level,
        "more foreground did not carry the crayon further"
    );
    assert!(
        crayon_mean(&crayoned(mid, 8, 15)) > level,
        "more background did not bring the paper further down"
    );
}

/// The midtones are drawn in the greys between the two swatches, and the
/// paper's tooth grains them.
///
/// Both halves matter and neither is seen by anything else here. The
/// obvious way to put a crayon on textured paper is to decide, per pixel,
/// whether the stick touched it — and that gives a picture with no greys
/// at all, every pixel one swatch or the other in a fine spatter. It is a
/// halftone screen, not a drawing, and it still carries the dark, leaves
/// the light, answers every slider and keeps alpha, so the rest of these
/// tests pass on it happily.
#[test]
fn conte_crayon_draws_the_midtones_in_greys() {
    let pm = crayoned(150, 8, 8);
    let greys = crayon_greys(&pm);
    assert!(
        greys > 0.9,
        "only {:.2} of a flat midtone came back a grey — the drawing went \
         two-colour",
        greys
    );
}

/// At a Relief of 0 the paper is flat, so a flat tone comes back flat —
/// and at any Relief above it the weave grains the drawing.
///
/// The paper reaches the picture only through its lighting, and an unlit
/// surface has nothing to show. It is the same reason a square weave
/// reads as horizontal striations under a light from the top.
#[test]
fn conte_crayon_grains_the_drawing_only_where_the_paper_is_lit() {
    let spread = |relief| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(150, 150, 150, 255));
        conte_crayon(
            &mut pm,
            8,
            8,
            Texture::Canvas,
            100,
            relief,
            Light::Top,
            false,
            BLACK,
            WHITE,
        );
        let values: Vec<f32> = pm.as_bytes().chunks_exact(4).map(|p| p[0] as f32).collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        (values.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / values.len() as f32).sqrt()
    };
    assert!(
        spread(0) < 0.5,
        "flat paper still grained the drawing: {:.1}",
        spread(0)
    );
    assert!(
        spread(25) > 4.0,
        "the lit paper left no grain: {:.1}",
        spread(25)
    );
}

/// Each texture leaves its own grain, and Scaling changes its size.
#[test]
fn conte_crayon_grain_follows_the_texture_and_its_scaling() {
    // Lit, because an unlit surface shows nothing at all — see
    // [`conte_crayon_grains_the_drawing_only_where_the_paper_is_lit`].
    let drawn = |texture, scaling| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(150, 150, 150, 255));
        conte_crayon(
            &mut pm,
            8,
            8,
            texture,
            scaling,
            25,
            Light::Top,
            false,
            BLACK,
            WHITE,
        );
        pm.as_bytes().to_vec()
    };
    let canvas = drawn(Texture::Canvas, 100);
    assert_ne!(
        canvas,
        drawn(Texture::Brick, 100),
        "the texture made no difference"
    );
    assert_ne!(
        canvas,
        drawn(Texture::Canvas, 200),
        "the scaling made no difference"
    );
}

/// Relief lights the paper, so raising it changes the drawing; and the
/// Light list decides which side of the grain catches it.
#[test]
fn conte_crayon_relief_lights_the_paper() {
    let drawn = |relief, light| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(150, 150, 150, 255));
        conte_crayon(
            &mut pm,
            8,
            8,
            Texture::Canvas,
            100,
            relief,
            light,
            false,
            BLACK,
            WHITE,
        );
        pm.as_bytes().to_vec()
    };
    let flat = drawn(0, Light::Top);
    assert_ne!(
        flat,
        drawn(40, Light::Top),
        "the relief did not light the paper"
    );
    assert_ne!(
        drawn(40, Light::Top),
        drawn(40, Light::Bottom),
        "the light came from the same side either way"
    );
}

/// Invert turns the paper inside out, so what took the crayon first now
/// takes it last.
#[test]
fn conte_crayon_invert_turns_the_paper_over() {
    let drawn = |invert| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(150, 150, 150, 255));
        conte_crayon(
            &mut pm,
            8,
            8,
            Texture::Canvas,
            100,
            20,
            Light::Top,
            invert,
            BLACK,
            WHITE,
        );
        pm.as_bytes().to_vec()
    };
    assert_ne!(drawn(false), drawn(true));
}

/// The weave runs the way the Light is set, not the way it is woven.
///
/// Canvas is a square weave, and the obvious thing to do with it — read
/// the height field and let a waxy stick catch the tops of the grain —
/// lays that square over the picture as a grid, the same whichever way
/// the light is set. CS6 shows horizontal striations under a light from
/// the top, because a slope is lit by how far it falls *away* from the
/// light: the threads running across the picture catch it and the ones
/// running down it do not. Nothing else here notices which way round the
/// grain went.
#[test]
fn conte_crayon_weave_runs_across_the_light() {
    // At CS6's own default Relief of 4. Wound higher the directional
    // lighting swamps everything and the measure stops telling the two
    // apart — a square weave laid straight over the picture still comes
    // out looking directional at a Relief of 30.
    let grain_along = |light| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(150, 150, 150, 255));
        conte_crayon(
            &mut pm,
            8,
            8,
            Texture::Canvas,
            100,
            4,
            light,
            false,
            BLACK,
            WHITE,
        );
        let across: u32 = (1..128)
            .flat_map(|y| (1..128).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x - 1, y).r as i32).unsigned_abs())
            .sum();
        let down: u32 = (1..128)
            .flat_map(|y| (1..128).map(move |x| (x, y)))
            .map(|(x, y)| (pm.get(x, y).r as i32 - pm.get(x, y - 1).r as i32).unsigned_abs())
            .sum();
        (across, down)
    };
    // Lit from the top: lines run across, so the tone changes going down.
    let (across, down) = grain_along(Light::Top);
    assert!(
        down > across * 3 / 2,
        "a light from the top did not lay the weave across: {} across against {} down",
        across,
        down
    );
    // Lit from the left: the other way about.
    let (across, down) = grain_along(Light::Left);
    assert!(
        across > down * 3 / 2,
        "a light from the left did not lay the weave down: {} across against {} down",
        across,
        down
    );
}

/// It draws in the swatches, not in the picture's own colours.
#[test]
fn conte_crayon_draws_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 255, 255));
    // Relief off, so only the crayon itself is being read.
    conte_crayon(
        &mut pm,
        11,
        7,
        Texture::Canvas,
        100,
        0,
        Light::Top,
        false,
        fore,
        back,
    );
    for p in pm.as_bytes().chunks_exact(4) {
        assert_eq!(
            p[0], p[1],
            "{:?} is not on the line between the swatches",
            p
        );
        assert!(
            p[2] >= p[0],
            "{:?} is not on the line between the swatches",
            p
        );
    }
}

#[test]
fn conte_crayon_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    conte_crayon(
        &mut pm,
        11,
        7,
        Texture::Canvas,
        100,
        20,
        Light::Top,
        false,
        BLACK,
        WHITE,
    );
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn conte_crayon_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    conte_crayon(
        &mut pm,
        11,
        7,
        Texture::Canvas,
        100,
        20,
        Light::Top,
        false,
        BLACK,
        WHITE,
    );
}

/// The dark takes the ink and the light is left as bare paper — there is
/// no middle tone, which is what makes this read as a pen rather than as a
/// tinted photograph.
#[test]
fn graphic_pen_takes_the_dark_and_leaves_the_light() {
    let mut pm = Pixmap::filled(128, 128, Rgba8::new(20, 20, 20, 255));
    pm.fill_rect(Rect::new(0, 0, 128, 32), Rgba8::new(245, 245, 245, 255));
    graphic_pen(&mut pm, 8, 50, StrokeDirection::Horizontal, BLACK, WHITE);
    // Well inside each band, clear of the boundary between them.
    let light = (4..28)
        .flat_map(|y| (8..120).map(move |x| (x, y)))
        .filter(|&(x, y)| inked(&pm, x, y))
        .count();
    let dark = (48..112)
        .flat_map(|y| (8..120).map(move |x| (x, y)))
        .filter(|&(x, y)| inked(&pm, x, y))
        .count();
    assert!(
        dark > light * 3,
        "the dark did not take the ink: {} against {}",
        dark,
        light
    );
}

/// Light/Dark Balance is how far the sheet is slid towards ink. Wound up
/// it darkens the sheet; wound down it leaves most of it bare.
#[test]
fn graphic_pen_balance_moves_the_sheet_towards_ink() {
    let cover = |balance| {
        let mut pm = pen_sheet();
        graphic_pen(
            &mut pm,
            6,
            balance,
            StrokeDirection::Horizontal,
            BLACK,
            WHITE,
        );
        pen_cover(&pm)
    };
    assert!(
        cover(80) > cover(20) + 0.2,
        "the balance did not slide the sheet: {} against {}",
        cover(80),
        cover(20)
    );
}

/// Wound well up, the strokes run together and a dark area goes solid
/// rather than staying striped — which is what CS6's heavier settings show
/// and what a plain line screen cannot do.
#[test]
fn graphic_pen_dark_areas_run_together_when_the_balance_is_high() {
    let covered = |balance| {
        let mut pm = Pixmap::filled(128, 128, Rgba8::new(30, 30, 30, 255));
        graphic_pen(
            &mut pm,
            8,
            balance,
            StrokeDirection::Horizontal,
            BLACK,
            WHITE,
        );
        pen_cover(&pm)
    };
    assert!(
        covered(60) > 0.97,
        "a dark area stayed striped at a heavy balance: {:.2} covered",
        covered(60)
    );
}

/// The lines run along Stroke Direction: a pixel and its neighbour up the
/// line agree far more often than a pixel and its neighbour across it.
///
/// This is the whole of the direction list. Any of the four drawn the same
/// way would still look like a pen drawing, so nothing else here sees it.
#[test]
fn graphic_pen_lays_the_lines_the_way_the_direction_says() {
    for (direction, along) in [
        (StrokeDirection::Horizontal, (1, 0)),
        (StrokeDirection::Vertical, (0, 1)),
        (StrokeDirection::RightDiagonal, (1, -1)),
        (StrokeDirection::LeftDiagonal, (1, 1)),
    ] {
        let mut pm = pen_sheet();
        graphic_pen(&mut pm, 12, 50, direction, BLACK, WHITE);
        let with = agreement(&pm, along.0, along.1);
        let across = agreement(&pm, -along.1, along.0);
        assert!(
            with > across + 0.1,
            "{:?}: the lines do not run along the direction ({} with, {} across)",
            direction,
            with,
            across
        );
    }
}

/// Stroke Length is how long a mark is: short, and the drawing is
/// dithering that breaks against every change of tone; long, and the marks
/// are dashes that carry across the picture.
///
/// Measured as how often a scanline changes from ink to paper, which is
/// the number of marks it crosses: many short ones, or few long ones.
#[test]
fn graphic_pen_stroke_length_lengthens_the_marks() {
    let turns = |length| {
        let mut pm = pen_sheet();
        graphic_pen(
            &mut pm,
            length,
            50,
            StrokeDirection::Horizontal,
            BLACK,
            WHITE,
        );
        let row: Vec<bool> = (0..160).map(|x| inked(&pm, x, 80)).collect();
        row.windows(2).filter(|pair| pair[0] != pair[1]).count()
    };
    assert!(turns(1) > turns(15), "{} against {}", turns(1), turns(15));
}

/// It draws in the swatches, not in the picture's own colours — the one
/// thing every filter in this family has in common.
#[test]
fn graphic_pen_paints_in_the_two_swatches() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(30, 160, 60, 255));
    pm.fill_rect(Rect::new(16, 16, 32, 32), Rgba8::new(200, 40, 90, 255));
    let (fore, back) = (Rgba8::new(0, 0, 200, 255), Rgba8::new(255, 255, 255, 255));
    graphic_pen(&mut pm, 8, 50, StrokeDirection::RightDiagonal, fore, back);
    for p in pm.as_bytes().chunks_exact(4) {
        let is_fore = p[0] == fore.r && p[1] == fore.g && p[2] == fore.b;
        let is_back = p[0] == back.r && p[1] == back.g && p[2] == back.b;
        assert!(is_fore || is_back, "{:?} is neither swatch", p);
    }
}

#[test]
fn graphic_pen_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(Rect::new(4, 4, 8, 8), Rgba8::new(20, 20, 20, 200));
    let before = pm.clone();
    graphic_pen(&mut pm, 8, 50, StrokeDirection::Horizontal, BLACK, WHITE);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn graphic_pen_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    graphic_pen(&mut pm, 8, 50, StrokeDirection::Horizontal, BLACK, WHITE);
}

/// Contrast is how hard the picture is pushed against the screen: wound
/// right down the screen shades it and the greys come through, wound up
/// they are cut away and nothing is left but the two swatches.
#[test]
fn halftone_pattern_contrast_cuts_the_greys_away() {
    let greys = |contrast| {
        let mut pm = screen_sheet();
        halftone_pattern(&mut pm, 6, contrast, HalftonePattern::Dot, BLACK, WHITE);
        screen_greys(&pm)
    };
    // At the top of the slider every pixel is one swatch or the other.
    assert_eq!(greys(50), 0, "contrast 50 left greys in the screen");
    assert!(
        greys(0) > 0,
        "the screen did not soften at the bottom of contrast"
    );
    assert!(greys(0) > greys(10), "contrast did not cut the greys away");
}

/// What the screen does to a tone at CS6's default Contrast: a mid grey —
/// and only a mid grey — comes back as the full black-and-white
/// chessboard, which is how the eye reads it back as a mid grey. Either
/// side of that the chessboard is of white against a light grey, or of
/// black against a dark one, so the photograph's own tones survive as the
/// *shade* of the chessboard. A curve steeper than this collapses whole
/// bands of tone onto the same flat chessboard and posterises the sheet.
#[test]
fn halftone_pattern_shades_the_chessboard_with_the_tone() {
    // The two swatches of the chessboard at one tone, dark one first.
    let squares = |grey: u8| {
        let mut pm = Pixmap::filled(16, 16, Rgba8::new(grey, grey, grey, 255));
        halftone_pattern(&mut pm, 1, 5, HalftonePattern::Dot, BLACK, WHITE);
        let (a, b) = (pm.get(8, 8).r, pm.get(9, 8).r);
        assert_ne!(a, b, "grey {grey} came back flat, with no chessboard in it");
        (a.min(b), a.max(b))
    };
    let (dark, light) = squares(128);
    assert!(
        dark < 4 && light > 251,
        "a mid grey is not the full chessboard: {dark} {light}"
    );
    let (dark, light) = squares(220);
    assert_eq!(light, 255, "a light grey lost its paper");
    assert!(
        (160..250).contains(&dark),
        "a light grey went to ink, not a light grey: {dark}"
    );
    let (dark, light) = squares(40);
    assert_eq!(dark, 0, "a dark grey lost its ink");
    assert!(
        (10..100).contains(&light),
        "a dark grey went to paper, not a dark grey: {light}"
    );
}

/// The trap this filter falls into: thresholding the picture to two
/// colours whatever Contrast says. CS6 at a low Contrast comes back as
/// the *photograph*, shaded by the screen — a ramp still reads as a ramp,
/// with its own tones in it, and only the top of the slider flattens it
/// into ink and paper.
#[test]
fn halftone_pattern_at_low_contrast_keeps_the_photograph() {
    let ramp = || {
        let mut pm = Pixmap::new(256, 64);
        for y in 0..64 {
            for x in 0..256 {
                let v = x as u8;
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        pm
    };
    // How many tones the ramp came back in, and how well what came back
    // still climbs with what went in.
    let levels = |pm: &Pixmap| {
        let mut seen = [false; 256];
        for p in pm.as_bytes().chunks_exact(4) {
            seen[p[0] as usize] = true;
        }
        seen.iter().filter(|&&s| s).count()
    };
    let column_mean =
        |pm: &Pixmap, x: i32| (0..64).map(|y| pm.get(x, y).r as f32).sum::<f32>() / 64.0;

    let mut low = ramp();
    halftone_pattern(&mut low, 4, 5, HalftonePattern::Dot, BLACK, WHITE);
    assert!(
        levels(&low) > 32,
        "the low end of contrast came back in {} tones, not a photograph",
        levels(&low)
    );
    // Dark end inked, light end bare, and the middle in between.
    let (dark, mid, light) = (
        column_mean(&low, 16),
        column_mean(&low, 128),
        column_mean(&low, 240),
    );
    assert!(
        dark < mid && mid < light,
        "the ramp did not climb: {dark} {mid} {light}"
    );

    // And at the top of the slider the ramp is flattened into the two
    // swatches — bar the odd pixel sitting on the threshold itself, which
    // the curve passes through however steep it is.
    let mut high = ramp();
    halftone_pattern(&mut high, 4, 50, HalftonePattern::Dot, BLACK, WHITE);
    let pure = high
        .as_bytes()
        .chunks_exact(4)
        .filter(|p| p[0] == 0 || p[0] == 255)
        .count();
    assert!(
        pure * 1000 >= 256 * 64 * 995,
        "the top of contrast left {} of {} pixels grey",
        256 * 64 - pure,
        256 * 64
    );
}

/// The dark takes the ink and the light is left as bare paper.
#[test]
fn halftone_pattern_takes_the_dark_and_leaves_the_light() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(20, 20, 20, 255));
    pm.fill_rect(Rect::new(0, 0, 64, 24), Rgba8::new(245, 245, 245, 255));
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Dot, BLACK, WHITE);
    let light = (4..20)
        .flat_map(|y| (4..60).map(move |x| (x, y)))
        .filter(|&(x, y)| inked(&pm, x, y))
        .count();
    let dark = (28..60)
        .flat_map(|y| (4..60).map(move |x| (x, y)))
        .filter(|&(x, y)| inked(&pm, x, y))
        .count();
    assert!(
        dark > light * 3,
        "the dark did not take the ink: {} against {}",
        dark,
        light
    );
}

/// The Dot screen is a chessboard of squares `size` pixels across, square
/// to the picture: it comes back round every two squares either way, a
/// step of one square along a row or a column lands on the other colour,
/// and a step of one along the diagonal on the same colour again.
#[test]
fn halftone_pattern_dot_rules_a_chessboard_of_squares() {
    let mut pm = screen_sheet();
    halftone_pattern(&mut pm, 6, 50, HalftonePattern::Dot, BLACK, WHITE);
    for y in 0..(64 - 12) {
        for x in 0..(64 - 12) {
            assert_eq!(
                pm.get(x, y),
                pm.get(x + 12, y),
                "not periodic in x at ({x}, {y})"
            );
            assert_eq!(
                pm.get(x, y),
                pm.get(x, y + 12),
                "not periodic in y at ({x}, {y})"
            );
            assert_eq!(
                pm.get(x, y),
                pm.get(x + 6, y + 6),
                "not diagonal at ({x}, {y})"
            );
        }
    }
    // Read at the middle of each square, which is where it is most
    // itself — the squares fade into one another at the join, so a pixel
    // picked there belongs to neither.
    for cy in (0..52).step_by(6) {
        for cx in (0..52).step_by(6) {
            let here = inked(&pm, cx, cy);
            assert_ne!(
                here,
                inked(&pm, cx + 6, cy),
                "squares do not alternate across"
            );
            assert_ne!(
                here,
                inked(&pm, cx, cy + 6),
                "squares do not alternate down"
            );
        }
    }
}
