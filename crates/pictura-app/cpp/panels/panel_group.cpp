#include "panel_group.h"

#include "icons.h"

#include <QtCore/QEvent>
#include <QtGui/QContextMenuEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {
constexpr int kIconButtonSize = 22;
constexpr int kIconPixmapSize = 16;
constexpr int kHeaderButtonSize = 18;
} // namespace

PanelGroup::PanelGroup(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    tabs_ = new QTabWidget(this);
    tabs_->setObjectName(QStringLiteral("panelGroupTabs"));
    tabs_->setTabPosition(QTabWidget::North);
    tabs_->setDocumentMode(true);
    tabs_->tabBar()->installEventFilter(this);
    // M43: name the panel-group tab bar so the scoped theme can make the active
    // tab use the pane `${base}` colour without touching the document tabs. The
    // tab bar elides instead of forcing width, and does not expand, so the
    // corner `▾` button keeps its place at the column minimum width.
    tabs_->tabBar()->setObjectName(QStringLiteral("panelTabBar"));
    tabs_->tabBar()->setElideMode(Qt::ElideRight);
    tabs_->tabBar()->setExpanding(false);
    layout->addWidget(tabs_);

    headerCorner_ = new QWidget(tabs_);
    auto* cornerLayout = new QHBoxLayout(headerCorner_);
    cornerLayout->setContentsMargins(0, 0, 0, 0);
    cornerLayout->setSpacing(0);

    headerButton_ = new QToolButton(headerCorner_);
    headerButton_->setObjectName(QStringLiteral("panelWidgetMenu"));
    headerButton_->setText(QStringLiteral("\u25BE"));
    headerButton_->setAutoRaise(true);
    headerButton_->setPopupMode(QToolButton::InstantPopup);
    headerButton_->setFixedSize(kHeaderButtonSize, kHeaderButtonSize);
    headerButton_->setVisible(false);
    cornerLayout->addWidget(headerButton_);

    floatCloseButton_ = new QToolButton(headerCorner_);
    floatCloseButton_->setObjectName(QStringLiteral("panelFloatClose"));
    floatCloseButton_->setAutoRaise(true);
    floatCloseButton_->setFixedSize(kHeaderButtonSize, kHeaderButtonSize);
    floatCloseButton_->setToolTip(tr("Close"));
    floatCloseButton_->setIcon(icon(QStringLiteral("panel.close")));
    floatCloseButton_->setVisible(false);
    cornerLayout->addWidget(floatCloseButton_);

    tabs_->setCornerWidget(headerCorner_, Qt::TopRightCorner);
    connect(tabs_, &QTabWidget::currentChanged, this, [this]() { updateHeaderMenu(); });

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
    updateHeaderMenu();
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

bool PanelGroup::setCurrentPanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->setCurrentIndex(i);
            return true;
        }
    }
    return false;
}

void PanelGroup::setCurrentToFirstVisible()
{
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            tabs_->setCurrentIndex(i);
            return;
        }
    }
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
    const int count = bar->count();
    index = qBound(0, index, count);
    // ponytail: hidden QTabBar tabs have empty rects, so scan for the first
    // visible tab instead of trusting tabRect(index).
    for (int i = index; i < count; ++i) {
        const QRect rect = bar->tabRect(i);
        if (rect.isValid() && !rect.isEmpty()) {
            return rect.left();
        }
    }
    for (int i = qMin(index, count) - 1; i >= 0; --i) {
        const QRect rect = bar->tabRect(i);
        if (rect.isValid() && !rect.isEmpty()) {
            return rect.right() + 1;
        }
    }
    return bar->rect().right() + 1;
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
        savedGroupMaxHeight_ = maximumHeight();
        const int barHeight = tabs_->tabBar() ? tabs_->tabBar()->sizeHint().height() : 24;
        savedTabsMinHeight_ = tabs_->minimumHeight();
        savedMinHeight_ = minimumHeight();
        tabs_->setMinimumHeight(barHeight);
        setMinimumHeight(barHeight);
        // M45 W5: clamp the group itself, not only its tab widget, so the
        // splitter gives it just the tab-bar height.
        tabs_->setMaximumHeight(barHeight);
        setMaximumHeight(barHeight);
        setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Fixed);
    } else {
        tabs_->setMinimumHeight(savedTabsMinHeight_);
        setMinimumHeight(savedMinHeight_);
        tabs_->setMaximumHeight(savedMaxHeight_);
        setMaximumHeight(savedGroupMaxHeight_);
        setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Expanding);
    }
    updateGeometry();
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
            applyMinimize();
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

void PanelGroup::setFloating(bool on)
{
    if (floatCloseButton_) {
        floatCloseButton_->setVisible(on);
    }
    updateHeaderMenu();
}

} // namespace pictura
