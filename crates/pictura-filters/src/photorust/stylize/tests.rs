use super::*;

/// A picture with an edge in it: light on the left, dark on the right.
fn edged() -> Pixmap {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(220, 220, 220, 255));
    for y in 0..64 {
        for x in 32..64 {
            pm.set(x, y, Rgba8::new(40, 40, 40, 255));
        }
    }
    pm
}

#[test]
fn diffusing_shuffles_pixels_without_inventing_colours() {
    // Normal takes a neighbour's colour whole, so on a two-tone picture
    // every pixel has to come back as one of the two tones. A filter that
    // averaged instead would leave a grey rim down the edge.
    let mut pm = edged();
    diffuse(&mut pm, DiffuseMode::Normal);
    for p in pm.as_bytes().chunks_exact(4) {
        assert!(p[0] == 220 || p[0] == 40, "{} is neither tone", p[0]);
    }
}

#[test]
fn the_edge_is_what_moves() {
    let mut pm = edged();
    diffuse(&mut pm, DiffuseMode::Normal);
    // Ragged along the seam...
    let mut ragged = 0;
    for y in 0..64 {
        if pm.get(31, y).r != 220 || pm.get(32, y).r != 40 {
            ragged += 1;
        }
    }
    assert!(
        ragged > 8,
        "the seam came back straight: only {ragged} rows moved"
    );
    // ...and untouched well away from it, where every neighbour is the
    // same colour anyway.
    for y in 0..64 {
        assert_eq!(pm.get(4, y).r, 220);
        assert_eq!(pm.get(60, y).r, 40);
    }
}

#[test]
fn darken_only_never_lightens_a_pixel_and_lighten_only_never_darkens_one() {
    let before = edged();
    let mut darkened = before.clone();
    diffuse(&mut darkened, DiffuseMode::DarkenOnly);
    let mut lightened = before.clone();
    diffuse(&mut lightened, DiffuseMode::LightenOnly);

    for y in 0..64 {
        for x in 0..64 {
            assert!(
                darkened.get(x, y).r <= before.get(x, y).r,
                "Darken Only lightened one"
            );
            assert!(
                lightened.get(x, y).r >= before.get(x, y).r,
                "Lighten Only darkened one"
            );
        }
    }
    // And each one actually did something: the dark half grows one way,
    // the light half the other.
    assert_ne!(darkened.as_bytes(), before.as_bytes());
    assert_ne!(lightened.as_bytes(), before.as_bytes());
    assert_ne!(darkened.as_bytes(), lightened.as_bytes());
}

/// Anisotropic smooths *along* an edge, not across it — which is the one
/// thing that separates it from a blur, and the thing that would go
/// unnoticed if it were wrong, since both look soft.
#[test]
fn anisotropic_smooths_the_grain_but_keeps_the_edge() {
    // Grain on both sides of the same edge.
    let mut pm = edged();
    for y in 0..64 {
        for x in 0..64 {
            let base = pm.get(x, y).r as i32;
            let jitter = if (x + y) % 2 == 0 { 12 } else { -12 };
            let level = (base + jitter).clamp(0, 255) as u8;
            pm.set(x, y, Rgba8::new(level, level, level, 255));
        }
    }
    let before = pm.clone();
    diffuse(&mut pm, DiffuseMode::Anisotropic);

    // The grain in the middle of a flat region is gone...
    let roughness = |img: &Pixmap| {
        let mut sum = 0i32;
        for y in 10..54 {
            for x in 6..26 {
                sum += (img.get(x, y).r as i32 - img.get(x + 1, y).r as i32).abs();
            }
        }
        sum
    };
    assert!(
        roughness(&pm) * 3 < roughness(&before),
        "the grain survived: {} before, {} after",
        roughness(&before),
        roughness(&pm)
    );

    // ...and the edge is still an edge, not a ramp.
    let step = (pm.get(31, 32).r as i32 - pm.get(32, 32).r as i32).abs();
    assert!(
        step > 120,
        "the edge was blurred away: a step of only {step}"
    );
}

#[test]
fn diffusing_is_the_same_every_time() {
    crate::photorust::with_seed(0, || {
        let run = || {
            let mut pm = edged();
            diffuse(&mut pm, DiffuseMode::Normal);
            pm
        };
        assert_eq!(run().as_bytes(), run().as_bytes());
    });
}

#[test]
fn diffusing_leaves_alpha_where_the_pixel_came_from() {
    // A pixel is moved whole, so a half-transparent one stays half
    // transparent wherever it lands — and the layer's edge is not eaten
    // by the transparency outside it.
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(100, 100, 100, 128));
    diffuse(&mut pm, DiffuseMode::Normal);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 128));
}

