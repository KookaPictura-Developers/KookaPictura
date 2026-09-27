//! Annotations: the marks the Eyedropper tool group leaves on a document to
//! *read* it rather than change it. None of them touch pixels.
//!
//! * **Color samplers** — persistent probes whose values the Info panel reads
//!   out; document state, capped at [`MAX_COLOR_SAMPLERS`].
//! * **Notes** — text pinned to a point; document state.
//! * **The ruler** — one measuring line. CS6 does not save it with the
//!   document, so it is view state and lives outside [`Annotations`].
//!
//! ponytail: samplers and notes are not yet written to or read from the PSD
//! (color samplers resource 1073, the `Anno` annotations block), which stay
//! preserved verbatim.
//!
//! Ported from photorust's `core/src/annotation.rs`
//! (<https://github.com/perfecto25/photorust>).

/// How many color samplers CS6 allows on one document
/// (`docs/03-tools/eyedropper-color-sampler-ruler.md`); later versions raised it.
pub const MAX_COLOR_SAMPLERS: usize = 4;

/// The two point-marker kinds, shared with the shell as plain integers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum MarkerKind {
    ColorSampler = 0,
    Note = 1,
    /// Count tool marks (Photoshop Extended); numbered in placement order.
    Count = 2,
}

impl MarkerKind {
    pub fn from_i32(v: i32) -> Option<MarkerKind> {
        match v {
            0 => Some(MarkerKind::ColorSampler),
            1 => Some(MarkerKind::Note),
            2 => Some(MarkerKind::Count),
            _ => None,
        }
    }
}

/// A point marker in document pixels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marker {
    pub x: i32,
    pub y: i32,
    /// Note text; empty for a color sampler.
    pub text: String,
}

/// The document's color samplers, notes, and count marks, each list in
/// placement order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Annotations {
    samplers: Vec<Marker>,
    notes: Vec<Marker>,
    counts: Vec<Marker>,
}

impl Annotations {
    fn list(&self, kind: MarkerKind) -> &Vec<Marker> {
        match kind {
            MarkerKind::ColorSampler => &self.samplers,
            MarkerKind::Note => &self.notes,
            MarkerKind::Count => &self.counts,
        }
    }

    fn list_mut(&mut self, kind: MarkerKind) -> &mut Vec<Marker> {
        match kind {
            MarkerKind::ColorSampler => &mut self.samplers,
            MarkerKind::Note => &mut self.notes,
            MarkerKind::Count => &mut self.counts,
        }
    }

    pub fn markers(&self, kind: MarkerKind) -> &[Marker] {
        self.list(kind)
    }

    pub fn marker(&self, kind: MarkerKind, index: usize) -> Option<&Marker> {
        self.list(kind).get(index)
    }

    /// Place a marker, returning its index; `None` (no change) once the
    /// samplers are full — CS6 refuses a fifth rather than evicting one.
    pub fn add(&mut self, kind: MarkerKind, x: i32, y: i32) -> Option<usize> {
        if kind == MarkerKind::ColorSampler && self.samplers.len() >= MAX_COLOR_SAMPLERS {
            return None;
        }
        let list = self.list_mut(kind);
        list.push(Marker {
            x,
            y,
            text: String::new(),
        });
        Some(list.len() - 1)
    }

    /// Move marker `index`; false for an out-of-range index.
    pub fn move_marker(&mut self, kind: MarkerKind, index: usize, x: i32, y: i32) -> bool {
        match self.list_mut(kind).get_mut(index) {
            Some(marker) => {
                marker.x = x;
                marker.y = y;
                true
            }
            None => false,
        }
    }

    /// Replace note `index`'s text; false for an out-of-range index.
    pub fn set_note_text(&mut self, index: usize, text: impl Into<String>) -> bool {
        match self.notes.get_mut(index) {
            Some(note) => {
                note.text = text.into();
                true
            }
            None => false,
        }
    }

