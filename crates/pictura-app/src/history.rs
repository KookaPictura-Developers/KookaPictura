use pictura_core::{Document, Layer, Plane};
use pictura_select::Selection;

#[derive(Clone)]
pub struct Snapshot {
    pub doc: Document,
    pub selection: Option<Selection>,
}

/// Tile edge for the region deltas: a changed plane keeps only the 64×64 tiles
/// whose bytes differ, so a small edit costs a small delta rather than a whole
/// 61 MiB plane.
const TILE: usize = 64;

enum Dir {
    Before,
    After,
}

/// One changed tile of one plane, laid out row-major with the plane's stride.
struct TileDelta {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    before: Vec<u8>,
    after: Vec<u8>,
}

/// The changed tiles of one tracked plane, anchored at the previous state.
struct PlaneDelta {
    index: usize,
    width: usize,
    tiles: Vec<TileDelta>,
}

/// A state's delta against the previous state within one geometry-stable
/// segment: the new metadata plus the changed tiles. `meta` is the new
/// document with every tracked plane cleared; overlaying the running planes
/// and then writing the after-tiles reconstructs the state exactly.
///
/// ponytail: tiles are uncompressed and keep both before- and after-bytes; drop
/// `before` (or RLE the tiles) only if a paint profile shows the payload hurts.
struct Delta {
    meta: Document,
    tiles: Vec<PlaneDelta>,
    selection: Option<Selection>,
}

/// A state is either a full anchor (the oldest state, or a geometry change that
/// breaks the delta chain) or a delta against the previous state.
enum Stored {
    Full(Snapshot),
    Delta(Delta),
}

struct Entry {
    label: String,
    stored: Stored,
}

struct Named {
    label: String,
    snapshot: Snapshot,
}

/// The History Brush's source: a state, a named snapshot, or a state the stack
/// has since dropped. The default is the oldest state, CS6's opening snapshot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrushSource {
    #[default]
    Oldest,
    State(usize),
    Snapshot(usize),
    /// Dropped by the depth limit or a new capture after undo; kept in
    /// `History::pinned` so the brush never reads a discarded state.
    Pinned,
}

// Clones are refcount bumps: the planes are copy-on-write, so a state costs
// one shared set of pixels plus the planes it has since forked.
/// Bounded undo/redo over labeled `(Document, Selection)` states plus up to
/// [`MAX_SNAPSHOTS`] named restore points.
///
/// Internally each state after the oldest is stored as the 64×64 tiles that
/// changed against the previous state, so retention scales with the edited
/// area, not with the document. A geometry or structure change stores that
/// state in full instead and starts a new anchor. `states` is the linear
/// history in capture order (oldest first) and `cursor` indexes the current
/// state; `current` is the materialized state at the cursor.
#[derive(Default)]
pub struct History {
    states: Vec<Entry>,
    cursor: usize,
    snapshots: Vec<Named>,
    current: Option<Snapshot>,
    brush_source: BrushSource,
    pinned: Option<Snapshot>,
}

const MAX_DEPTH: usize = 20;
const MAX_SNAPSHOTS: usize = 10;

impl History {
    /// Diff `snapshot` against the current state and store it as changed tiles,
    /// or in full when the tracked-plane geometry changed.
    pub fn capture(&mut self, snapshot: Snapshot, label: &str) {
        if matches!(self.brush_source, BrushSource::State(i) if i > self.cursor) {
            self.pin_source();
        }
        self.states.truncate(self.cursor + 1);
        let stored = match self.current.as_ref() {
            Some(prev) => match build_delta(prev, &snapshot) {
                Some(delta) => Stored::Delta(delta),
                None => Stored::Full(snapshot.clone()),
            },
            None => Stored::Full(snapshot.clone()),
        };
        self.states.push(Entry {
            label: label.to_string(),
            stored,
        });
        self.cursor = self.states.len() - 1;
        self.current = Some(snapshot);
        while self.states.len() > MAX_DEPTH + 1 {
            match self.brush_source {
                BrushSource::Oldest | BrushSource::State(0) => self.pin_source(),
                BrushSource::State(i) => self.brush_source = BrushSource::State(i - 1),
                _ => {}
            }
            self.drop_oldest();
        }
    }

