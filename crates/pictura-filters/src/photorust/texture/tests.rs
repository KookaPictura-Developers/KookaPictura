use super::*;
use crate::photorust::pixmap::Rgba8;

fn glass_at(pixmap: &Pixmap, x: i32, y: i32) -> [u8; 3] {
    let p = pixmap.get(x, y);
    [p.r, p.g, p.b]
}

#[test]
fn stained_glass_leads_in_the_foreground_colour_and_frames_the_image() {
    // The lead is the foreground colour, as CS6's is, and it runs round
    // the edge of the image as well as between the panes.
    let lead = Rgba8::new(200, 20, 30, 255);
    let mut px = Pixmap::filled(120, 90, Rgba8::new(40, 120, 220, 255));
    stained_glass(&mut px, 10, 6, 0, lead);
    for (x, y) in [(0, 0), (119, 0), (0, 89), (119, 89), (60, 0), (0, 45)] {
        assert_eq!(glass_at(&px, x, y), [200, 20, 30], "no lead at {x},{y}");
    }
    // Everything that is not lead is the one colour the picture had:
    // a flat picture makes flat panes.
    let mut glass = 0;
    for y in 0..90 {
        for x in 0..120 {
            let c = glass_at(&px, x, y);
            if c == [40, 120, 220] {
                glass += 1;
            }
        }
    }
    assert!(
        glass > 120 * 90 / 2,
        "only {glass} pixels came back as glass"
    );
}

#[test]
fn stained_glass_panes_are_flat_averages() {
    // A ramp comes back as a handful of flat panes rather than a ramp:
    // many fewer distinct colours than it went in with.
    let mut px = Pixmap::new(200, 60);
    for y in 0..60 {
        for x in 0..200 {
            px.set(x, y, Rgba8::new(x as u8, 255 - x as u8, 128, 255));
        }
    }
    stained_glass(&mut px, 12, 1, 0, Rgba8::BLACK);
    let mut colours = std::collections::HashMap::new();
    for y in 0..60 {
        for x in 0..200 {
            *colours.entry(glass_at(&px, x, y)).or_insert(0) += 1;
        }
    }
    // A colour to a pane, and those cover the picture; what is left is
    // the soft edge of the lead, a pixel here and there.
    let panes: Vec<i32> = colours.values().copied().filter(|&n| n >= 10).collect();
    let covered: i32 = panes.iter().sum();
    assert!(
        panes.len() < 60,
        "{} colours each cover ten pixels or more",
        panes.len()
    );
    assert!(
        covered > 200 * 60 * 7 / 10,
        "the panes cover only {covered} pixels"
    );
}

#[test]
fn a_thicker_border_is_more_lead() {
    let lead = |border| {
        let mut px = Pixmap::filled(160, 160, Rgba8::new(255, 255, 255, 255));
        stained_glass(&mut px, 10, border, 0, Rgba8::BLACK);
        px.as_bytes().chunks_exact(4).filter(|p| p[0] < 128).count()
    };
    let thin = lead(2);
    let thick = lead(10);
    assert!(
        thick > thin * 2,
        "Border Thickness 10 leads {thick} pixels and 2 leads {thin}"
    );
}

#[test]
fn bigger_cells_are_fewer_panes() {
    // Leads crossed along the middle row.
    let crossings = |cell| {
        let mut px = Pixmap::filled(400, 100, Rgba8::new(255, 255, 255, 255));
        stained_glass(&mut px, cell, 3, 0, Rgba8::BLACK);
        (1..400)
            .filter(|&x| px.get(x, 50).r < 128 && px.get(x - 1, 50).r >= 128)
            .count()
    };
    let small = crossings(5);
    let big = crossings(20);
    assert!(
        small > big * 2,
        "Cell Size 5 crossed {small} leads and 20 crossed {big}"
    );
}

#[test]
fn light_intensity_lights_the_middle_and_not_the_lead() {
    // The glow is centred on the image: the glass in the middle goes
    // towards white, the glass in a corner barely moves, and the lead
    // stays the colour it was given.
    let mut px = Pixmap::filled(300, 200, Rgba8::new(60, 60, 60, 255));
    stained_glass(&mut px, 8, 6, 10, Rgba8::BLACK);
    let brightest = |x0: i32, y0: i32| {
        (y0..y0 + 20)
            .flat_map(|y| (x0..x0 + 20).map(move |x| (x, y)))
            .map(|(x, y)| px.get(x, y).r)
            .max()
            .unwrap()
    };
    let middle = brightest(140, 90);
    let corner = brightest(4, 4);
    assert!(middle > 200, "the middle only reached {middle}");
    assert!(corner < 90, "the corner was lit to {corner}");
    assert_eq!(
        glass_at(&px, 0, 100),
        [0, 0, 0],
        "the glow reached the lead"
    );
}

