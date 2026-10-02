// The Type tools' text-field model. Ported from photorust's CanvasView
// typeKeyPress / typeMoveCaret / typeSelectWord / typeDeleteSelection.

#include "type_text_edit.h"

namespace pictura {

namespace {

// Letters, digits, and underscore make a word, as in any text field.
bool isWordChar(QChar c)
{
    return c.isLetterOrNumber() || c == QLatin1Char('_');
}

QString normalized(QString text)
{
    text.replace(QStringLiteral("\r\n"), QStringLiteral("\r"));
    text.replace(QLatin1Char('\n'), QLatin1Char('\r'));
    return text;
}

} // namespace

void TypeTextEdit::reset(const QString& text, int caret)
{
    text_ = normalized(text);
    caret_ = anchor_ = clampToChar(caret, -1);
}

QString TypeTextEdit::selectedText() const
{
    return text_.mid(selectionStart(), selectionEnd() - selectionStart());
}

void TypeTextEdit::insert(const QString& text)
{
    const int from = selectionStart();
    const QString clean = normalized(text);
    text_.replace(from, selectionEnd() - from, clean);
    caret_ = anchor_ = from + int(clean.size());
}

void TypeTextEdit::backspace()
{
    if (hasSelection()) {
        insert(QString());
    } else if (caret_ > 0) {
        anchor_ = clampToChar(caret_ - 1, -1);
        insert(QString());
    }
}

void TypeTextEdit::deleteForward()
{
    if (hasSelection()) {
        insert(QString());
    } else if (caret_ < text_.size()) {
        anchor_ = clampToChar(caret_ + 1, 1);
        insert(QString());
    }
}

void TypeTextEdit::moveTo(int index, bool extend)
{
    caret_ = clampToChar(qBound(0, index, int(text_.size())), -1);
    if (!extend) {
        anchor_ = caret_;
    }
}

void TypeTextEdit::step(int direction, bool byWord, bool extend)
{
    if (!extend && hasSelection()) {
        moveTo(direction < 0 ? selectionStart() : selectionEnd(), false);
        return;
    }
    const int target = byWord ? wordStep(caret_, direction) : clampToChar(caret_ + direction, direction);
    moveTo(target, extend);
}

void TypeTextEdit::stepLine(int direction, bool extend)
{
    const int start = lineStart(caret_);
    const int column = caret_ - start;
    if (direction < 0) {
        if (start == 0) {
            moveTo(0, extend);
            return;
        }
        const int prevStart = lineStart(start - 1);
        moveTo(prevStart + qMin(column, start - 1 - prevStart), extend);
        return;
    }
    const int end = lineEnd(caret_);
    if (end >= text_.size()) {
        moveTo(int(text_.size()), extend);
        return;
    }
    const int nextStart = end + 1;
    moveTo(nextStart + qMin(column, lineEnd(nextStart) - nextStart), extend);
}

void TypeTextEdit::home(bool whole, bool extend)
{
    moveTo(whole ? 0 : lineStart(caret_), extend);
}

void TypeTextEdit::end(bool whole, bool extend)
{
    moveTo(whole ? int(text_.size()) : lineEnd(caret_), extend);
}

void TypeTextEdit::selectAll()
{
    anchor_ = 0;
    caret_ = int(text_.size());
}

void TypeTextEdit::selectWord(int index)
{
    const int at = qBound(0, index, int(text_.size()));
    // On a space, the run of spaces: the gesture always grabs something.
    const bool word = at < text_.size() && isWordChar(text_.at(at));
    const auto matches = [word](QChar c) { return isWordChar(c) == word && c != QLatin1Char('\r'); };
    int from = at;
    while (from > 0 && matches(text_.at(from - 1))) {
        --from;
    }
    int to = at;
    while (to < text_.size() && matches(text_.at(to))) {
        ++to;
    }
    anchor_ = from;
    caret_ = to;
}

int TypeTextEdit::lineStart(int index) const
{
    const int cr = index > 0 ? int(text_.lastIndexOf(QLatin1Char('\r'), index - 1)) : -1;
    return cr + 1;
}

int TypeTextEdit::lineEnd(int index) const
{
    const int cr = int(text_.indexOf(QLatin1Char('\r'), index));
    return cr < 0 ? int(text_.size()) : cr;
}

// Past any run of separators, then past the word itself.
int TypeTextEdit::wordStep(int from, int direction) const
{
    const int size = int(text_.size());
    const auto charAt = [this, direction](int at) { return text_.at(direction < 0 ? at - 1 : at); };
    const auto inside = [size, direction](int at) { return direction < 0 ? at > 0 : at < size; };
    int at = from;
    while (inside(at) && !isWordChar(charAt(at))) {
        at += direction;
    }
    while (inside(at) && isWordChar(charAt(at))) {
        at += direction;
    }
    return at;
}

// Never between the halves of a surrogate pair: step past it in `direction`.
int TypeTextEdit::clampToChar(int index, int direction) const
{
    index = qBound(0, index, int(text_.size()));
    if (index > 0 && index < text_.size() && text_.at(index).isLowSurrogate()
        && text_.at(index - 1).isHighSurrogate()) {
        index += direction < 0 ? -1 : 1;
    }
    return index;
}

} // namespace pictura