    pub fn undo(&mut self) -> Option<Snapshot> {
        if self.cursor == 0 {
            return None;
        }
        Some(self.step_to(self.cursor - 1))
    }

    pub fn redo(&mut self) -> Option<Snapshot> {
        if self.cursor + 1 >= self.states.len() {
            return None;
        }
        Some(self.step_to(self.cursor + 1))
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.states.len()
    }

    /// Number of undoable steps (`0` at the oldest state).
    pub fn depth(&self) -> usize {
        self.cursor
    }

    /// Number of labeled states, including the current one.
    pub fn count(&self) -> usize {
        self.states.len()
    }

    /// Position of the current state, in `0..count()`.
    pub fn index(&self) -> usize {
        self.cursor
    }

    /// Label of state `i`, or `""` when out of range.
    pub fn label(&self, i: usize) -> &str {
        self.states.get(i).map_or("", |e| e.label.as_str())
    }

    /// Restore state `i` and move the cursor to it, or `None` when out of range.
    pub fn jump(&mut self, i: usize) -> Option<Snapshot> {
        if i >= self.states.len() {
            return None;
        }
        Some(self.step_to(i))
    }

    pub fn add_snapshot(&mut self, label: &str, snapshot: Snapshot) {
        self.snapshots.push(Named {
            label: label.to_string(),
            snapshot,
        });
        while self.snapshots.len() > MAX_SNAPSHOTS {
            match self.brush_source {
                BrushSource::Snapshot(0) => self.pin_source(),
                BrushSource::Snapshot(i) => self.brush_source = BrushSource::Snapshot(i - 1),
                _ => {}
            }
            self.snapshots.remove(0);
        }
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Label of named snapshot `i`, or `""` when out of range.
    pub fn snapshot_label(&self, i: usize) -> &str {
        self.snapshots.get(i).map_or("", |s| s.label.as_str())
    }

    /// The state stored in named snapshot `i`, or `None` when out of range.
    pub fn snapshot(&self, i: usize) -> Option<Snapshot> {
        self.snapshots.get(i).map(|s| s.snapshot.clone())
    }

    /// Move the materialized current state to `target`, applying or reverting
    /// deltas. Crossing a full anchor backward rematerializes from that anchor.
    fn step_to(&mut self, target: usize) -> Snapshot {
        while self.cursor < target {
            self.step_forward();
        }
        while self.cursor > target {
            if matches!(self.states[self.cursor].stored, Stored::Full(_)) {
                self.current = Some(self.materialize(target));
                self.cursor = target;
            } else {
                self.step_backward();
            }
        }
        self.current
            .clone()
            .expect("current state is always materialized")
    }

    fn step_forward(&mut self) {
        let next = self.cursor + 1;
        match &self.states[next].stored {
            Stored::Full(snapshot) => self.current = Some(snapshot.clone()),
            Stored::Delta(delta) => {
                let mut cur = self.current.take().expect("current state");
                adopt_metadata(&mut cur.doc, &delta.meta);
                apply_tiles(&mut cur.doc, &delta.tiles, Dir::After);
                cur.selection = delta.selection.clone();
                self.current = Some(cur);
            }
        }
        self.cursor = next;
    }

    fn step_backward(&mut self) {
        let here = self.cursor;
        let mut cur = self.current.take().expect("current state");
        match &self.states[here].stored {
            Stored::Full(_) => unreachable!("a full anchor is handled by step_to"),
            Stored::Delta(delta) => {
                apply_tiles(&mut cur.doc, &delta.tiles, Dir::Before);
                let prev = here - 1;
                match &self.states[prev].stored {
                    Stored::Full(snapshot) => cur = snapshot.clone(),
                    Stored::Delta(prev_delta) => {
                        adopt_metadata(&mut cur.doc, &prev_delta.meta);
                        cur.selection = prev_delta.selection.clone();
                    }
                }
            }
        }
        self.cursor = here - 1;
        self.current = Some(cur);
    }

    /// Rebuild state `target` from the nearest full anchor at or before it.
    fn materialize(&self, target: usize) -> Snapshot {
        let anchor = (0..=target)
            .rev()
            .find(|&i| matches!(self.states[i].stored, Stored::Full(_)))
            .expect("state 0 is always a full anchor");
        let Stored::Full(base) = &self.states[anchor].stored else {
            unreachable!()
        };
        if anchor == target {
            return base.clone();
        }
        let mut doc = base.doc.clone();
        let mut selection = base.selection.clone();
        for k in anchor + 1..=target {
            match &self.states[k].stored {
                Stored::Full(snapshot) => {
                    doc = snapshot.doc.clone();
                    selection = snapshot.selection.clone();
                }
                Stored::Delta(delta) => {
                    adopt_metadata(&mut doc, &delta.meta);
                    apply_tiles(&mut doc, &delta.tiles, Dir::After);
                    selection = delta.selection.clone();
                }
            }
        }
        Snapshot { doc, selection }
    }

    /// Promote state 1 to a full anchor, then drop state 0.
    fn drop_oldest(&mut self) {
        let state1 = self.materialize(1);
        let label = std::mem::take(&mut self.states[1].label);
        self.states[1] = Entry {
            label,
            stored: Stored::Full(state1),
        };
        self.states.remove(0);
        self.cursor -= 1;
    }

    #[cfg(test)]
    fn retained_tile_bytes(&self) -> usize {
        self.states
            .iter()
            .filter_map(|e| match &e.stored {
                Stored::Delta(d) => Some(d),
                Stored::Full(_) => None,
            })
            .flat_map(|d| d.tiles.iter())
            .flat_map(|p| p.tiles.iter())
            .map(|t| t.before.len() + t.after.len())
            .sum()
    }

    #[cfg(test)]
    fn full_state_count(&self) -> usize {
        self.states
            .iter()
            .filter(|e| matches!(e.stored, Stored::Full(_)))
            .count()
    }
}

/// Build a delta from `prev` to `next`, or `None` when the tracked-plane
/// geometry (count, stride, or height) differs and a full snapshot is needed.
///
/// ponytail: a geometry or structure change retains that one state in full and
/// restarts the tile chain there, so deltas never span incompatible planes.
fn build_delta(prev: &Snapshot, next: &Snapshot) -> Option<Delta> {
    let mut prev_planes: Vec<(Plane<u8>, usize, usize)> = Vec::new();
    each_plane(&prev.doc, &mut |p, w, h| {
        prev_planes.push((p.clone(), w, h))
    });
    let mut next_planes: Vec<(Plane<u8>, usize, usize)> = Vec::new();
    each_plane(&next.doc, &mut |p, w, h| {
        next_planes.push((p.clone(), w, h))
    });
    if prev_planes.len() != next_planes.len() {
        return None;
    }
    if prev_planes
        .iter()
        .zip(&next_planes)
        .any(|(a, b)| a.0.len() != b.0.len() || a.1 != b.1 || a.2 != b.2)
    {
        return None;
    }

    let mut tiles = Vec::new();
    for (index, (before, width, _)) in prev_planes.iter().enumerate() {
        let after = &next_planes[index];
        if shares_plane(before, &after.0) || before.as_slice() == after.0.as_slice() {
            continue;
        }
        let changed = diff_tiles(before.as_slice(), after.0.as_slice(), *width);
        if !changed.is_empty() {
            tiles.push(PlaneDelta {
                index,
                width: *width,
                tiles: changed,
            });
        }
    }

    let mut meta = next.doc.clone();
    each_plane_mut(&mut meta, &mut |p, _, _| *p = Plane::default());
    Some(Delta {
        meta,
        tiles,
        selection: next.selection.clone(),
    })
}

fn shares_plane(a: &Plane<u8>, b: &Plane<u8>) -> bool {
    a.len() == b.len() && std::ptr::eq(a.as_slice().as_ptr(), b.as_slice().as_ptr())
}

fn diff_tiles(before: &[u8], after: &[u8], width: usize) -> Vec<TileDelta> {
    if width == 0 {
        return Vec::new();
    }
    let height = before.len() / width;
    let mut out = Vec::new();
    let mut y0 = 0;
    while y0 < height {
        let h = TILE.min(height - y0);
        let mut x0 = 0;
        while x0 < width {
            let w = TILE.min(width - x0);
            let changed = (y0..y0 + h).any(|row| {
                let start = row * width + x0;
                before[start..start + w] != after[start..start + w]
            });
            if changed {
                let mut b = Vec::with_capacity(w * h);
                let mut a = Vec::with_capacity(w * h);
                for row in y0..y0 + h {
                    let start = row * width + x0;
                    b.extend_from_slice(&before[start..start + w]);
                    a.extend_from_slice(&after[start..start + w]);
                }
                out.push(TileDelta {
                    x: x0,
                    y: y0,
                    w,
                    h,
                    before: b,
                    after: a,
                });
            }
            x0 += TILE;
        }
        y0 += TILE;
    }
    out
}

fn apply_tiles(doc: &mut Document, deltas: &[PlaneDelta], dir: Dir) {
    for delta in deltas {
        let mut index = 0;
        each_plane_mut(doc, &mut |plane, _, _| {
            if index == delta.index {
                for tile in &delta.tiles {
                    let bytes = match dir {
                        Dir::Before => &tile.before,
                        Dir::After => &tile.after,
                    };
                    write_tile(plane, delta.width, tile, bytes);
                }
            }
            index += 1;
        });
    }
}

fn write_tile(plane: &mut Plane<u8>, width: usize, tile: &TileDelta, bytes: &[u8]) {
    if width == 0 {
        return;
    }
    for row in 0..tile.h {
        let dst = (tile.y + row) * width + tile.x;
        let src = row * tile.w;
        if dst + tile.w > plane.len() {
            return;
        }
        plane[dst..dst + tile.w].copy_from_slice(&bytes[src..src + tile.w]);
    }
}

/// Replace `doc`'s metadata with `meta`'s, keeping `doc`'s tracked-plane bytes.
fn adopt_metadata(doc: &mut Document, meta: &Document) {
    let mut planes: Vec<Plane<u8>> = Vec::new();
    each_plane(doc, &mut |p, _, _| planes.push(p.clone()));
    *doc = meta.clone();
    let mut index = 0;
    each_plane_mut(doc, &mut |p, _, _| {
        if let Some(src) = planes.get(index) {
            *p = src.clone();
        }
        index += 1;
    });
}

fn plane_height(len: usize, width: usize) -> usize {
    len.checked_div(width).unwrap_or(0)
}

/// Visit every tracked plane (composite, document channels, layer channels,
/// layer masks, depth-first) with its row stride and height. The traversal is
/// fixed by the document structure, not the plane lengths, so it enumerates the
/// same indices before and after a plane is cleared or written.
fn each_plane(doc: &Document, f: &mut impl FnMut(&Plane<u8>, usize, usize)) {
    let width = doc.width as usize;
    let h = plane_height(doc.composite.data.len(), width);
    f(&doc.composite.data, width, h);
    for channel in &doc.channels {
        let h = plane_height(channel.data.len(), width);
        f(&channel.data, width, h);
    }
    for layer in &doc.layers {
        each_layer_plane(layer, f);
    }
}

fn each_layer_plane(layer: &Layer, f: &mut impl FnMut(&Plane<u8>, usize, usize)) {
    let layer_width = layer.rect.width().max(0) as usize;
    for channel in &layer.channels {
        let width = if layer_width > 0 {
            layer_width
        } else {
            channel.data.len()
        };
        let h = plane_height(channel.data.len(), width);
        f(&channel.data, width, h);
    }
    if let Some(mask) = &layer.mask {
        if let Some(data) = &mask.data {
            let mask_width = mask.rect.width().max(0) as usize;
            let width = if mask_width > 0 {
                mask_width
            } else {
                data.len()
            };
            let h = plane_height(data.len(), width);
            f(data, width, h);
        }
    }
    for child in &layer.children {
        each_layer_plane(child, &mut *f);
    }
}

fn each_plane_mut(doc: &mut Document, f: &mut impl FnMut(&mut Plane<u8>, usize, usize)) {
    let width = doc.width as usize;
    let h = plane_height(doc.composite.data.len(), width);
    f(&mut doc.composite.data, width, h);
    for channel in doc.channels.iter_mut() {
        let h = plane_height(channel.data.len(), width);
        f(&mut channel.data, width, h);
    }
    for layer in doc.layers.iter_mut() {
        each_layer_plane_mut(layer, f);
    }
}

fn each_layer_plane_mut(layer: &mut Layer, f: &mut impl FnMut(&mut Plane<u8>, usize, usize)) {
    let layer_width = layer.rect.width().max(0) as usize;
    for channel in layer.channels.iter_mut() {
        let width = if layer_width > 0 {
            layer_width
        } else {
            channel.data.len()
        };
        let h = plane_height(channel.data.len(), width);
        f(&mut channel.data, width, h);
    }
    if let Some(mask) = layer.mask.as_mut() {
        if let Some(data) = mask.data.as_mut() {
            let mask_width = mask.rect.width().max(0) as usize;
            let width = if mask_width > 0 {
                mask_width
            } else {
                data.len()
            };
            let h = plane_height(data.len(), width);
            f(data, width, h);
        }
    }
    for child in layer.children.iter_mut() {
        each_layer_plane_mut(child, &mut *f);
    }
}

impl History {
    pub fn brush_source(&self) -> BrushSource {
        self.brush_source
    }

