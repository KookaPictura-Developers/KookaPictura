#include "panel_group.h"

#include <QtCore/QEvent>
#include <QtGui/QContextMenuEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {
constexpr int kIconButtonSize = 22;
constexpr int kIconPixmapSize = 16;
} // namespace

PanelGroup::PanelGroup(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    tabs_ = new QTabWidget(this);
    tabs_->setTabPosition(QTabWidget::North);
    tabs_->setDocumentMode(true);
    tabs_->tabBar()->installEventFilter(this);
    layout->addWidget(tabs_);

    iconRow_ = new QWidget(this);
    iconRow_->setObjectName(QStringLiteral("panelGroupIconRow"));
    iconRowLayout_ = new QHBoxLayout(iconRow_);
    iconRowLayout_->setContentsMargins(2, 2, 2, 2);
    iconRowLayout_->setSpacing(2);
    iconRowLayout_->addStretch(1);
    iconRow_->setVisible(false);
    layout->addWidget(iconRow_);
}

void PanelGroup::addPanel(QWidget* panel, const QString& title, const QIcon& icon)
{
    if (!panel) {
        return;
    }
    const int index = tabs_->addTab(panel, icon, title);
    tabs_->setTabToolTip(index, title);
    if (tabs_->count() == 1) {
        setObjectName(QStringLiteral("panelGroup_") + panel->objectName());
    }
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
}

QStringList PanelGroup::titles() const
{
    QStringList out;
    for (int i = 0; i < tabs_->count(); ++i) {
        out << tabs_->tabText(i);
    }
    return out;
}

void PanelGroup::setPanelOrder(const QStringList& order)
{
    const int count = tabs_->count();
    QList<QWidget*> panels;
    QStringList titles;
    QList<QIcon> icons;
    for (int i = 0; i < count; ++i) {
        panels << tabs_->widget(i);
        titles << tabs_->tabText(i);
        icons << tabs_->tabIcon(i);
    }
    QList<int> sequence;
    for (const QString& name : order) {
        for (int i = 0; i < panels.size(); ++i) {
            if (panels.at(i) && panels.at(i)->objectName() == name && !sequence.contains(i)) {
                sequence << i;
                break;
            }
        }
    }
    for (int i = 0; i < panels.size(); ++i) {
        if (!sequence.contains(i)) {
            sequence << i;
        }
    }
    // QTabBar::moveTab reorders the tab bar but not QTabWidget's page stack, so
    // re-insert the pages to keep `panels()` and `titles()` in the same order.
    for (int i = tabs_->count() - 1; i >= 0; --i) {
        tabs_->removeTab(i);
    }
    for (int index : sequence) {
        tabs_->addTab(panels.at(index), icons.at(index), titles.at(index));
    }
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
}

bool PanelGroup::isPanelVisible(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->isTabVisible(i);
        }
    }
    return false;
}

QStringList PanelGroup::visibleTitles() const
{
    QStringList out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            out << tabs_->tabText(i);
        }
    }
    return out;
}

QList<QWidget*> PanelGroup::panels() const
{
    QList<QWidget*> out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (QWidget* panel = tabs_->widget(i)) {
            out << panel;
        }
    }
    return out;
}

QList<QWidget*> PanelGroup::visiblePanels() const
{
    QList<QWidget*> out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            if (QWidget* panel = tabs_->widget(i)) {
                out << panel;
            }
        }
    }
    return out;
}

bool PanelGroup::containsPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return true;
        }
    }
    return false;
}

bool PanelGroup::removePanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return true;
        }
    }
    return false;
}

bool PanelGroup::setPanelVisible(const QString& objectName, bool visible)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->setTabVisible(i, visible);
            if (visible) {
                tabs_->setCurrentIndex(i);
            }
            setVisible(!visibleTitles().isEmpty());
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return true;
        }
    }
    return false;
}

QString PanelGroup::currentPanelName() const
{
    QWidget* panel = tabs_->currentWidget();
    return panel ? panel->objectName() : QString();
}

QString PanelGroup::titleForPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->tabText(i);
        }
    }
    return QString();
}

QIcon PanelGroup::iconForPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->tabIcon(i);
        }
    }
    return QIcon();
}