#[test]
fn texturizer_leaves_a_picture_alone_at_no_relief() {
    let mut px = Pixmap::filled(40, 40, Rgba8::new(90, 140, 200, 255));
    let before = px.clone();
    texturizer(&mut px, Texture::Burlap, 100, 0, Light::Top, false);
    assert_eq!(px.as_bytes(), before.as_bytes());
}

#[test]
fn texturizer_shows_as_plainly_in_black_as_in_white() {
    // CS6 lights every tone alike: its brick throws white edges across a
    // black horse as plainly as dark ones across the sky. A surface that
    // only scaled the picture's own brightness would vanish in the black.
    let spread = |level: u8| {
        let mut px = Pixmap::filled(120, 120, Rgba8::new(level, level, level, 255));
        texturizer(&mut px, Texture::Sandstone, 100, 16, Light::Top, false);
        let values: Vec<f32> = px.as_bytes().chunks_exact(4).map(|p| p[0] as f32).collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        (values.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / values.len() as f32).sqrt()
    };
    let dark = spread(30);
    let light = spread(200);
    assert!(dark > 20.0, "the surface barely shows in black: {dark:.1}");
    assert!(
        dark > light * 0.6,
        "black shows {dark:.1} of texture and white {light:.1}"
    );
}

#[test]
fn texturizer_brick_draws_its_courses() {
    // Lit from the top, each course has a lit lip and a dark joint: down
    // a column the picture swings far both ways once every course.
    let mut px = Pixmap::filled(80, 90, Rgba8::new(128, 128, 128, 255));
    texturizer(&mut px, Texture::Brick, 100, 24, Light::Top, false);
    let column: Vec<u8> = (0..90).map(|y| px.get(40, y).r).collect();
    let bright = column.iter().filter(|&&v| v > 220).count();
    let dark = column.iter().filter(|&&v| v < 40).count();
    assert!(
        bright >= 8,
        "only {bright} lit pixels down a column of ten courses"
    );
    assert!(
        dark >= 8,
        "only {dark} dark pixels down a column of ten courses"
    );
}

#[test]
fn texturizer_invert_turns_the_surface_inside_out() {
    let run = |invert| {
        let mut px = Pixmap::filled(60, 60, Rgba8::new(128, 128, 128, 255));
        texturizer(&mut px, Texture::Canvas, 100, 12, Light::Top, invert);
        px
    };
    let (plain, inverted) = (run(false), run(true));
    assert_ne!(plain.as_bytes(), inverted.as_bytes());
}

#[test]
fn stained_glass_is_deterministic() {
    let run = || {
        let mut px = Pixmap::new(90, 70);
        for y in 0..70 {
            for x in 0..90 {
                px.set(x, y, Rgba8::new((x * 3) as u8, (y * 3) as u8, 90, 255));
            }
        }
        stained_glass(&mut px, 6, 3, 4, Rgba8::BLACK);
        px
    };
    assert_eq!(run().as_bytes(), run().as_bytes());
}

#[test]
fn every_texture_stays_in_range_and_is_not_flat() {
    for texture in [
        Texture::Brick,
        Texture::Burlap,
        Texture::Canvas,
        Texture::Sandstone,
    ] {
        let field = height_map(texture, 64, 64, 100);
        let (lo, hi) = field
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)));
        assert!(lo >= 0.0 && hi <= 1.2, "{texture:?} ran from {lo} to {hi}");
        assert!(hi - lo > 0.3, "{texture:?} is nearly flat: {lo}..{hi}");
    }
}

/// Brick's mortar runs in rows, so at double the scaling the rows are
/// twice as far apart.
#[test]
fn scaling_sizes_the_texture() {
    // Averaged across a row, so the grit of the face evens out and the
    // joints between courses stand out.
    let rows = |scaling| {
        let field = height_map(Texture::Brick, 160, 200, scaling);
        let mean = |y: usize| field[y * 160..(y + 1) * 160].iter().sum::<f32>() / 160.0;
        (1..200)
            .filter(|&y| mean(y) < 0.1 && mean(y - 1) >= 0.1)
            .count()
    };
    let (normal, double) = (rows(100), rows(200));
    assert!(
        normal >= double * 2 - 1 && normal <= double * 2 + 1,
        "{normal} rows against {double}"
    );
}