    /// Point the History Brush at state or snapshot `source`; false (and no
    /// change) when it does not exist.
    pub fn set_brush_source(&mut self, source: BrushSource) -> bool {
        let exists = match source {
            BrushSource::Oldest => true,
            BrushSource::State(i) => i < self.states.len(),
            BrushSource::Snapshot(i) => i < self.snapshots.len(),
            BrushSource::Pinned => false,
        };
        if exists {
            self.brush_source = source;
            self.pinned = None;
        }
        exists
    }

    /// The snapshot the History Brush paints from.
    pub fn brush_source_doc(&self) -> Option<Snapshot> {
        match self.brush_source {
            BrushSource::Oldest => Some(self.materialize(0)),
            BrushSource::State(i) => (i < self.states.len()).then(|| self.materialize(i)),
            BrushSource::Snapshot(i) => self.snapshots.get(i).map(|s| s.snapshot.clone()),
            BrushSource::Pinned => self.pinned.clone(),
        }
    }

    fn pin_source(&mut self) {
        if matches!(self.brush_source, BrushSource::Pinned) {
            return;
        }
        let snapshot = match self.brush_source {
            BrushSource::Oldest => Some(self.materialize(0)),
            BrushSource::State(i) => (i < self.states.len()).then(|| self.materialize(i)),
            BrushSource::Snapshot(i) => self.snapshots.get(i).map(|s| s.snapshot.clone()),
            BrushSource::Pinned => None,
        };
        self.pinned = snapshot;
        self.brush_source = BrushSource::Pinned;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, Channel, ColorMode};