QWidget* PanelGroup::takePanel(const QString& objectName, QString* title, QIcon* icon, int* index)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            if (title) {
                *title = tabs_->tabText(i);
            }
            if (icon) {
                *icon = tabs_->tabIcon(i);
            }
            if (index) {
                *index = i;
            }
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return panel;
        }
    }
    return nullptr;
}

void PanelGroup::insertPanel(QWidget* panel, const QString& title, const QIcon& icon, int index)
{
    if (!panel) {
        return;
    }
    index = qBound(0, index, tabs_->count());
    const int actual = tabs_->insertTab(index, panel, icon, title);
    tabs_->setTabToolTip(actual, title);
    tabs_->setCurrentIndex(actual);
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
}

QTabBar* PanelGroup::tabBar() const
{
    return tabs_ ? tabs_->tabBar() : nullptr;
}

int PanelGroup::indexOfPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return i;
        }
    }
    return -1;
}

QRect PanelGroup::tabBarGlobalRect() const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return QRect();
    }
    return QRect(bar->mapToGlobal(QPoint(0, 0)), bar->size());
}

int PanelGroup::tabInsertionIndexAt(const QPoint& globalPos) const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return -1;
    }
    const QPoint pos = bar->mapFromGlobal(globalPos);
    if (!bar->rect().contains(pos)) {
        return -1;
    }
    for (int i = 0; i < bar->count(); ++i) {
        if (pos.x() < bar->tabRect(i).center().x()) {
            return i;
        }
    }
    return bar->count();
}

int PanelGroup::tabInsertionX(int index) const
{
    QTabBar* bar = tabBar();
    if (!bar || bar->count() == 0) {
        return 0;
    }
    index = qBound(0, index, bar->count());
    if (index < bar->count()) {
        return bar->tabRect(index).left();
    }
    return bar->tabRect(bar->count() - 1).right() + 1;
}

QPoint PanelGroup::tabInsertionGlobalPointForTest(int index) const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return QPoint();
    }
    return bar->mapToGlobal(QPoint(tabInsertionX(index), bar->height() / 2));
}

void PanelGroup::setMinimized(bool minimized)
{
    if (minimized_ == minimized) {
        return;
    }
    minimized_ = minimized;
    applyMinimize();
}

void PanelGroup::applyMinimize()
{
    if (minimized_) {
        if (collapsedToIcons_) {
            setCollapsedToIcons(false);
        }
        savedMaxHeight_ = tabs_->maximumHeight();
        const int barHeight = tabs_->tabBar() ? tabs_->tabBar()->sizeHint().height() : 24;
        tabs_->setMaximumHeight(barHeight);
    } else {
        tabs_->setMaximumHeight(savedMaxHeight_);
    }
}

void PanelGroup::setCollapsedToIcons(bool collapsed)
{
    if (collapsedToIcons_ == collapsed) {
        return;
    }
    collapsedToIcons_ = collapsed;
    if (collapsed) {
        if (minimized_) {
            minimized_ = false;
            tabs_->setMaximumHeight(savedMaxHeight_);
        }
        rebuildIconRow();
    }
    tabs_->setVisible(!collapsed);
    iconRow_->setVisible(collapsed);
}

void PanelGroup::rebuildIconRow()
{
    while (QLayoutItem* item = iconRowLayout_->takeAt(0)) {
        if (QWidget* widget = item->widget()) {
            delete widget;
        }
        delete item;
    }
    for (int i = 0; i < tabs_->count(); ++i) {
        if (!tabs_->isTabVisible(i)) {
            continue;
        }
        QWidget* panel = tabs_->widget(i);
        if (!panel) {
            continue;
        }
        iconRowLayout_->addWidget(
            makeIconButton(tabs_->tabIcon(i), tabs_->tabText(i), panel->objectName()));
    }
    iconRowLayout_->addStretch(1);
}

