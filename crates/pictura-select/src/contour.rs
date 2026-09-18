//! 50 %-coverage marching-squares outline of a selection mask.
//!
//! The overlay needs the committed edge (marching ants), not pixel data. We walk
//! the boundary as unit edges on the integer pixel lattice and chain them into
//! closed polylines. The drawn union of edges is the exact 4-connected outline;
//! degree-4 saddle vertices may split a loop into more than one polyline, which
//! is fine for drawing. Derived and view-only: never stored.

use std::collections::HashMap;

use crate::Selection;

/// Closed polylines (integer pixel-corner lattice, `[0, width] x [0, height]`)
/// tracing the boundary of the pixels whose coverage is `>= threshold`.
///
/// The first vertex is not repeated at the end; draw each loop closed.
pub fn contour(selection: &Selection, threshold: u8) -> Vec<Vec<[i32; 2]>> {
    let (w, h) = (selection.width as usize, selection.height as usize);
    if w == 0 || h == 0 || selection.data.len() != w * h {
        return Vec::new();
    }
    let inside = |x: i64, y: i64| -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < w
            && (y as usize) < h
            && selection.data[y as usize * w + x as usize] >= threshold
    };

    // One pass for the inside bounding box, so a small selection in a huge
    // document never scans the whole mask twice.
    let mut min_x = w as i64;
    let mut min_y = h as i64;
    let mut max_x = -1i64;
    let mut max_y = -1i64;
    let mut inside_count = 0usize;
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            if selection.data[y as usize * w + x as usize] >= threshold {
                inside_count += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if inside_count == 0 {
        return Vec::new();
    }
    if inside_count == w * h {
        // Select All: the outline is the document rectangle; skip the edge scan.
        return vec![vec![
            [0, 0],
            [0, h as i32],
            [w as i32, h as i32],
            [w as i32, 0],
        ]];
    }

    let vw = w as i64 + 1;
    let key = |x: i64, y: i64| -> i64 { y * vw + x };
    let decode = |k: i64| -> [i32; 2] { [(k % vw) as i32, (k / vw) as i32] };

    let mut edges: Vec<[i64; 2]> = Vec::new();
    let mut adj: HashMap<i64, Vec<usize>> = HashMap::new();
    {
        let mut push_edge = |a: (i64, i64), b: (i64, i64)| {
            let (ka, kb) = (key(a.0, a.1), key(b.0, b.1));
            let id = edges.len();
            edges.push([ka, kb]);
            adj.entry(ka).or_default().push(id);
            adj.entry(kb).or_default().push(id);
        };

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if !inside(x, y) {
                    continue;
                }
                if !inside(x, y - 1) {
                    push_edge((x, y), (x + 1, y));
                }
                if !inside(x, y + 1) {
                    push_edge((x, y + 1), (x + 1, y + 1));
                }
                if !inside(x - 1, y) {
                    push_edge((x, y), (x, y + 1));
                }
                if !inside(x + 1, y) {
                    push_edge((x + 1, y), (x + 1, y + 1));
                }
            }
        }
    }

    let mut used = vec![false; edges.len()];
    let mut loops: Vec<Vec<[i32; 2]>> = Vec::new();
    for start in 0..edges.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let first = edges[start][0];
        let mut cur = edges[start][1];
        let mut poly = vec![first, cur];
        loop {
            let mut next_edge = None;
            if let Some(list) = adj.get(&cur) {
                for &e in list {
                    if !used[e] {
                        next_edge = Some(e);
                        break;
                    }
                }
            }
            let Some(e) = next_edge else {
                break;
            };
            used[e] = true;
            let [p, q] = edges[e];
            let next = if p == cur { q } else { p };
            if next == first {
                break;
            }
            cur = next;
            poly.push(cur);
        }
        loops.push(poly.into_iter().map(decode).collect());
    }
    loops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(w: u32, h: u32, x0: i64, y0: i64, x1: i64, y1: i64) -> Selection {
        let mut s = Selection::none(w, h);
        for y in y0.max(0)..y1.min(h as i64) {
            for x in x0.max(0)..x1.min(w as i64) {
                s.data[(y * w as i64 + x) as usize] = 255;
            }
        }
        s
    }

    fn has_vertex(loop_: &[[i32; 2]], p: [i32; 2]) -> bool {
        loop_.contains(&p)
    }

    #[test]
    fn rectangle_is_one_loop_with_corners() {
        let sel = filled(6, 6, 1, 1, 5, 4);
        let loops = contour(&sel, 128);
        assert_eq!(loops.len(), 1);
        let l = &loops[0];
        assert!(has_vertex(l, [1, 1]), "top-left corner");
        assert!(has_vertex(l, [5, 1]), "top-right corner");
        assert!(has_vertex(l, [5, 4]), "bottom-right corner");
        assert!(has_vertex(l, [1, 4]), "bottom-left corner");
    }

    #[test]
    fn interior_hole_yields_two_loops() {
        let mut sel = filled(8, 8, 0, 0, 8, 8);
        for y in 3..5 {
            for x in 3..5 {
                sel.data[y * 8 + x] = 0;
            }
        }
        let loops = contour(&sel, 128);
        assert_eq!(loops.len(), 2);
        let outer = loops
            .iter()
            .any(|l| has_vertex(l, [0, 0]) && has_vertex(l, [8, 8]));
        let hole = loops
            .iter()
            .any(|l| has_vertex(l, [3, 3]) && has_vertex(l, [5, 5]));
        assert!(outer, "outer document boundary");
        assert!(hole, "inner hole boundary");
    }

    #[test]
    fn empty_selection_yields_none() {
        assert!(contour(&Selection::none(4, 4), 128).is_empty());
        assert!(contour(&filled(4, 4, 9, 9, 11, 11), 128).is_empty());
    }

    #[test]
    fn threshold_respects_partial_coverage() {
        let mut sel = Selection::none(4, 4);
        sel.data[0] = 100;
        assert!(contour(&sel, 128).is_empty(), "100 < 128 is outside");
        let loops = contour(&sel, 50);
        assert_eq!(loops.len(), 1);
        assert!(has_vertex(&loops[0], [0, 0]));
        assert!(has_vertex(&loops[0], [1, 1]));
    }

    #[test]
    fn select_all_is_the_document_rectangle() {
        let loops = contour(&Selection::all(4, 5), 128);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0], vec![[0, 0], [0, 5], [4, 5], [4, 0]]);
    }
}