    fn doc(w: u32, h: u32, seed: u8) -> Document {
        let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        doc.composite.data = vec![seed; (w * h * 3) as usize].into();
        doc
    }

    fn snap(seed: u8) -> Snapshot {
        Snapshot {
            doc: doc(1, 1, seed),
            selection: None,
        }
    }

    #[test]
    fn capture_undo_round_trip_restores_doc_and_selection() {
        let mut history = History::default();
        let open = Snapshot {
            doc: doc(2, 1, 7),
            selection: None,
        };
        let selection = Selection {
            width: 2,
            height: 1,
            data: vec![255, 0],
        };
        let selected = Snapshot {
            doc: doc(2, 1, 7),
            selection: Some(selection.clone()),
        };
        history.capture(open, "Open");
        history.capture(selected, "Select All");

        let restored = history.undo().expect("undo after capture");
        assert_eq!((restored.doc.width, restored.doc.height), (2, 1));
        assert!(restored.selection.is_none());

        let forwarded = history.redo().expect("redo after undo");
        assert_eq!(forwarded.selection, Some(selection));
        assert_eq!(forwarded.doc.composite.data, doc(2, 1, 7).composite.data);
    }

    #[test]
    fn selection_none_round_trips() {
        let mut history = History::default();
        let doc0 = doc(2, 2, 3);
        history.capture(
            Snapshot {
                doc: doc0.clone(),
                selection: Some(Selection {
                    width: 2,
                    height: 2,
                    data: vec![255; 4],
                }),
            },
            "Select All",
        );
        history.capture(
            Snapshot {
                doc: doc(3, 3, 4),
                selection: None,
            },
            "Deselect",
        );

        let cleared = history.undo().expect("undo to selection");
        assert_eq!(cleared.selection.unwrap().data, vec![255; 4]);

        let restored = history.redo().expect("redo to none");
        assert_eq!(restored.doc, doc(3, 3, 4));
        assert!(restored.selection.is_none());
    }

