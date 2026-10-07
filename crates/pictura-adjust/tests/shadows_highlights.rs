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

fn plane(width: u32, height: u32, at: impl Fn(u32, u32) -> u8) -> PixelBuffer {
    let n = (width * height) as usize;
    let mut data = vec![0u8; n * 3];
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            let v = at(x, y);
            data[i] = v;
            data[n + i] = v;
            data[2 * n + i] = v;
        }
    }
    PixelBuffer {
        width,
        height,
        channels: 3,
        data: data.into(),
    }
}

#[test]
fn lifts_shadows_and_pulls_highlights() {
    let dark = [20u8, 30, 10];
    let mut b = buf3(&[dark]);
    apply(&params(100.0, 0.0), &mut b).unwrap();
    let lifted = px3(&b, 0);
    assert!(lifted[1] > 80, "a dark area opens up strongly: {lifted:?}");
    assert!(
        lifted[1] > lifted[0] && lifted[0] > lifted[2],
        "the hue is kept: {lifted:?}"
    );

    let bright = [230u8, 240, 245];
    let mut b = buf3(&[bright]);
    apply(&params(0.0, 100.0), &mut b).unwrap();
    assert!(px3(&b, 0)[0] < 200, "a bright area comes down");
    // Pure black and white have nowhere to go.
    let mut b = buf3(&[[0, 0, 0], [255, 255, 255]]);
    apply(&params(100.0, 100.0), &mut b).unwrap();
    assert_eq!((px3(&b, 0), px3(&b, 1)), ([0; 3], [255; 3]));
}

#[test]
fn the_correction_follows_the_neighbourhood() {
    // The same dark pixel lifts far more inside a dark area than as a speck
    // in a bright one, so local contrast survives.
    let (w, h) = (61, 61);
    let mut dark_area = plane(w, h, |_, _| 30);
    let mut bright_area = plane(w, h, |x, y| if (x, y) == (30, 30) { 30 } else { 220 });
    apply(&params(100.0, 0.0), &mut dark_area).unwrap();
    apply(&params(100.0, 0.0), &mut bright_area).unwrap();
    let centre = (30 * w + 30) as usize;
    assert!(dark_area.data[centre] > 90, "{}", dark_area.data[centre]);
    assert!(
        bright_area.data[centre] < 40,
        "{}",
        bright_area.data[centre]
    );
}

#[test]
fn zero_is_a_noop_and_alpha_is_kept() {
    let mut b = buf4(&[[20, 30, 10, 77], [60, 70, 50, 128]]);
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
