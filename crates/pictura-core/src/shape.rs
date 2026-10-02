//! The geometry behind the Rectangle, Rounded Rectangle, Ellipse, and Polygon
//! tools (`docs/03-tools/shape-tools.md`).
//!
//! The tools differ only in the outline a drag produces; what becomes of it (a
//! shape layer, a Work Path component, or pixels) is the caller's business. The
//! outline is one closed [`Subpath`] in the Work Path's own model, so curves
//! stay cubic Beziers rather than being flattened here: the live preview, the
//! path, and the vector mask all draw the same curve.
//!
//! Ported from photorust's `core/src/shape.rs`, which flattened every outline
//! to points; here arcs are quarter-circle cubics.

use crate::path::{PathPoint, Subpath};

/// The cubic handle length, as a fraction of the radius, that best fits a
/// quarter circle.
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// Flattening tolerance for [`coverage`], in document pixels.
const COVERAGE_TOLERANCE: f64 = 0.1;

/// Vertical samples per pixel row in [`coverage`].
const SUBROWS: usize = 4;

/// Which shape tool drew the outline.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ShapeKind {
    #[default]
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Polygon,
}

impl ShapeKind {
    /// The bridge's integer code: 0 Rectangle, 1 Rounded Rectangle, 2 Ellipse,
    /// 3 Polygon.
    pub fn from_i32(v: i32) -> Option<ShapeKind> {
        match v {
            0 => Some(ShapeKind::Rectangle),
            1 => Some(ShapeKind::RoundedRectangle),
            2 => Some(ShapeKind::Ellipse),
            3 => Some(ShapeKind::Polygon),
            _ => None,
        }
    }

    /// A shape layer is named after the tool that drew it.
    pub fn layer_name(self) -> &'static str {
        match self {
            ShapeKind::Rectangle => "Rectangle",
            ShapeKind::RoundedRectangle => "Rounded Rectangle",
            ShapeKind::Ellipse => "Ellipse",
            ShapeKind::Polygon => "Polygon",
        }
    }
}

/// The geometry options a tool reads: Rounded Rectangle's corner `radius` in
/// pixels, Polygon's `sides` (clamped to 3–100).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ShapeOptions {
    pub kind: ShapeKind,
    pub radius: f64,
    pub sides: u32,
}

/// The closed outline a drag from `from` to `to` draws, or `None` when the
/// drag encloses no area.
///
/// The rectangle-based tools square off under `shift` (Circle for the
/// Ellipse) and grow from `from` under `alt`. The Polygon is always centred on
/// `from`, the drag is its radius, its first vertex sits under the pointer, and
/// `shift` snaps that rotation to 15°.
pub fn outline(
    options: ShapeOptions,
    from: (f64, f64),
    to: (f64, f64),
    shift: bool,
    alt: bool,
) -> Option<Subpath> {
    let subpath = match options.kind {
        ShapeKind::Polygon => polygon(from, to, options.sides, shift)?,
        kind => {
            let rect = drag_rect(from, to, shift, alt);
            if rect.2 <= 0.0 || rect.3 <= 0.0 {
                return None;
            }
            match kind {
                ShapeKind::RoundedRectangle => rounded_rectangle(rect, options.radius),
                ShapeKind::Ellipse => ellipse(rect),
                _ => rectangle(rect),
            }
        }
    };
    Some(subpath)
}

/// The `(x, y, w, h)` a drag marks out. Shift squares it off by the longer
/// side, so squaring never pulls the shape back from the pointer; Alt makes the
/// drag a half-diagonal from the centre.
fn drag_rect(from: (f64, f64), to: (f64, f64), shift: bool, alt: bool) -> (f64, f64, f64, f64) {
    let (mut dx, mut dy) = (to.0 - from.0, to.1 - from.1);
    if shift {
        let side = dx.abs().max(dy.abs());
        dx = side.copysign(dx);
        dy = side.copysign(dy);
    }
    if alt {
        (
            from.0 - dx.abs(),
            from.1 - dy.abs(),
            dx.abs() * 2.0,
            dy.abs() * 2.0,
        )
    } else {
        (
            from.0.min(from.0 + dx),
            from.1.min(from.1 + dy),
            dx.abs(),
            dy.abs(),
        )
    }
}

fn point(
    anchor: (f64, f64),
    in_handle: Option<(f64, f64)>,
    out_handle: Option<(f64, f64)>,
) -> PathPoint {
    PathPoint {
        anchor,
        in_handle,
        out_handle,
        smooth: in_handle.is_some() && out_handle.is_some(),
    }
}