/// The same edge is lit from one side and shaded from the other, and
/// Invert swaps the two.
#[test]
fn the_light_and_invert_decide_which_side_is_lit() {
    let lit = |light, invert| {
        let mut pm = Pixmap::filled(40, 40, Rgba8::new(128, 128, 128, 255));
        apply_relief(&mut pm, Texture::Brick, 100, 50, light, invert);
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[0] as i64)
            .collect::<Vec<_>>()
    };
    let (top, bottom, inverted) = (
        lit(Light::Top, false),
        lit(Light::Bottom, false),
        lit(Light::Bottom, true),
    );
    assert_ne!(top, bottom);
    let moved: i64 = top.iter().zip(&inverted).map(|(a, b)| (a - b).abs()).sum();
    assert!(
        moved < 40 * 40,
        "inverting the surface did not swap the light: {moved}"
    );
}

#[test]
fn no_relief_changes_nothing() {
    let mut pm = Pixmap::filled(16, 16, Rgba8::new(10, 200, 30, 90));
    let before = pm.clone();
    apply_relief(&mut pm, Texture::Sandstone, 100, 0, Light::Top, false);
    assert_eq!(pm.as_bytes(), before.as_bytes());
}
fn crack_tone(pm: &Pixmap) -> Vec<f32> {
    pm.as_bytes().chunks_exact(4).map(|p| p[0] as f32).collect()
}

/// A flat sheet comes out cracked: some of it much darker than the rest,
/// where the cracks are, and the more spacing the fewer of them.
#[test]
fn craquelure_cracks_a_flat_sheet_and_spacing_thins_the_cracks() {
    let dark_share = |spacing| {
        let mut pm = Pixmap::filled(160, 160, Rgba8::new(200, 200, 200, 255));
        craquelure(&mut pm, spacing, 6, 0);
        let tone = crack_tone(&pm);
        tone.iter().filter(|&&v| v < 100.0).count() as f32 / tone.len() as f32
    };
    let (tight, wide) = (dark_share(10), dark_share(80));
    assert!(tight > 0.05, "hardly any cracks at spacing 10: {tight}");
    assert!(wide > 0.005, "no cracks at spacing 80: {wide}");
    assert!(tight > wide * 2.0, "{tight} against {wide}");
}

/// Crack Brightness lifts the bottom of the cracks towards the picture.
#[test]
fn craquelure_brightness_lightens_the_cracks() {
    let darkest = |brightness| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(200, 200, 200, 255));
        craquelure(&mut pm, 15, 0, brightness);
        crack_tone(&pm).into_iter().fold(f32::MAX, f32::min)
    };
    assert!(
        darkest(10) > darkest(0) + 80.0,
        "{} vs {}",
        darkest(10),
        darkest(0)
    );
}

/// Crack Depth is how hard the plates are lit: at 0 a plate is flat, and
/// deeper spreads its tones apart.
#[test]
fn craquelure_depth_deepens_the_relief() {
    let spread = |depth| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(128, 128, 128, 255));
        // Cracks as light as the plates, so only the lighting spreads.
        craquelure(&mut pm, 15, depth, 10);
        let tone = crack_tone(&pm);
        let mean = tone.iter().sum::<f32>() / tone.len() as f32;
        (tone.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / tone.len() as f32).sqrt()
    };
    assert!(
        spread(10) > spread(0) + 10.0,
        "{} vs {}",
        spread(10),
        spread(0)
    );
}

