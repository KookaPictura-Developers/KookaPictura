#[allow(unused_imports)]
use super::*;

#[test]
fn the_pencil_draws_the_detail_and_leaves_the_flat_parts_as_paper() {
    let mut pm = half_detailed();
    colored_pencil(&mut pm, 2, 8, 25, Rgba8::WHITE);
    let detailed = ink(&pm, 0..64);
    let flat = ink(&pm, 144..160);
    assert!(
        detailed > flat * 4,
        "the flat wash took nearly as much pencil as the detail: {detailed} vs {flat}"
    );
}

/// A drawn shape keeps its own colour: the paper streaks across a petal,
/// it does not swallow it. CS6's flowers come back pink.
///
/// This is the regression the hatch is one number away from at all times.
/// The strokes cover about a quarter of the page, so a hatch that takes
/// nearly everything back in its gaps leaves a third of the colour at best
/// — everywhere, detail or no detail — and the filter turns into a washed
/// photograph on grey rather than a drawing.
#[test]
fn a_drawn_shape_keeps_most_of_its_colour() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(200, 60, 90, 255));
    // Fine texture, so the whole field counts as something to draw — what
    // the veins in a petal do — without moving the colour itself much.
    for y in 0..64 {
        for x in (0..64).step_by(3) {
            pm.set(x, y, Rgba8::new(220, 75, 105, 255));
        }
    }
    let before = ink(&pm, 0..64);
    let mut drawn = pm.clone();
    colored_pencil(&mut drawn, 2, 8, 25, Rgba8::WHITE);
    let after = ink(&drawn, 0..64);
    assert!(
        after * 2 > before,
        "the petal lost most of its colour to the paper: {after} of {before}"
    );
}

/// Paper Brightness sets the level of the ground. A flat wash has no
/// detail to draw, so the pencil leaves it as a light stroke of its own
/// colour over the sheet — which pulls it towards the paper rather than
/// leaving it at the picture's own level. 0 is black paper and 50 is
/// white.
#[test]
fn paper_brightness_sets_the_ground() {
    // Well away from the middle, so "towards the paper" is unambiguous at
    // every setting.
    let source = Rgba8::new(60, 80, 100, 255);
    for (setting, paper) in [(0u32, 0i32), (25, 128), (50, 255)] {
        let mut pm = Pixmap::filled(32, 32, source);
        colored_pencil(&mut pm, 4, 8, setting, Rgba8::WHITE);
        let got = pm.get(16, 16).r as i32;
        assert!(
            (got - paper).abs() < (got - source.r as i32).abs(),
            "paper brightness {setting}: {got} was not pulled towards {paper}"
        );
    }
}

/// A field the colour of the sheet has nothing to show, so it comes back
/// as the paper — within the hatch's own faint stroke, which the hand
/// leaves even where it had nothing to draw.
#[test]
fn a_field_the_colour_of_the_paper_stays_paper() {
    for (setting, paper) in [(0u32, 0i32), (25, 128), (50, 255)] {
        let mut pm = Pixmap::filled(
            32,
            32,
            Rgba8::new(paper as u8, paper as u8, paper as u8, 255),
        );
        colored_pencil(&mut pm, 4, 8, setting, Rgba8::WHITE);
        let got = pm.get(16, 16).r as i32;
        assert!(
            (got - paper).abs() <= HATCH_INK as i32,
            "paper brightness {setting}: a paper-coloured field came back at {got}"
        );
    }
}

/// Stroke Pressure is the gain on everything the hand does, the hatch
/// included, so at zero the page stays blank however much there was to
/// draw.
#[test]
fn no_pressure_leaves_the_page_blank() {
    let mut pm = half_detailed();
    colored_pencil(&mut pm, 2, 0, 25, Rgba8::WHITE);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[0] == 128));
}

/// More pressure, more pencil.
#[test]
fn pressure_lays_more_colour() {
    let laid = |pressure| {
        let mut pm = half_detailed();
        colored_pencil(&mut pm, 2, pressure, 25, Rgba8::WHITE);
        ink(&pm, 0..64)
    };
    assert!(laid(14) > laid(4), "pressing harder laid no more colour");
}

/// The hatch is seeded from where each pixel is, so an undo/redo replay
/// draws the same strokes.
#[test]
fn the_same_picture_is_drawn_the_same_way_twice() {
    crate::photorust::with_seed(0, || {
        let mut first = half_detailed();
        colored_pencil(&mut first, 6, 8, 25, Rgba8::WHITE);
        let mut second = half_detailed();
        colored_pencil(&mut second, 6, 8, 25, Rgba8::WHITE);
        assert_eq!(first.as_bytes(), second.as_bytes());
    });
}