    #[test]
    fn empty_stacks_refuse_undo_and_redo() {
        let mut history = History::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.depth(), 0);
        assert_eq!(history.count(), 0);

        assert!(history.undo().is_none());
        assert!(history.redo().is_none());
        assert!(history.jump(0).is_none());
        assert_eq!(history.depth(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn redo_returns_the_stashed_post_state() {
        let mut history = History::default();
        history.capture(snap(0), "open");
        history.capture(snap(1), "op");

        let restored = history.undo().expect("undo state");
        assert_eq!(restored.doc.composite.data, vec![0, 0, 0]);
        assert_eq!(history.depth(), 0);
        assert!(history.can_redo());

        let forwarded = history.redo().expect("redo state");
        assert_eq!(forwarded.doc.composite.data, vec![1, 1, 1]);
        assert_eq!(history.depth(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn depth_bounded_at_20_and_drops_oldest() {
        let mut history = History::default();
        history.capture(snap(99), "initial");
        for i in 0..21u8 {
            history.capture(snap(i), "op");
        }
        // One state per undoable step plus the current state, capped at depth 20.
        assert_eq!(history.depth(), 20);
        assert_eq!(history.count(), 21);

        let mut last = None;
        for _ in 0..20 {
            last = history.undo();
        }
        let oldest_kept = last.expect("state 20");
        assert_eq!(oldest_kept.doc.composite.data, vec![0, 0, 0]);
        assert_eq!(history.depth(), 0);
        assert!(history.undo().is_none());
    }

    #[test]
    fn new_capture_truncates_redo() {
        let mut history = History::default();
        history.capture(snap(0), "open");
        history.capture(snap(1), "op");
        assert!(history.undo().is_some());
        assert!(history.can_redo());

        history.capture(snap(2), "op2");
        assert!(!history.can_redo());
        assert_eq!(history.depth(), 1);
        assert!(history.can_undo());
    }

    #[test]
    fn labels_track_captures_and_undo_redo_position() {
        let mut history = History::default();
        history.capture(snap(0), "Open");
        history.capture(snap(1), "Select All");
        history.capture(snap(2), "Filter");

        assert_eq!(history.count(), 3);
        assert_eq!(history.index(), 2);
        assert_eq!(history.label(0), "Open");
        assert_eq!(history.label(1), "Select All");
        assert_eq!(history.label(2), "Filter");
        assert_eq!(history.label(9), "");

        history.undo();
        assert_eq!(history.index(), 1);
        history.undo();
        assert_eq!(history.index(), 0);
        history.redo();
        assert_eq!(history.index(), 1);
    }

    #[test]
    fn jump_restores_state_and_moves_position() {
        let mut history = History::default();
        history.capture(snap(0), "Open");
        history.capture(snap(1), "A");
        history.capture(snap(2), "B");

        let jumped = history.jump(0).expect("state 0");
        assert_eq!(jumped.doc.composite.data, vec![0, 0, 0]);
        assert_eq!(history.index(), 0);
        assert!(history.can_redo());
        assert!(!history.can_undo());

        let forward = history.jump(2).expect("state 2");
        assert_eq!(forward.doc.composite.data, vec![2, 2, 2]);
        assert_eq!(history.index(), 2);

        assert!(history.jump(9).is_none());
        assert_eq!(history.index(), 2);
    }

    #[test]
    fn snapshots_are_capped_and_restorable() {
        let mut history = History::default();
        for i in 0..12u8 {
            history.add_snapshot(&format!("snap {i}"), snap(i));
        }
        assert_eq!(history.snapshot_count(), 10);
        // The two oldest snapshots were dropped.
        assert_eq!(history.snapshot_label(0), "snap 2");
        assert_eq!(history.snapshot_label(9), "snap 11");
        assert_eq!(history.snapshot_label(10), "");

        let restored = history.snapshot(3).expect("snapshot 3");
        assert_eq!(restored.doc.composite.data, vec![5, 5, 5]);
        assert!(history.snapshot(99).is_none());
    }

    #[test]
    fn the_brush_source_follows_its_state_and_is_pinned_before_it_is_dropped() {
        let mut history = History::default();
        history.capture(snap(0), "Open");
        assert_eq!(history.brush_source(), BrushSource::Oldest);
        assert_eq!(
            history.brush_source_doc().unwrap().doc.composite.data,
            vec![0, 0, 0]
        );
        for i in 1..=3u8 {
            history.capture(snap(i), "Brush");
        }
        assert!(history.set_brush_source(BrushSource::State(2)));
        assert!(!history.set_brush_source(BrushSource::State(9)));
        assert_eq!(history.brush_source(), BrushSource::State(2));
        // Depth pruning shifts the index down with its state.
        for i in 4..=22u8 {
            history.capture(snap(i), "Brush");
        }
        assert_eq!(history.brush_source(), BrushSource::State(0));
        assert_eq!(
            history.brush_source_doc().unwrap().doc.composite.data,
            vec![2, 2, 2]
        );
        // Dropping it pins a copy instead of reading another state.
        history.capture(snap(23), "Brush");
        assert_eq!(history.brush_source(), BrushSource::Pinned);
        assert_eq!(
            history.brush_source_doc().unwrap().doc.composite.data,
            vec![2, 2, 2]
        );

        // A redo state discarded by a new capture is pinned the same way.
        assert!(history.set_brush_source(BrushSource::State(20)));
        history.undo();
        history.undo();
        history.capture(snap(99), "Brush");
        assert_eq!(history.brush_source(), BrushSource::Pinned);
        assert_eq!(
            history.brush_source_doc().unwrap().doc.composite.data,
            vec![23, 23, 23]
        );

        for i in 0..11u8 {
            history.add_snapshot("s", snap(100 + i));
            if i == 0 {
                assert!(history.set_brush_source(BrushSource::Snapshot(0)));
            }
        }
        assert_eq!(history.brush_source(), BrushSource::Pinned);
        assert_eq!(
            history.brush_source_doc().unwrap().doc.composite.data,
            vec![100, 100, 100]
        );
    }

    #[test]
    fn region_deltas_retain_only_changed_tiles() {
        let mut history = History::default();
        let mut doc = doc(256, 256, 0);
        let full_state_bytes = doc.composite.data.len();
        history.capture(
            Snapshot {
                doc: doc.clone(),
                selection: None,
            },
            "Open",
        );

        // Ten edits, each touching only the first 64×64 tile.
        for i in 1..=10u8 {
            doc.composite.data[0] = i;
            history.capture(
                Snapshot {
                    doc: doc.clone(),
                    selection: None,
                },
                "Paint",
            );
        }

        assert_eq!(
            history.full_state_count(),
            1,
            "only the base is a full state"
        );
        let retained = history.retained_tile_bytes();
        assert!(
            retained <= 10 * TILE * TILE * 2,
            "retained {retained} bytes exceeds the changed-tile bound"
        );
        assert!(
            retained < full_state_bytes,
            "retained {retained} bytes should be far below a full state ({full_state_bytes})"
        );

        // Every state still materializes its exact edited byte.
        for i in 0..=10u8 {
            let restored = history.jump(i as usize).expect("state");
            assert_eq!(restored.doc.composite.data[0], i);
        }
    }

    #[test]
    fn every_state_matches_a_reference_copy() {
        let mut doc = doc(200, 130, 5);
        doc.layers = vec![Layer {
            name: "paint".into(),
            rect: pictura_core::PsdRect {
                top: 0,
                left: 0,
                bottom: 130,
                right: 200,
            },
            channels: vec![Channel {
                id: 0,
                data: vec![9u8; 200 * 130].into(),
            }],
            ..Default::default()
        }];

        let mut history = History::default();
        let mut reference: Vec<Snapshot> = Vec::new();
        history.capture(
            Snapshot {
                doc: doc.clone(),
                selection: None,
            },
            "Open",
        );
        reference.push(Snapshot {
            doc: doc.clone(),
            selection: None,
        });

        // A mix of pixel edits, a metadata-only change, and the region path.
        let mut seed = 7u8;
        for step in 0..8usize {
            seed = seed.wrapping_mul(31).wrapping_add(step as u8);
            for k in 0..500usize {
                let idx = (seed as usize * 137 + k * 61) % doc.composite.data.len();
                doc.composite.data[idx] ^= (seed ^ k as u8) | 1;
            }
            let layer = &mut doc.layers[0];
            for k in 0..300usize {
                let idx = (seed as usize * 53 + k * 97) % layer.channels[0].data.len();
                layer.channels[0].data[idx] = seed.wrapping_add(k as u8);
            }
            if step == 3 {
                doc.layers[0].visible = false;
            }
            let selection = Some(Selection {
                width: 200,
                height: 130,
                data: vec![seed; 200 * 130],
            });
            doc.composite.data[0] = seed;
            let snapshot = Snapshot {
                doc: doc.clone(),
                selection,
            };
            history.capture(snapshot.clone(), "Edit");
            reference.push(snapshot);
        }

        for (i, expected) in reference.iter().enumerate() {
            let jumped = history.jump(i).expect("state");
            assert_eq!(&jumped.doc, &expected.doc, "doc differs at state {i}");
            assert_eq!(jumped.selection, expected.selection, "selection at {i}");
        }

        // Walk down and up through undo/redo, still byte-identical.
        while let Some(snapshot) = history.undo() {
            let i = history.index();
            assert_eq!(&snapshot.doc, &reference[i].doc);
        }
        while let Some(snapshot) = history.redo() {
            let i = history.index();
            assert_eq!(&snapshot.doc, &reference[i].doc);
        }
    }

    #[test]
    fn geometry_change_uses_a_full_anchor_and_still_undoes() {
        let mut history = History::default();
        let open = doc(32, 32, 4);
        history.capture(
            Snapshot {
                doc: open.clone(),
                selection: None,
            },
            "Open",
        );

        let resized = doc(48, 48, 9);
        history.capture(
            Snapshot {
                doc: resized.clone(),
                selection: None,
            },
            "Resize",
        );

        let restored = history.undo().expect("undo across geometry");
        assert_eq!(restored.doc, open);
        let forwarded = history.redo().expect("redo across geometry");
        assert_eq!(forwarded.doc, resized);
    }

    #[test]
    fn paint_commit_undo_restores_the_prior_pixels() {
        let mut doc = Document::new(64, 64, ColorMode::Rgb, BitDepth::Eight);
        doc.layers = vec![Layer {
            name: "base".into(),
            rect: pictura_core::PsdRect {
                top: 0,
                left: 0,
                bottom: 64,
                right: 64,
            },
            channels: vec![
                Channel {
                    id: 0,
                    data: vec![10u8; 64 * 64].into(),
                },
                Channel {
                    id: -1,
                    data: vec![255u8; 64 * 64].into(),
                },
            ],
            ..Default::default()
        }];
        let prior_layer = doc.layers[0].channels[0].data.clone();
        let prior_composite = doc.composite.data.clone();

        let mut history = History::default();
        history.capture(
            Snapshot {
                doc: doc.clone(),
                selection: None,
            },
            "Open",
        );

        // A paint commit: dab bytes into the layer plane and recomposite.
        let mut painted = doc.clone();
        for p in painted.layers[0].channels[0].data[..64 * 32].iter_mut() {
            *p = 200;
        }
        for p in painted.composite.data[..64 * 32].iter_mut() {
            *p = 200;
        }
        history.capture(
            Snapshot {
                doc: painted,
                selection: None,
            },
            "Brush",
        );

        let undone = history.undo().expect("undo the paint");
        assert_eq!(undone.doc.layers[0].channels[0].data, prior_layer);
        assert_eq!(undone.doc.composite.data, prior_composite);

        let redone = history.redo().expect("redo the paint");
        assert_eq!(redone.doc.layers[0].channels[0].data[0], 200);
        assert_eq!(redone.doc.layers[0].channels[0].data[32 * 64], 10);
        assert_eq!(redone.doc.composite.data[0], 200);
    }
}
