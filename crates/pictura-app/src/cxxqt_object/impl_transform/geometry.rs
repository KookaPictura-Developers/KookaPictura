use super::super::state::TransformSession;
use pictura_core::PsdRect;

/// Screen-pixel tolerance for a scale-handle hit.
const HANDLE_TOLERANCE: f64 = 6.0;
/// Screen-pixel band outside a corner that starts a rotation.
const ROTATE_BAND: f64 = 20.0;
/// Handle indices above the eight scale handles.
pub(super) const ROTATE_HANDLE: i32 = 8;
pub(super) const MOVE_HANDLE: i32 = 9;

/// Centre of `rect` as a document-space point.
fn rect_center(rect: PsdRect) -> (f64, f64) {
    (
        (rect.left as f64 + rect.right as f64) / 2.0,
        (rect.top as f64 + rect.bottom as f64) / 2.0,
    )
}

/// Source-rect corners in order top-left, top-right, bottom-right, bottom-left.
fn source_corners(rect: PsdRect) -> [(f64, f64); 4] {
    [
        (rect.left as f64, rect.top as f64),
        (rect.right as f64, rect.top as f64),
        (rect.right as f64, rect.bottom as f64),
        (rect.left as f64, rect.bottom as f64),
    ]
}

/// `p' = c + R(θ)·(S·(p − c)) + (dx, dy)` for one point.
fn forward_point(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
    p: (f64, f64),
) -> (f64, f64) {
    let (c_x, c_y) = rect_center(rect);
    let (sin, cos) = angle.sin_cos();
    let ux = (p.0 - c_x) * sx;
    let uy = (p.1 - c_y) * sy;
    (
        c_x + cos * ux - sin * uy + dx,
        c_y + sin * ux + cos * uy + dy,
    )
}

/// The transformed quad corners in [`source_corners`] order.
pub(super) fn transform_quad_points(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
) -> [(f64, f64); 4] {
    let mut out = source_corners(rect);
    for corner in &mut out {
        *corner = forward_point(rect, sx, sy, angle, dx, dy, *corner);
    }
    out
}

/// Source coordinate of handle `h` (4..=7 are edge midpoints).
fn handle_source_point(rect: PsdRect, h: i32) -> (f64, f64) {
    let (left, top) = (rect.left as f64, rect.top as f64);
    let (right, bottom) = (rect.right as f64, rect.bottom as f64);
    let (mx, my) = ((left + right) / 2.0, (top + bottom) / 2.0);
    match h {
        0 => (left, top),
        1 => (right, top),
        2 => (right, bottom),
        3 => (left, bottom),
        4 => (mx, top),
        5 => (right, my),
        6 => (mx, bottom),
        _ => (left, my),
    }
}

/// The anchor handle opposite `h` (the pivot a scale keeps fixed).
fn opposite_handle(h: i32) -> i32 {
    match h {
        0 => 2,
        1 => 3,
        2 => 0,
        3 => 1,
        4 => 6,
        5 => 7,
        6 => 4,
        _ => 5,
    }
}