/// A wider pencil is a coarser hatch — the same drawing in fewer, bigger
/// strokes, so neighbouring pixels agree with each other more often.
#[test]
fn a_wider_pencil_makes_a_coarser_hatch() {
    let roughness = |width| {
        let mut pm = half_detailed();
        colored_pencil(&mut pm, width, 12, 25, Rgba8::WHITE);
        (0..32)
            .map(|x| {
                (0..64)
                    .map(|y| (pm.get(x, y).r as i32 - pm.get(x + 1, y).r as i32).unsigned_abs())
                    .sum::<u32>()
            })
            .sum::<u32>()
    };
    assert!(
        roughness(20) < roughness(2),
        "a wide pencil changed as often across the page as a fine one"
    );
}

#[test]
fn colored_pencil_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    colored_pencil(&mut pm, 4, 8, 25, Rgba8::WHITE);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn colored_pencil_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    colored_pencil(&mut pm, 4, 8, 25, Rgba8::WHITE);
}

/// The point of the filter: what comes back is a handful of flat areas,
/// not a photograph.
#[test]
fn cutout_leaves_a_few_flat_pieces() {
    let mut pm = disc_on_a_ground();
    let before = shades(&pm).len();
    cutout(&mut pm, 4, 4, 2);
    let after = shades(&pm).len();
    assert!(before >= 10, "the test picture was already flat: {before}");
    assert!(
        after <= 4,
        "a two-colour picture came back in {after} shades"
    );
}

/// A piece is painted the colour the picture had under it, not a value off
/// a fixed grid — the difference between Cutout and Posterize. The disc
/// stays the pink it was; the ground stays the green it was.
#[test]
fn a_piece_keeps_the_colour_it_covered() {
    let mut pm = disc_on_a_ground();
    cutout(&mut pm, 4, 4, 2);
    // The grain averages out to four levels over the picture's own colour.
    for (at, want) in [((48, 48), [214, 104, 154]), ((4, 4), [44, 94, 34])] {
        let got = pm.get(at.0, at.1);
        for (c, want) in [got.r, got.g, got.b].into_iter().zip(want) {
            assert!(
                (c as i32 - want).abs() <= 8,
                "a piece came back {got:?} where the picture was {want:?}"
            );
        }
    }
}

/// Cutting more finely leaves more pieces, which is what Number of Levels
/// is for.
#[test]
fn more_levels_cut_more_pieces() {
    let gradient = || {
        let mut pm = Pixmap::new(128, 32);
        for y in 0..32 {
            for x in 0..128 {
                let v = (x * 2) as u8;
                pm.set(x, y, Rgba8::new(v, v / 2, 255 - v, 255));
            }
        }
        pm
    };
    let pieces = |levels| {
        let mut pm = gradient();
        cutout(&mut pm, levels, 0, 3);
        shades(&pm).len()
    };
    assert!(
        pieces(8) > pieces(2),
        "eight levels cut no more finely than two"
    );
}

/// Edge Simplicity rubs out what is too small to cut around. A speck a few
/// pixels across survives a light hand and not a heavy one.
#[test]
fn edge_simplicity_rubs_out_the_small_stuff() {
    let speck = || {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(40, 90, 30, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(30, 30, 3, 3),
            Rgba8::new(230, 40, 60, 255),
        );
        pm
    };
    let survives = |simplicity| {
        let mut pm = speck();
        cutout(&mut pm, 4, simplicity, 3);
        pm.get(31, 31).r > 150
    };
    assert!(survives(0), "a light hand cut round the speck anyway");
    assert!(!survives(8), "a heavy hand left the speck standing");
}

/// Two areas exactly as bright as each other are still two pieces. Cutting
/// on brightness alone would join them and paint the pair some average
/// mud, which is the thing this filter must not do to a flower on grass.
#[test]
fn equally_bright_colours_are_cut_apart() {
    // Matched to within a level of each other by the usual weighting.
    let (pink, green) = (Rgba8::new(214, 100, 160, 255), Rgba8::new(40, 208, 60, 255));
    assert!((luma(pink) - luma(green)).abs() < 1.0);
    let mut pm = Pixmap::filled(64, 32, pink);
    pm.fill_rect(crate::photorust::pixmap::Rect::new(32, 0, 32, 32), green);
    cutout(&mut pm, 4, 2, 3);
    for (at, want) in [((8, 16), pink), ((56, 16), green)] {
        let got = pm.get(at.0, at.1);
        assert!(
            (got.r as i32 - want.r as i32).abs() <= 8 && (got.g as i32 - want.g as i32).abs() <= 8,
            "{got:?} where the picture was {want:?} — the two were cut as one piece"
        );
    }
}

