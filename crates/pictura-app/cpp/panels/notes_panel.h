#pragma once

#include <QtWidgets/QWidget>

class QLabel;
class QPlainTextEdit;
class QToolButton;

namespace pictura {

class PictureView;

// Window > Notes: the text of the current note, with previous/next and delete.
// The Note tool picks the current note; the panel only edits it. Typed text
// is committed as one "Edit Note" state when the editor loses focus or the
// panel moves to another note.
class NotesPanel : public QWidget {
    Q_OBJECT

public:
    explicit NotesPanel(QWidget* parent = nullptr);

    void setView(PictureView* view);
    // Show note `index` (-1: none), committing any edit to the previous one.
    void showNote(int index);
    void refresh();
    // Commit the editor's text to the current note.
    void commitText();

    int currentNoteForTest() const { return current_; }
    QPlainTextEdit* editorForTest() const { return text_; }

signals:
    // Previous/next/delete ask for another current note; the frame routes this
    // to the tool controller, which owns the current note.
    void noteRequested(int index);

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    int noteCount() const;

    PictureView* view_ = nullptr;
    int current_ = -1;
    QLabel* counter_ = nullptr;
    QPlainTextEdit* text_ = nullptr;
    QToolButton* prev_ = nullptr;
    QToolButton* next_ = nullptr;
    QToolButton* delete_ = nullptr;
};

} // namespace pictura