#[test]
fn craquelure_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
    a.fill_rect(
        crate::photorust::pixmap::Rect::new(4, 4, 8, 8),
        Rgba8::new(20, 20, 20, 200),
    );
    let before = a.clone();
    let mut b = a.clone();
    craquelure(&mut a, 15, 6, 9);
    craquelure(&mut b, 15, 6, 9);
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
fn craquelure_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    craquelure(&mut pm, 15, 6, 9);
}
/// The network follows the picture: across a light sheet the cracks run
/// mostly across, so a cracked pixel's neighbour to the side is cracked
/// more often than the one below it. Across a dark sheet the paint
/// crumples every which way, and the two come out alike.
#[test]
fn craquelure_courses_the_light_and_crumples_the_dark() {
    // How much more often cracks run across than down.
    let across_over_down = |tone: u8| {
        let n = 192usize;
        let mut pm = Pixmap::filled(n as u32, n as u32, Rgba8::new(tone, tone, tone, 255));
        // No lighting and black-bottomed cracks: only the cracks show.
        craquelure(&mut pm, 20, 0, 0);
        let cracked: Vec<bool> = crack_tone(&pm)
            .iter()
            .map(|&v| v < tone as f32 * 0.6)
            .collect();
        let (mut across, mut down) = (0, 0);
        for y in 0..n - 1 {
            for x in 0..n - 1 {
                let i = y * n + x;
                if cracked[i] {
                    across += cracked[i + 1] as u32;
                    down += cracked[i + n] as u32;
                }
            }
        }
        across as f32 / down.max(1) as f32
    };
    let (light, dark) = (across_over_down(220), across_over_down(40));
    assert!(
        light > 1.3,
        "the light sheet's cracks do not run across: {light}"
    );
    assert!(
        (0.75..1.3).contains(&dark),
        "the dark sheet's cracks lean one way: {dark}"
    );
}

/// The cracks do not close into plates. CS6's are runs that meet now and
/// then and stop short, so the uncracked paint is mostly one surface
/// running round their ends; a network closed all round cuts it into
/// jigsaw pieces, none of them more than a sliver of the whole.
#[test]
fn craquelure_cracks_stop_short_rather_than_cutting_out_pieces() {
    for tone in [220u8, 40] {
        let n = 192usize;
        let mut pm = Pixmap::filled(n as u32, n as u32, Rgba8::new(tone, tone, tone, 255));
        craquelure(&mut pm, 20, 0, 0);
        let paint: Vec<bool> = crack_tone(&pm)
            .iter()
            .map(|&v| v > tone as f32 * 0.8)
            .collect();
        // The largest run of connected paint, by flood fill.
        let mut seen = vec![false; n * n];
        let mut largest = 0;
        for start in 0..n * n {
            if !paint[start] || seen[start] {
                continue;
            }
            let (mut stack, mut size) = (vec![start], 0);
            seen[start] = true;
            while let Some(i) = stack.pop() {
                size += 1;
                let (x, y) = (i % n, i / n);
                let mut visit = |j: usize| {
                    if paint[j] && !seen[j] {
                        seen[j] = true;
                        stack.push(j);
                    }
                };
                if x > 0 {
                    visit(i - 1);
                }
                if x + 1 < n {
                    visit(i + 1);
                }
                if y > 0 {
                    visit(i - n);
                }
                if y + 1 < n {
                    visit(i + n);
                }
            }
            largest = largest.max(size);
        }
        let total = paint.iter().filter(|&&p| p).count();
        // The crumple's contours do ring off the odd island, as CS6's
        // crumpled horse does; the courses hardly ever close.
        let share = if tone > 128 { 0.8 } else { 0.5 };
        assert!(
            largest as f32 > total as f32 * share,
            "tone {tone}: the cracks cut the paint into pieces, the largest {largest} of {total}"
        );
    }
}
/// The cracks are grooves, and their lit rims throw shadow. With the
/// crack floors lifted to the paint's own tone, anything darker than the
/// paint is shade, and there should be plenty of it when the cracks are
/// deep and none when they are flat.
#[test]
fn craquelure_grooves_cast_shadows() {
    let shaded = |depth| {
        let mut pm = Pixmap::filled(160, 160, Rgba8::new(128, 128, 128, 255));
        craquelure(&mut pm, 20, depth, 10);
        let tone = crack_tone(&pm);
        tone.iter().filter(|&&v| v < 128.0 * 0.7).count() as f32 / tone.len() as f32
    };
    let (deep, flat) = (shaded(10), shaded(0));
    assert!(deep > 0.04, "deep cracks threw hardly any shadow: {deep}");
    assert!(flat < 0.001, "flat cracks threw shadow: {flat}");
}
fn spread_of(pm: &Pixmap) -> f32 {
    let t = crack_tone(pm);
    let m = t.iter().sum::<f32>() / t.len() as f32;
    (t.iter().map(|v| (v - m).powi(2)).sum::<f32>() / t.len() as f32).sqrt()
}