/// Areas touching only at a corner are two pieces, as a pair of scissors
/// would leave them, and each takes its own colour.
#[test]
fn areas_meeting_at_a_corner_are_two_pieces() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(20, 20, 20, 255));
    let red = Rgba8::new(220, 40, 40, 255);
    pm.fill_rect(crate::photorust::pixmap::Rect::new(0, 0, 16, 16), red);
    pm.fill_rect(crate::photorust::pixmap::Rect::new(16, 16, 16, 16), red);
    // No flattening and no vote, so the corner is left exactly as drawn.
    cutout(&mut pm, 8, 0, 3);
    assert_eq!(pm.get(4, 4).r, pm.get(20, 20).r);
}

#[test]
fn cutout_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    cutout(&mut pm, 4, 4, 2);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

/// Every pixel belongs to exactly one piece, whatever the settings — the
/// fill must leave none of the picture uncut.
#[test]
fn cutout_covers_the_whole_picture() {
    for (levels, simplicity, fidelity) in [(2, 0, 1), (8, 10, 3), (4, 4, 2), (2, 10, 1), (8, 0, 3)]
    {
        let mut pm = disc_on_a_ground();
        cutout(&mut pm, levels, simplicity, fidelity);
        assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 255));
    }
}

#[test]
fn cutout_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    cutout(&mut pm, 4, 4, 2);
}

/// The brush lays flat colour — the field it paints over comes back
/// calmer than it was.
#[test]
fn the_brush_flattens_what_it_paints_over() {
    let before = two_noisy_fields();
    let mut after = before.clone();
    dry_brush(&mut after, 6, 10, 1);
    let (was, now) = (restlessness(&before, 4..28), restlessness(&after, 4..28));
    assert!(
        now * 3 < was,
        "the brush left the surface as restless as it found it: {now} against {was}"
    );
}

/// ...and it loads from one side of a boundary, never across it, so the
/// boundary is exactly where it was.
#[test]
fn the_brush_does_not_paint_across_an_edge() {
    let mut pm = two_noisy_fields();
    dry_brush(&mut pm, 6, 10, 1);
    // The step from one side to the other, a pixel either way.
    let step = pm.get(32, 32).r as i32 - pm.get(31, 32).r as i32;
    assert!(step > 120, "the edge came back softened to {step} levels");
}

/// A wider brush lays broader patches. Stripes eight pixels apart stand up
/// to a brush narrower than they are and are painted over by one wider than
/// they are, which is the whole of what Brush Size does.
#[test]
fn a_bigger_brush_paints_broader_patches() {
    let stripes = || {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let v = if (x / 8) % 2 == 0 { 60 } else { 200 };
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        pm
    };
    // How much of the stripes' contrast is left across the middle of the
    // picture, away from the frame's edges.
    let surviving = |size| {
        let mut pm = stripes();
        dry_brush(&mut pm, size, 10, 1);
        let band: Vec<i32> = (16..48).map(|x| pm.get(x, 32).r as i32).collect();
        band.iter().max().unwrap() - band.iter().min().unwrap()
    };
    assert!(
        surviving(10) * 2 < surviving(2),
        "a wide brush left as much standing as a narrow one: {} against {}",
        surviving(10),
        surviving(2)
    );
}

/// Brush Detail is how many colours the paint is mixed from.
#[test]
fn brush_detail_sets_how_many_colours_the_paint_is_mixed_from() {
    let mixed = |detail| {
        let mut pm = two_noisy_fields();
        dry_brush(&mut pm, 2, detail, 1);
        shades(&pm).len()
    };
    assert!(
        mixed(10) > mixed(0),
        "the top of the slider mixed no more colours than the bottom"
    );
}

/// Texture is the body of the paint: laid on thickly, the facets the brush
/// left stand further from one another.
///
/// Measured on a picture with brushwork in it, because that is what the
/// slider works on. A flat wash has no facets to raise and comes back the
/// same at every setting, which is the honest answer — there is nothing
/// there to lay on thickly.
#[test]
fn texture_is_the_body_of_the_paint() {
    let laid = |texture| {
        let mut pm = two_noisy_fields();
        dry_brush(&mut pm, 4, 10, texture);
        restlessness(&pm, 4..60)
    };
    assert!(
        laid(3) > laid(1),
        "paint laid on thickly stood no further out: {} against {}",
        laid(3),
        laid(1)
    );
}

