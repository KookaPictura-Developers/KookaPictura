//! Shadows/Highlights (`shdH`) kernel behaviour.

use pictura_adjust::{apply, AdjustError, Adjustment, ShadowsHighlightsParams};
use pictura_core::PixelBuffer;

fn buf3(px: &[[u8; 3]]) -> PixelBuffer {
    let n = px.len();
    let mut data = vec![0u8; n * 3];
    for (i, p) in px.iter().enumerate() {
        data[i] = p[0];
        data[n + i] = p[1];
        data[2 * n + i] = p[2];
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels: 3,
        data: data.into(),
    }
}

fn buf4(px: &[[u8; 4]]) -> PixelBuffer {
    let n = px.len();
    let mut data = vec![0u8; n * 4];
    for (i, p) in px.iter().enumerate() {
        data[i] = p[0];
        data[n + i] = p[1];
        data[2 * n + i] = p[2];
        data[3 * n + i] = p[3];
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels: 4,
        data: data.into(),
    }
}

fn px3(b: &PixelBuffer, i: usize) -> [u8; 3] {
    let n = b.pixel_count();
    [b.data[i], b.data[n + i], b.data[2 * n + i]]
}

fn params(shadows_amount: f64, highlights_amount: f64) -> Adjustment {
    Adjustment::ShadowsHighlights(ShadowsHighlightsParams {
        shadows_amount,
        highlights_amount,
    })
}

#[test]
fn lifts_shadows_and_pulls_highlights() {
    let dark = [20u8, 30, 10];
    let mut b = buf3(&[dark]);
    apply(&params(100.0, 0.0), &mut b).unwrap();
    assert!(px3(&b, 0)[0] > dark[0], "a dark pixel should lighten");
    assert!(px3(&b, 0)[1] > dark[1]);

    let bright = [230u8, 240, 245];
    let mut b = buf3(&[bright]);
    apply(&params(0.0, 100.0), &mut b).unwrap();
    assert!(px3(&b, 0)[0] < bright[0], "a bright pixel should darken");
}

#[test]
fn zero_is_a_noop_and_alpha_is_kept() {
    let mut b = buf4(&[[20, 30, 10, 77], [230, 240, 245, 128]]);
    let orig = b.clone();
    apply(&params(0.0, 0.0), &mut b).unwrap();
    assert_eq!(b, orig, "0/0 is byte-identical");

    apply(&params(50.0, 50.0), &mut b).unwrap();
    let n = b.pixel_count();
    assert_eq!(&b.data[3 * n..], &[77, 128], "alpha is untouched");
    assert_ne!(&b.data[..3 * n], &orig.data[..3 * n]);
}

#[test]
fn rejects_invalid_amounts() {
    let mut b = buf3(&[[10, 10, 10]]);
    for (s, h) in [(101.0, 0.0), (0.0, 101.0), (-1.0, 0.0), (f64::NAN, 0.0)] {
        let before = b.clone();
        assert!(matches!(
            apply(&params(s, h), &mut b),
            Err(AdjustError::InvalidParams(_))
        ));
        assert_eq!(b, before, "a rejected amount must not mutate");
    }
}
