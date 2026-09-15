#include "theme.h"

#include <QtGui/QColor>
#include <QtGui/QPalette>
#include <QtWidgets/QApplication>
#include <QtWidgets/QStyleFactory>

namespace pictura {

namespace {

struct Ramp {
    QColor window;
    QColor windowText;
    QColor base;
    QColor alternateBase;
    QColor text;
    QColor button;
    QColor buttonText;
    QColor highlight;
    QColor highlightedText;
    QColor toolTipBase;
    QColor toolTipText;
    QColor link;
    QColor disabledText;
};

// 0 = darkest .. 3 = lightest. Values are a defensible dark ramp; the corpus
// does not pin exact CS6 colours (M16 open question).
const Ramp kRamps[Theme::kLevelCount] = {
    {QColor(26, 26, 26),
     QColor(216, 216, 216),
     QColor(18, 18, 18),
     QColor(33, 33, 33),
     QColor(216, 216, 216),
     QColor(38, 38, 38),
     QColor(216, 216, 216),
     QColor(61, 111, 153),
     QColor(255, 255, 255),
     QColor(42, 42, 42),
     QColor(232, 232, 232),
     QColor(111, 168, 220),
     QColor(96, 96, 96)},
    {QColor(43, 43, 43),
     QColor(224, 224, 224),
     QColor(35, 35, 35),
     QColor(50, 50, 50),
     QColor(224, 224, 224),
     QColor(56, 56, 56),
     QColor(224, 224, 224),
     QColor(61, 111, 153),
     QColor(255, 255, 255),
     QColor(58, 58, 58),
     QColor(232, 232, 232),
     QColor(111, 168, 220),
     QColor(104, 104, 104)},
    {QColor(58, 58, 58),
     QColor(232, 232, 232),
     QColor(50, 50, 50),
     QColor(65, 65, 65),
     QColor(232, 232, 232),
     QColor(71, 71, 71),
     QColor(232, 232, 232),
     QColor(61, 111, 153),
     QColor(255, 255, 255),
     QColor(74, 74, 74),
     QColor(236, 236, 236),
     QColor(111, 168, 220),
     QColor(112, 112, 112)},
    {QColor(74, 74, 74),
     QColor(240, 240, 240),
     QColor(66, 66, 66),
     QColor(81, 81, 81),
     QColor(240, 240, 240),
     QColor(87, 87, 87),
     QColor(240, 240, 240),
     QColor(61, 111, 153),
     QColor(255, 255, 255),
     QColor(90, 90, 90),
     QColor(240, 240, 240),
     QColor(111, 168, 220),
     QColor(120, 120, 120)},
};

QPalette paletteFor(int level)
{
    const Ramp& ramp = kRamps[level];
    QPalette pal;
    pal.setColor(QPalette::Window, ramp.window);
    pal.setColor(QPalette::WindowText, ramp.windowText);
    pal.setColor(QPalette::Base, ramp.base);
    pal.setColor(QPalette::AlternateBase, ramp.alternateBase);
    pal.setColor(QPalette::Text, ramp.text);
    pal.setColor(QPalette::Button, ramp.button);
    pal.setColor(QPalette::ButtonText, ramp.buttonText);
    pal.setColor(QPalette::Highlight, ramp.highlight);
    pal.setColor(QPalette::HighlightedText, ramp.highlightedText);
    pal.setColor(QPalette::ToolTipBase, ramp.toolTipBase);
    pal.setColor(QPalette::ToolTipText, ramp.toolTipText);
    pal.setColor(QPalette::Link, ramp.link);
    pal.setColor(QPalette::Disabled, QPalette::Text, ramp.disabledText);
    pal.setColor(QPalette::Disabled, QPalette::WindowText, ramp.disabledText);
    pal.setColor(QPalette::Disabled, QPalette::ButtonText, ramp.disabledText);
    return pal;
}

} // namespace

int Theme::clampLevel(int level)
{
    if (level < 0) {
        return 0;
    }
    if (level >= kLevelCount) {
        return kLevelCount - 1;
    }
    return level;
}

void Theme::apply(int level)
{
    const int clamped = clampLevel(level);
    QApplication::setStyle(QStyleFactory::create(QStringLiteral("Fusion")));
    const QPalette pal = paletteFor(clamped);
    qApp->setPalette(pal);
}

} // namespace pictura