/// The canvas is seeded from where each pixel is, so an undo/redo replay
/// paints on the same one.
#[test]
fn the_same_picture_is_painted_the_same_way_twice() {
    crate::photorust::with_seed(0, || {
        let mut first = two_noisy_fields();
        dry_brush(&mut first, 4, 8, 2);
        let mut second = two_noisy_fields();
        dry_brush(&mut second, 4, 8, 2);
        assert_eq!(first.as_bytes(), second.as_bytes());
    });
}

#[test]
fn dry_brush_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    dry_brush(&mut pm, 4, 8, 2);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

/// Brush Size 0 is no brush: nothing is painted, and what comes back is
/// the picture with the paint mixed and the canvas under it.
#[test]
fn no_brush_paints_nothing() {
    let source = two_noisy_fields();
    let mut pm = source.clone();
    dry_brush(&mut pm, 0, 10, 1);
    for (a, b) in pm
        .as_bytes()
        .chunks_exact(4)
        .zip(source.as_bytes().chunks_exact(4))
    {
        assert!(
            (a[0] as i32 - b[0] as i32).abs() <= 8,
            "size 0 moved a pixel from {} to {}",
            b[0],
            a[0]
        );
    }
}

#[test]
fn dry_brush_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    dry_brush(&mut pm, 2, 8, 2);
}

/// The knife lays a noisy surface down flat and stops dead at a boundary:
/// Kuwahara takes each pixel from the calmest box beside it, which is never
/// one straddling the edge.
#[test]
fn the_knife_flattens_a_surface_and_stops_at_a_boundary() {
    let before = two_noisy_fields();
    let mut pm = before.clone();
    palette_knife(&mut pm, 12, 3, 0);
    assert!(
        restlessness(&pm, 4..28) * 4 < restlessness(&before, 4..28),
        "the knife left the noise in: {} against {}",
        restlessness(&pm, 4..28),
        restlessness(&before, 4..28)
    );
    let left = pm.get(28, 32).r as i32;
    let right = pm.get(35, 32).r as i32;
    assert!(
        left < 80 && right > 180,
        "the knife carried one field into the other: {left} and {right}"
    );
}

/// A wide knife works in bigger patches than a fine one, and so leaves the
/// picture further from where it started.
#[test]
fn a_wider_knife_strays_further_from_the_picture() {
    let strayed = |size| {
        let before = two_noisy_fields();
        let mut pm = before.clone();
        palette_knife(&mut pm, size, 3, 0);
        pm.as_bytes()
            .iter()
            .zip(before.as_bytes())
            .map(|(&a, &b)| (a as i32 - b as i32).unsigned_abs())
            .sum::<u32>()
    };
    assert!(
        strayed(50) > strayed(3),
        "a wide knife stayed as close to the picture as a fine one: {} against {}",
        strayed(50),
        strayed(3)
    );
}

/// Stroke Detail is the palette, `2 + 3·detail` levels, and only half of each
/// channel is rounded onto it. A flat 150 rounds to 127.5 on the four-step
/// palette and to 153 on the ten-step one.
#[test]
fn stroke_detail_is_the_palette() {
    let flat = |detail| {
        let mut pm = Pixmap::filled(16, 16, Rgba8::new(150, 150, 150, 255));
        palette_knife(&mut pm, 12, detail, 0);
        pm.get(8, 8).r
    };
    assert_eq!(flat(1), 139);
    assert_eq!(flat(3), 152);
}

/// Softness eases the hard join the knife leaves between two patches.
#[test]
fn softness_eases_the_joins_between_patches() {
    let hardest_join = |softness| {
        let mut pm = two_noisy_fields();
        palette_knife(&mut pm, 12, 3, softness);
        (0..63)
            .map(|x| (pm.get(x, 32).r as i32 - pm.get(x + 1, 32).r as i32).abs())
            .max()
            .unwrap_or(0)
    };
    assert!(
        hardest_join(10) < hardest_join(0),
        "the soft blade cut as hard as the sharp one: {} against {}",
        hardest_join(10),
        hardest_join(0)
    );
}

#[test]
fn palette_knife_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    palette_knife(&mut pm, 25, 3, 5);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn palette_knife_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    palette_knife(&mut pm, 25, 3, 0);
}