QToolButton* PanelGroup::makeIconButton(const QIcon& icon, const QString& title,
                                        const QString& objectName)
{
    auto* button = new QToolButton(iconRow_);
    button->setObjectName(QStringLiteral("panelGroupIcon_") + objectName);
    if (!icon.isNull()) {
        button->setIcon(icon);
        button->setIconSize(QSize(kIconPixmapSize, kIconPixmapSize));
    } else {
        button->setText(title.left(1));
    }
    button->setToolTip(title);
    button->setAutoRaise(true);
    button->setFixedSize(kIconButtonSize, kIconButtonSize);
    connect(button, &QToolButton::clicked, this, [this, objectName, button]() {
        emit panelActivated(objectName, button->mapToGlobal(QPoint(button->width(), 0)));
    });
    return button;
}

QWidget* PanelGroup::detachPanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            detached_.panel = panel;
            detached_.title = tabs_->tabText(i);
            detached_.icon = tabs_->tabIcon(i);
            detached_.index = i;
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return panel;
        }
    }
    return nullptr;
}

bool PanelGroup::attachPanel(const QString& objectName)
{
    if (!detached_.panel || detached_.panel->objectName() != objectName) {
        return false;
    }
    const int index = qBound(0, detached_.index, tabs_->count());
    tabs_->insertTab(index, detached_.panel, detached_.icon, detached_.title);
    tabs_->setTabToolTip(index, detached_.title);
    detached_ = {};
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
    return true;
}

bool PanelGroup::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == tabs_->tabBar()) {
        const QEvent::Type type = event->type();
        if (type == QEvent::ContextMenu) {
            auto* menuEvent = static_cast<QContextMenuEvent*>(event);
            emit tabContextMenuRequested(menuEvent->globalPos());
            return true;
        }
        if (type == QEvent::MouseButtonPress) {
            auto* mouseEvent = static_cast<QMouseEvent*>(event);
            if (mouseEvent->button() == Qt::LeftButton) {
                pressPending_ = true;
                dragging_ = false;
                pressGlobal_ = mouseEvent->globalPosition().toPoint();
                const int index = tabs_->tabBar()->tabAt(mouseEvent->position().toPoint());
                QWidget* panel = index >= 0 ? tabs_->widget(index) : nullptr;
                pressedPanel_ = panel ? panel->objectName() : QString();
            }
        } else if (type == QEvent::MouseMove) {
            auto* mouseEvent = static_cast<QMouseEvent*>(event);
            const QPoint globalPos = mouseEvent->globalPosition().toPoint();
            if (pressPending_ && !dragging_
                && (globalPos - pressGlobal_).manhattanLength()
                       >= QApplication::startDragDistance()) {
                pressPending_ = false;
                dragging_ = true;
                if (pressedPanel_.isEmpty()) {
                    emit groupDragStarted(globalPos);
                } else {
                    emit tabDragStarted(pressedPanel_, globalPos);
                }
            }
            if (dragging_) {
                emit dragMoved(globalPos);
                return true;
            }
        } else if (type == QEvent::MouseButtonRelease) {
            pressPending_ = false;
            if (dragging_) {
                dragging_ = false;
                auto* mouseEvent = static_cast<QMouseEvent*>(event);
                emit dragFinished(mouseEvent->globalPosition().toPoint());
                pressedPanel_.clear();
                return true;
            }
            pressedPanel_.clear();
        }
    }
    return QWidget::eventFilter(watched, event);
}

int PanelGroup::tabPositionForTest() const
{
    return static_cast<int>(tabs_->tabPosition());
}

int PanelGroup::titleCountForTest() const
{
    return tabs_->count();
}

QStringList PanelGroup::titleTextsForTest() const
{
    return titles();
}

QIcon PanelGroup::titleIconForTest(const QString& title) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->tabText(i) == title) {
            return tabs_->tabIcon(i);
        }
    }
    return QIcon();
}

bool PanelGroup::groupLabelForTest() const
{
    return findChild<QLabel*>(QStringLiteral("panelGroupLabel")) != nullptr;
}

bool PanelGroup::contentHiddenForTest() const
{
    return tabs_->maximumHeight() < QWIDGETSIZE_MAX;
}

bool PanelGroup::tabBarVisibleForTest() const
{
    return tabs_->tabBar() && tabs_->tabBar()->isVisible();
}

bool PanelGroup::iconRowVisibleForTest() const
{
    return iconRow_ && iconRow_->isVisible();
}

} // namespace pictura
