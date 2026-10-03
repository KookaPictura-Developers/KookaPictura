//! The Line tool's outline. CS6's Line draws a filled shape rather than a
//! stroke (that is why it has a Weight, not a brush), so the outline is the
//! quad a `weight`-wide segment sweeps, with optional arrowheads in place of
//! its ends. Ported from photorust's `line_points`; the arrowheads are CS6's
//! (photorust has none), their geometry an approximation.

use super::{closed, point, Arrowheads};
use crate::path::Subpath;

/// The line from `from` to `to`, `weight` pixels wide (at least 1); Shift snaps
/// its angle to 45°. `None` for a line with no length.
pub(super) fn line(
    from: (f64, f64),
    to: (f64, f64),
    weight: f64,
    arrows: Arrowheads,
    shift: bool,
) -> Option<Subpath> {
    let (mut dx, mut dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.hypot(dy);
    if length <= 0.0 {
        return None;
    }
    if shift {
        let step = std::f64::consts::FRAC_PI_4;
        let angle = (dy.atan2(dx) / step).round() * step;
        (dx, dy) = (length * angle.cos(), length * angle.sin());
    }
    let half = weight.max(1.0) / 2.0;
    // Arrowhead size, never narrower than the line itself.
    let head_half = (weight.max(1.0) * arrows.width.clamp(10.0, 1000.0) / 100.0 / 2.0).max(half);
    let head_len = weight.max(1.0) * arrows.length.clamp(10.0, 5000.0) / 100.0;
    let concave = arrows.concavity.clamp(-50.0, 50.0) / 100.0 * head_len;
    // Where a head's sloped base meets the line's edge, measured from its tip:
    // the base runs from the corner (head_len back) to the centre
    // (head_len - concave back).
    let meet = head_len - concave * (1.0 - half / head_half);

    // Local frame: x along the line from 0 to `length`, y across it.
    let mut local: Vec<(f64, f64)> = Vec::with_capacity(10);
    if arrows.start {
        local.extend([(0.0, 0.0), (head_len, -head_half), (meet, -half)]);
    } else {
        local.push((0.0, -half));
    }
    if arrows.end {
        local.extend([
            (length - meet, -half),
            (length - head_len, -head_half),
            (length, 0.0),
            (length - head_len, head_half),
            (length - meet, half),
        ]);
    } else {
        local.extend([(length, -half), (length, half)]);
    }
    if arrows.start {
        local.extend([(meet, half), (head_len, head_half)]);
    } else {
        local.push((0.0, half));
    }

    let (ux, uy) = (dx / length, dy / length);
    Some(closed(
        local
            .into_iter()
            .map(|(x, y)| {
                point(
                    (from.0 + x * ux - y * uy, from.1 + x * uy + y * ux),
                    None,
                    None,
                )
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchors(s: &Subpath) -> Vec<(f64, f64)> {
        s.points.iter().map(|p| p.anchor).collect()
    }

    #[test]
    fn a_line_is_a_quad_of_its_weight() {
        let s = line(
            (10.0, 10.0),
            (10.0, 50.0),
            6.0,
            Arrowheads::default(),
            false,
        )
        .unwrap();
        assert_eq!(
            anchors(&s),
            vec![(13.0, 10.0), (13.0, 50.0), (7.0, 50.0), (7.0, 10.0)]
        );
        assert!(line((5.0, 5.0), (5.0, 5.0), 4.0, Arrowheads::default(), false).is_none());
    }

    #[test]
    fn shift_snaps_the_angle_to_45_degrees() {
        let s = line((0.0, 0.0), (100.0, 8.0), 2.0, Arrowheads::default(), true).unwrap();
        assert!(s
            .points
            .iter()
            .all(|p| (p.anchor.1.abs() - 1.0).abs() < 1e-9));
    }

    #[test]
    fn arrowheads_replace_the_ends() {
        let arrows = Arrowheads {
            start: true,
            end: true,
            width: 500.0,
            length: 1000.0,
            concavity: 0.0,
        };
        let s = line((0.0, 0.0), (100.0, 0.0), 2.0, arrows, false).unwrap();
        let a = anchors(&s);
        assert_eq!(a.len(), 10);
        assert!(
            a.contains(&(0.0, 0.0)) && a.contains(&(100.0, 0.0)),
            "tips at the ends"
        );
        assert!(
            a.contains(&(20.0, -5.0)) && a.contains(&(80.0, 5.0)),
            "heads 10 wide, 20 long"
        );
        assert!(
            a.contains(&(20.0, -1.0)),
            "a flat base meets the line where the head ends"
        );

        let concave = Arrowheads {
            concavity: 50.0,
            start: false,
            ..arrows
        };
        let c = line((0.0, 0.0), (100.0, 0.0), 2.0, concave, false).unwrap();
        // The base's corners stay 20 px back (x = 80) and its centre comes
        // 10 px nearer the tip (x = 90), so it crosses the line's edge
        // (y = -1, four fifths of the way in) at x = 88.
        assert!(anchors(&c)
            .iter()
            .any(|&(x, y)| (x - 88.0).abs() < 1e-9 && (y + 1.0).abs() < 1e-9));
    }
}