/// The daubs wash a surface together and stop dead at a boundary, which is
/// the whole of the first pass.
#[test]
fn a_daub_washes_a_surface_together_and_stops_at_a_boundary() {
    let before = two_noisy_fields();
    let mut after = before.clone();
    paint_daubs(&mut after, 10, 0, DaubBrush::Simple);
    assert!(
        restlessness(&after, 4..28) * 4 < restlessness(&before, 4..28),
        "the daub left the surface as it found it"
    );
    let step = after.get(32, 32).r as i32 - after.get(31, 32).r as i32;
    assert!(
        step >= 100,
        "the daub washed across the edge: {step} levels"
    );
}

/// A bigger brush lays a bigger daub, and anything smaller than the daub
/// goes into it. A mark a few pixels across survives a small brush and is
/// painted over by a large one — as long as it is near enough in colour to
/// belong to the same thing, which is the next test's business.
#[test]
fn a_bigger_brush_lays_a_bigger_daub() {
    let survives = |size| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(120, 120, 120, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(30, 30, 6, 6),
            Rgba8::new(150, 150, 150, 255),
        );
        paint_daubs(&mut pm, size, 0, DaubBrush::Simple);
        pm.get(32, 32).r as i32 - 120
    };
    assert!(
        survives(2) > 20,
        "a small brush painted over the mark anyway"
    );
    assert!(survives(20) < 8, "a large brush left the mark standing");
}

/// Dark Rough draws a dark outline along the dark side of a boundary —
/// darker than either field — and Light Rough a light one along the
/// bright side.
#[test]
fn the_rough_brushes_draw_their_halo_on_one_side() {
    let halo = |brush| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(60, 60, 60, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(32, 0, 32, 64),
            Rgba8::new(200, 200, 200, 255),
        );
        paint_daubs(&mut pm, 8, 20, brush);
        let mean = |x: i32| (0..64).map(|y| pm.get(x, y).r as u32).sum::<u32>() / 64;
        (mean(30), mean(33))
    };
    let (dark_side, _) = halo(DaubBrush::DarkRough);
    assert!(dark_side < 30, "Dark Rough drew no outline: {dark_side}");
    let (dark_side, light_side) = halo(DaubBrush::LightRough);
    assert!(light_side > 230, "Light Rough drew no rim: {light_side}");
    assert!(
        dark_side > 40,
        "Light Rough darkened the dark side: {dark_side}"
    );
}

/// Sparkle draws contour lines across a slow gradient, which the plain
/// brush leaves smooth, and lifts the light parts of the picture.
#[test]
fn sparkle_draws_contours_and_lights_the_picture_up() {
    let ramp = || {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let v = (60 + x * 2) as u8;
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        pm
    };
    let across = |pm: &Pixmap| {
        (1..64)
            .map(|x| (pm.get(x, 32).r as i32 - pm.get(x - 1, 32).r as i32).unsigned_abs())
            .sum::<u32>()
    };
    let (mut plain, mut sparkle) = (ramp(), ramp());
    paint_daubs(&mut plain, 4, 17, DaubBrush::Simple);
    paint_daubs(&mut sparkle, 4, 17, DaubBrush::Sparkle);
    assert!(
        across(&sparkle) > across(&plain) * 3,
        "no contours: {} against {}",
        across(&sparkle),
        across(&plain)
    );

    // The lines are light: they rise well above the plain brush and nothing
    // sinks below it.
    let row = |pm: &Pixmap| (0..64).map(|x| pm.get(x, 32).r as i32).collect::<Vec<_>>();
    let (lit, base) = (row(&sparkle), row(&plain));
    let lines = lit.iter().zip(&base).filter(|(s, p)| *s - *p > 40).count();
    assert!(
        lines >= 4,
        "only {lines} pixels of light lines across the ramp"
    );
    assert!(
        lit.iter().zip(&base).all(|(s, p)| s - p > -8),
        "Sparkle drew dark lines: {lit:?} against {base:?}"
    );

    // A flat field has no bands to draw, so what lifts it is the tone curve
    // alone — gently, so the picture keeps its colour under the lines.
    let mut light = Pixmap::filled(32, 32, Rgba8::new(200, 200, 200, 255));
    paint_daubs(&mut light, 4, 0, DaubBrush::Sparkle);
    assert!(
        light.get(16, 16).r > 205,
        "not lit up: {}",
        light.get(16, 16).r
    );
}