/// The eight handle positions of the current quad.
fn handle_points(quad: &[(f64, f64); 4]) -> [(f64, f64); 8] {
    [
        quad[0],
        quad[1],
        quad[2],
        quad[3],
        ((quad[0].0 + quad[1].0) / 2.0, (quad[0].1 + quad[1].1) / 2.0),
        ((quad[1].0 + quad[2].0) / 2.0, (quad[1].1 + quad[2].1) / 2.0),
        ((quad[2].0 + quad[3].0) / 2.0, (quad[2].1 + quad[3].1) / 2.0),
        ((quad[3].0 + quad[0].0) / 2.0, (quad[3].1 + quad[0].1) / 2.0),
    ]
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

/// Translate the session by the pointer delta since press.
pub(super) fn gesture_translate(session: &mut TransformSession, x: f64, y: f64) {
    session.dx = session.start[3] + (x - session.press_x);
    session.dy = session.start[4] + (y - session.press_y);
}

/// Rotate about the session centre; Shift snaps to 15° steps.
pub(super) fn gesture_rotate(session: &mut TransformSession, x: f64, y: f64, shift: bool) {
    let (c_x, c_y) = rect_center(session.orig_rect);
    let pivot = (c_x + session.start[3], c_y + session.start[4]);
    let a0 = (session.press_y - pivot.1).atan2(session.press_x - pivot.0);
    let a1 = (y - pivot.1).atan2(x - pivot.0);
    let mut angle = session.start[2] + (a1 - a0);
    if shift {
        let step = std::f64::consts::PI / 12.0;
        angle = (angle / step).round() * step;
    }
    session.angle = angle;
}

/// Scale about the opposite handle so it stays fixed; Shift locks the aspect
/// ratio and the transformed rect is clamped to at least 1 px per axis.
pub(super) fn gesture_scale(session: &mut TransformSession, x: f64, y: f64, shift: bool) -> bool {
    let rect = session.orig_rect;
    let [start_sx, start_sy, start_angle, start_dx, start_dy] = session.start;
    let (c_x, c_y) = rect_center(rect);
    let h = session.handle;
    let drag_src = handle_source_point(rect, h);
    let anchor_src = handle_source_point(rect, opposite_handle(h));
    let anchor_doc = forward_point(
        rect,
        start_sx,
        start_sy,
        start_angle,
        start_dx,
        start_dy,
        anchor_src,
    );
    let (sin, cos) = start_angle.sin_cos();
    let vx = x - anchor_doc.0;
    let vy = y - anchor_doc.1;
    let wx = cos * vx + sin * vy;
    let wy = -sin * vx + cos * vy;
    let ux = drag_src.0 - anchor_src.0;
    let uy = drag_src.1 - anchor_src.1;
    let mut new_sx = if ux.abs() > 1e-12 { wx / ux } else { start_sx };
    let mut new_sy = if uy.abs() > 1e-12 { wy / uy } else { start_sy };
    // Shift locks the aspect ratio on a corner only; an edge handle always
    // scales one axis.
    if shift && h <= 3 {
        let uniform = if new_sx.abs() > new_sy.abs() {
            new_sx
        } else {
            new_sy
        };
        new_sx = uniform;
        new_sy = uniform;
    }
    let min_sx = 1.0 / rect.width() as f64;
    let min_sy = 1.0 / rect.height() as f64;
    if new_sx.abs() < min_sx {
        new_sx = if new_sx < 0.0 { -min_sx } else { min_sx };
    }
    if new_sy.abs() < min_sy {
        new_sy = if new_sy < 0.0 { -min_sy } else { min_sy };
    }
    if !new_sx.is_finite() || !new_sy.is_finite() {
        return false;
    }
    let a00 = cos * new_sx;
    let a01 = -sin * new_sy;
    let a10 = sin * new_sx;
    let a11 = cos * new_sy;
    let anchor_mapped = (
        a00 * anchor_src.0 + a01 * anchor_src.1,
        a10 * anchor_src.0 + a11 * anchor_src.1,
    );
    let t_x = anchor_doc.0 - anchor_mapped.0;
    let t_y = anchor_doc.1 - anchor_mapped.1;
    session.scale_x = new_sx;
    session.scale_y = new_sy;
    session.angle = start_angle;
    session.dx = t_x - c_x + a00 * c_x + a01 * c_y;
    session.dy = t_y - c_y + a10 * c_x + a11 * c_y;
    true
}

/// Convex point-in-quad test using consistent cross-product signs.
fn point_in_quad(p: (f64, f64), quad: &[(f64, f64); 4]) -> bool {
    let mut sign = 0i32;
    for i in 0..4 {
        let a = quad[i];
        let b = quad[(i + 1) % 4];
        let cross = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        if cross.abs() < 1e-9 {
            continue;
        }
        let s = if cross > 0.0 { 1 } else { -1 };
        if sign == 0 {
            sign = s;
        } else if sign != s {
            return false;
        }
    }
    true
}

/// Which part of the session `(x, y)` hits: 0..=7 handle, 8 rotate, 9 move, -1
/// nothing. Tolerances are in screen pixels and converted through `zoom`.
pub(super) fn hit_test(
    rect: PsdRect,
    sx: f64,
    sy: f64,
    angle: f64,
    dx: f64,
    dy: f64,
    x: f64,
    y: f64,
    zoom: f64,
) -> i32 {
    if !x.is_finite() || !y.is_finite() {
        return -1;
    }
    let zoom = if zoom.is_finite() && zoom > 1e-9 {
        zoom
    } else {
        1.0
    };
    let quad = transform_quad_points(rect, sx, sy, angle, dx, dy);
    let p = (x, y);
    let tol = HANDLE_TOLERANCE / zoom;
    for (i, h) in handle_points(&quad).iter().enumerate() {
        if dist2(p, *h) <= tol * tol {
            return i as i32;
        }
    }
    if point_in_quad(p, &quad) {
        return MOVE_HANDLE;
    }
    let band = ROTATE_BAND / zoom;
    for corner in &quad {
        if dist2(p, *corner) <= band * band {
            return ROTATE_HANDLE;
        }
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect4() -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: 4,
            right: 4,
        }
    }

    fn rect100() -> PsdRect {
        PsdRect {
            top: 0,
            left: 0,
            bottom: 100,
            right: 100,
        }
    }

    fn session() -> TransformSession {
        TransformSession {
            path: "0".to_string(),
            orig_rect: rect4(),
            scale_x: 1.0,
            scale_y: 1.0,
            angle: 0.0,
            dx: 0.0,
            dy: 0.0,
            handle: -1,
            press_x: 0.0,
            press_y: 0.0,
            start: [1.0, 1.0, 0.0, 0.0, 0.0],
            dragging: true,
        }
    }

    #[test]
    fn hit_test_classifies_handles_move_and_rotate() {
        let r = rect100();
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0), 0);
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 100.0, 100.0, 1.0), 2);
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 50.0, 0.0, 1.0), 4);
        assert_eq!(
            hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 50.0, 50.0, 1.0),
            MOVE_HANDLE
        );
        assert_eq!(
            hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, -10.0, -10.0, 1.0),
            ROTATE_HANDLE
        );
        assert_eq!(hit_test(r, 1.0, 1.0, 0.0, 0.0, 0.0, 300.0, 300.0, 1.0), -1);
    }

    #[test]
    fn identity_quad_is_the_source_rect() {
        let q = transform_quad_points(rect4(), 1.0, 1.0, 0.0, 0.0, 0.0);
        assert_eq!(q, [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)]);
    }

    #[test]
    fn opposite_handles_are_paired() {
        assert_eq!(opposite_handle(0), 2);
        assert_eq!(opposite_handle(1), 3);
        assert_eq!(opposite_handle(4), 6);
        assert_eq!(opposite_handle(5), 7);
        for h in 0..8 {
            assert_eq!(opposite_handle(opposite_handle(h)), h);
        }
    }

    #[test]
    fn corner_scale_keeps_the_opposite_corner_fixed() {
        let mut s = session();
        s.handle = 0;
        s.press_x = 0.0;
        s.press_y = 0.0;
        assert!(gesture_scale(&mut s, -4.0, -4.0, false));
        assert!((s.scale_x - 2.0).abs() < 1e-9);
        assert!((s.scale_y - 2.0).abs() < 1e-9);
        let anchor = forward_point(
            s.orig_rect,
            s.scale_x,
            s.scale_y,
            s.angle,
            s.dx,
            s.dy,
            (4.0, 4.0),
        );
        assert!((anchor.0 - 4.0).abs() < 1e-9 && (anchor.1 - 4.0).abs() < 1e-9);
        let dragged = forward_point(
            s.orig_rect,
            s.scale_x,
            s.scale_y,
            s.angle,
            s.dx,
            s.dy,
            (0.0, 0.0),
        );
        assert!((dragged.0 + 4.0).abs() < 1e-9 && (dragged.1 + 4.0).abs() < 1e-9);
    }

    #[test]
    fn shift_on_a_corner_locks_the_two_scales_equal() {
        let mut s = session();
        s.handle = 0;
        assert!(gesture_scale(&mut s, -4.0, -8.0, true));
        assert!((s.scale_x - s.scale_y).abs() < 1e-12);
    }

    #[test]
    fn shift_on_an_edge_handle_leaves_the_other_axis() {
        let mut s = session();
        s.handle = 4;
        assert!(gesture_scale(&mut s, 2.0, -4.0, true));
        assert!((s.scale_x - 1.0).abs() < 1e-9, "x stays 1.0");
        assert!((s.scale_y - 2.0).abs() < 1e-9, "y scales to 2.0");
    }

    #[test]
    fn scale_clamps_to_at_least_one_pixel_per_axis() {
        let mut s = session();
        s.handle = 0;
        assert!(gesture_scale(&mut s, 4.0 - 0.001, 4.0 - 0.001, false));
        assert!((s.scale_x.abs() - 0.25).abs() < 1e-9);
        assert!((s.scale_y.abs() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn rotation_tracks_the_pointer_and_snaps() {
        let mut s = session();
        s.handle = ROTATE_HANDLE;
        s.press_x = 4.0;
        s.press_y = 2.0;
        gesture_rotate(&mut s, 2.0, 4.0, false);
        assert!((s.angle - std::f64::consts::FRAC_PI_2).abs() < 1e-9);

        let mut snapped = session();
        snapped.handle = ROTATE_HANDLE;
        snapped.press_x = 4.0;
        snapped.press_y = 2.0;
        let target: f64 = 0.2;
        gesture_rotate(
            &mut snapped,
            2.0 + target.cos() * 2.0,
            2.0 + target.sin() * 2.0,
            true,
        );
        assert!((snapped.angle - std::f64::consts::PI / 12.0).abs() < 1e-9);
    }

    #[test]
    fn translation_accumulates_from_press() {
        let mut s = session();
        s.handle = MOVE_HANDLE;
        s.press_x = 1.0;
        s.press_y = 1.0;
        gesture_translate(&mut s, 3.0, 5.0);
        assert!((s.dx - 2.0).abs() < 1e-12 && (s.dy - 4.0).abs() < 1e-12);
    }
}
