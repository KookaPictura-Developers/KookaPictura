//! Lighting Effects property tests, ported from photorust's render tests and
//! extended to the rig and its CS6 geometry.

use super::*;

const W: u32 = 240;
const H: u32 = 180;

fn filled(w: u32, h: u32, rgb: [u8; 3], alpha: u8) -> PixelBuffer {
    let n = w as usize * h as usize;
    let mut data = vec![0u8; n * 4];
    for (c, &v) in rgb.iter().chain([alpha].iter()).enumerate() {
        data[c * n..c * n + n].fill(v);
    }
    PixelBuffer {
        width: w,
        height: h,
        channels: 4,
        data: data.into(),
    }
}

fn red_at(buf: &PixelBuffer, x: usize, y: usize) -> i32 {
    buf.data[y * buf.width as usize + x] as i32
}

/// An evenly-toned frame, so what comes back is the lighting and nothing the
/// picture brought with it.
fn under(rig: &Lighting) -> PixelBuffer {
    let mut buf = filled(W, H, [160, 160, 160], 255);
    lighting_effects(&mut buf, rig).unwrap();
    buf
}

fn one(light: Light) -> Lighting {
    Lighting {
        lights: vec![light],
        ..Lighting::default()
    }
}

/// A lamp decides what the picture reflects: outside a spot's ellipse, with no
/// ambience, there is nothing to reflect and the picture goes black.
#[test]
fn a_spot_lights_its_ellipse_and_leaves_the_rest_dark() {
    let buf = under(&Lighting::default());
    assert!(
        red_at(&buf, 120, 90) > 80,
        "the middle of the ellipse is not lit"
    );
    assert!(
        red_at(&buf, 4, 4) < 12,
        "outside the ellipse did not go dark"
    );
}

/// CS6's hotspot sits at the far end of the ellipse, the way the spot aims, so
/// the aimed-at end is brighter than the end behind the centre.
#[test]
fn the_hotspot_lies_the_way_the_spot_aims() {
    let buf = under(&one(Light {
        angle: 0.0,
        size: 0.5,
        width: 0.3,
        ..Light::default()
    }));
    let span = 0.5 * (W as f32).hypot(H as f32);
    let reach = (0.4 * span) as usize;
    let ahead = red_at(&buf, 120 + reach, 90);
    let behind = red_at(&buf, 120 - reach, 90);
    assert!(ahead > behind + 40, "{ahead} ahead, {behind} behind");
}

#[test]
fn ambience_lifts_what_the_lamp_does_not_reach() {
    let dark = red_at(&under(&Lighting::default()), 4, 4);
    let lifted = red_at(
        &under(&Lighting {
            ambience: 50.0,
            ..Lighting::default()
        }),
        4,
        4,
    );
    assert!(lifted > dark + 40, "{dark} unlit, {lifted} with ambience");
}

/// The sun reaches everywhere equally: no falloff, no hotspot, no dark corners.
#[test]
fn an_infinite_light_falls_evenly_across_the_frame() {
    let buf = under(&one(Light {
        kind: LightType::Infinite,
        ..Light::default()
    }));
    let middle = red_at(&buf, 120, 90);
    let corner = red_at(&buf, 4, 4);
    assert!(middle > 40);
    assert!(
        (middle - corner).abs() <= 1,
        "{middle} in the middle, {corner} in the corner"
    );
}

/// A point light falls off with distance; a spot is flat across its hotspot.
#[test]
fn a_point_light_falls_off_where_a_spots_hotspot_is_still_flat() {
    let point = under(&one(Light {
        kind: LightType::Point,
        ..Light::default()
    }));
    let spot = under(&one(Light {
        angle: 90.0,
        hotspot: 100.0,
        size: 0.5,
        width: 0.5,
        ..Light::default()
    }));
    let fade = |buf: &PixelBuffer| red_at(buf, 120, 110) - red_at(buf, 120, 130);
    assert!(
        fade(&point) > fade(&spot),
        "the point light did not fall off faster than the spot's hotspot"
    );
}

#[test]
fn intensity_and_exposure_both_turn_the_light_up() {
    let middle = |light: Light, exposure: f32| {
        red_at(
            &under(&Lighting {
                exposure,
                ..one(light)
            }),
            120,
            90,
        )
    };
    let base = Light {
        intensity: 10.0,
        ..Light::default()
    };
    let seen = middle(base, 0.0);
    assert!(
        middle(
            Light {
                intensity: 20.0,
                ..base
            },
            0.0
        ) > seen
    );
    assert!(middle(base, 50.0) > seen);
    // Negative intensity takes light away rather than adding it.
    let ambient = |intensity: f32| {
        red_at(
            &under(&Lighting {
                ambience: 60.0,
                ..one(Light { intensity, ..base })
            }),
            120,
            90,
        )
    };
    assert!(ambient(-25.0) < ambient(0.0));
}

#[test]
fn lights_add_up_and_a_hidden_light_adds_nothing() {
    let spot = Light {
        intensity: 20.0,
        ..Light::default()
    };
    let single = red_at(&under(&one(spot)), 120, 90);
    let pair = Lighting {
        lights: vec![spot, spot],
        ..Lighting::default()
    };
    assert!(red_at(&under(&pair), 120, 90) > single + 20);
    let hidden = Lighting {
        lights: vec![spot, Light { on: false, ..spot }],
        ..Lighting::default()
    };
    assert_eq!(under(&hidden).data, under(&one(spot)).data);
}