/// The Rough brushes lay texture where the painting ones leave a surface
/// smooth.
#[test]
fn the_rough_brushes_lay_texture() {
    let texture = |brush| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(120, 120, 120, 255));
        paint_daubs(&mut pm, 8, 10, brush);
        restlessness(&pm, 4..60)
    };
    assert_eq!(texture(DaubBrush::Simple), 0);
    assert!(
        texture(DaubBrush::DarkRough) > 500,
        "no texture: {}",
        texture(DaubBrush::DarkRough)
    );
}

/// The Wide brushes stretch the daub sideways, so a horizontal stripe a
/// round daub of the same size paints over survives them.
#[test]
fn the_wide_brushes_lay_a_wide_daub() {
    let stripe = |brush| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(60, 60, 60, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(0, 30, 64, 4),
            Rgba8::new(200, 200, 200, 255),
        );
        paint_daubs(&mut pm, 12, 0, brush);
        pm.get(32, 31).r
    };
    assert!(
        stripe(DaubBrush::Simple) < 70,
        "the round daub kept the stripe"
    );
    assert!(
        stripe(DaubBrush::WideSharp) > 190,
        "the wide daub lost the stripe"
    );
}

#[test]
fn paint_daubs_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(0, 0, 3, 3),
        Rgba8::new(120, 140, 160, 200),
    );
    let before = pm.clone();
    paint_daubs(&mut pm, 8, 7, DaubBrush::DarkRough);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

/// Relief catches the light: a bright disc on a dark ground comes back
/// with highlights round it, and a flat surface comes back with none.
#[test]
fn plastic_wrap_highlights_relief_and_not_a_flat_surface() {
    let mut flat = Pixmap::filled(48, 48, Rgba8::new(90, 90, 90, 255));
    plastic_wrap(&mut flat, 20, 9, 7);
    let brightest = |pm: &Pixmap| pm.as_bytes().chunks_exact(4).map(|p| p[0]).max().unwrap();
    assert!(
        brightest(&flat) <= 90,
        "the flat surface shone: {}",
        brightest(&flat)
    );

    let mut disc = Pixmap::filled(64, 64, Rgba8::new(40, 40, 40, 255));
    for y in 0..64 {
        for x in 0..64 {
            if (x - 32) * (x - 32) + (y - 32) * (y - 32) < 12 * 12 {
                disc.set(x, y, Rgba8::new(200, 200, 200, 255));
            }
        }
    }
    plastic_wrap(&mut disc, 20, 9, 7);
    assert!(
        brightest(&disc) > 240,
        "the relief did not shine: {}",
        brightest(&disc)
    );
}

/// Highlight Strength is how bright the highlights are, and at 0 there
/// are none.
#[test]
fn plastic_wrap_highlight_strength_sets_the_gloss() {
    let lit = |strength| {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let v = if (x / 8 + y / 8) % 2 == 0 { 60 } else { 180 };
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        let before = pm.clone();
        plastic_wrap(&mut pm, strength, 9, 7);
        pm.as_bytes()
            .chunks_exact(4)
            .zip(before.as_bytes().chunks_exact(4))
            .map(|(a, b)| (a[0] as i32 - b[0] as i32).max(0) as u32)
            .sum::<u32>()
    };
    assert_eq!(lit(0), 0);
    assert!(lit(20) > lit(8) * 2, "{} against {}", lit(20), lit(8));
}

#[test]
fn plastic_wrap_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(250, 250, 250, 200),
    );
    let before = pm.clone();
    plastic_wrap(&mut pm, 15, 9, 7);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn plastic_wrap_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    plastic_wrap(&mut pm, 15, 9, 7);
}

/// The dark side of a boundary is inked, and the light side and a flat
/// field are not.
#[test]
fn poster_edges_inks_the_dark_side_of_a_boundary() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(60, 60, 60, 255));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(32, 0, 32, 64),
        Rgba8::new(200, 200, 200, 255),
    );
    poster_edges(&mut pm, 2, 1, 6);
    assert!(
        pm.get(31, 32).r < 15,
        "no ink on the dark side: {}",
        pm.get(31, 32).r
    );
    assert!(
        pm.get(33, 32).r > 150,
        "ink on the light side: {}",
        pm.get(33, 32).r
    );
    assert!(
        pm.get(8, 32).r > 40,
        "ink on a flat field: {}",
        pm.get(8, 32).r
    );
}

