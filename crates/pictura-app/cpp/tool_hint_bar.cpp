#include "tool_hint_bar.h"

#include <QtGui/QFontMetrics>
#include <QtGui/QPainter>

namespace pictura {

ToolHintBar::ToolHintBar(QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("toolHintBar"));
    setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Fixed);
}

void ToolHintBar::setHints(const QList<ToolHint>& hints, const QString& fallbackText)
{
    hints_ = hints;
    fallbackText_ = fallbackText;
    pressed_ = -1;
    updateGeometry();
    update();
}

void ToolHintBar::setPressedKey(const QString& key)
{
    int index = -1;
    if (!key.isEmpty()) {
        for (int i = 0; i < hints_.size(); ++i) {
            if (hints_.at(i).key.compare(key, Qt::CaseInsensitive) == 0) {
                index = i;
                break;
            }
        }
    }
    if (index != pressed_) {
        pressed_ = index;
        update();
    }
}

QString ToolHintBar::keyForTest(int index) const
{
    return index >= 0 && index < hints_.size() ? hints_.at(index).key : QString();
}

QString ToolHintBar::textForTest(int index) const
{
    return index >= 0 && index < hints_.size() ? hints_.at(index).text : QString();
}

QSize ToolHintBar::sizeHint() const
{
    const QFontMetrics fm(font());
    int width = 8;
    if (hints_.isEmpty()) {
        width += fm.horizontalAdvance(fallbackText_);
    } else {
        for (const ToolHint& hint : hints_) {
            width += fm.horizontalAdvance(hint.key) + 18;
            width += fm.horizontalAdvance(hint.text) + 12;
        }
    }
    return QSize(width, fm.height() + 8);
}

void ToolHintBar::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    const QFontMetrics fm(font());
    if (hints_.isEmpty()) {
        painter.setPen(palette().color(QPalette::WindowText));
        painter.drawText(rect().adjusted(4, 0, -4, 0), Qt::AlignVCenter | Qt::AlignLeft,
                         fallbackText_);
        return;
    }
    const int keyHeight = fm.height() + 4;
    const int y = (height() - keyHeight) / 2;
    int x = 4;
    for (int i = 0; i < hints_.size(); ++i) {
        const ToolHint& hint = hints_.at(i);
        const int keyWidth = fm.horizontalAdvance(hint.key) + 12;
        const QRect keyRect(x, y, keyWidth, keyHeight);
        const bool hot = i == pressed_;
        painter.setPen(QPen(palette().color(QPalette::WindowText), 1));
        painter.setBrush(hot ? palette().color(QPalette::Highlight)
                             : palette().color(QPalette::Button));
        painter.drawRoundedRect(keyRect, 3, 3);
        painter.setPen(hot ? palette().color(QPalette::HighlightedText)
                           : palette().color(QPalette::WindowText));
        painter.drawText(keyRect, Qt::AlignCenter, hint.key);
        x += keyWidth + 6;
        painter.setPen(palette().color(QPalette::WindowText));
        painter.drawText(QRect(x, y, fm.horizontalAdvance(hint.text), keyHeight),
                         Qt::AlignVCenter | Qt::AlignLeft, hint.text);
        x += fm.horizontalAdvance(hint.text) + 14;
    }
}

} // namespace pictura