fn closed(points: Vec<PathPoint>) -> Subpath {
    Subpath {
        points,
        closed: true,
    }
}

/// Four corners, clockwise from the top-left.
fn rectangle((x, y, w, h): (f64, f64, f64, f64)) -> Subpath {
    closed(
        [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
            .into_iter()
            .map(|a| point(a, None, None))
            .collect(),
    )
}

/// A rectangle whose corners are quarter circles of `radius`, clamped to half
/// the shorter side (a larger radius would cross opposite corners). Each
/// corner is two anchors joined by one cubic; where the clamp leaves no
/// straight edge between two arcs, their anchors merge.
fn rounded_rectangle(rect: (f64, f64, f64, f64), radius: f64) -> Subpath {
    let (x, y, w, h) = rect;
    let r = radius.max(0.0).min(w.min(h) / 2.0);
    if r <= 0.0 {
        return rectangle(rect);
    }
    let k = r * KAPPA;
    let (r0, r1, b0, b1) = (x + r, x + w - r, y + r, y + h - r);
    let (right, bottom) = (x + w, y + h);
    let raw = [
        point((r0, y), Some((r0 - k, y)), None),
        point((r1, y), None, Some((r1 + k, y))),
        point((right, b0), Some((right, b0 - k)), None),
        point((right, b1), None, Some((right, b1 + k))),
        point((r1, bottom), Some((r1 + k, bottom)), None),
        point((r0, bottom), None, Some((r0 - k, bottom))),
        point((x, b1), Some((x, b1 + k)), None),
        point((x, b0), None, Some((x, b0 - k))),
    ];
    let mut points: Vec<PathPoint> = Vec::with_capacity(raw.len());
    for p in raw {
        match points.last_mut() {
            Some(last) if same(last.anchor, p.anchor) => {
                last.out_handle = p.out_handle;
                last.smooth = true;
            }
            _ => points.push(p),
        }
    }
    closed(points)
}

/// Anchors the radius clamp meant to coincide, up to rounding.
fn same(a: (f64, f64), b: (f64, f64)) -> bool {
    (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
}

/// The ellipse inscribed in the rectangle: four smooth anchors, clockwise from
/// the top.
fn ellipse((x, y, w, h): (f64, f64, f64, f64)) -> Subpath {
    let (rx, ry) = (w / 2.0, h / 2.0);
    let (cx, cy) = (x + rx, y + ry);
    let (kx, ky) = (rx * KAPPA, ry * KAPPA);
    closed(vec![
        point((cx, y), Some((cx - kx, y)), Some((cx + kx, y))),
        point((x + w, cy), Some((x + w, cy - ky)), Some((x + w, cy + ky))),
        point((cx, y + h), Some((cx + kx, y + h)), Some((cx - kx, y + h))),
        point((x, cy), Some((x, cy + ky)), Some((x, cy - ky))),
    ])
}

/// A regular polygon of `sides` corners on the circle through `to` centred on
/// `centre`, the first corner under `to`.
fn polygon(centre: (f64, f64), to: (f64, f64), sides: u32, shift: bool) -> Option<Subpath> {
    let sides = sides.clamp(3, 100);
    let (dx, dy) = (to.0 - centre.0, to.1 - centre.1);
    let radius = dx.hypot(dy);
    if radius <= 0.0 {
        return None;
    }
    let mut rotation = dy.atan2(dx);
    if shift {
        let step = std::f64::consts::PI / 12.0;
        rotation = (rotation / step).round() * step;
    }
    Some(closed(
        (0..sides)
            .map(|i| {
                let angle = rotation + f64::from(i) / f64::from(sides) * std::f64::consts::TAU;
                let anchor = (
                    centre.0 + radius * angle.cos(),
                    centre.1 + radius * angle.sin(),
                );
                point(anchor, None, None)
            })
            .collect(),
    ))
}

/// A `width` × `height` document-sized, row-major, anti-aliased coverage mask
/// (0–255) of `subpath`'s interior, even-odd. Each pixel row is sampled on
/// [`SUBROWS`] scanlines and each span's ends are weighted by the fraction of
/// the pixel they cover. An open subpath is closed implicitly.
pub fn coverage(subpath: &Subpath, width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    let mut mask = vec![0u8; w * h];
    let contour = subpath.flatten(COVERAGE_TOLERANCE);
    if contour.len() < 3 || w == 0 || h == 0 {
        return mask;
    }
    let (top, bottom) = contour
        .iter()
        .fold((f64::MAX, f64::MIN), |(t, b), p| (t.min(p.1), b.max(p.1)));
    let first_row = top.floor().max(0.0) as usize;
    let last_row = (bottom.ceil().max(0.0) as usize).min(h);
    let mut row = vec![0f32; w + 1];
    let mut crossings: Vec<f64> = Vec::new();
    for y in first_row..last_row {
        row.iter_mut().for_each(|c| *c = 0.0);
        for s in 0..SUBROWS {
            let sy = y as f64 + (s as f64 + 0.5) / SUBROWS as f64;
            crossings.clear();
            let mut j = contour.len() - 1;
            for (i, &(xi, yi)) in contour.iter().enumerate() {
                let (xj, yj) = contour[j];
                if (yi > sy) != (yj > sy) {
                    crossings.push(xi + (sy - yi) / (yj - yi) * (xj - xi));
                }
                j = i;
            }
            crossings.sort_by(f64::total_cmp);
            for &[x0, x1] in crossings.as_chunks::<2>().0 {
                add_span(&mut row, x0.max(0.0), x1.min(w as f64));
            }
        }
        for (x, c) in row[..w].iter().enumerate() {
            mask[y * w + x] = (c / SUBROWS as f32 * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    mask
}

/// Accumulate horizontal coverage of `[x0, x1)` into `row` (one cell per pixel).
fn add_span(row: &mut [f32], x0: f64, x1: f64) {
    if x1 <= x0 {
        return;
    }
    let (first, last) = (x0.floor() as usize, x1.floor() as usize);
    if first == last {
        row[first] += (x1 - x0) as f32;
        return;
    }
    row[first] += (first as f64 + 1.0 - x0) as f32;
    for c in &mut row[first + 1..last] {
        *c += 1.0;
    }
    if last < row.len() {
        row[last] += (x1 - last as f64) as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::VectorPath;

    fn opts(kind: ShapeKind) -> ShapeOptions {
        ShapeOptions {
            kind,
            radius: 10.0,
            sides: 5,
        }
    }

    fn anchors(s: &Subpath) -> Vec<(f64, f64)> {
        s.points.iter().map(|p| p.anchor).collect()
    }

    /// The flattened curve's `(left, top, right, bottom)`.
    fn bounds(s: &Subpath) -> (f64, f64, f64, f64) {
        let mut path = VectorPath::default();
        path.add_subpath(s.clone());
        path.subpath_bounds(0).expect("non-empty")
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn a_rectangle_is_four_corners_clockwise() {
        let s = outline(
            opts(ShapeKind::Rectangle),
            (40.0, 60.0),
            (10.0, 20.0),
            false,
            false,
        )
        .unwrap();
        assert!(s.closed);
        assert_eq!(
            anchors(&s),
            vec![(10.0, 20.0), (40.0, 20.0), (40.0, 60.0), (10.0, 60.0)]
        );
        assert!(s
            .points
            .iter()
            .all(|p| p.in_handle.is_none() && p.out_handle.is_none()));
    }

    #[test]
    fn shift_squares_by_the_longer_side_and_alt_grows_from_the_centre() {
        let sq = outline(
            opts(ShapeKind::Rectangle),
            (0.0, 0.0),
            (40.0, -10.0),
            true,
            false,
        )
        .unwrap();
        assert_eq!(bounds(&sq), (0.0, -40.0, 40.0, 0.0));
        let centred = outline(
            opts(ShapeKind::Rectangle),
            (50.0, 50.0),
            (60.0, 70.0),
            false,
            true,
        )
        .unwrap();
        assert_eq!(bounds(&centred), (40.0, 30.0, 60.0, 70.0));
    }

    #[test]
    fn a_drag_that_encloses_nothing_draws_nothing() {
        for kind in [
            ShapeKind::Rectangle,
            ShapeKind::RoundedRectangle,
            ShapeKind::Ellipse,
            ShapeKind::Polygon,
        ] {
            assert!(outline(opts(kind), (5.0, 5.0), (5.0, 5.0), false, false).is_none());
        }
        assert!(outline(
            opts(ShapeKind::Rectangle),
            (5.0, 5.0),
            (30.0, 5.0),
            false,
            false
        )
        .is_none());
    }

    #[test]
    fn rounded_corners_are_quarter_circles_inside_the_box() {
        let s = outline(
            opts(ShapeKind::RoundedRectangle),
            (0.0, 0.0),
            (100.0, 60.0),
            false,
            false,
        )
        .unwrap();
        assert_eq!(s.points.len(), 8);
        let (l, t, r, b) = bounds(&s);
        assert!(near(l, 0.0) && near(t, 0.0) && near(r, 100.0) && near(b, 60.0));
        // The top-left arc's midpoint sits on the circle of radius 10 about (10, 10).
        let seg = s.segment(7).unwrap();
        let mid = (
            (seg[0].0 + 3.0 * seg[1].0 + 3.0 * seg[2].0 + seg[3].0) / 8.0,
            (seg[0].1 + 3.0 * seg[1].1 + 3.0 * seg[2].1 + seg[3].1) / 8.0,
        );
        assert!(((mid.0 - 10.0).hypot(mid.1 - 10.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn a_huge_radius_clamps_to_a_stadium_without_duplicate_anchors() {
        let mut o = opts(ShapeKind::RoundedRectangle);
        o.radius = 500.0;
        let s = outline(o, (0.0, 0.0), (100.0, 40.0), false, false).unwrap();
        assert_eq!(
            s.points.len(),
            6,
            "the vertical edges collapse into arc joins"
        );
        let (l, t, r, b) = bounds(&s);
        assert!(near(l, 0.0) && near(t, 0.0) && near(r, 100.0) && near(b, 40.0));
        let circle = outline(o, (0.0, 0.0), (40.0, 40.0), false, false).unwrap();
        assert_eq!(circle.points.len(), 4);
        assert!(circle.points.iter().all(|p| p.smooth));
    }

    #[test]
    fn an_ellipse_is_four_smooth_anchors_filling_its_box() {
        let s = outline(
            opts(ShapeKind::Ellipse),
            (0.0, 0.0),
            (80.0, 40.0),
            false,
            false,
        )
        .unwrap();
        assert_eq!(
            anchors(&s),
            vec![(40.0, 0.0), (80.0, 20.0), (40.0, 40.0), (0.0, 20.0)]
        );
        assert!(s.points.iter().all(|p| p.smooth));
        let (l, t, r, b) = bounds(&s);
        assert!(near(l, 0.0) && near(t, 0.0) && near(r, 80.0) && near(b, 40.0));
        let circle = outline(
            opts(ShapeKind::Ellipse),
            (0.0, 0.0),
            (30.0, 10.0),
            true,
            false,
        )
        .unwrap();
        assert_eq!(bounds(&circle), (0.0, 0.0, 30.0, 30.0));
    }

    #[test]
    fn a_polygon_centres_on_the_press_and_turns_to_the_pointer() {
        let mut o = opts(ShapeKind::Polygon);
        o.sides = 6;
        let s = outline(o, (50.0, 50.0), (50.0, 20.0), false, false).unwrap();
        assert_eq!(s.points.len(), 6);
        assert!(near(s.points[0].anchor.0, 50.0) && near(s.points[0].anchor.1, 20.0));
        for (x, y) in anchors(&s) {
            assert!(near((x - 50.0).hypot(y - 50.0), 30.0));
        }
        // 50° snaps to 45° under Shift.
        let a = 50f64.to_radians();
        let snapped =
            outline(o, (0.0, 0.0), (10.0 * a.cos(), 10.0 * a.sin()), true, false).unwrap();
        let first = snapped.points[0].anchor;
        assert!(near(first.0, first.1));
        o.sides = 1;
        assert_eq!(
            outline(o, (0.0, 0.0), (5.0, 0.0), false, false)
                .unwrap()
                .points
                .len(),
            3
        );
    }

    #[test]
    fn coverage_fills_the_interior_and_antialiases_the_edge() {
        let s = outline(
            opts(ShapeKind::Rectangle),
            (2.0, 2.0),
            (6.5, 6.0),
            false,
            false,
        )
        .unwrap();
        let m = coverage(&s, 8, 8);
        assert_eq!(m[3 * 8 + 3], 255);
        assert_eq!(m[3 * 8 + 1], 0);
        assert_eq!(m[3 * 8 + 6], 128, "half the pixel is covered");
        assert_eq!(m[7 * 8 + 3], 0);
        let total: u32 = m.iter().map(|&v| u32::from(v)).sum();
        assert!((total as f64 / 255.0 - 18.0).abs() < 0.1, "area is 4.5 x 4");
        let off = outline(
            opts(ShapeKind::Ellipse),
            (-20.0, -20.0),
            (40.0, 40.0),
            false,
            false,
        )
        .unwrap();
        assert_eq!(coverage(&off, 8, 8).len(), 64, "clipped to the canvas");
    }
}