/// Posterization bands the brightness and keeps the colour: a ramp comes
/// back in few values, and a green stays green.
#[test]
fn poster_edges_bands_brightness_and_keeps_colour() {
    let mut ramp = Pixmap::new(256, 8);
    for y in 0..8 {
        for x in 0..256 {
            ramp.set(x, y, Rgba8::new(x as u8, x as u8, x as u8, 255));
        }
    }
    poster_edges(&mut ramp, 0, 0, 0);
    let mut values: Vec<u8> = (0..256).map(|x| ramp.get(x, 4).r).collect();
    // A level either way is rounding, not another band.
    values.dedup_by(|a, b| a.abs_diff(*b) <= 2);
    assert!(values.len() <= 8, "the ramp was not banded: {values:?}");

    let mut green = Pixmap::filled(32, 32, Rgba8::new(50, 100, 20, 255));
    poster_edges(&mut green, 2, 1, 0);
    let g = green.get(16, 16);
    assert!(
        g.g > g.r * 3 / 2 && g.g > g.b * 3,
        "the green lost its colour: {g:?}"
    );
}

#[test]
fn poster_edges_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    poster_edges(&mut pm, 2, 1, 2);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn poster_edges_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    poster_edges(&mut pm, 2, 1, 2);
}

/// A longer stroke carries a mark further along the diagonal — but only
/// a little: the picture stays sharp, and the long streaks are grain.
#[test]
fn rough_pastels_strokes_run_diagonally_and_lengthen() {
    let reach = |length| {
        let mut pm = Pixmap::filled(80, 80, Rgba8::new(40, 40, 40, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(38, 38, 4, 4),
            Rgba8::new(255, 255, 255, 255),
        );
        pastel(&mut pm, length, 1, 0);
        // Up and to the right of the mark, and straight to its right.
        let diagonal: u32 = (3..5).map(|d| pm.get(41 + d, 38 - d).r as u32).sum();
        let across: u32 = (4..7).map(|d| pm.get(41 + d, 40).r as u32).sum::<u32>() * 2 / 3;
        (diagonal, across)
    };
    let (short, _) = reach(0);
    let (long, across) = reach(40);
    assert!(
        long > short,
        "a longer stroke did not reach further: {long} against {short}"
    );
    assert!(
        long > across,
        "the stroke did not run diagonally: {long} against {across}"
    );
}

/// The pastel is paler than the picture, and grained where it was flat.
#[test]
fn rough_pastels_pales_and_grains_the_picture() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(100, 100, 100, 255));
    pastel(&mut pm, 6, 4, 0);
    let mean = pm
        .as_bytes()
        .chunks_exact(4)
        .map(|p| p[0] as u32)
        .sum::<u32>()
        / (64 * 64);
    assert!(mean > 110, "not paler: {mean}");
    assert!(
        restlessness(&pm, 4..60) > 300,
        "no grain: {}",
        restlessness(&pm, 4..60)
    );
}

#[test]
fn rough_pastels_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    pastel(&mut pm, 6, 4, 20);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn rough_pastels_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    pastel(&mut pm, 6, 4, 20);
}

/// The lights are carried towards white, further as Highlight Area goes
/// up, and not at all at Intensity 0 beyond the smudge.
#[test]
fn smudge_stick_brightens_the_lights() {
    let lit = |highlight, intensity| {
        let mut pm = Pixmap::filled(32, 32, Rgba8::new(170, 170, 170, 255));
        smudge_stick(&mut pm, 2, highlight, intensity);
        pm.get(16, 16).r
    };
    assert!(
        lit(0, 10) < 200,
        "a mid-light burnt out at Highlight Area 0: {}",
        lit(0, 10)
    );
    assert!(
        lit(20, 10) > 240,
        "Highlight Area 20 did not burn it out: {}",
        lit(20, 10)
    );
    assert!(
        lit(20, 0) < 180,
        "Intensity 0 still brightened: {}",
        lit(20, 0)
    );
}

/// Dark is smeared along the diagonal into light, and light is not
/// smeared into dark.
#[test]
fn smudge_stick_drags_dark_along_the_diagonal() {
    // Straight edges, which the median keeps; a corner it rounds away.
    // A diagonal stroke smears both a band and a stripe; a horizontal or
    // vertical one only smears one of them.
    let smeared = |rect| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(150, 150, 150, 255));
        pm.fill_rect(rect, Rgba8::new(0, 0, 0, 255));
        smudge_stick(&mut pm, 10, 0, 0);
        pm
    };
    let band = smeared(crate::photorust::pixmap::Rect::new(0, 20, 64, 20));
    let stripe = smeared(crate::photorust::pixmap::Rect::new(20, 0, 20, 64));
    assert!(
        band.get(30, 17).r < 140,
        "nothing smeared over the band: {}",
        band.get(30, 17).r
    );
    assert!(
        stripe.get(43, 30).r < 140,
        "nothing smeared past the stripe: {}",
        stripe.get(43, 30).r
    );
    assert!(
        band.get(30, 30).r < 30,
        "the dark was washed out: {}",
        band.get(30, 30).r
    );
}

