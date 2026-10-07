#[allow(unused_imports)]
use super::*;

#[test]
fn watercolor_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    watercolor(&mut pm, 9, 1, 1);
}

#[test]
fn paint_daubs_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    paint_daubs(&mut pm, 8, 7, DaubBrush::Simple);
}

/// The picture is rendered between the two swatches: what was dark comes
/// back as the foreground colour, what was light as the background.
#[test]
fn the_picture_is_rendered_between_the_swatches() {
    let mut pm = ramp();
    // No spread and a lamp turned right down, so nothing but the mapping
    // is being measured.
    lit_plainly(&mut pm, 0, 0);
    assert!(pm.get(1, 32).r < 12, "the dark end did not go to black");
    assert!(pm.get(62, 32).r > 200, "the light end did not go to white");
}

/// The lit end is carried towards the glow colour, so a blue tube leaves
/// the lights blue and the darks alone.
#[test]
fn the_tube_tints_what_it_lights() {
    let mut pm = ramp();
    lit(&mut pm, 0, 10);
    let light = pm.get(60, 32);
    assert!(
        light.b > light.r + 60,
        "the lit end came back untinted: {light:?}"
    );
    let dark = pm.get(2, 32);
    assert!(dark.b < 40, "the tube reached the shadows: {dark:?}");
}

/// Glow Size is how far the picture's own light carries into what is dark.
#[test]
fn glow_size_is_how_far_the_light_carries() {
    let carried = |size| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::BLACK);
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(0, 0, 48, 96),
            Rgba8::WHITE,
        );
        neon_glow(&mut pm, size, 40, TUBE, Rgba8::BLACK, Rgba8::WHITE);
        // A band well inside the dark half, where the only thing that can
        // have reached is light that carried.
        (58..78)
            .map(|x| (0..96).map(|y| pm.get(x, y).b as u32).sum::<u32>())
            .sum::<u32>()
    };
    assert!(
        carried(20) > carried(4) * 4,
        "a wide glow carried no further than a narrow one: {} against {}",
        carried(20),
        carried(4)
    );
}

/// Wound below zero it is the shadows that light up instead, which is what
/// the negative half of CS6's slider is for.
#[test]
fn a_negative_glow_size_lights_the_shadows() {
    let ground = |size| {
        let mut pm = Pixmap::filled(96, 96, Rgba8::BLACK);
        pm.fill_rect(
            crate::photorust::pixmap::Rect::new(32, 32, 32, 32),
            Rgba8::WHITE,
        );
        neon_glow(&mut pm, size, 12, TUBE, Rgba8::BLACK, Rgba8::WHITE);
        pm.get(8, 8).b as i32
    };
    assert!(
        ground(-12) > ground(12) + 60,
        "the dark ground did not light up: {} against {}",
        ground(-12),
        ground(12)
    );
}

/// Past the middle of the Glow Brightness slider the light carries the
/// brightest part of the picture past white and back out the other side,
/// so it returns as the foreground colour. This is what makes the filter's
/// two extremes look like negatives of one another, and it is deliberate.
#[test]
fn a_lamp_turned_past_white_folds_back_to_the_foreground() {
    let white_end = |brightness| {
        let mut pm = ramp();
        lit_plainly(&mut pm, 0, brightness);
        pm.get(62, 32).r as i32
    };
    assert!(white_end(10) > 180, "the lamp did not light the picture");
    assert!(
        white_end(50) < 40,
        "the lamp never folded: the light end came back at {}",
        white_end(50)
    );
}

#[test]
fn neon_glow_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    lit(&mut pm, 5, 15);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn neon_glow_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    lit(&mut pm, 5, 15);
}

/// The plaster takes the darks down and leaves the lights where they are.
/// That is the whole of what separates Fresco from Dry Brush, so it is
/// measured against Dry Brush at the same settings rather than in the
/// abstract.
#[test]
fn the_plaster_takes_the_picture_down() {
    let source = Pixmap::filled(64, 64, Rgba8::new(60, 100, 80, 255));
    let (mut painted, mut plastered) = (source.clone(), source.clone());
    dry_brush(&mut painted, 2, 8, 1);
    fresco(&mut plastered, 2, 8, 1);
    assert!(
        (plastered.get(32, 32).g as i32) * 2 < painted.get(32, 32).g as i32,
        "the plaster left the picture where the brush did: {:?} against {:?}",
        plastered.get(32, 32),
        painted.get(32, 32)
    );
}

/// ...and it takes a colour's weakest channel down hardest, which deepens
/// the colour rather than merely dimming it.
#[test]
fn the_plaster_deepens_rather_than_dims() {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(60, 180, 60, 255));
    fresco(&mut pm, 0, 10, 1);
    let got = pm.get(32, 32);
    let (was, now) = (180.0 / 60.0, got.g as f32 / (got.r as f32).max(1.0));
    assert!(
        now > was * 1.5,
        "the colour came back as flat as it went in: {got:?}"
    );
}

/// White is plaster's one exception: the top of the range stays put, so a
/// highlight is still a highlight.
#[test]
fn the_lights_stay_where_they_are() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(255, 255, 250, 255));
    fresco(&mut pm, 0, 10, 1);
    assert!(
        pm.get(16, 16).r > 240,
        "the plaster took the highlight down"
    );
}

/// Fresco paints in the same dabs as Dry Brush, so it flattens a surface
/// and keeps the boundary through it.
#[test]
fn fresco_paints_in_dabs_like_the_brush_it_shares() {
    let mut pm = two_noisy_fields();
    fresco(&mut pm, 6, 10, 1);
    let step = pm.get(32, 32).r as i32 - pm.get(31, 32).r as i32;
    assert!(step > 60, "the edge came back softened to {step} levels");
}

