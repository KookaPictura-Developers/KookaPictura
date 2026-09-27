//! Edge snapping for the Magnetic Lasso: a *live wire*. The segment from the
//! last fastening point to the pointer is the cheapest path through an
//! edge-cost field, so it clings to the boundary being traced. The field is
//! built once per gesture; each pointer move is one bounded Dijkstra query.
//!
//! **Contrast** (1–100 %) is the gradient strength below which nothing counts
//! as an edge; **Width** is how far either side of the straight segment the
//! search may wander. Both map onto CS6's options
//! (`docs/03-tools/lasso-selection.md`).
//!
//! ponytail: CS6's cost function is closed; this is a Sobel-magnitude live wire
//! (no gradient-direction or Laplacian terms), so it approximates Photoshop's
//! snapping rather than matching it.
//!
//! Ported from photorust's `core/src/magnetic.rs`
//! (<https://github.com/perfecto25/photorust>).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use pictura_core::PixelBuffer;

/// Largest corridor, in pixels, a single trace explores. The wire is re-traced
/// on every pointer move, so past this the query returns a straight line — a
/// jump that long means the user is no longer tracing an edge.
const MAX_TRACE_PIXELS: usize = 1 << 20;

/// Cost charged per step regardless of edge strength, so the wire prefers the
/// short route over a long detour along a slightly stronger edge.
const STEP_COST: f32 = 0.05;

/// A per-pixel traversal cost field: 0 on a strong edge, 1 on a flat area.
#[derive(Clone, Debug)]
pub struct EdgeMap {
    width: u32,
    height: u32,
    cost: Vec<f32>,
}

