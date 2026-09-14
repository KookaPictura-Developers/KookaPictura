use pictura_core::Document;
use pictura_select::Selection;

pub struct Snapshot {
    pub doc: Document,
    pub selection: Option<Selection>,
}

// ponytail: full-document clones; COW or tile diffs if PSB-size docs hit RAM.
#[derive(Default)]
pub struct History {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
}

const MAX_STATES: usize = 20;

impl History {
    pub fn capture(&mut self, snapshot: Snapshot) {
        self.redo_stack.clear();
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > MAX_STATES {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let previous = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(previous)
    }

    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn depth(&self) -> usize {
        self.undo_stack.len()
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

    #[test]
    fn capture_undo_round_trip_restores_doc_and_selection() {
        let mut history = History::default();
        let doc0 = doc(2, 1, 7);
        let selection = Selection {
            width: 2,
            height: 1,
            data: vec![255, 0],
        };
        history.capture(Snapshot {
            doc: doc0.clone(),
            selection: Some(selection.clone()),
        });

        let restored = history
            .undo(Snapshot {
                doc: doc(4, 2, 9),
                selection: Some(selection.clone()),
            })
            .expect("undo after capture");
        assert_eq!((restored.doc.width, restored.doc.height), (2, 1));
        assert_eq!(restored.doc.composite.data, doc0.composite.data);
        assert_eq!(restored.selection, Some(selection));
    }

    #[test]
    fn selection_none_round_trips() {
        let mut history = History::default();
        let doc0 = doc(2, 2, 3);
        history.capture(Snapshot {
            doc: doc0.clone(),
            selection: None,
        });

        let restored = history
            .undo(Snapshot {
                doc: doc(3, 3, 4),
                selection: Some(Selection {
                    width: 3,
                    height: 3,
                    data: vec![1; 9],
                }),
            })
            .expect("undo after capture");
        assert_eq!(restored.doc, doc0);
        assert!(restored.selection.is_none());
    }

    #[test]
    fn empty_stacks_refuse_undo_and_redo() {
        let mut history = History::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.depth(), 0);

        assert!(history
            .undo(Snapshot {
                doc: doc(1, 1, 5),
                selection: None,
            })
            .is_none());
        assert!(history
            .redo(Snapshot {
                doc: doc(1, 1, 5),
                selection: None,
            })
            .is_none());
        assert_eq!(history.depth(), 0);
        assert!(!history.can_redo());
    }

    #[test]
    fn redo_returns_the_stashed_post_state() {
        let mut history = History::default();
        history.capture(Snapshot {
            doc: doc(2, 2, 0),
            selection: None,
        });
        let post = || Snapshot {
            doc: doc(3, 3, 1),
            selection: None,
        };

        let restored = history.undo(post()).expect("undo state");
        assert_eq!(restored.doc.width, 2);
        assert_eq!(history.depth(), 0);
        assert!(history.can_redo());

        let forwarded = history.redo(post()).expect("redo state");
        assert_eq!(forwarded.doc.width, 3);
        assert_eq!(forwarded.doc.composite.data, vec![1; 27]);
        assert_eq!(history.depth(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn depth_bounded_at_20_and_drops_oldest() {
        let mut history = History::default();
        for i in 0..21u8 {
            history.capture(Snapshot {
                doc: doc(1, 1, i),
                selection: None,
            });
        }
        assert_eq!(history.depth(), 20);

        let dropped = history
            .undo(Snapshot {
                doc: doc(1, 1, 200),
                selection: None,
            })
            .expect("state 20");
        assert_eq!(dropped.doc.composite.data, vec![20, 20, 20]);

        let mut last = None;
        for _ in 0..19 {
            last = history.undo(Snapshot {
                doc: doc(1, 1, 200),
                selection: None,
            });
        }
        let oldest_kept = last.expect("state 1");
        assert_eq!(oldest_kept.doc.composite.data, vec![1, 1, 1]);
        assert_eq!(history.depth(), 0);
        assert!(history
            .undo(Snapshot {
                doc: doc(1, 1, 200),
                selection: None,
            })
            .is_none());
    }

    #[test]
    fn new_capture_truncates_redo() {
        let mut history = History::default();
        history.capture(Snapshot {
            doc: doc(1, 1, 0),
            selection: None,
        });
        assert!(history
            .undo(Snapshot {
                doc: doc(1, 1, 1),
                selection: None,
            })
            .is_some());
        assert!(history.can_redo());

        history.capture(Snapshot {
            doc: doc(1, 1, 2),
            selection: None,
        });
        assert!(!history.can_redo());
        assert_eq!(history.depth(), 1);
        assert!(history.can_undo());
    }
}