/// Intensity is how strong the grain is: none at 0, plenty at 100.
#[test]
fn grain_intensity_strengthens_the_grain() {
    let run = |intensity| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(128, 128, 128, 255));
        grain(
            &mut pm,
            intensity,
            50,
            GrainType::Regular,
            Rgba8::BLACK,
            Rgba8::WHITE,
        );
        spread_of(&pm)
    };
    assert!(run(0) < 0.5, "grain at Intensity 0: {}", run(0));
    assert!(run(100) > 30.0, "hardly any grain at 100: {}", run(100));
}

/// Clumped grain is gathered into clumps: neighbouring pixels move
/// together, where Regular's are independent.
#[test]
fn grain_clumped_gathers_the_noise() {
    let neighbour_likeness = |kind| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::new(128, 128, 128, 255));
        grain(&mut pm, 60, 50, kind, Rgba8::BLACK, Rgba8::WHITE);
        let t = crack_tone(&pm);
        let m = t.iter().sum::<f32>() / t.len() as f32;
        let (mut together, mut apart) = (0.0, 0.0);
        for i in 0..t.len() - 1 {
            together += (t[i] - m) * (t[i + 1] - m);
            apart += (t[i] - m).powi(2);
        }
        together / apart
    };
    assert!(neighbour_likeness(GrainType::Regular) < 0.2);
    assert!(neighbour_likeness(GrainType::Clumped) > 0.5);
}

/// Stippled cuts the picture into the two swatches, and more of it into
/// the foreground where the picture is dark.
#[test]
fn grain_stippled_is_the_two_swatches() {
    let (ink, paper) = (Rgba8::new(0, 0, 255, 255), Rgba8::new(255, 255, 0, 255));
    let inked = |tone: u8| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(tone, tone, tone, 255));
        grain(&mut pm, 50, 50, GrainType::Stippled, ink, paper);
        let mut count = 0;
        for p in pm.as_bytes().chunks_exact(4) {
            assert!(p[..3] == [0, 0, 255] || p[..3] == [255, 255, 0], "{p:?}");
            count += (p[2] == 255 && p[0] == 0) as u32;
        }
        count
    };
    assert!(
        inked(60) > inked(190) + 1000,
        "{} vs {}",
        inked(60),
        inked(190)
    );
}

/// Horizontal grain bands the picture in rows: averaged along each row
/// the bands stand out, averaged down each column they cancel. Vertical
/// the other way.
#[test]
fn grain_streaks_run_their_way() {
    let lean = |kind| {
        let n = 128usize;
        let mut pm = Pixmap::filled(n as u32, n as u32, Rgba8::new(160, 160, 160, 255));
        grain(&mut pm, 80, 50, kind, Rgba8::BLACK, Rgba8::WHITE);
        let t = crack_tone(&pm);
        let spread = |means: Vec<f32>| {
            let m = means.iter().sum::<f32>() / means.len() as f32;
            (means.iter().map(|x| (x - m).powi(2)).sum::<f32>() / means.len() as f32).sqrt()
        };
        let rows = spread(
            (0..n)
                .map(|y| t[y * n..(y + 1) * n].iter().sum::<f32>() / n as f32)
                .collect(),
        );
        let cols = spread(
            (0..n)
                .map(|x| (0..n).map(|y| t[y * n + x]).sum::<f32>() / n as f32)
                .collect(),
        );
        rows / cols
    };
    assert!(
        lean(GrainType::Horizontal) > 3.0,
        "{}",
        lean(GrainType::Horizontal)
    );
    assert!(
        lean(GrainType::Vertical) < 0.33,
        "{}",
        lean(GrainType::Vertical)
    );
}

/// Speckle throws the foreground over the dark and leaves the light
/// clean; Sprinkles throws the background.
#[test]
fn grain_speckle_and_sprinkles_throw_the_swatches() {
    let ink = Rgba8::new(255, 0, 0, 255);
    let paper = Rgba8::new(0, 255, 0, 255);
    let count = |tone: u8, kind, swatch: Rgba8| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(tone, tone, tone, 255));
        grain(&mut pm, 60, 50, kind, ink, paper);
        pm.as_bytes()
            .chunks_exact(4)
            .filter(|p| p[..3] == [swatch.r, swatch.g, swatch.b])
            .count()
    };
    assert!(count(30, GrainType::Speckle, ink) > 800);
    assert!(count(250, GrainType::Speckle, ink) < 20);
    assert!(count(128, GrainType::Sprinkles, paper) > 800);
}

