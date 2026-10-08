//! Spec scenarios from the replaced `stylize/trace_contour.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

fn row(values: &[u8], channels: u8, alpha: &[u8]) -> PixelBuffer {
    let n = values.len();
    let mut data = Vec::new();
    for _ in 0..3 {
        data.extend_from_slice(values);
    }
    if channels == 4 {
        data.extend_from_slice(alpha);
    }
    PixelBuffer {
        width: n as u32,
        height: 1,
        channels,
        data: data.into(),
    }
}

fn plane(buf: &PixelBuffer, c: usize) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[c * n..c * n + n].to_vec()
}

#[test]
fn trace_contour_level_zero_and_max_differ() {
    let values: Vec<u8> = (0..=255).collect();
    let base = row(&values, 3, &[]);
    let mut low = base.clone();
    trace_contour(&mut low, 0, ContourEdge::Upper).unwrap();
    let mut high = base.clone();
    trace_contour(&mut high, 255, ContourEdge::Upper).unwrap();
    assert!(
        plane(&low, 0).iter().all(|&v| v == 255),
        "level 0 crosses nothing"
    );
    assert!(plane(&high, 0).contains(&0), "level 255 inks the peak");
    assert_ne!(low.data, high.data);
}

#[test]
fn trace_contour_edge_picks_the_side_of_the_step() {
    let base = row(&[0, 0, 0, 255, 255, 255], 3, &[]);
    let mut upper = base.clone();
    trace_contour(&mut upper, 128, ContourEdge::Upper).unwrap();
    let mut lower = base.clone();
    trace_contour(&mut lower, 128, ContourEdge::Lower).unwrap();
    assert_eq!(plane(&upper, 0)[3], 0, "Upper inks the bright side");
    assert_eq!(plane(&lower, 0)[2], 0, "Lower inks the dark side");
    assert!(plane(&upper, 0)[2] == 255 && plane(&lower, 0)[3] == 255);
}

#[test]
fn trace_contour_leaves_a_flat_field_blank_and_alpha_untouched() {
    let n = 12;
    let values = vec![100u8; n];
    let alpha: Vec<u8> = (0..n).map(|i| (i * 5) as u8).collect();
    let base = row(&values, 4, &alpha);
    let mut out = base.clone();
    trace_contour(&mut out, 100, ContourEdge::Upper).unwrap();
    for c in 0..3 {
        assert!(
            plane(&out, c).iter().all(|&v| v == 255),
            "flat field must stay white"
        );
    }
    assert_eq!(plane(&out, 3), alpha);
}