    /// Delete marker `index`, shifting later ones down; false when out of range.
    pub fn remove(&mut self, kind: MarkerKind, index: usize) -> bool {
        let list = self.list_mut(kind);
        if index >= list.len() {
            return false;
        }
        list.remove(index);
        true
    }

    /// Delete every marker of `kind`; false when there were none.
    pub fn clear(&mut self, kind: MarkerKind) -> bool {
        let list = self.list_mut(kind);
        let had = !list.is_empty();
        list.clear();
        had
    }

    /// The marker of `kind` within `radius` of `(x, y)`, nearest first. The
    /// shell scales `radius` by zoom so the grab area keeps its screen size.
    pub fn marker_at(&self, kind: MarkerKind, x: f64, y: f64, radius: f64) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for (i, marker) in self.list(kind).iter().enumerate() {
            let d = (f64::from(marker.x) - x).hypot(f64::from(marker.y) - y);
            if d <= radius && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }
}

/// A Ruler measuring line from `a` to `b`, in document pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ruler {
    pub a: (f64, f64),
    pub b: (f64, f64),
}

/// The Ruler readout: CS6's X, Y, W, H, A, D1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Degrees in `-180..=180`, anticlockwise from east as seen on screen, so a
    /// line running down-right reads negative, as CS6 reports it.
    pub angle: f64,
    pub distance: f64,
}

impl Ruler {
    pub fn measure(&self) -> Measurement {
        let dx = self.b.0 - self.a.0;
        let dy = self.b.1 - self.a.1;
        Measurement {
            x: self.a.0,
            y: self.a.1,
            width: dx,
            height: dy,
            // Document y grows downward; the reported angle does not.
            angle: (-dy).atan2(dx).to_degrees(),
            distance: dx.hypot(dy),
        }
    }