// ------------------------------------------------------------- emboss --

/// A pale disc on a darker ground, for the light to catch one side of.
fn disc() -> Pixmap {
    let mut pm = Pixmap::filled(120, 120, Rgba8::new(90, 90, 90, 255));
    for y in 0..120 {
        for x in 0..120 {
            let (dx, dy) = ((x - 60) as f32, (y - 60) as f32);
            if dx * dx + dy * dy < 35.0 * 35.0 {
                pm.set(x, y, Rgba8::new(200, 200, 200, 255));
            }
        }
    }
    pm
}

/// Everything that did not change comes back mid grey — inside the disc
/// as well as outside it. An emboss that left the picture's own tones in
/// place would not be an emboss; it would be a sharpen.
#[test]
fn embossing_leaves_flat_ground_grey() {
    let mut pm = disc();
    emboss(&mut pm, 135.0, 3.0, 100.0);
    for (x, y) in [(5, 5), (60, 60), (114, 114)] {
        let p = pm.get(x, y);
        assert_eq!(
            (p.r, p.g, p.b),
            (128, 128, 128),
            "({x}, {y}) is not flat grey"
        );
    }
}

/// The side the light names is the lit side. Getting this backwards reads
/// as a picture stamped in from behind, and it is invisible in a test that
/// only checks *that* the edges changed.
#[test]
fn the_light_falls_on_the_side_the_angle_names() {
    let mut pm = disc();
    emboss(&mut pm, 135.0, 3.0, 100.0);
    // 135° is up and to the left, so that rim catches it and the far one
    // is in shadow.
    assert!(pm.get(36, 36).r > 170, "the lit rim came back dark");
    assert!(pm.get(84, 84).r < 86, "the shadowed rim came back light");

    // Turned right around, the two swap over.
    let mut other = disc();
    emboss(&mut other, -45.0, 3.0, 100.0);
    assert!(other.get(36, 36).r < 86);
    assert!(other.get(84, 84).r > 170);
}

/// An edge running along the light's own line has no slope across it, so
/// there is nothing for the light to catch — which is why an embossed
/// picture loses whatever runs parallel to the angle.
#[test]
fn an_edge_along_the_light_disappears() {
    let mut pm = Pixmap::filled(80, 80, Rgba8::new(90, 90, 90, 255));
    for y in 0..80i32 {
        for x in 0..80i32 {
            if (x - y).abs() < 3 {
                pm.set(x, y, Rgba8::new(220, 220, 220, 255));
            }
        }
    }
    let across = {
        let mut copy = pm.clone();
        emboss(&mut copy, 135.0, 3.0, 100.0);
        copy
    };
    emboss(&mut pm, 45.0, 3.0, 100.0);

    // Read on a line *across* the stripe rather than down the middle of
    // it: in the middle both samples land inside the stripe whichever way
    // the light comes from, and everything looks flat.
    let boldest = |img: &Pixmap| {
        (-8..=8)
            .map(|k| (img.get(40 + k, 40 - k).r as i32 - 128).abs())
            .max()
            .unwrap_or(0)
    };
    // 135° runs along the stripe, so there is no slope for it to catch.
    assert!(
        boldest(&across) < 12,
        "a stripe along the light still showed"
    );
    assert!(boldest(&pm) > 60, "a stripe across the light did not show");
}

#[test]
fn amount_presses_the_relief_harder() {
    let rim = |amount| {
        let mut pm = disc();
        emboss(&mut pm, 135.0, 3.0, amount);
        (pm.get(36, 36).r as i32 - 128).abs()
    };
    assert!(rim(300.0) > rim(100.0));
    assert!(rim(100.0) > rim(20.0));
}

/// Height is the thickness of the relief, so a taller one spreads further
/// from the edge that made it.
#[test]
fn height_widens_the_band_the_relief_covers() {
    let width = |height| {
        let mut pm = disc();
        emboss(&mut pm, 135.0, height, 100.0);
        // How many pixels along a line through the rim are not flat grey.
        (0..60)
            .filter(|&i| (pm.get(i, i).r as i32 - 128).abs() > 4)
            .count()
    };
    assert!(width(9.0) > width(3.0), "a taller relief was no wider");
}

/// Differenced per channel, which is what puts the coloured fringes along
/// an edge between two colours of the same brightness. Working on
/// brightness alone would come back flat grey there — no edge at all.
#[test]
fn an_edge_between_two_colours_of_one_tone_still_shows() {
    let mut pm = Pixmap::filled(60, 60, Rgba8::new(180, 100, 100, 255));
    for y in 0..60 {
        for x in 30..60 {
            // Much the same luma, a long way apart in colour.
            pm.set(x, y, Rgba8::new(100, 140, 180, 255));
        }
    }
    emboss(&mut pm, 0.0, 3.0, 100.0);
    let seam = pm.get(30, 30);
    assert!(
        (seam.r as i32 - seam.b as i32).abs() > 40,
        "the colour edge came back grey: {seam:?}"
    );
}

