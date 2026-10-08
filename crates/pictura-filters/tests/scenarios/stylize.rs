//! Spec scenarios from the replaced `stylize.rs` unit tests, run through
//! `apply` against the ported photorust engine.

use super::shims::*;
use pictura_core::PixelBuffer;
use pictura_filters::*;

fn planar(width: u32, height: u32, channels: u8, planes: &[Vec<u8>]) -> PixelBuffer {
    let mut data = Vec::new();
    for p in planes {
        data.extend_from_slice(p);
    }
    PixelBuffer {
        width,
        height,
        channels,
        data: data.into(),
    }
}

fn gray_row(values: &[u8]) -> PixelBuffer {
    planar(
        values.len() as u32,
        1,
        3,
        &[values.to_vec(), values.to_vec(), values.to_vec()],
    )
}

fn gray_at(buf: &PixelBuffer, x: usize, y: usize) -> u8 {
    let w = buf.width as usize;
    let n = buf.pixel_count();
    let i = y * w + x;
    assert_eq!(buf.data[i], buf.data[n + i], "R != G at {x},{y}");
    assert_eq!(buf.data[i], buf.data[2 * n + i], "R != B at {x},{y}");
    buf.data[i]
}

fn alpha_plane(buf: &PixelBuffer) -> Vec<u8> {
    let n = buf.pixel_count();
    buf.data[3 * n..4 * n].to_vec()
}

#[test]
fn emboss_flat_field_is_neutral_gray() {
    let n = 5 * 4;
    let base = planar(
        5,
        4,
        4,
        &[vec![40; n], vec![90; n], vec![160; n], vec![37; n]],
    );
    let mut out = base.clone();
    emboss(&mut out, 135.0, 3.0, 100.0).unwrap();
    for i in 0..n {
        for c in 0..3 {
            let v = out.data[c * n + i] as i32;
            assert!((v - 128).abs() <= 1, "channel {c} pixel {i} = {v}");
        }
        assert_eq!(out.data[i], out.data[n + i], "not achromatic");
        assert_eq!(out.data[i], out.data[2 * n + i], "not achromatic");
    }
    assert_eq!(alpha_plane(&out), base.data[3 * n..4 * n].to_vec());
}

#[test]
fn emboss_rejects_invalid_parameters() {
    let base = gray_row(&[10, 20, 30, 40]);
    for (angle, height, amount) in [
        (400.0, 1.0, 100.0),
        (f64::NAN, 1.0, 100.0),
        (0.0, 0.0, 100.0),
        (0.0, -1.0, 100.0),
        (0.0, f64::INFINITY, 100.0),
        (0.0, 1.0, 0.0),
        (0.0, 1.0, -5.0),
        (0.0, 1.0, f64::NAN),
        (0.0, 101.0, 100.0),
        (0.0, 1e300, 100.0),
        (0.0, 1.0, 501.0),
    ] {
        let mut out = base.clone();
        assert!(
            emboss(&mut out, angle, height, amount).is_err(),
            "expected reject for angle={angle} height={height} amount={amount}"
        );
        assert_eq!(out, base, "rejected parameters must not modify the buffer");
    }
    let mut ok = base.clone();
    assert!(emboss(&mut ok, 360.0, 1.0, 100.0).is_ok());
    assert!(emboss(&mut ok, 0.0, 100.0, 500.0).is_ok());
}

#[test]
fn find_edges_is_light_on_a_flat_field() {
    let base = gray_row(&[77; 8]);
    let mut out = base.clone();
    find_edges(&mut out).unwrap();
    assert!(out.data[..base.pixel_count()].iter().all(|&v| v == 255));
}

#[test]
fn find_edges_is_dark_at_a_step_edge() {
    let base = gray_row(&[0, 0, 0, 0, 255, 255, 255, 255]);
    let mut out = base.clone();
    find_edges(&mut out).unwrap();
    assert!(out.data[3] < 128, "edge column 3 = {}", out.data[3]);
    assert!(out.data[4] < 128, "edge column 4 = {}", out.data[4]);
    assert_eq!(out.data[0], 255, "flat area must stay light");
    assert_eq!(out.data[7], 255, "flat area must stay light");
}

#[test]
fn stylize_preserves_alpha() {
    let expected: Vec<u8> = (0..20u32).map(|i| (i * 11) as u8).collect();
    let n = expected.len();
    let r: Vec<u8> = (0..n).map(|i| (i * 13) as u8).collect();
    let g: Vec<u8> = (0..n).map(|i| (i * 17) as u8).collect();
    let b: Vec<u8> = (0..n).map(|i| (i * 19) as u8).collect();
    let base = planar(5, 4, 4, &[r, g, b, expected.clone()]);

    let mut e = base.clone();
    emboss(&mut e, 45.0, 2.0, 120.0).unwrap();
    assert_eq!(alpha_plane(&e), expected);

    let mut f = base.clone();
    find_edges(&mut f).unwrap();
    assert_eq!(alpha_plane(&f), expected);

    let mut s = base.clone();
    solarize(&mut s).unwrap();
    assert_eq!(alpha_plane(&s), expected);
}

