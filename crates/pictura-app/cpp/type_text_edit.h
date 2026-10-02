#pragma once

#include <QtCore/QString>

namespace pictura {

// The text a Type tool session edits, with a caret and a selection anchor —
// the text-field model photorust's CanvasView keeps in m_typeText /
// m_typeCaret / m_typeAnchor. Indices are UTF-16 positions; lines are
// separated by '\r', as `TySh` separates them. The selection runs between the
// anchor and the caret.
class TypeTextEdit {
public:
    void reset(const QString& text, int caret);

    const QString& text() const { return text_; }
    int caret() const { return caret_; }
    int anchor() const { return anchor_; }
    bool hasSelection() const { return caret_ != anchor_; }
    int selectionStart() const { return qMin(caret_, anchor_); }
    int selectionEnd() const { return qMax(caret_, anchor_); }
    QString selectedText() const;

    // Replace the selection (or insert at the caret) with `text`; '\n' and
    // "\r\n" become '\r'.
    void insert(const QString& text);
    // Backspace / Delete: the selection if there is one, else the character
    // before / after the caret (a surrogate pair as one).
    void backspace();
    void deleteForward();

    // Move the caret; `extend` keeps the anchor (Shift).
    void moveTo(int index, bool extend);
    // Left / Right along the text (`direction` -1 / +1), by word with Ctrl.
    // Without `extend` a selection collapses to its near edge first.
    void step(int direction, bool byWord, bool extend);
    // Up / Down to the neighbouring line, keeping the column.
    void stepLine(int direction, bool extend);
    // Home / End of the caret's line, or of the whole text with Ctrl.
    void home(bool whole, bool extend);
    void end(bool whole, bool extend);
    void selectAll();
    // Double-click: the word (or run of spaces) around `index`.
    void selectWord(int index);

private:
    int lineStart(int index) const;
    int lineEnd(int index) const;
    int wordStep(int from, int direction) const;
    int clampToChar(int index, int direction) const;

    QString text_;
    int caret_ = 0;
    int anchor_ = 0;
};

} // namespace pictura