#[test]
fn fresco_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    fresco(&mut pm, 2, 8, 1);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn fresco_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    fresco(&mut pm, 2, 8, 1);
}

/// Grain is grain: more of it, more silver on the picture.
#[test]
fn grain_lays_silver_on_the_picture() {
    let laid = |grain| {
        let mut pm = ramp();
        film_grain(&mut pm, grain, 0, 10);
        scatter(&pm, 8..56)
    };
    assert!(laid(0) < 0.5, "no grain still moved the picture");
    assert!(
        laid(20) > laid(10) && laid(10) > laid(4),
        "the slider did nothing"
    );
}

/// CS6's defaults are Grain 4, Highlight Area 0, Intensity 10, and what
/// they give is the picture with grain on it and nothing else — the
/// highlight pair has nothing to work on until Highlight Area opens it up.
#[test]
fn with_no_highlight_area_intensity_has_nothing_to_do() {
    let lit = |highlight| {
        let mut pm = ramp();
        film_grain(&mut pm, 0, highlight, 10);
        // The light end of the ramp, where any lifting would show.
        (48..64)
            .map(|x| pm.get(x, 32).r as i32 - x * 4)
            .sum::<i32>()
    };
    assert_eq!(lit(0), 0, "the highlights moved with the area shut");
    assert!(lit(20) > 200, "opening the area lifted nothing");
}

/// Intensity is how hard the lit part is carried towards white, so it does
/// nothing at 0 and everything at 10 — once there is an area to carry.
#[test]
fn intensity_carries_the_lit_part_towards_white() {
    let lifted = |intensity| {
        let mut pm = ramp();
        film_grain(&mut pm, 0, 20, intensity);
        pm.get(60, 32).r as i32
    };
    assert_eq!(lifted(0), 240, "intensity 0 lifted the highlight anyway");
    assert!(lifted(10) > lifted(5) && lifted(5) > lifted(0));
}

/// What is under the line is left where it was. A film's shadows do not
/// glow.
#[test]
fn what_is_under_the_line_is_left_alone() {
    let mut pm = ramp();
    film_grain(&mut pm, 0, 10, 10);
    for x in 0..8 {
        assert_eq!(pm.get(x, 32).r as i32, x * 4, "a shadow was lifted");
    }
}

/// Each channel is asked where *it* stands, not the pixel's brightness. So
/// a colour with one channel over the line and two under it comes back
/// more vivid rather than washed out — CS6's grass goes lime, not grey.
#[test]
fn a_colour_with_one_channel_in_the_light_goes_vivid() {
    let grass = Rgba8::new(40, 90, 30, 255);
    let mut pm = Pixmap::filled(32, 32, grass);
    film_grain(&mut pm, 0, 16, 10);
    let got = pm.get(16, 16);
    assert_eq!(
        (got.r, got.b),
        (grass.r, grass.b),
        "a channel under the line was carried anyway"
    );
    assert!(
        got.g > grass.g + 40,
        "the channel over the line was not carried: {got:?}"
    );
}

/// "A smoother pattern is added to the image's lighter areas" — the grain
/// thins out where the picture is light, rather than lying evenly over
/// everything the way Add Noise would.
#[test]
fn the_grain_is_smoother_in_the_light() {
    let mut pm = ramp();
    // Half open, so the light end of the ramp is over the line and the
    // dark end is under it. Intensity 0, so nothing has been lifted and
    // the only difference between the two ends is how much silver landed
    // on them.
    film_grain(&mut pm, 20, 10, 0);
    let (shadow, highlight) = (scatter(&pm, 4..20), scatter(&pm, 44..60));
    assert!(
        highlight * 2.0 < shadow,
        "the light end took as much grain as the dark: {highlight} against {shadow}"
    );
}

/// ...and the grain it keeps there is coloured rather than grey, which is
/// the other half of CS6's sentence.
#[test]
fn the_grain_in_the_light_is_coloured() {
    let colourfulness = |xs: std::ops::Range<i32>| {
        let mut pm = ramp();
        film_grain(&mut pm, 20, 10, 0);
        ink(&pm, xs)
    };
    assert!(
        colourfulness(44..60) > colourfulness(4..20),
        "the light end's grain was as grey as the dark end's"
    );
}

/// The film is seeded from where each pixel is, so an undo/redo replay
/// exposes the same frame.
#[test]
fn the_same_picture_takes_the_same_grain_twice() {
    let mut first = ramp();
    film_grain(&mut first, 8, 10, 5);
    let mut second = ramp();
    film_grain(&mut second, 8, 10, 5);
    assert_eq!(first.as_bytes(), second.as_bytes());
}

#[test]
fn film_grain_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 140, 160, 77));
    film_grain(&mut pm, 8, 10, 5);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn film_grain_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    film_grain(&mut pm, 8, 10, 5);
}

/// The brush reaches past the frame at every edge and has to dip inside
/// it instead.
#[test]
fn dry_brush_handles_a_picture_smaller_than_its_brush() {
    for (w, h) in [(1, 1), (3, 40), (40, 3)] {
        let mut pm = Pixmap::filled(w, h, Rgba8::new(90, 120, 30, 255));
        dry_brush(&mut pm, 10, 10, 1);
        assert!((pm.get(0, 0).g as i32 - 120).abs() <= 12);
    }
}

/// A single row and a single column still cut — the fill's walk above and
/// below the run has nowhere to go.
#[test]
fn cutout_handles_a_one_pixel_picture() {
    for (w, h) in [(1, 32), (32, 1), (1, 1)] {
        let mut pm = Pixmap::filled(w, h, Rgba8::new(90, 120, 30, 255));
        cutout(&mut pm, 4, 4, 2);
        assert_eq!(pm.get(0, 0).g, 120);
    }
}