#[test]
fn embossing_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 120, 120, 77));
    emboss(&mut pm, 135.0, 3.0, 100.0);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

// --------------------------------------------------------- find edges --

/// Flat ground has no gradient anywhere, so the whole picture comes back
/// white — the ground Find Edges draws its lines on.
#[test]
fn finding_edges_leaves_flat_ground_white() {
    let mut pm = Pixmap::filled(48, 48, Rgba8::new(120, 120, 120, 255));
    find_edges(&mut pm);
    for y in 0..48 {
        for x in 0..48 {
            assert_eq!(pm.get(x, y), Rgba8::WHITE, "({x}, {y}) was not left white");
        }
    }
}

/// A step is an edge, and the line lands on it rather than a pixel to one
/// side — which is what separates a Sobel line from a one-sided
/// difference. The gradient is raw, so a step this strong saturates.
#[test]
fn finding_edges_draws_a_line_on_the_step() {
    let mut pm = edged();
    find_edges(&mut pm);
    // Dark down the seam...
    let seam = (pm.get(31, 20).r as i32).min(pm.get(32, 20).r as i32);
    assert!(seam < 20, "the seam came back light: {seam}");
    // ...and white well away from it, on both sides.
    for x in [4, 60] {
        assert_eq!(
            pm.get(x, 20),
            Rgba8::WHITE,
            "flat ground at x={x} was not white"
        );
    }
}

/// The gradient is the raw Sobel rather than one normalised to a
/// full-contrast step, so the picture's own texture comes through: a
/// gentle step is a grey line, not white. Normalising it is the difference
/// between a Find Edges that reads as a pencil sketch and one that leaves
/// a photograph almost blank.
#[test]
fn finding_edges_shows_a_gentle_step() {
    let mut pm = Pixmap::filled(48, 48, Rgba8::new(200, 200, 200, 255));
    for y in 0..48 {
        for x in 24..48 {
            pm.set(x, y, Rgba8::new(192, 192, 192, 255));
        }
    }
    find_edges(&mut pm);
    // An eight-level step is a Sobel of 32: visible grey, not white.
    let seam = pm.get(24, 24).r;
    assert!(seam < 240, "a gentle step vanished: {seam}");
}

/// Differenced per channel, so a step in one channel alone comes back as
/// that channel's colour rather than grey. A detector working on
/// brightness would still draw a line here — red carries brightness too —
/// so the line's colour is the whole point.
#[test]
fn finding_edges_keeps_the_colour_of_an_edge() {
    let mut pm = Pixmap::filled(60, 60, Rgba8::new(200, 60, 60, 255));
    for y in 0..60 {
        for x in 30..60 {
            // Only red steps; green and blue are flat throughout.
            pm.set(x, y, Rgba8::new(60, 60, 60, 255));
        }
    }
    find_edges(&mut pm);
    let seam = pm.get(30, 30);
    assert!(
        seam.g as i32 - seam.r as i32 > 80,
        "the red edge came back grey: {seam:?}"
    );
    assert_eq!(seam.g, seam.b, "green and blue should be untouched");
}

#[test]
fn finding_edges_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 120, 120, 77));
    find_edges(&mut pm);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn finding_edges_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    find_edges(&mut pm);
}

// ------------------------------------------------------------ extrude --

/// A pale disc on a dark ground, big enough for a grid of towers.
fn scene() -> Pixmap {
    let mut pm = Pixmap::filled(180, 180, Rgba8::new(30, 90, 35, 255));
    for y in 0..180 {
        for x in 0..180 {
            let (dx, dy) = ((x - 90) as f32, (y - 90) as f32);
            if dx * dx + dy * dy < 60.0 * 60.0 {
                pm.set(x, y, Rgba8::new(230, 130, 160, 255));
            }
        }
    }
    pm
}

/// The middle of the frame is where the viewer is, so a tower there is
/// thrown nowhere at all and its face lands exactly on its own square.
/// Every other tower's position is measured from that one, so if this is
/// wrong the whole grid is sliding.
#[test]
fn the_tower_in_the_middle_does_not_move() {
    let mut pm = scene();
    extrude(
        &mut pm,
        ExtrudeOptions {
            size: 30,
            solid_front: true,
            ..ExtrudeOptions::default()
        },
    );
    // 180 across in 30s: the middle falls on the corner of four tiles, so
    // read a little inside one of them.
    let p = pm.get(80, 80);
    assert_eq!(
        (p.r, p.g, p.b),
        (230, 130, 160),
        "the middle tower moved off its square"
    );
}