    /// Shift-drag: snap `b` to the nearest 45° ray from `a`, keeping the length
    /// of `b`'s projection onto that ray.
    pub fn constrained(a: (f64, f64), b: (f64, f64)) -> Ruler {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let step = std::f64::consts::FRAC_PI_4;
        let angle = (dy.atan2(dx) / step).round() * step;
        let (s, c) = angle.sin_cos();
        let along = dx * c + dy * s;
        Ruler {
            a,
            b: (a.0 + along * c, a.1 + along * s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn samplers_and_notes_are_independent() {
        let mut a = Annotations::default();
        a.add(MarkerKind::ColorSampler, 1, 1);
        a.add(MarkerKind::Note, 2, 2);
        a.add(MarkerKind::Note, 3, 3);
        assert_eq!(a.markers(MarkerKind::ColorSampler).len(), 1);
        assert!(a.clear(MarkerKind::Note));
        assert!(
            !a.clear(MarkerKind::Note),
            "clearing nothing reported a change"
        );
        assert_eq!(a.markers(MarkerKind::ColorSampler).len(), 1);
    }

    #[test]
    fn color_samplers_stop_at_four() {
        let mut a = Annotations::default();
        for i in 0..MAX_COLOR_SAMPLERS {
            assert_eq!(a.add(MarkerKind::ColorSampler, i as i32, 0), Some(i));
        }
        assert_eq!(a.add(MarkerKind::ColorSampler, 9, 9), None);
        assert_eq!(a.markers(MarkerKind::ColorSampler).len(), 4);
        assert!(a.remove(MarkerKind::ColorSampler, 0));
        assert_eq!(a.add(MarkerKind::ColorSampler, 9, 9), Some(3));
        for i in 0..20 {
            assert!(
                a.add(MarkerKind::Note, i, 0).is_some(),
                "notes are unlimited"
            );
        }
    }

    #[test]
    fn hit_testing_picks_the_nearest_within_range() {
        let mut a = Annotations::default();
        a.add(MarkerKind::ColorSampler, 10, 10);
        a.add(MarkerKind::ColorSampler, 14, 10);
        a.add(MarkerKind::Note, 40, 40);
        assert_eq!(
            a.marker_at(MarkerKind::ColorSampler, 11.0, 10.0, 5.0),
            Some(0)
        );
        assert_eq!(
            a.marker_at(MarkerKind::ColorSampler, 13.0, 10.0, 5.0),
            Some(1)
        );
        assert_eq!(a.marker_at(MarkerKind::ColorSampler, 40.0, 40.0, 5.0), None);
        assert_eq!(a.marker_at(MarkerKind::Note, 41.0, 41.0, 5.0), Some(0));
    }

    #[test]
    fn edits_address_markers_by_index() {
        let mut a = Annotations::default();
        for i in 0..3 {
            a.add(MarkerKind::Note, i * 10, 0);
        }
        assert!(a.set_note_text(1, "check this edge"));
        assert!(!a.set_note_text(9, "nope"));
        assert!(a.move_marker(MarkerKind::Note, 1, 30, 40));
        assert!(!a.move_marker(MarkerKind::Note, 9, 0, 0));
        assert!(a.remove(MarkerKind::Note, 0));
        assert!(!a.remove(MarkerKind::Note, 9));
        let moved = a.marker(MarkerKind::Note, 0).unwrap();
        assert_eq!(
            (moved.x, moved.y, moved.text.as_str()),
            (30, 40, "check this edge")
        );
    }

    #[test]
    fn the_ruler_reads_cs6_angles_and_lengths() {
        let flat = Ruler {
            a: (10.0, 20.0),
            b: (60.0, 20.0),
        }
        .measure();
        assert!(close(flat.x, 10.0) && close(flat.y, 20.0) && close(flat.width, 50.0));
        assert!(close(flat.angle, 0.0) && close(flat.distance, 50.0));
        let down = Ruler {
            a: (0.0, 0.0),
            b: (3.0, 4.0),
        }
        .measure();
        assert!(close(down.distance, 5.0) && close(down.height, 4.0));
        assert!(
            (down.angle + 53.130_102).abs() < 1e-4,
            "angle was {}",
            down.angle
        );
        let up = Ruler {
            a: (0.0, 10.0),
            b: (10.0, 0.0),
        }
        .measure();
        assert!(close(up.angle, 45.0));
    }

    #[test]
    fn shift_snaps_the_ruler_to_45_degrees() {
        let r = Ruler::constrained((0.0, 0.0), (10.0, 1.0));
        assert!(close(r.b.1, 0.0) && close(r.b.0, 10.0));
        let r = Ruler::constrained((0.0, 0.0), (10.0, 9.0));
        assert!(close(r.b.0, r.b.1) && close(r.measure().angle, -45.0));
        let r = Ruler::constrained((5.0, 5.0), (5.5, -20.0));
        assert!(close(r.b.0, 5.0) && close(r.measure().angle, 90.0));
    }

    #[test]
    fn marker_kind_round_trips_through_its_integer() {
        for kind in [
            MarkerKind::ColorSampler,
            MarkerKind::Note,
            MarkerKind::Count,
        ] {
            assert_eq!(MarkerKind::from_i32(kind as i32), Some(kind));
        }
        assert_eq!(MarkerKind::from_i32(3), None);
    }

    #[test]
    fn count_marks_are_an_independent_numbered_list() {
        let mut a = Annotations::default();
        assert_eq!(a.add(MarkerKind::Count, 3, 3), Some(0));
        assert_eq!(a.add(MarkerKind::Count, 8, 4), Some(1));
        assert_eq!(a.add(MarkerKind::Note, 1, 1), Some(0));
        assert_eq!(a.markers(MarkerKind::Count).len(), 2);
        assert_eq!(a.markers(MarkerKind::Note).len(), 1);
        // Removing a mark shifts later numbers down.
        assert!(a.remove(MarkerKind::Count, 0));
        assert_eq!(a.marker(MarkerKind::Count, 0).unwrap().x, 8);
    }
}
