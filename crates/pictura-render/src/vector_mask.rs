//! Vector-mask coverage for the compositor.
//!
//! The mask path is document-relative, so sampling happens in document pixels.
//! `ponytail:` hard 0/255 coverage and a per-pixel O(edges) scan over every
//! closed subpath, no active-edge table and no antialiasing; add both when
//! curved masks need smoother edges.

use pictura_core::{VectorFillRule, VectorMask, VectorSubpath};

/// Coverage at a canvas pixel: `255` where the layer shows, `0` where the mask
/// hides it. An absent or disabled mask, or one with no closed subpath, is
/// fully permissive.
pub(crate) fn coverage(mask: Option<&VectorMask>, x: i32, y: i32) -> u8 {
    let Some(mask) = mask else {
        return 255;
    };
    if !mask.has_fill() {
        return 255;
    }
    // D4: union only. Every closed subpath contributes edges to one fill; if
    // any declares non-zero, the whole path is sampled non-zero.
    let non_zero = mask
        .subpaths
        .iter()
        .any(|s| s.closed && s.points.len() >= 3 && s.fill_rule == VectorFillRule::NonZero);

    let cx = x as f64 * 256.0 + 128.0;
    let cy = y as f64 * 256.0 + 128.0;
    let inside = if non_zero {
        winding(&mask.subpaths, cx, cy) != 0
    } else {
        crossings(&mask.subpaths, cx, cy) % 2 == 1
    };
    let c = if inside { 255 } else { 0 };
    if mask.invert {
        255 - c
    } else {
        c
    }
}

fn crossings(subpaths: &[VectorSubpath], cx: f64, cy: f64) -> u32 {
    let mut count = 0;
    for subpath in subpaths {
        if !subpath.closed || subpath.points.len() < 3 {
            continue;
        }
        let points = &subpath.points;
        for i in 0..points.len() {
            let (ax, ay) = (points[i][0] as f64, points[i][1] as f64);
            let next = points[(i + 1) % points.len()];
            let (bx, by) = (next[0] as f64, next[1] as f64);
            if (ay > cy) != (by > cy) {
                let xint = ax + (cy - ay) * (bx - ax) / (by - ay);
                if cx < xint {
                    count += 1;
                }
            }
        }
    }
    count
}

fn winding(subpaths: &[VectorSubpath], cx: f64, cy: f64) -> i32 {
    let mut winding = 0;
    for subpath in subpaths {
        if !subpath.closed || subpath.points.len() < 3 {
            continue;
        }
        let points = &subpath.points;
        for i in 0..points.len() {
            let (ax, ay) = (points[i][0] as f64, points[i][1] as f64);
            let next = points[(i + 1) % points.len()];
            let (bx, by) = (next[0] as f64, next[1] as f64);
            let left = (bx - ax) * (cy - ay) - (cx - ax) * (by - ay);
            if ay <= cy {
                if by > cy && left > 0.0 {
                    winding += 1;
                }
            } else if by <= cy && left < 0.0 {
                winding -= 1;
            }
        }
    }
    winding
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::VectorSubpath;

    fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> VectorSubpath {
        let p = |x: i32, y: i32| [x * 256, y * 256];
        VectorSubpath {
            closed: true,
            operation: 1,
            fill_rule: VectorFillRule::EvenOdd,
            points: vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)],
        }
    }

    fn mask(subpaths: Vec<VectorSubpath>) -> VectorMask {
        VectorMask {
            subpaths,
            invert: false,
            disabled: false,
        }
    }

    #[test]
    fn absent_or_disabled_is_permissive() {
        assert_eq!(coverage(None, 0, 0), 255);
        let mut m = mask(vec![rect(0, 0, 2, 2)]);
        m.disabled = true;
        assert_eq!(coverage(Some(&m), 0, 0), 255);
    }

    #[test]
    fn no_closed_subpath_is_permissive() {
        assert_eq!(coverage(Some(&mask(Vec::new())), 0, 0), 255);
        let mut open = rect(0, 0, 2, 2);
        open.closed = false;
        assert_eq!(coverage(Some(&mask(vec![open])), 0, 0), 255);
    }

    #[test]
    fn even_odd_clips_inside_the_rectangle() {
        let m = mask(vec![rect(1, 1, 3, 3)]);
        assert_eq!(coverage(Some(&m), 1, 1), 255);
        assert_eq!(coverage(Some(&m), 2, 2), 255);
        assert_eq!(coverage(Some(&m), 0, 0), 0);
        assert_eq!(coverage(Some(&m), 3, 3), 0);
    }

    #[test]
    fn invert_flips_the_inside() {
        let mut m = mask(vec![rect(1, 1, 3, 3)]);
        m.invert = true;
        assert_eq!(coverage(Some(&m), 1, 1), 0);
        assert_eq!(coverage(Some(&m), 0, 0), 255);
    }

    #[test]
    fn an_inner_winding_subpath_cuts_a_hole() {
        // Two same-winding rectangles nest: even-odd leaves the inner one out.
        let m = mask(vec![rect(0, 0, 4, 4), rect(1, 1, 3, 3)]);
        assert_eq!(coverage(Some(&m), 2, 2), 0, "the inner rectangle is a hole");
        assert_eq!(coverage(Some(&m), 0, 0), 255);
    }
}