impl EdgeMap {
    /// Build the cost field from a planar composite (1–2 planes read as gray,
    /// 3+ as RGB). Gradients below `contrast` % of the image's peak are
    /// flattened to "no edge"; what survives is rescaled across the remaining
    /// range.
    pub fn from_buffer(buffer: &PixelBuffer, contrast: u32) -> Self {
        let (width, height) = (buffer.width, buffer.height);
        let (w, h) = (width as usize, height as usize);
        let mut cost = vec![1.0f32; w * h];
        if width < 3 || height < 3 || buffer.data.len() < w * h * buffer.channels as usize {
            return Self {
                width,
                height,
                cost,
            };
        }

        let plane = w * h;
        let luma: Vec<f32> = (0..plane)
            .map(|i| {
                if buffer.channels >= 3 {
                    0.299 * buffer.data[i] as f32
                        + 0.587 * buffer.data[plane + i] as f32
                        + 0.114 * buffer.data[2 * plane + i] as f32
                } else {
                    buffer.data[i] as f32
                }
            })
            .collect();

        // Sobel, skipping the one-pixel border; those pixels keep the maximum
        // cost, which also stops the wire running along the canvas edge.
        let mut magnitude = vec![0.0f32; plane];
        let mut peak = 0.0f32;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let at = |dx: usize, dy: usize| luma[(y + dy - 1) * w + (x + dx - 1)];
                let gx =
                    -at(0, 0) - 2.0 * at(0, 1) - at(0, 2) + at(2, 0) + 2.0 * at(2, 1) + at(2, 2);
                let gy =
                    -at(0, 0) - 2.0 * at(1, 0) - at(2, 0) + at(0, 2) + 2.0 * at(1, 2) + at(2, 2);
                let g = (gx * gx + gy * gy).sqrt();
                magnitude[y * w + x] = g;
                peak = peak.max(g);
            }
        }
        if peak <= 0.0 {
            // A flat image has nothing to snap to; the wire comes out straight.
            return Self {
                width,
                height,
                cost,
            };
        }

        let threshold = contrast.clamp(1, 100) as f32 / 100.0;
        let span = (1.0 - threshold).max(1e-6);
        for (c, &g) in cost.iter_mut().zip(&magnitude) {
            let strength = ((g / peak - threshold) / span).clamp(0.0, 1.0);
            *c = 1.0 - strength;
        }
        Self {
            width,
            height,
            cost,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn contains(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    /// The cheapest 8-connected path from `from` to `to`, both inclusive,
    /// searched within the segment's bounding box grown by `width` px. Falls
    /// back to a straight line when an endpoint is off-canvas, the corridor is
    /// too large to search interactively, or nothing better exists, so it never
    /// fails.
    pub fn trace(&self, from: (i32, i32), to: (i32, i32), width: u32) -> Vec<(i32, i32)> {
        if !self.contains(from) || !self.contains(to) {
            return straight_line(from, to);
        }
        if from == to {
            return vec![from];
        }

        let grow = width.max(1) as i32;
        let left = (from.0.min(to.0) - grow).max(0);
        let top = (from.1.min(to.1) - grow).max(0);
        let right = (from.0.max(to.0) + grow + 1).min(self.width as i32);
        let bottom = (from.1.max(to.1) + grow + 1).min(self.height as i32);
        let (rw, rh) = ((right - left) as usize, (bottom - top) as usize);
        if rw * rh > MAX_TRACE_PIXELS {
            return straight_line(from, to);
        }
        let index = |x: i32, y: i32| (y - top) as usize * rw + (x - left) as usize;

        let mut best = vec![f32::INFINITY; rw * rh];
        let mut came_from = vec![usize::MAX; rw * rh];
        let mut settled = vec![false; rw * rh];
        let (start, goal) = (index(from.0, from.1), index(to.0, to.1));
        best[start] = 0.0;
        let mut queue = BinaryHeap::from([Step {
            cost: 0.0,
            at: start,
        }]);

        // Diagonals pay their real length so the wire does not staircase.
        const SQRT_2: f32 = std::f32::consts::SQRT_2;
        const NEIGHBOURS: [(i32, i32, f32); 8] = [
            (-1, 0, 1.0),
            (1, 0, 1.0),
            (0, -1, 1.0),
            (0, 1, 1.0),
            (-1, -1, SQRT_2),
            (1, -1, SQRT_2),
            (-1, 1, SQRT_2),
            (1, 1, SQRT_2),
        ];

        while let Some(Step { at, .. }) = queue.pop() {
            if settled[at] {
                continue;
            }
            settled[at] = true;
            if at == goal {
                break;
            }
            let x = left + (at % rw) as i32;
            let y = top + (at / rw) as i32;
            for &(dx, dy, length) in &NEIGHBOURS {
                let (nx, ny) = (x + dx, y + dy);
                if nx < left || ny < top || nx >= right || ny >= bottom {
                    continue;
                }
                let next = index(nx, ny);
                if settled[next] {
                    continue;
                }
                let edge = self.cost[ny as usize * self.width as usize + nx as usize];
                let candidate = best[at] + (edge + STEP_COST) * length;
                if candidate < best[next] {
                    best[next] = candidate;
                    came_from[next] = at;
                    queue.push(Step {
                        cost: candidate,
                        at: next,
                    });
                }
            }
        }
        if !settled[goal] {
            return straight_line(from, to);
        }

        let mut path = Vec::new();
        let mut cursor = goal;
        loop {
            path.push((left + (cursor % rw) as i32, top + (cursor / rw) as i32));
            if cursor == start {
                break;
            }
            cursor = came_from[cursor];
            if cursor == usize::MAX {
                return straight_line(from, to);
            }
        }
        path.reverse();
        path
    }
}

/// A Dijkstra frontier entry, ordered reversed so the max-heap pops the
/// cheapest step first.
struct Step {
    cost: f32,
    at: usize,
}

impl PartialEq for Step {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Step {}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .total_cmp(&self.cost)
            .then_with(|| self.at.cmp(&other.at))
    }
}

