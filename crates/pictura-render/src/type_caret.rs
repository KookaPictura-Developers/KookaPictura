//! Caret geometry for the Type tools: where the caret stands at every
//! character boundary of point type, from the same layout the render uses, so
//! a click lands between the letters it is drawn between. Ported from
//! photorust's `CanvasView::typeCaretRect` / `typeFlowOffset`, which measured
//! with Qt's font metrics; here the spec's face measures (`fonts::face_for`).

use pictura_core::TypeSpec;

use crate::fonts::face_for;
use crate::type_layer::{align_factor, flat_rect, map, text_box_any, type_lines};

/// One caret segment per UTF-16 boundary of `spec.text` (`len + 1` of them,
/// the boundary inside a surrogate pair repeating the one before it), each
/// `[x0, y0, x1, y1]` in document pixels after `spec.matrix`. Horizontal type's
/// caret runs ascent to descent across the line at the boundary's pen position
/// (the line shifted by its alignment); vertical type's runs across the column
/// above the boundary's cell. Empty for a non-positive size or missing face.
pub fn type_caret_stops(spec: &TypeSpec) -> Vec<[f64; 4]> {
    let (Some(bundled), Some((left, top, width, height))) =
        (face_for(&spec.font), text_box_any(spec))
    else {
        return Vec::new();
    };
    let rect = flat_rect(left, top, width, height);
    let (rl, rt) = (rect.left as f32, rect.top as f32);
    let (rw, rh) = (rect.width() as f32, rect.height() as f32);
    let size = spec.size as f32;
    let ascent = bundled.ascent(size);
    let descent = bundled.descent(size);
    let leading = size * 1.2;
    let cell = ascent + descent;
    let factor = align_factor(spec.justification);
    let mut stops = Vec::new();
    let mut push = |c: Option<char>, a: (f32, f32), b: (f32, f32)| {
        let (ox, oy) = (spec.origin.0, spec.origin.1);
        let rel = |p: (f32, f32)| map(spec, f64::from(rl + p.0) - ox, f64::from(rt + p.1) - oy);
        let (p, q) = (rel(a), rel(b));
        for _ in 0..c.map_or(1, char::len_utf16) {
            stops.push([p.0, p.1, q.0, q.1]);
        }
    };
    for (i, line) in type_lines(&spec.text).iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        if spec.vertical {
            let centre = rw - leading * (i as f32 + 0.5);
            let run = (rh - chars.len() as f32 * cell) * factor;
            for k in 0..=chars.len() {
                let y = run + k as f32 * cell;
                let a = (centre - leading / 2.0, y);
                push(chars.get(k).copied(), a, (centre + leading / 2.0, y));
            }
        } else {
            let advance = |n: usize| -> f32 {
                let prefix: String = chars[..n].iter().collect();
                bundled
                    .shape_line(&prefix, size, 0.0)
                    .iter()
                    .map(|g| g.advance)
                    .sum()
            };
            let shift = (rw - advance(chars.len())) * factor;
            let baseline = ascent + i as f32 * leading;
            for k in 0..=chars.len() {
                let x = shift + advance(k);
                let a = (x, baseline - ascent);
                push(chars.get(k).copied(), a, (x, baseline + descent));
            }
        }
    }
    stops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(text: &str, vertical: bool, justification: u8) -> TypeSpec {
        TypeSpec {
            text: text.into(),
            font: "Liberation Sans".into(),
            size: 32.0,
            color: [0, 0, 0, 255],
            justification,
            vertical,
            antialias: true,
            origin: (100.0, 80.0),
            matrix: TypeSpec::IDENTITY,
        }
    }

    #[test]
    fn stops_step_along_lines_and_down_the_columns() {
        let empty = type_caret_stops(&spec("", false, 0));
        assert_eq!(empty.len(), 1);
        assert!((empty[0][0] - 100.0).abs() <= 1.0 && empty[0][1] < 80.0 && empty[0][3] > 80.0);

        // "ab\rc": stops 0..=2 on line one, rising; 3 and 4 on line two.
        let stops = type_caret_stops(&spec("ab\rc", false, 0));
        assert_eq!(stops.len(), 5);
        assert!(stops[0][0] < stops[1][0] && stops[1][0] < stops[2][0]);
        assert!((stops[3][0] - stops[0][0]).abs() < 0.01);
        assert!((stops[3][1] - stops[0][1] - 38.4).abs() < 0.01);

        // Right alignment ends the line at the click.
        let right = type_caret_stops(&spec("ab", false, 1));
        assert!((right[2][0] - 100.0).abs() <= 1.5, "{:?}", right[2]);

        // Vertical: each boundary one cell further down, flat across the column.
        let column = type_caret_stops(&spec("ab", true, 0));
        assert_eq!(column.len(), 3);
        assert_eq!(column[0][1], column[0][3]);
        assert!(column[1][1] > column[0][1] && column[0][0] < 100.0 && column[0][2] > 100.0);

        // A surrogate pair repeats its leading stop; a turn maps every stop.
        assert_eq!(type_caret_stops(&spec("a\u{1F600}", false, 0)).len(), 4);
        let mut turned = spec("ab", false, 0);
        turned.matrix = [0.0, 1.0, -1.0, 0.0];
        let t = type_caret_stops(&turned);
        assert!((t[0][1] - t[0][3]).abs() < 0.01 && t[2][1] > t[0][1]);
    }
}