#[test]
fn grain_is_deterministic_and_leaves_alpha_alone() {
    use GrainType::*;
    for kind in [
        Regular, Soft, Sprinkles, Clumped, Contrasty, Enlarged, Stippled, Horizontal, Vertical,
        Speckle,
    ] {
        let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
        let before = a.clone();
        let mut b = a.clone();
        grain(&mut a, 60, 60, kind, Rgba8::BLACK, Rgba8::WHITE);
        grain(&mut b, 60, 60, kind, Rgba8::BLACK, Rgba8::WHITE);
        assert_eq!(a.as_bytes(), b.as_bytes(), "{kind:?}");
        let alpha = |pm: &Pixmap| {
            pm.as_bytes()
                .chunks_exact(4)
                .map(|p| p[3])
                .collect::<Vec<_>>()
        };
        assert_eq!(alpha(&a), alpha(&before), "{kind:?}");
    }
    grain(
        &mut Pixmap::new(0, 0),
        40,
        50,
        GrainType::Regular,
        Rgba8::BLACK,
        Rgba8::WHITE,
    );
}
/// Contrasty is contrasty at the slider's flat middle: a dark sheet goes
/// darker and a light one lighter than Regular leaves them.
#[test]
fn grain_contrasty_pushes_the_tones_apart_on_its_own() {
    let mean = |tone: u8, kind| {
        let mut pm = Pixmap::filled(64, 64, Rgba8::new(tone, tone, tone, 255));
        grain(&mut pm, 40, 50, kind, Rgba8::BLACK, Rgba8::WHITE);
        let t = crack_tone(&pm);
        t.iter().sum::<f32>() / t.len() as f32
    };
    let span = |kind| mean(200, kind) - mean(60, kind);
    assert!(
        span(GrainType::Contrasty) > span(GrainType::Regular) * 1.4,
        "{} vs {}",
        span(GrainType::Contrasty),
        span(GrainType::Regular)
    );
}

/// CS6's streaks are long: a row of Horizontal grain stays alike far along
/// it, not only between neighbours.
#[test]
fn grain_streaks_run_long() {
    let mut pm = Pixmap::filled(256, 64, Rgba8::new(140, 140, 140, 255));
    grain(
        &mut pm,
        60,
        50,
        GrainType::Horizontal,
        Rgba8::BLACK,
        Rgba8::WHITE,
    );
    let t = crack_tone(&pm);
    let m = t.iter().sum::<f32>() / t.len() as f32;
    // How alike pixels 40 apart along a row are, against 40 apart down.
    let (mut along, mut across, mut own) = (0.0, 0.0, 0.0);
    for y in 0..24 {
        for x in 0..200 {
            let i = y * 256 + x;
            along += (t[i] - m) * (t[i + 40] - m);
            across += (t[i] - m) * (t[i + 40 * 256] - m);
            own += (t[i] - m).powi(2);
        }
    }
    assert!(along / own > 0.3, "the streaks are short: {}", along / own);
    assert!(
        (across / own).abs() < 0.1,
        "rows move together: {}",
        across / own
    );
}

/// Speckle's specks sit on a mesh: the rows and columns of the mesh take
/// far more of them than the pixels between.
#[test]
fn grain_speckle_lays_its_specks_on_a_mesh() {
    let mut pm = Pixmap::filled(96, 96, Rgba8::new(90, 90, 90, 255));
    grain(
        &mut pm,
        50,
        50,
        GrainType::Speckle,
        Rgba8::new(255, 0, 0, 255),
        Rgba8::WHITE,
    );
    let (mut on, mut off, mut n_on, mut n_off) = (0.0, 0.0, 0.0, 0.0);
    for (i, p) in pm.as_bytes().chunks_exact(4).enumerate() {
        let (x, y) = (i % 96, i / 96);
        let speck = (p[..3] == [255, 0, 0]) as u32 as f32;
        if x % GRAIN_MESH == 0 || y % GRAIN_MESH == 0 {
            on += speck;
            n_on += 1.0;
        } else {
            off += speck;
            n_off += 1.0;
        }
    }
    assert!(
        on / n_on > 2.0 * off / n_off,
        "{} vs {}",
        on / n_on,
        off / n_off
    );
}
/// How grey each pixel of a flat sheet comes out after tiling, and
/// whether it is grout: the grout is pulled towards grey, so on a black
/// sheet it is the lighter part.
fn tiled(size: u32, grout: u32, lighten: u32) -> Vec<f32> {
    let mut pm = Pixmap::filled(200, 200, Rgba8::new(0, 0, 0, 255));
    mosaic_tiles(&mut pm, size, grout, lighten);
    crack_tone(&pm)
}

