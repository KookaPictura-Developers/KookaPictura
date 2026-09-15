use pictura_core::Document;
use pictura_select::Selection;

#[derive(Clone)]
pub struct Snapshot {
    pub doc: Document,
    pub selection: Option<Selection>,
}

struct Entry {
    snapshot: Snapshot,
    label: String,
}

// ponytail: full-document clones; COW or tile diffs if PSB-size docs hit RAM.
/// Bounded undo/redo over labeled `(Document, Selection)` states plus up to
/// [`MAX_SNAPSHOTS`] named restore points.
///
/// `states` is the linear history in capture order (oldest first) and `cursor`
/// indexes the current state. Capturing after an undo discards the states after
/// the cursor, and the oldest state is dropped once the stack would hold more
/// than one state per undoable step.
#[derive(Default)]
pub struct History {
    states: Vec<Entry>,
    cursor: usize,
    snapshots: Vec<Entry>,
}

const MAX_DEPTH: usize = 20;
const MAX_SNAPSHOTS: usize = 10;

impl History {
    pub fn capture(&mut self, snapshot: Snapshot, label: &str) {
        self.states.truncate(self.cursor + 1);
        self.states.push(Entry {
            snapshot,
            label: label.to_string(),
        });
        self.cursor = self.states.len() - 1;
        while self.states.len() > MAX_DEPTH + 1 {
            self.states.remove(0);
            self.cursor -= 1;
        }
    }

    pub fn undo(&mut self) -> Option<Snapshot> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        Some(self.states[self.cursor].snapshot.clone())
    }

    pub fn redo(&mut self) -> Option<Snapshot> {
        if self.cursor + 1 >= self.states.len() {
            return None;
        }
        self.cursor += 1;
        Some(self.states[self.cursor].snapshot.clone())
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
        let snapshot = self.states.get(i)?.snapshot.clone();
        self.cursor = i;
        Some(snapshot)
    }

    pub fn add_snapshot(&mut self, label: &str, snapshot: Snapshot) {
        self.snapshots.push(Entry {
            snapshot,
            label: label.to_string(),
        });
        while self.snapshots.len() > MAX_SNAPSHOTS {
            self.snapshots.remove(0);
        }
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Label of named snapshot `i`, or `""` when out of range.
    pub fn snapshot_label(&self, i: usize) -> &str {
        self.snapshots.get(i).map_or("", |e| e.label.as_str())
    }

    /// The state stored in named snapshot `i`, or `None` when out of range.
    pub fn snapshot(&self, i: usize) -> Option<Snapshot> {
        self.snapshots.get(i).map(|e| e.snapshot.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pictura_core::{BitDepth, ColorMode};

    fn doc(w: u32, h: u32, seed: u8) -> Document {
        let mut doc = Document::new(w, h, ColorMode::Rgb, BitDepth::Eight);
        doc.composite.data = vec![seed; (w * h * 3) as usize];
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
}