/// Solid Front Faces is the difference between a face carrying its piece
/// of the picture and one flat colour. Both have to be possible, or the
/// tick box does nothing.
#[test]
fn solid_front_faces_flattens_what_a_face_carries() {
    // A tile with a gradient across it, so a carried face is not flat by
    // accident.
    let mut graded = Pixmap::filled(60, 60, Rgba8::BLACK);
    for y in 0..60 {
        for x in 0..60 {
            graded.set(x, y, Rgba8::new((x * 4) as u8, 100, 100, 255));
        }
    }
    let run = |solid| {
        let mut pm = graded.clone();
        extrude(
            &mut pm,
            ExtrudeOptions {
                size: 30,
                depth: 1.0,
                solid_front: solid,
                ..ExtrudeOptions::default()
            },
        );
        // Across the middle of the top-left tile's face.
        (5..25).map(|x| pm.get(x, 15).r as i32).collect::<Vec<_>>()
    };
    let carried = run(false);
    let flat = run(true);
    assert!(
        carried.windows(2).any(|w| w[0] != w[1]),
        "a carried face came back flat"
    );
    assert!(
        flat.windows(2).all(|w| w[0] == w[1]),
        "a solid face was not one colour"
    );
}

/// Pyramids have no face to carry anything, which is why CS6 greys the
/// tick box out for them — and why it must make no difference here.
#[test]
fn pyramids_ignore_a_setting_that_only_blocks_have() {
    let run = |solid| {
        let mut pm = scene();
        extrude(
            &mut pm,
            ExtrudeOptions {
                kind: ExtrudeType::Pyramids,
                solid_front: solid,
                ..ExtrudeOptions::default()
            },
        );
        pm
    };
    assert_eq!(run(false).as_bytes(), run(true).as_bytes());
}

/// Level-based reads the height off the picture, so the same picture
/// always gives the same towers — and a bright subject stands out of a
/// dark ground rather than being scattered at random through it.
#[test]
fn level_based_stands_the_bright_tiles_tallest() {
    let mut level = scene();
    extrude(
        &mut level,
        ExtrudeOptions {
            size: 30,
            depth: 200.0,
            level_based: true,
            solid_front: true,
            ..ExtrudeOptions::default()
        },
    );
    // The pale disc is thrown a long way out; the dark ground barely
    // moves. So a ring outside the disc's original edge now carries the
    // disc's colour.
    let thrown = (0..180)
        .filter(|&y| {
            let p = level.get(20, y);
            p.r > 150
        })
        .count();
    assert!(
        thrown > 0,
        "the bright tiles did not stand out over the dark ones"
    );

    // ...and it is the same picture every time, since nothing here is
    // random.
    let again = {
        let mut pm = scene();
        extrude(
            &mut pm,
            ExtrudeOptions {
                size: 30,
                depth: 200.0,
                level_based: true,
                solid_front: true,
                ..ExtrudeOptions::default()
            },
        );
        pm
    };
    assert_eq!(level.as_bytes(), again.as_bytes());
}

/// Random heights are drawn from where the tile is, so a preview and the
/// commit behind it agree — the same rule Diffuse follows.
#[test]
fn random_heights_are_the_same_every_time() {
    crate::photorust::with_seed(0, || {
        let run = || {
            let mut pm = scene();
            extrude(&mut pm, ExtrudeOptions::default());
            pm
        };
        assert_eq!(run().as_bytes(), run().as_bytes());
    });
}

/// The grid rarely divides the picture evenly, and the tick box says what
/// to do with the part-tiles left along the edges.
#[test]
fn masking_incomplete_blocks_leaves_the_ragged_edge_alone() {
    // 100 across in 30s leaves a strip of 10 down the right-hand side.
    let mut original = Pixmap::filled(100, 100, Rgba8::new(200, 40, 40, 255));
    for y in 0..100 {
        for x in 0..100 {
            original.set(x, y, Rgba8::new((x * 2) as u8, (y * 2) as u8, 90, 255));
        }
    }
    let run = |mask| {
        let mut pm = original.clone();
        extrude(
            &mut pm,
            ExtrudeOptions {
                size: 30,
                // Barely thrown, so nothing else reaches the strip.
                depth: 1.0,
                solid_front: true,
                mask_incomplete: mask,
                ..ExtrudeOptions::default()
            },
        );
        pm
    };
    let masked = run(true);
    let built = run(false);
    for y in 0..90 {
        for x in 90..100 {
            assert_eq!(
                masked.get(x, y),
                original.get(x, y),
                "a part-tile at ({x}, {y}) was built anyway"
            );
        }
    }
    assert_ne!(
        masked.as_bytes(),
        built.as_bytes(),
        "the tick box made no difference at all"
    );
}

