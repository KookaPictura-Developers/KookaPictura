//! Levels' per-channel records.

use super::{buf3, px3};
use crate::{apply, Adjustment, LevelsChannel, LevelsParams};

#[test]
fn levels_channel_record_moves_only_its_plane_before_the_composite() {
    let mut b = buf3(1, 1, &[[100, 100, 100]]);
    let red = LevelsChannel {
        input_black: 0,
        input_white: 200,
        gamma: 1.0,
        output_black: 0,
        output_white: 255,
    };
    apply(
        &Adjustment::Levels(LevelsParams {
            input_black: 0,
            input_white: 255,
            gamma: 1.0,
            output_black: 0,
            output_white: 127,
            red: Some(red),
            green: None,
            blue: None,
        }),
        &mut b,
    )
    .unwrap();
    // Red 100 -> 128 by its record, then every plane halves by the composite.
    assert_eq!(px3(&b, 0), [64, 50, 50]);

    let mut bad = red;
    bad.input_black = 250;
    let mut c = buf3(1, 1, &[[1, 2, 3]]);
    let params = LevelsParams {
        input_black: 0,
        input_white: 255,
        gamma: 1.0,
        output_black: 0,
        output_white: 255,
        red: None,
        green: Some(bad),
        blue: None,
    };
    assert!(apply(&Adjustment::Levels(params), &mut c).is_err());
}