/// The grout runs in a grid Tile Size apart: across a black sheet, the
/// rows that are mostly grout come round every Tile Size.
#[test]
fn mosaic_tiles_lays_a_grid_tile_size_apart() {
    let lines = |size| {
        let t = tiled(size, 4, 10);
        let row = |y: usize| t[y * 200..(y + 1) * 200].iter().sum::<f32>() / 200.0;
        let rows: Vec<f32> = (0..200).map(row).collect();
        let mean = rows.iter().sum::<f32>() / 200.0;
        (1..200)
            .filter(|&y| rows[y] > mean && rows[y - 1] <= mean)
            .count()
    };
    let (small, big) = (lines(20), lines(40));
    assert!((8..=12).contains(&small), "{small} grout lines at size 20");
    assert!((4..=6).contains(&big), "{big} grout lines at size 40");
}

/// Grout Width widens the grout, and Lighten Grout lightens it.
#[test]
fn mosaic_tiles_grout_widens_and_lightens() {
    let share = |grout| tiled(30, grout, 10).iter().filter(|&&v| v > 60.0).count();
    assert!(share(12) > share(2) * 2, "{} vs {}", share(12), share(2));
    let lightest = |lighten| tiled(30, 8, lighten).into_iter().fold(0.0f32, f32::max);
    assert!(
        lightest(10) > lightest(0) + 60.0,
        "{} vs {}",
        lightest(10),
        lightest(0)
    );
}

/// The tiles keep the picture's colour: a red sheet's tiles are red.
#[test]
fn mosaic_tiles_keep_the_picture_on_the_tiles() {
    let mut pm = Pixmap::filled(120, 120, Rgba8::new(200, 30, 30, 255));
    mosaic_tiles(&mut pm, 40, 3, 9);
    let reds = pm
        .as_bytes()
        .chunks_exact(4)
        .filter(|p| p[0] as i32 > p[1] as i32 + 100)
        .count();
    assert!(reds > 120 * 120 / 2, "only {reds} red pixels");
}