#[test]
fn smudge_stick_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    smudge_stick(&mut pm, 2, 10, 10);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn smudge_stick_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    smudge_stick(&mut pm, 2, 0, 10);
}

/// A flat field comes back blotched, lighter and darker, and more so as
/// Definition rises; at Definition 0 it stays flat.
#[test]
fn sponge_blotches_a_flat_field() {
    let spread = |definition| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(128, 128, 128, 255));
        sponge(&mut pm, 2, definition, 5);
        let values: Vec<u8> = pm.as_bytes().chunks_exact(4).map(|p| p[0]).collect();
        (
            values.iter().min().copied().unwrap(),
            values.iter().max().copied().unwrap(),
        )
    };
    assert_eq!(spread(0), (128, 128));
    let (lo, hi) = spread(12);
    assert!(lo < 118 && hi > 138, "not blotched: {lo}..{hi}");
    let (lo2, hi2) = spread(25);
    assert!(hi2 - lo2 > hi - lo, "more Definition did not blotch harder");
}

/// A bigger brush lays bigger blotches: fewer changes from one pixel to
/// the next.
#[test]
fn sponge_brush_size_sizes_the_blotches() {
    let busy = |size| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(128, 128, 128, 255));
        sponge(&mut pm, size, 20, 5);
        restlessness(&pm, 4..92)
    };
    assert!(busy(0) > busy(10) * 2, "{} against {}", busy(0), busy(10));
}

#[test]
fn sponge_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    sponge(&mut pm, 2, 12, 5);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn sponge_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    sponge(&mut pm, 2, 12, 5);
}

/// A bigger brush washes a sharp edge out further.
#[test]
fn underpainting_brush_size_softens_the_picture() {
    let edge = |size| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(40, 40, 40, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(32, 0, 32, 64),
            Rgba8::new(220, 220, 220, 255),
        );
        underpaint(&mut pm, size, 0, 0);
        pm.get(29, 32).r
    };
    assert!(edge(20) > edge(0) + 30, "{} against {}", edge(20), edge(0));
}

/// Texture Coverage breaks a straight edge into the texture's pattern.
#[test]
fn underpainting_coverage_breaks_an_edge() {
    let ragged = |coverage| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(40, 40, 40, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(32, 0, 32, 64),
            Rgba8::new(220, 220, 220, 255),
        );
        underpaint(&mut pm, 0, coverage, 0);
        restlessness(&pm, 26..38)
    };
    assert!(
        ragged(40) > ragged(0) + 500,
        "{} against {}",
        ragged(40),
        ragged(0)
    );
}

#[test]
fn underpainting_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    underpaint(&mut pm, 6, 16, 4);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}

#[test]
fn underpainting_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    underpaint(&mut pm, 6, 16, 4);
}

/// Shadow Intensity takes a mid-dark tone to black once it is up, and
/// leaves a light one alone.
#[test]
fn watercolor_shadow_intensity_blackens_the_darker_tones() {
    let tone = |value, shadow| {
        let mut pm = Pixmap::filled(32, 32, Rgba8::new(value, value, value, 255));
        watercolor(&mut pm, 9, shadow, 1);
        pm.get(16, 16).r
    };
    assert!(
        tone(100, 0) > 60,
        "a mid-dark went black at 0: {}",
        tone(100, 0)
    );
    assert!(
        tone(100, 6) < 15,
        "a mid-dark stayed at 6: {}",
        tone(100, 6)
    );
    assert!(
        tone(220, 6) > 180,
        "a light tone went dark: {}",
        tone(220, 6)
    );
}

/// Less detail washes a small mark away; more keeps it.
#[test]
fn watercolor_brush_detail_sets_the_wash() {
    let kept = |detail| {
        let mut pm = Pixmap::filled(48, 48, Rgba8::new(200, 200, 200, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(22, 22, 5, 5),
            Rgba8::new(90, 90, 90, 255),
        );
        watercolor(&mut pm, detail, 0, 1);
        pm.get(24, 24).r
    };
    assert!(kept(14) < 150, "full detail lost the mark: {}", kept(14));
    assert!(
        kept(1) > 170,
        "the broadest wash kept the mark: {}",
        kept(1)
    );
}

#[test]
fn watercolor_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = pm.clone();
    watercolor(&mut pm, 9, 1, 1);
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&pm), alpha(&before));
}