#[test]
fn a_coloured_light_tints_what_it_lights() {
    let buf = under(&one(Light {
        color: [255, 40, 40],
        ..Light::default()
    }));
    let n = (W * H) as usize;
    let at = 90 * W as usize + 120;
    assert!(buf.data[at] as i32 > buf.data[n + at] as i32 + 40);
}

/// Without a texture every pixel faces the lamp alike; a raised channel gives
/// the light slopes to catch on.
#[test]
fn a_texture_is_what_gives_the_light_something_to_catch() {
    let shiny = Lighting {
        gloss: 80.0,
        ..one(Light {
            intensity: 40.0,
            ..Light::default()
        })
    };
    let flat = under(&shiny);
    let mut bumpy = filled(W, H, [160, 160, 160], 255);
    let n = (W * H) as usize;
    for y in 0..H as usize {
        for x in 0..W as usize {
            bumpy.data[n + y * W as usize + x] = if (x / 8) % 2 == 0 { 40 } else { 200 };
        }
    }
    lighting_effects(
        &mut bumpy,
        &Lighting {
            texture: TextureChannel::Green,
            height: 90.0,
            ..shiny.clone()
        },
    )
    .unwrap();
    let spread = |buf: &PixelBuffer, plane: usize| {
        let row = &buf.data[plane * n + 90 * W as usize..][..W as usize];
        let window = &row[100..140];
        (*window.iter().max().unwrap() as i32) - (*window.iter().min().unwrap() as i32)
    };
    // Red is flat in both; it is the relief that spreads it.
    assert!(
        spread(&bumpy, 0) > spread(&flat, 0) + 30,
        "the texture did not shape the light: {} flat, {} bumpy",
        spread(&flat, 0),
        spread(&bumpy, 0)
    );
}

#[test]
fn lighting_leaves_alpha_alone() {
    let mut buf = filled(64, 64, [200, 200, 200], 90);
    lighting_effects(&mut buf, &Lighting::default()).unwrap();
    assert!(buf.data[3 * 64 * 64..].iter().all(|&a| a == 90));
}

/// The workspace previews a shrunk proxy, so the rig is placed and sized as a
/// fraction of the frame.
#[test]
fn a_rig_is_the_same_light_at_any_size() {
    let rig = Lighting {
        lights: vec![
            Light {
                center: (0.35, 0.6),
                ..Light::default()
            },
            Light {
                kind: LightType::Point,
                center: (0.7, 0.3),
                size: 0.3,
                ..Light::default()
            },
        ],
        ..Lighting::default()
    };
    let render = |w: u32, h: u32| {
        let mut buf = filled(w, h, [160, 160, 160], 255);
        lighting_effects(&mut buf, &rig).unwrap();
        buf
    };
    let (big, small) = (render(600, 400), render(150, 100));
    let mut worst = 0i32;
    for y in 0..100 {
        for x in 0..150 {
            let mut block = 0i32;
            for dy in 0..4 {
                for dx in 0..4 {
                    block += red_at(&big, x * 4 + dx, y * 4 + dy);
                }
            }
            worst = worst.max((block / 16 - red_at(&small, x, y)).abs());
        }
    }
    assert!(worst <= 8, "proxy and full size disagree by {worst} levels");
}

#[test]
fn each_light_type_lights_differently() {
    let mut seen = Vec::new();
    for kind in [LightType::Spot, LightType::Point, LightType::Infinite] {
        let buf = under(&one(Light {
            kind,
            ..Light::default()
        }));
        assert!(!seen.contains(&buf.data), "{kind:?} matched another type");
        seen.push(buf.data);
    }
}

#[test]
fn the_hotspot_ellipse_grows_with_hotspot_and_stays_inside() {
    let (a0, b0, off0) = spot_hotspot(0.0);
    let (a1, b1, off1) = spot_hotspot(100.0);
    assert!(a1 > a0 && b1 > b0);
    assert!(a0 + off0 < 1.0 && a1 + off1 < 1.0 && off1 < off0);
    assert_eq!(spot_hotspot(-100.0).0, 0.0);
}

#[test]
fn bad_rigs_are_refused_and_leave_the_buffer_untouched() {
    let bad = [
        Lighting {
            lights: Vec::new(),
            ..Lighting::default()
        },
        Lighting {
            lights: vec![Light::default(); MAX_LIGHTS + 1],
            ..Lighting::default()
        },
        Lighting {
            gloss: 101.0,
            ..Lighting::default()
        },
        Lighting {
            height: -1.0,
            ..Lighting::default()
        },
        one(Light {
            intensity: f32::NAN,
            ..Light::default()
        }),
        one(Light {
            size: 0.0,
            ..Light::default()
        }),
        one(Light {
            elevation: 91.0,
            ..Light::default()
        }),
        one(Light {
            center: (f32::INFINITY, 0.5),
            ..Light::default()
        }),
    ];
    for rig in bad {
        let mut buf = filled(8, 8, [10, 20, 30], 255);
        let before = buf.data.clone();
        let err = lighting_effects(&mut buf, &rig).unwrap_err();
        assert!(matches!(err, FilterError::InvalidParams(_)), "{rig:?}");
        assert_eq!(buf.data, before);
    }
    let mut full = filled(8, 8, [10, 20, 30], 255);
    let rig = Lighting {
        lights: vec![Light::default(); MAX_LIGHTS],
        ..Lighting::default()
    };
    assert!(lighting_effects(&mut full, &rig).is_ok());
}
