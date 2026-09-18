#include "panel_group.h"

#include <QtGui/QAction>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

int PanelGroup::headerCornerWidthForTest() const
{
    int width = headerButton_ ? headerButton_->width() : 0;
    if (floatCloseButton_ && floatCloseButton_->isVisible()) {
        width += floatCloseButton_->width();
    }
    return width;
}

bool PanelGroup::floatCloseVisibleForTest() const
{
    return floatCloseButton_ && floatCloseButton_->isVisible();
}

QPoint PanelGroup::tabInsertionGlobalPointForTest(int index) const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return QPoint();
    }
    return bar->mapToGlobal(QPoint(tabInsertionX(index), bar->height() / 2));
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

int PanelGroup::firstVisibleTabIndexForTest() const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            return i;
        }
    }
    return -1;
}

int PanelGroup::currentTabIndexForTest() const
{
    return tabs_->currentIndex();
}

bool PanelGroup::headerMenuAtRightForTest() const
{
    if (!headerButton_ || !headerButton_->isVisible()
        || tabs_->cornerWidget(Qt::TopRightCorner) != headerCorner_
        || !tabs_->isAncestorOf(headerButton_)) {
        return false;
    }
    QTabBar* bar = tabs_->tabBar();
    if (!bar) {
        return false;
    }
    // The corner widget sits immediately to the right of the tab bar.
    const QPoint topLeft = headerButton_->mapTo(tabs_, QPoint(0, 0));
    return topLeft.x() >= bar->width() - 2
           && topLeft.x() + headerButton_->width() <= tabs_->width();
}

QStringList PanelGroup::panelMenuTextsForTest() const
{
    QStringList texts;
    if (!headerMenu_) {
        return texts;
    }
    for (QAction* action : headerMenu_->actions()) {
        if (!action->isSeparator()) {
            texts.push_back(action->text());
        }
    }
    return texts;
}

static QAction* findMenuAction(QMenu* menu, const QString& text)
{
    if (!menu) {
        return nullptr;
    }
    for (QAction* action : menu->actions()) {
        if (action->text() == text) {
            return action;
        }
        if (QMenu* child = action->menu()) {
            for (QAction* sub : child->actions()) {
                if (sub->text() == text) {
                    return sub;
                }
            }
        }
    }
    return nullptr;
}

bool PanelGroup::panelMenuEnabledForTest(const QString& text) const
{
    QAction* action = findMenuAction(headerMenu_, text);
    return action && action->isEnabled();
}

QString PanelGroup::panelMenuToolTipForTest(const QString& text) const
{
    QAction* action = findMenuAction(headerMenu_, text);
    return action ? action->toolTip() : QString();
}

bool PanelGroup::triggerPanelMenuForTest(const QString& text)
{
    QAction* action = findMenuAction(headerMenu_, text);
    if (!action || !action->isEnabled()) {
        return false;
    }
    action->trigger();
    return true;
}

} // namespace pictura