#[test]
fn extruding_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(90, 90, Rgba8::new(120, 60, 60, 200));
    extrude(&mut pm, ExtrudeOptions::default());
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 200));
}

#[test]
fn extruding_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    extrude(&mut pm, ExtrudeOptions::default());
}

#[test]
fn embossing_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    emboss(&mut pm, 135.0, 3.0, 100.0);
}

#[test]
fn diffusing_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    diffuse(&mut pm, DiffuseMode::Normal);
}

// ----------------------------------------------------------- solarize --

// -------------------------------------------------------------- tiles --

/// A picture of one colour, tiled onto a background of another: what is
/// left of the ground is exactly the gaps the tiles moved out of, so both
/// colours have to be there and no third one may appear.
fn tiled_flat(fill: TileFill) -> Pixmap {
    let mut pm = Pixmap::filled(200, 200, Rgba8::new(40, 90, 160, 255));
    tiles(
        &mut pm,
        TileOptions {
            count: 10,
            offset: 30,
            fill,
            foreground: Rgba8::new(255, 0, 0, 255),
            background: Rgba8::new(0, 255, 0, 255),
        },
    );
    pm
}

#[test]
fn tiles_leave_gaps_in_the_fill_colour() {
    for (fill, expected) in [
        (TileFill::BackgroundColor, Rgba8::new(0, 255, 0, 255)),
        (TileFill::ForegroundColor, Rgba8::new(255, 0, 0, 255)),
    ] {
        let pm = tiled_flat(fill);
        let picture = Rgba8::new(40, 90, 160, 255);
        let mut gaps = 0;
        for y in 0..200 {
            for x in 0..200 {
                let px = pm.get(x, y);
                if px == expected {
                    gaps += 1;
                } else {
                    assert_eq!(px, picture, "a third colour appeared at {x},{y}");
                }
            }
        }
        assert!(gaps > 0, "nothing moved: no gap was left to fill");
        // The tiles still cover most of the frame — a 30% offset cannot
        // shift a tile off more than a little over half of itself.
        assert!(gaps < 200 * 200 / 2, "the tiles covered less than half");
    }
}

/// Inverse Image puts the negative of the picture in the gaps rather than
/// a swatch colour.
#[test]
fn the_inverse_fill_shows_the_negative() {
    let pm = tiled_flat(TileFill::InverseImage);
    let negative = Rgba8::new(215, 165, 95, 255);
    assert!(
        (0..200).any(|x| (0..200).any(|y| pm.get(x, y) == negative)),
        "no part of the negative showed through"
    );
}

/// Unaltered Image leaves the ground as it was, so on a flat picture the
/// filter has nothing to show at all — which is CS6's behaviour, and why
/// the option is only useful on a picture with detail in it.
#[test]
fn the_unaltered_fill_leaves_a_flat_picture_alone() {
    let pm = tiled_flat(TileFill::UnalteredImage);
    assert!(pm
        .as_bytes()
        .chunks_exact(4)
        .all(|p| p[0] == 40 && p[1] == 90 && p[2] == 160));
}

/// More tiles means smaller ones, which means more of them and so more
/// edges where a gap can open.
#[test]
fn more_tiles_cut_the_picture_finer() {
    let gaps = |count| {
        let mut pm = Pixmap::filled(200, 200, Rgba8::new(40, 90, 160, 255));
        tiles(
            &mut pm,
            TileOptions {
                count,
                offset: 20,
                fill: TileFill::BackgroundColor,
                background: Rgba8::new(0, 255, 0, 255),
                ..TileOptions::default()
            },
        );
        pm.as_bytes()
            .chunks_exact(4)
            .filter(|p| p[1] == 255)
            .count()
    };
    assert!(
        gaps(20) > gaps(5),
        "twenty tiles left no more gap than five"
    );
}

/// The offsets come from where a tile sits in the grid, not from a RNG, so
/// an undo/redo replay lands every tile where it was the first time.
#[test]
fn tiles_fall_the_same_way_every_time() {
    crate::photorust::with_seed(0, || {
        let first = tiled_flat(TileFill::BackgroundColor);
        let second = tiled_flat(TileFill::BackgroundColor);
        assert_eq!(first.as_bytes(), second.as_bytes());
    });
}

