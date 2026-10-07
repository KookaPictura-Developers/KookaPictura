//! Selective Color weighs each range by how much of its colour a pixel holds.

use super::{buf3, px3};
use crate::{apply, Adjustment, SelectiveColorMethod, SelectiveColorParams, SelectiveRange};

#[test]
fn a_colour_range_leaves_near_gray_pixels_nearly_alone() {
    // Cyans: Magenta -100, Black -94, Relative (the horse photo's sky edit).
    let mut ranges = [SelectiveRange::default(); 9];
    ranges[3] = SelectiveRange {
        c: 0,
        m: -100,
        y: 0,
        k: -94,
    };
    let adjustment = Adjustment::SelectiveColor(SelectiveColorParams {
        method: SelectiveColorMethod::Relative,
        ranges,
    });
    // Near-white foam with a faint cyan cast, a saturated cyan, and a red.
    let mut b = buf3(3, 1, &[[236, 242, 244], [0, 200, 220], [200, 40, 30]]);
    apply(&adjustment, &mut b).unwrap();
    let foam = px3(&b, 0);
    assert!(
        foam.iter()
            .zip([236, 242, 244])
            .all(|(a, b)| a.abs_diff(b) <= 2),
        "{foam:?}"
    );
    let cyan = px3(&b, 1);
    assert!(
        cyan[0] > 100 && cyan[1] > 200,
        "the cyan lightens: {cyan:?}"
    );
    assert_eq!(px3(&b, 2), [200, 40, 30], "a red holds no cyan");
}
