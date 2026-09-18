#include "panel_column.h"

#include "panel_column_internal.h"

#include "frame.h"
#include "icons.h"
#include "panel_group.h"
#include "theme.h"

#include <QtCore/QEvent>
#include <QtCore/QJsonObject>
#include <QtCore/QMetaObject>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QAction>
#include <QtGui/QCursor>
#include <QtGui/QFontMetrics>
#include <QtGui/QGuiApplication>
#include <QtGui/QHideEvent>
#include <QtGui/QMouseEvent>
#include <QtGui/QPalette>
#include <QtGui/QScreen>
#include <QtGui/QShowEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenu>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QScrollBar>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

const char* const kTabMenuTexts[] = {
    "Close",
    "Close Panel Group",
    "Minimize",
    "Collapse to Icons",
    "Auto-Collapse Iconic Panels",
    "Auto-Show Hidden Panels",
    "Interface Options\u2026",
};

} // namespace

void PanelColumn::showTabMenu(PanelGroup* group, const QPoint& globalPos)
{
    menuGroup_ = group;
    QMenu* menu = buildTabMenu(group);
    menu->setAttribute(Qt::WA_DeleteOnClose);
    menu->popup(globalPos);
}

QMenu* PanelColumn::buildTabMenu(PanelGroup* group)
{
    auto* menu = new QMenu(this);
    menu->setObjectName(QStringLiteral("panelTabMenu"));

    QAction* close = menu->addAction(tr("Close"));
    connect(close, &QAction::triggered, this, [this, group]() {
        if (!group) {
            return;
        }
        const QString name = group->currentPanelName();
        if (!name.isEmpty()) {
            showPanel(name, false);
        }
    });

    QAction* closeGroupAction = menu->addAction(tr("Close Panel Group"));
    connect(closeGroupAction, &QAction::triggered, this,
            [this, group]() { closeGroup(group); });

    const bool minimized = group && group->isMinimized();
    QAction* minimize = menu->addAction(minimized ? tr("Expand Panel") : tr("Minimize"));
    connect(minimize, &QAction::triggered, this, [this, group]() {
        group->setMinimized(!group->isMinimized());
        emit stateChanged();
    });

    QAction* collapse = menu->addAction(tr("Collapse to Icons"));
    connect(collapse, &QAction::triggered, this, [this, group]() {
        group->setCollapsedToIcons(!group->isCollapsedToIcons());
        emit stateChanged();
    });

    menu->addSeparator();

    QAction* autoCollapse = menu->addAction(tr("Auto-Collapse Iconic Panels"));
    autoCollapse->setCheckable(true);
    autoCollapse->setChecked(autoCollapseIconic_);
    connect(autoCollapse, &QAction::triggered, this, [this](bool on) {
        setAutoCollapseIconic(on);
    });

    QAction* autoShow = menu->addAction(tr("Auto-Show Hidden Panels"));
    autoShow->setCheckable(true);
    autoShow->setChecked(autoShowHidden_);
    connect(autoShow, &QAction::triggered, this, [this](bool on) { setAutoShowHidden(on); });

    menu->addSeparator();

    QAction* options = menu->addAction(tr("Interface Options\u2026"));
    connect(options, &QAction::triggered, this, [this]() { emit interfaceOptionsRequested(); });

    return menu;
}

QStringList PanelColumn::tabMenuTextsFor(const PanelGroup* group) const
{
    QStringList out;
    for (const char* text : kTabMenuTexts) {
        out << QString::fromUtf8(text);
    }
    // M45 W5: the minimize entry is a function of the group's state.
    if (group && group->isMinimized() && out.size() > 2) {
        out[2] = QStringLiteral("Expand Panel");
    }
    return out;
}

} // namespace pictura
