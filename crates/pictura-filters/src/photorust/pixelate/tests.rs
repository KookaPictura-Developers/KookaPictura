use super::*;

fn flat(size: u32, colour: Rgba8) -> Pixmap {
    Pixmap::filled(size, size, colour)
}

/// How much of the image ended up dark, as a fraction.
fn inked(px: &Pixmap) -> f32 {
    let total = (px.width() * px.height()) as f32;
    let dark = (0..px.height() as i32)
        .flat_map(|y| (0..px.width() as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let p = px.get(x, y);
            (p.r as u32 + p.g as u32 + p.b as u32) < 600
        })
        .count();
    dark as f32 / total
}

#[test]
fn facet_never_invents_a_colour() {
    // The property that makes it a facet rather than a blur: every colour
    // it lays down was really in the picture somewhere. An average, or a
    // per-channel median taking its red from one neighbour and its green
    // from another, would fail this and would soften the picture instead
    // of clumping it.
    let source = many_colours(64);
    let mut after = source.clone();
    facet(&mut after);

    let mut present = std::collections::HashSet::new();
    for chunk in source.as_bytes().chunks_exact(4) {
        present.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    for chunk in after.as_bytes().chunks_exact(4) {
        assert!(
            present.contains(&[chunk[0], chunk[1], chunk[2], chunk[3]]),
            "Facet invented the colour {:?}",
            chunk
        );
    }
}

#[test]
fn facet_lays_flat_patches_across_a_smooth_gradient() {
    // The failure this pins is the one that shipped: a Facet built only on
    // the medoid handed every smoothly graded area straight back, because
    // the most typical of nine values along a gradient is the one already
    // in the middle. Applied to a photograph it tidied the grain and left
    // every petal and every sky exactly as it found them.
    //
    // A clean ramp, so there is no grain to hide behind, and a gentle one
    // — the shallow gradients of a petal or a sky are the ones the old
    // version left untouched.
    let size = 128i32;
    let mut px = Pixmap::new(size as u32, size as u32);
    for y in 0..size {
        for x in 0..size {
            let v = (100 + x * 60 / size) as u8;
            px.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    let before = distinct_colours(&px);
    facet(&mut px);
    let after = distinct_colours(&px);

    assert!(
        after * 2 < before,
        "a smooth ramp came back with {} of its {} tones, so no patches were laid down",
        after,
        before
    );

    // ...and the patches are flat: somewhere along the middle row there
    // has to be a run of pixels all the same.
    let mut longest = 1;
    let mut run = 1;
    for x in 1..size {
        if px.get(x, 64) == px.get(x - 1, 64) {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 1;
        }
    }
    assert!(longest >= 3, "the longest flat run was {} pixels", longest);
}

#[test]
fn facet_swallows_a_lone_speck() {
    // Clumping similar colours together means an odd one out loses.
    let mut px = flat(32, Rgba8::new(120, 120, 120, 255));
    px.set(16, 16, Rgba8::new(255, 0, 0, 255));
    facet(&mut px);
    assert_eq!(
        px.get(16, 16),
        Rgba8::new(120, 120, 120, 255),
        "the speck survived"
    );
}

/// A gradient with grain on it — a photograph, in miniature.
fn speckled(size: u32) -> Pixmap {
    let mut px = Pixmap::new(size, size);
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            let ramp = (x + y) * 200 / (size as i32 * 2);
            let grain = ((x * 37 + y * 91) % 23) - 11;
            let v = (ramp + grain).clamp(0, 255) as u8;
            px.set(x, y, Rgba8::new(v, v / 2 + 60, 255 - v, 255));
        }
    }
    px
}

#[test]
fn repeating_facet_grows_the_patches() {
    // CS6's Facet takes no settings, so repeating it is the only control
    // there is — which means each pass has to go further than the last.
    let source = speckled(96);
    let after_passes = |n: usize| {
        let mut px = source.clone();
        for _ in 0..n {
            facet(&mut px);
        }
        distinct_colours(&px)
    };
    assert!(
        after_passes(4) < after_passes(1),
        "a fourth pass of Facet left as many colours as the first: {} against {}",
        after_passes(4),
        after_passes(1)
    );
}

#[test]
fn mezzotint_leaves_nothing_between_the_corners_of_the_colour_cube() {
    // The defining property: every channel goes all the way to one end or
    // the other. Anything in between means the dither has become a blend,
    // and the picture would come back grey rather than as confetti.
    let mut px = many_colours(64);
    mezzotint(&mut px, MezzotintType::GrainyDots);
    for chunk in px.as_bytes().chunks_exact(4) {
        for &channel in &chunk[..3] {
            assert!(
                channel == 0 || channel == 255,
                "a channel came back as {}",
                channel
            );
        }
    }
}

#[test]
fn white_and_black_survive_untouched() {
    // The two ends of the dither. A threshold that let white flicker would
    // leave grubby specks all over a highlight.
    for (level, expected) in [(255u8, 255u8), (0, 0)] {
        let mut px = flat(48, Rgba8::new(level, level, level, 255));
        mezzotint(&mut px, MezzotintType::FineDots);
        for y in 0..48 {
            for x in 0..48 {
                assert_eq!(
                    px.get(x, y).r,
                    expected,
                    "{} came back wrong at {},{}",
                    level,
                    x,
                    y
                );
            }
        }
    }
}

#[test]
fn one_draw_decides_all_three_channels() {
    // Drawing a threshold per channel is the obvious reading of "each
    // plate is screened" and it is wrong: independent draws let every
    // pixel pick its three answers out of a hat, so all eight corners of
    // the colour cube turn up everywhere and a dark green ground comes
    // back carrying red and blue specks it has no red or blue to justify.
    //
    // With one draw the channels cross it in order of their own strength,
    // so a region can only land on the colours between black and itself.
    // Measured on the ground of the photograph this was found on.
    let ground = Rgba8::new(48, 71, 10, 255);
    let mut px = flat(200, ground);
    mezzotint(&mut px, MezzotintType::FineDots);

    // Green is the strongest channel here, then red, then blue — so the
    // only colours reachable are black, green, yellow and white.
    let mut seen = std::collections::HashSet::new();
    for chunk in px.as_bytes().chunks_exact(4) {
        seen.insert((chunk[0] > 0, chunk[1] > 0, chunk[2] > 0));
    }
    let allowed = [
        (false, false, false),
        (false, true, false),
        (true, true, false),
        (true, true, true),
    ];
    for colour in &seen {
        assert!(
            allowed.contains(colour),
            "a dark green ground produced {:?}, which it has nothing to make it from",
            colour
        );
    }
    // ...and it is not trivially passing by coming back all black.
    assert!(seen.contains(&(false, true, false)), "no green at all");
    assert!(seen.contains(&(true, true, false)), "no yellow at all");
}

#[test]
fn the_dither_is_drawn_across_the_whole_range() {
    // The tempting alternative is a threshold confined to a band around
    // mid-grey, so that lights go solid, darks stay dark and only the
    // midtones break up. It looks more like a mezzotint in the abstract
    // and it is wrong: CS6 speckles a near-white petal and carpets a dark
    // ground, and a band does neither. Pinned by the numbers, because
    // both models produce something that looks like a mezzotint.
    let lit = |level: u8| {
        let mut px = flat(200, Rgba8::new(level, level, level, 255));
        mezzotint(&mut px, MezzotintType::FineDots);
        let count = (0..200i32)
            .flat_map(|y| (0..200i32).map(move |x| (x, y)))
            .filter(|&(x, y)| px.get(x, y).r > 0)
            .count();
        count as f32 / (200.0 * 200.0)
    };

    for level in [40u8, 90, 128, 190, 225] {
        let wanted = level as f32 / 255.0;
        let got = lit(level);
        assert!(
            (got - wanted).abs() < 0.05,
            "a tone of {} came back {:.0}% lit where the tone itself is {:.0}%",
            level,
            got * 100.0,
            wanted * 100.0
        );
    }
}

#[test]
fn a_lighter_area_keeps_more_of_its_ink() {
    // The same thing over a wider span and without the arithmetic: three
    // greys, and the count of lit pixels has to follow them.
    let lit = |level: u8| {
        let mut px = flat(128, Rgba8::new(level, level, level, 255));
        mezzotint(&mut px, MezzotintType::FineDots);
        (0..128i32)
            .flat_map(|y| (0..128i32).map(move |x| (x, y)))
            .filter(|&(x, y)| px.get(x, y).r > 0)
            .count()
    };
    let dark = lit(48);
    let mid = lit(128);
    let light = lit(208);
    assert!(
        dark < mid && mid < light,
        "the dither did not follow the tone: {}, {}, {}",
        dark,
        mid,
        light
    );
}

#[test]
fn the_line_types_run_sideways() {
    // What separates a line from a dot is that its grain has a direction.
    // Built on a square cell by mistake, Long Lines would still produce a
    // perfectly convincing mezzotint — just the same one as Coarse Dots.
    let mut px = flat(200, Rgba8::new(128, 128, 128, 255));
    mezzotint(&mut px, MezzotintType::LongLines);

    // How often the pattern changes along a row, against down a column.
    let changes = |horizontal: bool| {
        let mut count = 0;
        for a in 20..180 {
            for b in 1..200 {
                let (x0, y0, x1, y1) = if horizontal {
                    (b - 1, a, b, a)
                } else {
                    (a, b - 1, a, b)
                };
                if px.get(x0, y0).r != px.get(x1, y1).r {
                    count += 1;
                }
            }
        }
        count
    };
    let across = changes(true);
    let down = changes(false);
    assert!(
        down > across * 3,
        "Long Lines changes {} times across and {} times down, which is not a line",
        across,
        down
    );
}

#[test]
fn lines_come_in_every_length() {
    // CS6's lines are streaks of random length, a pixel or two up to a few
    // hundred. Laid on cells of a fixed width, every dash comes out the
    // same length and the result reads as ruled hatching.
    let mut px = flat(400, Rgba8::new(64, 64, 64, 255));
    mezzotint(&mut px, MezzotintType::MediumLines);
    let mut runs = Vec::new();
    for y in 0..400 {
        let mut start = 0;
        for x in 1..=400 {
            if x == 400 || px.get(x, y).r != px.get(x - 1, y).r {
                if start > 0 && x < 400 {
                    runs.push(x - start);
                }
                start = x;
            }
        }
    }
    runs.sort_unstable();
    let at = |q: f32| runs[((runs.len() - 1) as f32 * q) as usize];
    assert!(
        at(0.25) <= 5,
        "a quarter of the runs should be short, but the quartile is {}",
        at(0.25)
    );
    assert!(
        at(0.9) >= 25,
        "a tenth of the runs should be long, but the 90th percentile is {}",
        at(0.9)
    );
}

#[test]
fn strokes_are_taller_than_lines() {
    // A stroke is a line with some height to it: CS6's rows agree with
    // the one below in a stroke pattern, and are independent in a line
    // pattern. Without that the two families are the same filter.
    let changes_down = |kind| {
        let mut px = flat(200, Rgba8::new(128, 128, 128, 255));
        mezzotint(&mut px, kind);
        (1..200)
            .flat_map(|y| (0..200).map(move |x| (x, y)))
            .filter(|&(x, y)| px.get(x, y).r != px.get(x, y - 1).r)
            .count()
    };
    let lines = changes_down(MezzotintType::ShortLines);
    let strokes = changes_down(MezzotintType::ShortStrokes);
    assert!(
        strokes * 4 < lines * 3,
        "Short Strokes changes {} times down a column and Short Lines {}",
        strokes,
        lines
    );
}

#[test]
fn longer_lines_are_harsher() {
    // CS6's longer line types throw a dark grey high less often: their
    // thresholds bunch closer to mid-grey, which is what lets a light sky
    // go nearly solid and its few dark streaks run on.
    let lit = |kind| {
        let mut px = flat(200, Rgba8::new(60, 60, 60, 255));
        mezzotint(&mut px, kind);
        px.as_bytes().chunks_exact(4).filter(|p| p[0] > 0).count()
    };
    let short = lit(MezzotintType::ShortLines);
    let medium = lit(MezzotintType::MediumLines);
    let long = lit(MezzotintType::LongLines);
    assert!(
        short > medium && medium > long,
        "a dark grey was lit {}, {} and {} times by Short, Medium and Long Lines",
        short,
        medium,
        long
    );
}

#[test]
fn a_coarser_grain_means_fewer_specks() {
    let specks = |kind| {
        let mut px = flat(160, Rgba8::new(128, 128, 128, 255));
        mezzotint(&mut px, kind);
        let mut count = 0;
        for y in 0..160 {
            for x in 1..160 {
                if px.get(x, y).r != px.get(x - 1, y).r {
                    count += 1;
                }
            }
        }
        count
    };
    assert!(
        specks(MezzotintType::FineDots) > specks(MezzotintType::CoarseDots),
        "Fine Dots is no finer than Coarse Dots"
    );
}

#[test]
fn mezzotint_is_deterministic() {
    let run = || {
        let mut px = many_colours(64);
        mezzotint(&mut px, MezzotintType::ShortStrokes);
        px
    };
    assert_eq!(run().as_bytes(), run().as_bytes());
}

#[test]
fn fragment_makes_four_copies_at_the_corners_of_a_square() {
    // This is the whole filter, and the arrangement is the part worth
    // pinning: four copies at the compass points instead average out to
    // something indistinguishable from a small blur.
    let mut px = Pixmap::filled(41, 41, Rgba8::BLACK);
    px.set(20, 20, Rgba8::WHITE);
    fragment(&mut px);

    let lit = |x: i32, y: i32| px.get(x, y).r > 0;
    for (dx, dy) in [(-4, -4), (4, -4), (-4, 4), (4, 4)] {
        assert!(lit(20 + dx, 20 + dy), "no copy landed at {},{}", dx, dy);
    }
    assert!(!lit(20, 20), "the original was left where it was");
    // The compass points, which a diamond of offsets would have lit.
    for (dx, dy) in [(0, -4), (0, 4), (-4, 0), (4, 0)] {
        assert!(!lit(20 + dx, 20 + dy), "a copy landed at {},{}", dx, dy);
    }
}

#[test]
fn fragment_leaves_a_flat_image_alone() {
    // Four copies of the same thing averaged is that thing. A rounding
    // error in the averaging would show up here as a shift of a level.
    let mut px = flat(48, Rgba8::new(77, 155, 211, 255));
    fragment(&mut px);
    assert_eq!(px.get(24, 24), Rgba8::new(77, 155, 211, 255));
}

/// A picture with a different colour nearly everywhere, so that flattening
/// it into cells is measurable as a loss of variety.
fn many_colours(size: u32) -> Pixmap {
    let mut px = Pixmap::new(size, size);
    for y in 0..size as i32 {
        for x in 0..size as i32 {
            px.set(
                x,
                y,
                Rgba8::new(
                    (x * 3 % 256) as u8,
                    (y * 5 % 256) as u8,
                    ((x + y) % 256) as u8,
                    255,
                ),
            );
        }
    }
    px
}

fn distinct_colours(px: &Pixmap) -> usize {
    let mut seen = std::collections::HashSet::new();
    for chunk in px.as_bytes().chunks_exact(4) {
        seen.insert([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    seen.len()
}

#[test]
fn pointillize_paints_the_gaps_in_the_background_colour() {
    // The part of the filter that is not obvious from looking at it: the
    // canvas between the dabs is the *background colour*, not what was
    // there before and not black. On a blue ground over a red picture,
    // every pixel has to be one or the other.
    let ground = Rgba8::new(0, 0, 255, 255);
    let mut px = flat(120, Rgba8::new(255, 0, 0, 255));
    pointillize(&mut px, 9, ground);

    let mut dabs = 0;
    let mut canvas = 0;
    for y in 0..120i32 {
        for x in 0..120i32 {
            match px.get(x, y) {
                p if p == Rgba8::new(255, 0, 0, 255) => dabs += 1,
                p if p == ground => canvas += 1,
                other => panic!("{:?} at {},{} is neither dab nor canvas", other, x, y),
            }
        }
    }
    assert!(dabs > 0, "nothing was painted");
    assert!(
        canvas > 0,
        "the dabs covered everything, so no ground shows"
    );
}

#[test]
fn the_dabs_carry_the_colour_they_cover() {
    // Half red and half green, and each half must come back in its own
    // colour rather than in one average of the two.
    let size = 120i32;
    let mut px = Pixmap::new(size as u32, size as u32);
    for y in 0..size {
        for x in 0..size {
            let colour = if x < size / 2 {
                Rgba8::new(220, 0, 0, 255)
            } else {
                Rgba8::new(0, 220, 0, 255)
            };
            px.set(x, y, colour);
        }
    }
    pointillize(&mut px, 8, Rgba8::WHITE);

    let reddest = |x0: i32, x1: i32| {
        let mut red = 0;
        let mut green = 0;
        for y in 10..110 {
            for x in x0..x1 {
                let p = px.get(x, y);
                if p.r > p.g {
                    red += 1;
                } else if p.g > p.r {
                    green += 1;
                }
            }
        }
        (red, green)
    };
    let (left_red, left_green) = reddest(5, 45);
    let (right_red, right_green) = reddest(75, 115);
    assert!(left_red > left_green * 4, "the red half came back green");
    assert!(right_green > right_red * 4, "the green half came back red");
}

#[test]
fn the_dabs_crowd_together_without_closing_up() {
    // Two failures either side of this. Too small and the picture reads as
    // specks on a white sheet rather than as paint; too large and the
    // dabs merge into a blur with no ground between them and none of the
    // texture the filter exists for.
    let mut px = flat(200, Rgba8::new(200, 40, 40, 255));
    pointillize(&mut px, 6, Rgba8::WHITE);
    let ground = (0..200i32)
        .flat_map(|y| (0..200i32).map(move |x| (x, y)))
        .filter(|&(x, y)| px.get(x, y) == Rgba8::WHITE)
        .count() as f32
        / (200.0 * 200.0);
    assert!(
        (0.10..0.40).contains(&ground),
        "{:.0}% of the picture came back as bare ground",
        ground * 100.0
    );
}

#[test]
fn neighbouring_dabs_do_not_all_carry_the_same_colour() {
    // Loading each dab with the average over its own cell is the obvious
    // thing to do, and it flattens the picture: neighbouring dabs come out
    // nearly the same colour and the result is a smooth field of dots
    // rather than something mixed on a palette. The texture in the picture
    // has to survive into the dabs.
    //
    // Measured at a coarse cell, where the two ways of loading a dab are
    // furthest apart: a twenty-pixel average leaves almost none of the
    // grain, and a three-by-three one leaves most of it.
    let size = 300i32;
    let ramp = |x: i32| 90.0 + x as f32 * 60.0 / size as f32;
    let mut src = Pixmap::new(size as u32, size as u32);
    for y in 0..size {
        for x in 0..size {
            let grain = ((x * 37 + y * 91) % 81) - 40;
            let v = (ramp(x) + grain as f32).clamp(0.0, 255.0) as u8;
            src.set(x, y, Rgba8::new(v, 200 - v / 2, 120, 255));
        }
    }

    let mut px = src.clone();
    pointillize(&mut px, 20, Rgba8::WHITE);

    // With the ramp taken out, whatever spread is left is the texture.
    let mut residuals = Vec::new();
    for y in 10..size - 10 {
        for x in 10..size - 10 {
            let p = px.get(x, y);
            if p != Rgba8::WHITE {
                residuals.push(p.r as f32 - ramp(x));
            }
        }
    }
    let mean = residuals.iter().sum::<f32>() / residuals.len() as f32;
    let spread = (residuals
        .iter()
        .map(|v| (v - mean) * (v - mean))
        .sum::<f32>()
        / residuals.len() as f32)
        .sqrt();
    assert!(
        spread > 4.0,
        "the dabs vary by only {:.1} levels around the tone they sit on, so the picture \
         was averaged flat before it was painted",
        spread
    );
}

#[test]
fn a_larger_cell_makes_larger_dabs() {
    // Cell Size is the only control there is. Measured as how often a row
    // crosses between a dab and the ground, which falls as the dabs grow.
    let crossings = |cell| {
        let mut px = flat(200, Rgba8::new(200, 40, 40, 255));
        pointillize(&mut px, cell, Rgba8::WHITE);
        let mut count = 0;
        for y in (10..190).step_by(5) {
            for x in 1..200 {
                if (px.get(x, y).g > 128) != (px.get(x - 1, y).g > 128) {
                    count += 1;
                }
            }
        }
        count
    };
    assert!(
        crossings(5) > crossings(25),
        "a fine pointillize has no more dabs across a row than a coarse one"
    );
}

#[test]
fn pointillize_is_deterministic() {
    let run = || {
        let mut px = many_colours(96);
        pointillize(&mut px, 7, Rgba8::WHITE);
        px
    };
    assert_eq!(run().as_bytes(), run().as_bytes());
}

#[test]
fn pointillize_does_not_paint_outside_the_layer() {
    // A layer's empty half is not canvas to be painted on; it is nothing.
    let mut px = Pixmap::new(80, 80);
    for y in 0..80 {
        for x in 0..40 {
            px.set(x, y, Rgba8::new(200, 60, 60, 255));
        }
    }
    pointillize(&mut px, 6, Rgba8::WHITE);
    for y in 0..80 {
        assert_eq!(
            px.get(70, y).a,
            0,
            "the empty half of the layer was painted on"
        );
    }
}

#[test]
fn mosaic_fills_each_square_with_one_colour() {
    // The defining property, and the one that separates it from a blur:
    // a tile is flat, and its edges land on the grid.
    let source = many_colours(100);
    let mut after = source.clone();
    let cell = 10i32;
    mosaic(&mut after, cell as u32);

    for tile_y in 0..10 {
        for tile_x in 0..10 {
            let first = after.get(tile_x * cell, tile_y * cell);
            for y in 0..cell {
                for x in 0..cell {
                    assert_eq!(
                        after.get(tile_x * cell + x, tile_y * cell + y),
                        first,
                        "the tile at {},{} is not one colour",
                        tile_x,
                        tile_y
                    );
                }
            }
        }
    }
}

#[test]
fn a_mosaic_tile_is_the_average_of_what_was_under_it() {
    // Sampling the middle of the square instead would look identical on a
    // photograph and would throw away the tone of everything else in it.
    let source = many_colours(64);
    let mut after = source.clone();
    mosaic(&mut after, 8);

    let mut total = 0u32;
    for y in 0..8i32 {
        for x in 0..8i32 {
            total += source.get(x, y).r as u32;
        }
    }
    let mean = (total / 64) as i32;
    assert!(
        (after.get(3, 3).r as i32 - mean).abs() <= 1,
        "the first tile came back {} where its square averages {}",
        after.get(3, 3).r,
        mean
    );
}

#[test]
fn a_clipped_edge_tile_averages_only_what_is_there() {
    // The squares at the right and bottom edges are cut short when the
    // picture is not a whole number of cells across. Counting the pixels
    // that are not there would darken those two strips — a shadow down
    // the edge of every mosaic, on some images and not others.
    let mut px = flat(101, Rgba8::WHITE);
    mosaic(&mut px, 10);
    for y in 0..101 {
        for x in 0..101 {
            assert_eq!(px.get(x, y), Rgba8::WHITE, "at {},{}", x, y);
        }
    }
}

#[test]
fn a_larger_mosaic_cell_means_fewer_of_them() {
    let source = many_colours(160);
    let colours = |cell| {
        let mut px = source.clone();
        mosaic(&mut px, cell);
        distinct_colours(&px)
    };
    assert!(
        colours(8) > colours(40),
        "a fine mosaic has no more tiles than a coarse one"
    );
}

#[test]
fn crystallize_flattens_the_picture_into_cells() {
    let source = many_colours(120);
    let mut after = source.clone();
    crystallize(&mut after, 12);

    let before = distinct_colours(&source);
    let now = distinct_colours(&after);
    assert!(
        now * 10 < before,
        "the picture kept {} of its {} colours, so it was not flattened into cells",
        now,
        before
    );
}

#[test]
fn a_larger_cell_size_means_fewer_cells() {
    // Cell Size is the whole interface. Wired to nothing, or to the wrong
    // end of the scale, the filter still produces a convincing crystal.
    let source = many_colours(160);
    let colours = |cell| {
        let mut px = source.clone();
        crystallize(&mut px, cell);
        distinct_colours(&px)
    };
    assert!(
        colours(8) > colours(40),
        "a fine crystal has no more cells than a coarse one: {} against {}",
        colours(8),
        colours(40)
    );
}

#[test]
fn the_cells_are_not_square_tiles() {
    // What separates Crystallize from Mosaic is that the seeds are nudged
    // off their lattice. Left on it, the cells come out as plain squares
    // and every boundary along a row falls on a multiple of the cell
    // size — which is a different filter with the same slider.
    let source = many_colours(200);
    let mut after = source.clone();
    let cell = 20u32;
    crystallize(&mut after, cell);

    let mut off_lattice = 0;
    for y in (10..190).step_by(7) {
        for x in 1..200 {
            if after.get(x, y) != after.get(x - 1, y) && !(x as u32).is_multiple_of(cell) {
                off_lattice += 1;
            }
        }
    }
    assert!(
        off_lattice > 20,
        "every cell boundary landed on the lattice, so these are square tiles"
    );
}

#[test]
fn crystallize_is_deterministic() {
    // The seeds are jittered from a hash of their own coordinates rather
    // than from a running random source, because undo and redo replay the
    // filter and a crystal that came out differently each time could not
    // be undone.
    let source = many_colours(96);
    let run = || {
        let mut px = source.clone();
        crystallize(&mut px, 9);
        px
    };
    assert_eq!(run().as_bytes(), run().as_bytes());
}

#[test]
fn a_flat_image_crystallizes_to_itself() {
    // Every cell averages the same colour, so nothing should move — and a
    // cell that reached outside the picture and averaged in nothing would
    // show up here as a darker patch.
    let mut px = flat(80, Rgba8::new(90, 140, 200, 255));
    crystallize(&mut px, 11);
    for y in 0..80 {
        for x in 0..80 {
            assert_eq!(
                px.get(x, y),
                Rgba8::new(90, 140, 200, 255),
                "at {},{}",
                x,
                y
            );
        }
    }
}

#[test]
fn white_paper_takes_no_ink() {
    // Nothing to print. A screen that laid down dots here would fog the
    // highlights of every image it touched.
    let mut px = flat(64, Rgba8::WHITE);
    color_halftone(&mut px, 6.0, DEFAULT_SCREEN_ANGLES);
    assert_eq!(inked(&px), 0.0, "a white image came back with dots on it");
}

#[test]
fn a_darker_image_takes_more_ink_than_a_lighter_one() {
    // The whole point of a halftone: the dots grow with the tone. Three
    // greys, and the coverage has to follow them in order.
    let coverage = |level: u8| {
        let mut px = flat(96, Rgba8::new(level, level, level, 255));
        color_halftone(&mut px, 6.0, DEFAULT_SCREEN_ANGLES);
        inked(&px)
    };
    let light = coverage(200);
    let mid = coverage(128);
    let dark = coverage(48);
    assert!(
        light < mid && mid < dark,
        "coverage did not follow the tone: {:.2}, {:.2}, {:.2}",
        light,
        mid,
        dark
    );
}

#[test]
fn the_screen_angles_change_where_the_dots_fall() {
    // Four numbers that all look alike in a dialog. Wired to nothing, the
    // filter would still produce a perfectly convincing halftone — just
    // one with every plate on top of its neighbour.
    let render = |angles: ScreenAngles| {
        let mut px = flat(96, Rgba8::new(150, 90, 60, 255));
        color_halftone(&mut px, 6.0, angles);
        px
    };
    let standard = render(DEFAULT_SCREEN_ANGLES);
    let turned = render([0.0, 30.0, 60.0, 15.0]);
    assert_ne!(
        standard.as_bytes(),
        turned.as_bytes(),
        "the screen angles made no difference"
    );
}

#[test]
fn a_bigger_radius_makes_a_coarser_screen() {
    // Max Radius sets the size of a dot at full ink, and the grid follows
    // from it, so a larger one means fewer and bigger dots — not the same
    // screen printed darker.
    let dots = |radius: f32| {
        let mut px = flat(120, Rgba8::new(128, 128, 128, 255));
        color_halftone(&mut px, radius, DEFAULT_SCREEN_ANGLES);
        // Count how often a row crosses from paper to ink, which is twice
        // the number of dots it passes through.
        let mut crossings = 0;
        let mut last = px.get(0, 60).r < 128;
        for x in 1..120 {
            let now = px.get(x, 60).r < 128;
            if now != last {
                crossings += 1;
                last = now;
            }
        }
        crossings
    };
    assert!(
        dots(4.0) > dots(12.0),
        "a fine screen has no more dots across a row than a coarse one"
    );
}

#[test]
fn transparent_pixels_stay_transparent() {
    // A screen is printed on the picture, not on the space around it.
    let mut px = Pixmap::new(64, 64);
    for y in 0..64 {
        for x in 0..32 {
            px.set(x, y, Rgba8::new(40, 40, 40, 255));
        }
    }
    color_halftone(&mut px, 5.0, DEFAULT_SCREEN_ANGLES);
    for y in 0..64 {
        assert_eq!(
            px.get(50, y).a,
            0,
            "the empty half of the layer was printed on"
        );
    }
}
/// Coarse Dots is grain, not a grid of squares: neighbouring pixels
/// clump together more than Fine Dots', but the clumps do not line up on
/// four-pixel cells — a pixel is as likely to differ from the one below
/// it across a cell boundary as within a cell. (Down, because rows of
/// square cells are shifted sideways from one another, so squares only
/// line up vertically.)
#[test]
fn coarse_dots_are_clumps_not_squares() {
    let run = |kind| {
        let mut px = flat(160, Rgba8::new(128, 128, 128, 255));
        mezzotint(&mut px, kind);
        px
    };
    // How often a pixel differs from the one below it, split by whether
    // that pair straddles a multiple of four.
    let changes = |px: &Pixmap| {
        let (mut on, mut n_on, mut off, mut n_off) = (0.0, 0.0, 0.0, 0.0);
        for y in 0..159 {
            for x in 0..160 {
                let d = (px.get(x, y).r != px.get(x, y + 1).r) as u32 as f32;
                if (y + 1) % 4 == 0 {
                    on += d;
                    n_on += 1.0;
                } else {
                    off += d;
                    n_off += 1.0;
                }
            }
        }
        (on / n_on, off / n_off)
    };
    let (_, fine_off) = changes(&run(MezzotintType::FineDots));
    let (coarse_on, coarse_off) = changes(&run(MezzotintType::CoarseDots));
    assert!(
        coarse_off < fine_off * 0.7,
        "coarse {coarse_off} vs fine {fine_off}: no clumping"
    );
    assert!(
        (coarse_on - coarse_off).abs() < 0.05,
        "changes fall on a four-pixel grid: {coarse_on} across cells, {coarse_off} within"
    );
}

/// Each pixel is decided on its own tone, so a hard edge in the picture
/// stays where it was rather than being smeared across a cell.
#[test]
fn coarse_dots_keep_a_hard_edge() {
    let mut px = flat(64, Rgba8::new(0, 0, 0, 255));
    px.fill_rect(
        crate::photorust::pixmap::Rect::new(31, 0, 33, 64),
        Rgba8::new(255, 255, 255, 255),
    );
    mezzotint(&mut px, MezzotintType::CoarseDots);
    for y in 0..64 {
        assert_eq!(px.get(30, y).r, 0, "the black side lit up at row {y}");
        assert_eq!(px.get(31, y).r, 255, "the white side went dark at row {y}");
    }
}