/// Bresenham between two points, used whenever the wire has nothing to snap to.
fn straight_line(from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
    let (mut x, mut y) = from;
    let dx = (to.0 - x).abs();
    let dy = -(to.1 - y).abs();
    let sx = if x < to.0 { 1 } else { -1 };
    let sy = if y < to.1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut points = Vec::new();
    loop {
        points.push((x, y));
        if (x, y) == to {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 1-plane gray buffer filled with `value`.
    fn gray(width: u32, height: u32, value: u8) -> PixelBuffer {
        PixelBuffer {
            width,
            height,
            channels: 1,
            data: vec![value; (width * height) as usize],
        }
    }

    fn set(buffer: &mut PixelBuffer, x: u32, y: u32, value: u8) {
        buffer.data[(y * buffer.width + x) as usize] = value;
    }

    /// Black left of `split`, white from it on: one strong vertical edge.
    fn vertical_edge(width: u32, height: u32, split: u32) -> PixelBuffer {
        let mut buffer = gray(width, height, 0);
        for y in 0..height {
            for x in split..width {
                set(&mut buffer, x, y, 255);
            }
        }
        buffer
    }

    #[test]
    fn a_flat_image_traces_a_straight_line() {
        let map = EdgeMap::from_buffer(&gray(32, 32, 255), 10);
        let path = map.trace((4, 4), (4, 20), 10);

        assert_eq!(path.first(), Some(&(4, 4)));
        assert_eq!(path.last(), Some(&(4, 20)));
        assert!(
            path.iter().all(|&(x, _)| x == 4),
            "wandered on a flat image"
        );
    }

    #[test]
    fn the_wire_snaps_onto_a_nearby_edge() {
        let map = EdgeMap::from_buffer(&vertical_edge(64, 64, 32), 10);
        let path = map.trace((32, 8), (32, 56), 12);

        assert!(path.len() >= 40, "path too short: {}", path.len());
        assert!(
            path.iter().all(|&(x, _)| (x - 32).abs() <= 1),
            "left the edge"
        );
    }

    #[test]
    fn the_wire_bows_toward_an_edge_beside_the_line() {
        let map = EdgeMap::from_buffer(&vertical_edge(64, 64, 32), 10);
        let path = map.trace((26, 8), (26, 56), 16);

        assert!(
            path.iter().any(|&(x, _)| (x - 32).abs() <= 1),
            "ignored the edge"
        );
        assert_eq!(path.first(), Some(&(26, 8)));
        assert_eq!(path.last(), Some(&(26, 56)));
    }

    #[test]
    fn a_narrow_width_keeps_the_wire_near_the_line() {
        let map = EdgeMap::from_buffer(&vertical_edge(64, 64, 32), 10);
        let path = map.trace((26, 8), (26, 56), 2);

        assert!(
            path.iter().all(|&(x, _)| (x - 26).abs() <= 2),
            "escaped its corridor"
        );
    }

    #[test]
    fn a_high_contrast_setting_ignores_a_weak_edge() {
        // Two grays a few levels apart, plus a strong edge elsewhere setting the
        // peak, so the weak one normalises well under a 60 % threshold.
        let mut buffer = gray(64, 64, 120);
        for y in 0..64 {
            for x in 32..64 {
                set(&mut buffer, x, y, 128);
            }
            set(&mut buffer, 8, y, 0);
        }
        let map = EdgeMap::from_buffer(&buffer, 60);
        let path = map.trace((26, 8), (26, 56), 16);

        assert!(
            path.iter().all(|&(x, _)| (x - 26).abs() <= 1),
            "chased a weak edge"
        );
    }

    #[test]
    fn rgb_buffers_read_luma_from_three_planes() {
        let edge = vertical_edge(16, 16, 8);
        let mut data = edge.data.clone();
        data.extend_from_slice(&edge.data);
        data.extend_from_slice(&edge.data);
        let rgb = PixelBuffer {
            channels: 3,
            data,
            ..edge
        };
        let path = EdgeMap::from_buffer(&rgb, 10).trace((8, 2), (8, 13), 4);

        assert!(path.iter().all(|&(x, _)| (x - 8).abs() <= 1));
    }

    #[test]
    fn off_canvas_endpoints_fall_back_to_a_straight_line() {
        let map = EdgeMap::from_buffer(&vertical_edge(64, 64, 32), 10);
        let path = map.trace((10, 10), (200, 10), 10);

        assert_eq!(path.first(), Some(&(10, 10)));
        assert_eq!(path.last(), Some(&(200, 10)));
        assert_eq!(path.len(), 191);
    }

    #[test]
    fn the_path_is_connected_and_honours_its_endpoints() {
        let map = EdgeMap::from_buffer(&vertical_edge(64, 64, 32), 10);
        let path = map.trace((3, 3), (60, 60), 20);

        assert_eq!(path.first(), Some(&(3, 3)));
        assert_eq!(path.last(), Some(&(60, 60)));
        for pair in path.windows(2) {
            let (dx, dy) = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            assert!(dx.abs() <= 1 && dy.abs() <= 1, "jumped by ({dx}, {dy})");
        }
    }

    #[test]
    fn a_tiny_or_short_buffer_is_all_flat() {
        let map = EdgeMap::from_buffer(&gray(2, 2, 0), 10);
        assert_eq!((map.width(), map.height()), (2, 2));
        assert_eq!(map.trace((0, 0), (1, 1), 4), vec![(0, 0), (1, 1)]);

        let short = PixelBuffer {
            data: vec![0; 4],
            ..gray(8, 8, 0)
        };
        assert_eq!(
            EdgeMap::from_buffer(&short, 10).trace((1, 1), (1, 1), 4),
            vec![(1, 1)]
        );
    }
}
