mod purge;

use super::qobject;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

impl qobject::PictureView {
    pub fn undo(mut self: Pin<&mut Self>) -> bool {
        let restored = self.as_mut().rust_mut().history.undo();
        let Some(snapshot) = restored else {
            return false;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            rust.doc = Some(snapshot.doc);
            rust.selection = snapshot.selection;
            // Undo during a stroke cancels it rather than committing it.
            rust.stroke = None;
            rust.reset_pyramid();
            rust.content_revision = rust.content_revision.wrapping_add(1);
        }
        self.changed();
        true
    }

    pub fn redo(mut self: Pin<&mut Self>) -> bool {
        let restored = self.as_mut().rust_mut().history.redo();
        let Some(snapshot) = restored else {
            return false;
        };
        {
            let mut rust = self.as_mut().rust_mut();
            rust.doc = Some(snapshot.doc);
            rust.selection = snapshot.selection;
            rust.stroke = None;
            rust.reset_pyramid();
            rust.content_revision = rust.content_revision.wrapping_add(1);
        }
        self.changed();
        true
    }

    pub fn can_undo(&self) -> bool {
        self.rust().history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.rust().history.can_redo()
    }

    pub fn history_depth(&self) -> i32 {
        self.rust().history.depth() as i32
    }

    pub fn history_count(&self) -> i32 {
        self.rust().history.count() as i32
    }

    pub fn history_index(&self) -> i32 {
        self.rust().history.index() as i32
    }

    pub fn history_label(&self, i: i32) -> QString {
        if i < 0 {
            return QString::default();
        }
        QString::from(self.rust().history.label(i as usize))
    }

    pub fn history_jump(mut self: Pin<&mut Self>, i: i32) -> bool {
        if i < 0 {
            return false;
        }
        let restored = self.as_mut().rust_mut().history.jump(i as usize);
        let Some(snapshot) = restored else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        rust.content_revision = rust.content_revision.wrapping_add(1);
        rust.stroke = None;
        self.as_mut().recomposite();
        true
    }

    pub fn history_add_snapshot(mut self: Pin<&mut Self>, label: &QString) -> bool {
        let Some(snapshot) = self.snapshot() else {
            return false;
        };
        self.as_mut()
            .rust_mut()
            .history
            .add_snapshot(&label.to_string(), snapshot);
        self.changed();
        true
    }

    pub fn history_snapshot_count(&self) -> i32 {
        self.rust().history.snapshot_count() as i32
    }

    pub fn history_snapshot_label(&self, i: i32) -> QString {
        if i < 0 {
            return QString::default();
        }
        QString::from(self.rust().history.snapshot_label(i as usize))
    }

    pub fn history_restore_snapshot(mut self: Pin<&mut Self>, i: i32) -> bool {
        if i < 0 {
            return false;
        }
        let Some(snapshot) = self.rust().history.snapshot(i as usize) else {
            return false;
        };
        let mut rust = self.as_mut().rust_mut();
        rust.doc = Some(snapshot.doc);
        rust.selection = snapshot.selection;
        rust.content_revision = rust.content_revision.wrapping_add(1);
        rust.stroke = None;
        self.as_mut().recomposite();
        true
    }
}