#[test]
fn tiles_leave_alpha_with_the_pixels_it_came_with() {
    let mut pm = Pixmap::filled(120, 120, Rgba8::new(120, 60, 60, 200));
    tiles(
        &mut pm,
        TileOptions {
            fill: TileFill::UnalteredImage,
            ..TileOptions::default()
        },
    );
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 200));
}

/// A tall thin region asked for more tiles than it has pixels across
/// still divides into something rather than into nothing.
#[test]
fn tiles_survive_a_region_narrower_than_the_tile_count() {
    let mut pm = Pixmap::filled(8, 300, Rgba8::new(200, 200, 200, 255));
    tiles(
        &mut pm,
        TileOptions {
            count: 99,
            offset: 50,
            ..TileOptions::default()
        },
    );
    assert_eq!(pm.width(), 8);
}

#[test]
fn tiling_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    tiles(&mut pm, TileOptions::default());
}

// ------------------------------------------------------ trace contour --

/// A grey picture split down the middle: dark on the left, light on the
/// right, with the level between the two.
fn step() -> Pixmap {
    let mut pm = Pixmap::new(8, 4);
    for y in 0..4 {
        for x in 0..8 {
            let v = if x < 4 { 60 } else { 200 };
            pm.set(x, y, Rgba8::new(v, v, v, 255));
        }
    }
    pm
}

/// Upper inks the light side of the step, Lower the dark side — one
/// column each, and the same boundary either way.
#[test]
fn the_edge_setting_picks_which_side_of_the_step_is_inked() {
    let mut upper = step();
    trace_contour(&mut upper, 128, ContourEdge::Upper);
    assert_eq!(upper.get(4, 2), Rgba8::new(0, 0, 0, 255), "the light side");
    assert_eq!(upper.get(3, 2), Rgba8::new(255, 255, 255, 255));

    let mut lower = step();
    trace_contour(&mut lower, 128, ContourEdge::Lower);
    assert_eq!(lower.get(3, 2), Rgba8::new(0, 0, 0, 255), "the dark side");
    assert_eq!(lower.get(4, 2), Rgba8::new(255, 255, 255, 255));
}

/// A level outside the picture's range crosses nothing, so there is no
/// contour to draw and the whole frame goes white.
#[test]
fn a_level_nothing_crosses_leaves_a_blank_page() {
    for level in [0, 255] {
        let mut pm = step();
        trace_contour(&mut pm, level, ContourEdge::Upper);
        assert!(
            pm.as_bytes().chunks_exact(4).all(|p| p[0] == 255),
            "level {level} inked something"
        );
    }
}

/// Moving the level moves the line, which is the whole point of the
/// control: it is a contour map, not an edge detector.
#[test]
fn the_level_decides_where_the_line_falls() {
    // A ramp across the frame, so each level lands on a different column.
    let mut pm = Pixmap::new(256, 1);
    for x in 0..256 {
        let v = x as u8;
        pm.set(x, 0, Rgba8::new(v, v, v, 255));
    }
    let inked = |level| {
        let mut copy = pm.clone();
        trace_contour(&mut copy, level, ContourEdge::Upper);
        (0..256).find(|&x| copy.get(x, 0).r == 0)
    };
    assert_eq!(inked(64), Some(64));
    assert_eq!(inked(192), Some(192));
}

/// Each channel is traced on its own, so a boundary only one of them
/// crosses leaves that channel at 0 and the others at 255 — a coloured
/// line, which is what a solid black one would have lost.
#[test]
fn a_boundary_in_one_channel_alone_draws_a_coloured_line() {
    let mut pm = Pixmap::new(8, 1);
    for x in 0..8 {
        // Red steps across the level; green and blue never do.
        let r = if x < 4 { 60 } else { 200 };
        pm.set(x, 0, Rgba8::new(r, 200, 200, 255));
    }
    trace_contour(&mut pm, 128, ContourEdge::Upper);
    // Red inked, the other two left white: cyan.
    assert_eq!(pm.get(4, 0), Rgba8::new(0, 255, 255, 255));
}

/// The frame's own border is not a crossing, or every picture would come
/// back with a box drawn round it.
#[test]
fn the_border_is_not_traced_as_an_edge() {
    let mut pm = Pixmap::filled(16, 16, Rgba8::new(200, 200, 200, 255));
    trace_contour(&mut pm, 128, ContourEdge::Upper);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[0] == 255));
}

#[test]
fn tracing_a_contour_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(16, 16, Rgba8::new(200, 200, 200, 77));
    trace_contour(&mut pm, 128, ContourEdge::Lower);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn tracing_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    trace_contour(&mut pm, 128, ContourEdge::Upper);
}

// --------------------------------------------------------------- wind --