#[test]
fn tiny_images_do_not_panic() {
    for (w, h) in [(1u32, 1u32), (1, 5), (5, 1)] {
        let n = (w * h) as usize;
        let base = planar(
            w,
            h,
            4,
            &[vec![10; n], vec![20; n], vec![30; n], vec![40; n]],
        );
        let mut e = base.clone();
        assert!(emboss(&mut e, 90.0, 1.0, 100.0).is_ok());
        let mut f = base.clone();
        assert!(find_edges(&mut f).is_ok());
        let mut s = base.clone();
        assert!(solarize(&mut s).is_ok());
    }
}

fn patch(w: u32, h: u32) -> PixelBuffer {
    let n = (w * h) as usize;
    let mut data = vec![0u8; n * 4];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let p = y * w as usize + x;
            data[p] = (x * 37 + y * 11) as u8;
            data[n + p] = (x * 5 + y * 61) as u8;
            data[2 * n + p] = (x * 29 + y * 3) as u8;
            data[3 * n + p] = 200;
        }
    }
    PixelBuffer {
        width: w,
        height: h,
        channels: 4,
        data: data.into(),
    }
}

#[test]
fn diffuse_modes_change_colour_and_preserve_alpha() {
    for mode in [
        DiffuseMode::Normal,
        DiffuseMode::DarkenOnly,
        DiffuseMode::LightenOnly,
        DiffuseMode::Anisotropic,
    ] {
        let base = patch(16, 16);
        let before = base.data.clone();
        let mut out = base.clone();
        diffuse(&mut out, mode).unwrap();
        let n = out.pixel_count();
        assert_ne!(out.data, before, "mode {mode:?} made no change");
        assert_eq!(
            out.data[3 * n..],
            before[3 * n..],
            "mode {mode:?} touched alpha"
        );
    }
}

#[test]
fn diffuse_is_deterministic() {
    let base = patch(16, 16);
    let mut a = base.clone();
    let mut b = base.clone();
    diffuse(&mut a, DiffuseMode::Normal).unwrap();
    diffuse(&mut b, DiffuseMode::Normal).unwrap();
    assert_eq!(a.data, b.data);
}

#[test]
fn glowing_edges_changes_colour_and_preserves_alpha() {
    let base = patch(24, 24);
    let before = base.data.clone();
    let mut out = base.clone();
    glowing_edges(&mut out, 2, 6, 1).unwrap();
    let n = out.pixel_count();
    assert_ne!(out.data, before);
    assert_eq!(out.data[3 * n..], before[3 * n..], "alpha untouched");
}

#[test]
fn glowing_edges_rejects_out_of_range_params_without_mutation() {
    let base = patch(16, 16);
    for (w, b, s) in [
        (0u32, 6u32, 1u32),
        (15, 6, 1),
        (2, 21, 1),
        (2, 6, 0),
        (2, 6, 16),
    ] {
        let mut out = base.clone();
        assert!(
            matches!(
                glowing_edges(&mut out, w, b, s),
                Err(FilterError::InvalidParams(_))
            ),
            "expected rejection for width={w} brightness={b} smoothness={s}"
        );
        assert_eq!(out, base, "rejected parameters must not modify the buffer");
    }
}

#[test]
fn emboss_opposite_lights_swap_highlight_and_shadow_and_gray_stays_gray() {
    let base = gray_row(&[50, 50, 50, 50, 90, 90, 90, 90]);
    let (mut from_right, mut from_left) = (base.clone(), base.clone());
    emboss(&mut from_right, 0.0, 1.0, 100.0).unwrap();
    emboss(&mut from_left, 180.0, 1.0, 100.0).unwrap();
    let (a, b) = (
        gray_at(&from_right, 4, 0) as i32,
        gray_at(&from_left, 4, 0) as i32,
    );
    assert!(
        (a - 128).signum() == -(b - 128).signum() && a != 128,
        "opposite lights must swap the edge's highlight and shadow ({a} vs {b})"
    );
    let n = base.pixel_count();
    for i in 0..n {
        assert_eq!(
            from_right.data[i],
            from_right.data[n + i],
            "gray in, gray out"
        );
        assert_eq!(
            from_right.data[i],
            from_right.data[2 * n + i],
            "gray in, gray out"
        );
    }
    assert!(
        gray_at(&from_right, 0, 0).abs_diff(128) <= 1,
        "flat stays mid-gray"
    );
}