#[test]
fn mosaic_tiles_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(48, 48, Rgba8::new(120, 140, 160, 77));
    let before = a.clone();
    let mut b = a.clone();
    mosaic_tiles(&mut a, 12, 3, 9);
    mosaic_tiles(&mut b, 12, 3, 9);
    assert_eq!(a.as_bytes(), b.as_bytes());
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&a), alpha(&before));
    mosaic_tiles(&mut Pixmap::new(0, 0), 12, 3, 9);
}
/// At CS6's defaults the tiles are flat and the grout thin, so most of
/// the picture comes through as it was: tiles rounded into domes, or
/// grout laid thick, would change nearly all of it.
#[test]
fn mosaic_tiles_leave_the_picture_on_flat_faces() {
    let mut pm = Pixmap::filled(160, 160, Rgba8::new(90, 90, 90, 255));
    mosaic_tiles(&mut pm, 12, 3, 9);
    let t = crack_tone(&pm);
    let kept = t.iter().filter(|&&v| (v - 90.0).abs() <= 12.0).count();
    assert!(
        kept > t.len() / 2,
        "only {kept} of {} pixels kept their tone",
        t.len()
    );
}
/// Every square is one colour, the average of the picture under it
/// lifted by [`PATCH_LIFT`]: at Relief 0 there is no light, and a
/// square's pixels all match. Squares are twice Square Size and one
/// pixels across.
#[test]
fn patchwork_fills_each_square_with_its_average() {
    let mut pm = Pixmap::new(27, 27);
    for y in 0..27 {
        for x in 0..27 {
            let v = if (x + y) % 2 == 0 { 40 } else { 200 };
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    patchwork(&mut pm, 4, 0);
    for y in 0..9 {
        for x in 0..9 {
            assert_eq!(pm.get(x, y), pm.get(0, 0));
        }
    }
    // 41 of one and 40 of the other.
    let mean = (41.0 * 40.0 + 40.0 * 200.0) / 81.0 / 255.0;
    let lifted = 255.0 * f32::powf(mean, PATCH_LIFT);
    assert!(
        (pm.get(0, 0).r as f32 - lifted).abs() <= 3.0,
        "{:?} vs {lifted}",
        pm.get(0, 0)
    );
    assert_ne!(
        pm.get(9, 0),
        pm.get(8, 0),
        "the next square began somewhere else"
    );
}

/// Relief builds every square into a block: across a flat sheet the
/// bottom edge of each square is shaded against its face, the more so
/// the higher Relief is, and the joints between rows are much deeper than
/// the ones along a row. At Relief 0 the sheet stays flat.
#[test]
fn patchwork_relief_rims_every_square() {
    let run = |relief| {
        let mut pm = Pixmap::filled(54, 54, Rgba8::new(160, 160, 160, 255));
        patchwork(&mut pm, 4, relief);
        pm
    };
    // Squares are nine pixels: row 22 is mid-face, row 26 a bottom edge,
    // column 26 a right-hand edge.
    let band = |pm: &Pixmap| pm.get(22, 22).r as i32 - pm.get(22, 26).r as i32;
    let side = |pm: &Pixmap| pm.get(22, 22).r as i32 - pm.get(26, 22).r as i32;
    let (flat, eight, sixteen) = (run(0), run(8), run(16));
    assert_eq!(band(&flat), 0);
    assert!(band(&eight) > 25, "{}", band(&eight));
    assert!(
        band(&sixteen) > band(&eight),
        "{} vs {}",
        band(&sixteen),
        band(&eight)
    );
    assert!(
        band(&eight) > 2 * side(&eight).abs(),
        "{} vs {}",
        band(&eight),
        side(&eight)
    );
}

/// A bright square standing over a dark one throws a crisp shadow onto
/// it, as CS6's pale sky does onto the horse.
#[test]
fn patchwork_casts_shadow_below_a_raised_square() {
    let mut pm = Pixmap::filled(18, 27, Rgba8::new(60, 60, 60, 255));
    pm.fill_rect(
        crate::photorust::pixmap::Rect::new(0, 0, 18, 9),
        Rgba8::new(230, 230, 230, 255),
    );
    let mut flat = Pixmap::filled(18, 27, Rgba8::new(60, 60, 60, 255));
    patchwork(&mut pm, 4, 8);
    patchwork(&mut flat, 4, 8);
    // Just inside the top of the square under the bright one: a real
    // shadow, not a shade darker.
    let (shadowed, open) = (pm.get(6, 10).r as f32, flat.get(6, 10).r as f32);
    assert!(shadowed < open * 0.7, "{shadowed} vs {open}");
}

#[test]
fn patchwork_is_deterministic_and_leaves_alpha_alone() {
    let mut a = Pixmap::filled(47, 31, Rgba8::new(120, 140, 160, 77));
    let before = a.clone();
    let mut b = a.clone();
    patchwork(&mut a, 4, 8);
    patchwork(&mut b, 4, 8);
    assert_eq!(a.as_bytes(), b.as_bytes());
    let alpha = |pm: &Pixmap| {
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[3])
            .collect::<Vec<_>>()
    };
    assert_eq!(alpha(&a), alpha(&before));
    patchwork(&mut Pixmap::new(0, 0), 4, 8);
    patchwork(&mut Pixmap::filled(3, 3, Rgba8::WHITE), 0, 25);
}
/// The joint between two squares follows the step between them, as
/// CS6's does: where a pale square stands over a dark one below it, the
/// joint is shaded; where a dark one stands over a pale one, the pale
/// square's top edge faces the light and the joint is lit.
#[test]
fn patchwork_joints_follow_the_step() {
    // The middle row of three: pale above dark, or dark above pale. Scored
    // as the band across the joint against the two faces.
    let joint = |upper: u8, lower: u8| {
        let mut pm = Pixmap::filled(27, 27, Rgba8::new(128, 128, 128, 255));
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(0, 0, 27, 9),
            Rgba8::new(upper, upper, upper, 255),
        );
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(0, 9, 27, 9),
            Rgba8::new(lower, lower, lower, 255),
        );
        patchwork(&mut pm, 4, 8);
        let band = (6..12).map(|y| pm.get(13, y).r as f32).sum::<f32>() / 6.0;
        let faces = (pm.get(13, 4).r as f32 + pm.get(13, 13).r as f32) / 2.0;
        band / faces
    };
    let (shaded, lit) = (joint(220, 90), joint(90, 220));
    assert!(lit > shaded + 0.15, "lit {lit} vs shaded {shaded}");
}
