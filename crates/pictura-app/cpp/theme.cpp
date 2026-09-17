#include "theme.h"

#include <QtCore/QString>
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

struct ColorToken {
    const char* key;
    QColor color;
};

// ponytail: derived from the existing dark ramp; exact CS6 swatches are
// unsourced (M23 open question), so the stylesheet only re-colours surfaces.
QString styleSheetFor(const Ramp& ramp)
{
    const ColorToken tokens[] = {
        {"${window}", ramp.window},
        {"${windowText}", ramp.windowText},
        {"${base}", ramp.base},
        {"${alternateBase}", ramp.alternateBase},
        {"${text}", ramp.text},
        {"${button}", ramp.button},
        {"${buttonText}", ramp.buttonText},
        {"${highlight}", ramp.highlight},
        {"${highlightedText}", ramp.highlightedText},
        {"${toolTipBase}", ramp.toolTipBase},
        {"${toolTipText}", ramp.toolTipText},
        {"${link}", ramp.link},
        {"${disabledText}", ramp.disabledText},
        {"${border}", ramp.window.darker(135)},
        {"${hover}", ramp.button.lighter(120)},
        {"${pressed}", ramp.button.darker(120)},
        {"${activeTab}", ramp.window.lighter(130)},
    };

    QString qss = QStringLiteral(R"(
QMainWindow { background: ${window}; }
QMainWindow::separator { background: ${border}; width: 3px; height: 3px; }

QMenuBar { background: ${window}; color: ${windowText}; border-bottom: 1px solid ${border}; }
QMenuBar::item { background: transparent; color: ${windowText}; padding: 4px 8px; }
QMenuBar::item:selected { background: ${highlight}; color: ${highlightedText}; }
QMenuBar::item:pressed { background: ${pressed}; color: ${buttonText}; }

QMenu { background: ${base}; color: ${text}; border: 1px solid ${border}; }
QMenu::item { background: transparent; padding: 4px 22px; }
QMenu::item:selected { background: ${highlight}; color: ${highlightedText}; }
QMenu::item:disabled { color: ${disabledText}; }
QMenu::separator { background: ${border}; height: 1px; margin: 4px 6px; }

QToolBar { background: ${window}; color: ${windowText}; border: 0; spacing: 2px; padding: 2px; }
QToolBar::separator { background: ${border}; width: 1px; margin: 3px 2px; }
QToolBar#optionsBar { border-bottom: 1px solid ${border}; }

QToolButton { background: ${button}; color: ${buttonText}; border: 1px solid ${border}; border-radius: 3px; padding: 3px; }
QToolButton:hover { background: ${hover}; border-color: ${highlight}; }
QToolButton:pressed { background: ${pressed}; }
QToolButton:checked { background: ${pressed}; border-color: ${highlight}; color: ${buttonText}; }
QToolButton:disabled { color: ${disabledText}; }

QDockWidget { color: ${windowText}; }
QDockWidget::title { background: ${window}; color: ${windowText}; padding: 3px 6px; border-bottom: 1px solid ${border}; }
QDockWidget::close-button, QDockWidget::float-button { background: transparent; border: 0; }

QTabBar::tab { background: ${window}; color: ${windowText}; border: 1px solid ${border}; border-bottom: 0; padding: 4px 10px; margin-right: 1px; }
QTabBar::tab:hover { background: ${hover}; }
QTabBar::tab:selected { background: ${activeTab}; color: ${windowText}; }
QTabBar::tab:disabled { color: ${disabledText}; }

/* M43: the panel-group tab bars are named `panelTabBar`, so their active tab
   takes the pane (`${base}`) colour while the document tabs keep the unscoped
   `QTabBar::tab` rules above. Inactive panel tabs stay `${window}`/`${hover}`. */
QTabBar#panelTabBar::tab { background: ${window}; color: ${windowText}; border: 1px solid ${border}; border-bottom: 0; padding: 4px 8px; margin-right: 1px; }
QTabBar#panelTabBar::tab:hover { background: ${hover}; }
QTabBar#panelTabBar::tab:selected { background: ${base}; color: ${windowText}; }

QTabWidget::pane { border: 1px solid ${border}; background: ${base}; }
QTabWidget::tab-bar { alignment: left; }

QStatusBar { background: ${window}; color: ${windowText}; border-top: 1px solid ${border}; }
QStatusBar::item { border: 0; }

QScrollBar:vertical { background: ${window}; width: 12px; border: 0; margin: 0; }
QScrollBar::handle:vertical { background: ${button}; min-height: 24px; border-radius: 3px; margin: 2px; }
QScrollBar::handle:vertical:hover { background: ${hover}; }
QScrollBar:horizontal { background: ${window}; height: 12px; border: 0; margin: 0; }
QScrollBar::handle:horizontal { background: ${button}; min-width: 24px; border-radius: 3px; margin: 2px; }
QScrollBar::handle:horizontal:hover { background: ${hover}; }
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical,
QScrollBar::add-line:horizontal, QScrollBar::sub-line:horizontal { background: transparent; border: 0; width: 0; height: 0; }
QScrollBar::add-page, QScrollBar::sub-page { background: transparent; }

QToolTip { background: ${toolTipBase}; color: ${toolTipText}; border: 1px solid ${border}; padding: 2px; }

QListView, QTreeView, QTableView { background: ${base}; color: ${text}; border: 1px solid ${border}; alternate-background-color: ${alternateBase}; selection-background-color: ${highlight}; selection-color: ${highlightedText}; }
QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox, QPlainTextEdit { background: ${base}; color: ${text}; border: 1px solid ${border}; border-radius: 2px; padding: 1px 2px; }
QLineEdit:disabled, QSpinBox:disabled, QDoubleSpinBox:disabled, QComboBox:disabled, QPlainTextEdit:disabled { color: ${disabledText}; }
QComboBox QAbstractItemView { background: ${base}; color: ${text}; border: 1px solid ${border}; selection-background-color: ${highlight}; selection-color: ${highlightedText}; }

QPushButton { background: ${button}; color: ${buttonText}; border: 1px solid ${border}; border-radius: 3px; padding: 4px 10px; }
QPushButton:hover { background: ${hover}; }
QPushButton:pressed { background: ${pressed}; }
QPushButton:disabled { color: ${disabledText}; }
)");

    for (const ColorToken& token : tokens) {
        qss.replace(QLatin1String(token.key), token.color.name(QColor::HexRgb));
    }
    return qss;
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

QString Theme::styleSheet(int level)
{
    return styleSheetFor(kRamps[clampLevel(level)]);
}

void Theme::apply(int level)
{
    const int clamped = clampLevel(level);
    QApplication::setStyle(QStyleFactory::create(QStringLiteral("Fusion")));
    qApp->setPalette(paletteFor(clamped));
    qApp->setStyleSheet(styleSheet(clamped));
}

} // namespace pictura