/// A white block in the middle of a black frame, with clear ground either
/// side of it — so which side the wind streaks off is visible.
fn block() -> Pixmap {
    let mut pm = Pixmap::filled(96, 32, Rgba8::new(0, 0, 0, 255));
    for y in 0..32 {
        for x in 32..64 {
            pm.set(x, y, Rgba8::new(255, 255, 255, 255));
        }
    }
    pm
}

/// How much white has run out onto the ground to the left of the block,
/// and how much onto the ground to its right.
fn spread(pm: &Pixmap) -> (i32, i32) {
    let lit = |range: std::ops::Range<i32>| {
        range
            .map(|x| (0..32).filter(|&y| pm.get(x, y).r > 40).count() as i32)
            .sum()
    };
    (lit(0..32), lit(64..96))
}

/// The wind comes from the side CS6 names, so the picture travels the
/// other way — and it streaks off *that* side of the block only. Getting
/// this backwards mirrors the whole filter; catching both sides of the
/// step streaks every shape from both at once.
#[test]
fn the_wind_streaks_one_side_of_a_shape_and_not_the_other() {
    let mut blown = block();
    wind(&mut blown, WindMethod::Wind, false);
    let (left, right) = spread(&blown);
    assert!(right > 0, "a wind from the left blew nothing to the right");
    assert_eq!(left, 0, "it streaked the upwind side as well");

    let mut blown = block();
    wind(&mut blown, WindMethod::Wind, true);
    let (left, right) = spread(&blown);
    assert!(left > 0, "a wind from the right blew nothing to the left");
    assert_eq!(right, 0, "it streaked the upwind side as well");
}

/// Blast is the same machine driven harder, so its streaks run further
/// than Wind's off the same edge.
#[test]
fn blast_reaches_further_than_wind() {
    let reach = |method| {
        let mut pm = block();
        wind(&mut pm, method, false);
        spread(&pm).1
    };
    assert!(
        reach(WindMethod::Blast) > reach(WindMethod::Wind),
        "blast blew no further than a breeze"
    );
}

/// Stagger's streaks wander between rows, so the far edge of what it
/// leaves is ragged rather than a clean column — which is the whole
/// difference between it and Wind.
#[test]
fn stagger_wanders_between_rows_and_the_others_do_not() {
    // Rows that alternate red and green, so a streak carrying a
    // neighbouring row's colour is visible as such. Wind's streaks stay
    // in their own row; Stagger's are the ones that wander.
    let striped = || {
        let mut pm = Pixmap::filled(64, 32, Rgba8::new(0, 0, 0, 255));
        for y in 0..32 {
            let colour = if y % 2 == 0 {
                Rgba8::new(255, 0, 0, 255)
            } else {
                Rgba8::new(0, 255, 0, 255)
            };
            for x in 0..32 {
                pm.set(x, y, colour);
            }
        }
        pm
    };
    let strays = |method| {
        let mut pm = striped();
        wind(&mut pm, method, false);
        (0..32)
            .map(|y| {
                let wrong = |p: Rgba8| if y % 2 == 0 { p.g > 40 } else { p.r > 40 };
                (32..64).filter(|&x| wrong(pm.get(x, y))).count()
            })
            .sum::<usize>()
    };
    assert_eq!(strays(WindMethod::Wind), 0, "a breeze crossed between rows");
    assert!(
        strays(WindMethod::Stagger) > 0,
        "stagger stayed in its own row"
    );
}

/// Flat ground has no edge to catch, so the wind leaves it exactly as it
/// was — if it did not, the filter would read as a smear rather than as
/// edges torn sideways.
#[test]
fn flat_ground_is_left_alone() {
    let mut pm = Pixmap::filled(48, 48, Rgba8::new(120, 120, 120, 255));
    let before = pm.clone();
    wind(&mut pm, WindMethod::Blast, false);
    assert_eq!(pm.as_bytes(), before.as_bytes());
}

/// Seeded from the pixel's own coordinates, so an undo/redo replay blows
/// the same way it did the first time.
#[test]
fn the_wind_blows_the_same_way_every_time() {
    crate::photorust::with_seed(0, || {
        let mut first = block();
        wind(&mut first, WindMethod::Stagger, true);
        let mut second = block();
        wind(&mut second, WindMethod::Stagger, true);
        assert_eq!(first.as_bytes(), second.as_bytes());
    });
}

#[test]
fn the_wind_leaves_alpha_alone() {
    let mut pm = block();
    for y in 0..pm.height() as i32 {
        for x in 0..pm.width() as i32 {
            let px = pm.get(x, y);
            pm.set(x, y, Rgba8::new(px.r, px.g, px.b, 77));
        }
    }
    wind(&mut pm, WindMethod::Blast, false);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn a_wind_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    wind(&mut pm, WindMethod::Wind, false);
}

