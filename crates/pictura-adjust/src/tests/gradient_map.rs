//! Gradient Map's opacity stops.

use super::{buf3, px3};
use crate::{apply, Adjustment, GradientMapParams, GradientStop, OpacityStop};

fn red_to_blue(transparency: Vec<OpacityStop>) -> Adjustment {
    Adjustment::GradientMap(GradientMapParams {
        stops: vec![
            GradientStop {
                location: 0,
                color: [255, 0, 0],
            },
            GradientStop {
                location: 4096,
                color: [0, 0, 255],
            },
        ],
        reverse: false,
        transparency,
    })
}

#[test]
fn opacity_stops_blend_the_map_over_the_original() {
    // Black maps to red at full opacity; white to blue at none, so it stays.
    let fade = vec![
        OpacityStop {
            location: 0,
            opacity: 100,
        },
        OpacityStop {
            location: 4096,
            opacity: 0,
        },
    ];
    let mut b = buf3(3, 1, &[[0, 0, 0], [255, 255, 255], [128, 128, 128]]);
    apply(&red_to_blue(fade), &mut b).unwrap();
    assert_eq!(px3(&b, 0), [255, 0, 0]);
    assert_eq!(px3(&b, 1), [255, 255, 255]);
    // Mid gray: half the way to the half-blended mid colour.
    assert_eq!(px3(&b, 2), [128, 64, 128]);

    let past_full = vec![OpacityStop {
        location: 0,
        opacity: 101,
    }];
    assert!(apply(&red_to_blue(past_full), &mut b).is_err());
}