// ------------------------------------------------------ glowing edges --

/// A white disc on black: one closed edge, and plenty of flat ground
/// either side of it that ought to go dark.
fn white_disc() -> Pixmap {
    let mut pm = Pixmap::filled(64, 64, Rgba8::new(0, 0, 0, 255));
    for y in 0..64 {
        for x in 0..64 {
            let (dx, dy) = ((x - 32) as f32, (y - 32) as f32);
            if (dx * dx + dy * dy).sqrt() < 16.0 {
                pm.set(x, y, Rgba8::new(255, 255, 255, 255));
            }
        }
    }
    pm
}

/// The lit band, measured along a line out from the middle.
fn band(pm: &Pixmap) -> usize {
    (0..64).filter(|&x| pm.get(x, 32).r > 30).count()
}

/// Flat ground goes black and the boundary lights up — the two halves of
/// what the filter is for.
#[test]
fn the_edge_lights_up_and_the_rest_goes_dark() {
    let mut pm = white_disc();
    glowing_edges(&mut pm, 3, 6, 5);
    assert!(pm.get(32, 32).r < 30, "the middle of the disc stayed lit");
    assert!(pm.get(2, 2).r < 30, "the corner of the ground stayed lit");
    assert!(band(&pm) > 0, "the edge did not light up at all");
}

/// Edge Width thickens the line, which is the only thing it does.
#[test]
fn edge_width_thickens_the_line() {
    let wide = {
        let mut pm = white_disc();
        glowing_edges(&mut pm, 12, 6, 5);
        band(&pm)
    };
    let thin = {
        let mut pm = white_disc();
        glowing_edges(&mut pm, 1, 6, 5);
        band(&pm)
    };
    assert!(wide > thin, "a wide edge was no thicker than a thin one");
}

/// Edge Brightness is the gain, so it lifts the line without moving it.
#[test]
fn edge_brightness_is_the_gain() {
    let lit = |brightness| {
        let mut pm = white_disc();
        glowing_edges(&mut pm, 3, brightness, 5);
        (0..64).map(|x| pm.get(x, 32).r as u32).sum::<u32>()
    };
    assert!(lit(14) > lit(6), "turning the brightness up did nothing");
    // CS6's slider runs down to zero, and zero means no light at all.
    assert_eq!(lit(0), 0);
}

/// Smoothness blurs before the gradient is taken, so fine texture stops
/// registering as an edge — which is the whole reason it is there.
#[test]
fn smoothness_stops_grain_reading_as_an_edge() {
    // Grain: all texture, no real boundary. Not a checkerboard — a
    // Sobel reads two pixels either side of the centre, which on a
    // two-pixel period are the same value, so it would measure nothing
    // at any setting and the test would pass for the wrong reason.
    let grainy = || {
        let mut pm = Pixmap::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let v = 110 + (hash(x as u32, y as u32) % 40) as u8;
                pm.set(x, y, Rgba8::new(v, v, v, 255));
            }
        }
        pm
    };
    let lit = |smoothness| {
        let mut pm = grainy();
        glowing_edges(&mut pm, 1, 6, smoothness);
        pm.as_bytes()
            .chunks_exact(4)
            .map(|p| p[0] as u32)
            .sum::<u32>()
    };
    assert!(
        lit(15) < lit(1),
        "smoothing left as much grain lit as no smoothing at all"
    );
}

/// The channels are taken separately, so an edge in one alone glows in
/// that colour rather than in white.
#[test]
fn an_edge_in_one_channel_glows_in_its_own_colour() {
    let mut pm = Pixmap::new(64, 16);
    for y in 0..16 {
        for x in 0..64 {
            // Red steps in the middle; green and blue never do.
            let r = if x < 32 { 40 } else { 220 };
            pm.set(x, y, Rgba8::new(r, 120, 120, 255));
        }
    }
    glowing_edges(&mut pm, 1, 10, 1);
    let lit = pm.get(32, 8);
    assert!(lit.r > 60, "the channel that stepped did not light up");
    assert_eq!((lit.g, lit.b), (0, 0), "the channels that did not step lit");
}

#[test]
fn glowing_edges_leaves_alpha_alone() {
    let mut pm = Pixmap::filled(32, 32, Rgba8::new(120, 120, 120, 77));
    glowing_edges(&mut pm, 3, 6, 5);
    assert!(pm.as_bytes().chunks_exact(4).all(|p| p[3] == 77));
}

#[test]
fn glowing_edges_over_an_empty_pixmap_does_nothing() {
    let mut pm = Pixmap::new(0, 0);
    glowing_edges(&mut pm, 3, 6, 5);
}
