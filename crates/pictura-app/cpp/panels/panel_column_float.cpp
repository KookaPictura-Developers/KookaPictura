#include "panel_column.h"

#include "panel_column_internal.h"

#include "frame.h"
#include "icons.h"
#include "panel_group.h"
#include "theme.h"

#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QIcon>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QScrollBar>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

PanelFloat* PanelColumn::createFloat(PanelGroup* group, const QPoint& globalPos)
{
    if (!group) {
        return nullptr;
    }
    PanelGroup* hosted = group;
    if (dragIsPanel_ && !dragPanel_.isEmpty()) {
        // M43: a tab drag floats a one-panel group holding only the dragged
        // panel; the source stack (docked group or old float) keeps the rest.
        QString title;
        QIcon iconValue;
        int index = -1;
        QWidget* panel = group->takePanel(dragPanel_, &title, &iconValue, &index);
        if (!panel) {
            return nullptr;
        }
        hosted = new PanelGroup(this);
        hosted->addPanel(panel, title, iconValue);
        panelVisible_[dragPanel_] = true;
        wireGroup(hosted);
        // M44 W4: keep the (possibly now empty) source group alive so its tab
        // bar keeps the implicit mouse grab until release; commitDrop/cancelDrag
        // clean it up once the drag ends.
    }
    const int index = groups_.indexOf(hosted);
    if (index >= 0) {
        groups_.removeAt(index);
        hosted->setParent(nullptr);
    }
    // The overlay parents to the main window (its central area is the clamp
    // rect) so it is clipped to the window; it must not parent to the
    // `centerSplitter`, which would absorb it as a splitter pane.
    QWidget* host = window();
    if (!host) {
        host = this;
    }
    auto* floatWindow = new PanelFloat(host);
    floatWindow->setGroup(hosted);
    floatWindow->onClose = [this, floatWindow]() { closeFloat(floatWindow); };
    hosted->setVisible(true);
    QSize size = hosted->sizeHint();
    size = size.expandedTo(QSize(220, 120));
    if (size.width() > 520) {
        size.setWidth(520);
    }
    floatWindow->resize(size);
    moveFloat(floatWindow, globalPos - dragGrabOffset_);
    floatWindow->show();
    // M47: an iconic hosted group needs room for its icon row.
    if (hosted->isCollapsedToIcons()
        && floatWindow->height() < PanelFloat::kFloatIconMinHeight) {
        floatWindow->resize(floatWindow->width(), PanelFloat::kFloatIconMinHeight);
    }
    floatWindow->syncToContent();
    floatWindow->raise();
    floats_ << floatWindow;
    // ponytail: the strip row stays stale during the drag (rebuilt on
    // commit/cancel), because rebuilding it deletes the widget holding the
    // mouse grab.
    // Re-dock routes through the float's group, so commitDrop takes the panel
    // out of the one-panel float (or re-inserts the whole group).
    dragGroup_ = hosted;
    return floatWindow;
}

QRect PanelColumn::floatBounds(QWidget* host) const
{
    if (!host) {
        return QRect();
    }
    // Keep the overlay in the central area plus the visible Tools pane, so a
    // float can cross the toolbar; fall back to the whole window. The menu bar
    // stays clear by clamping the top to the central widget's top.
    if (auto* mainWindow = qobject_cast<QMainWindow*>(host)) {
        if (QWidget* central = mainWindow->centralWidget()) {
            const QRect centralRect(central->mapTo(host, QPoint(0, 0)), central->size());
            QRect bounds = centralRect;
            if (auto* pictura = qobject_cast<PicturaMainWindow*>(host)) {
                if (QWidget* tools = pictura->findChild<QWidget*>(QStringLiteral("toolsPanel"))) {
                    if (tools->isVisible()) {
                        bounds = bounds.united(
                            QRect(tools->mapTo(host, QPoint(0, 0)), tools->size()));
                    }
                }
            }
            if (bounds.top() < centralRect.top()) {
                bounds.setTop(centralRect.top());
            }
            return bounds;
        }
    }
    return host->rect();
}

void PanelColumn::moveFloat(PanelFloat* floatWindow, const QPoint& globalTopLeft)
{
    if (!floatWindow) {
        return;
    }
    QWidget* host = floatWindow->parentWidget();
    if (!host) {
        return;
    }
    const QRect bounds = floatBounds(host);
    const QPoint local = host->mapFromGlobal(globalTopLeft);
    const int maxX = qMax(bounds.left(), bounds.right() - floatWindow->width() + 1);
    const int maxY = qMax(bounds.top(), bounds.bottom() - floatWindow->height() + 1);
    floatWindow->move(qBound(bounds.left(), local.x(), maxX),
                      qBound(bounds.top(), local.y(), maxY));
}

void PanelColumn::destroyFloat(PanelFloat* floatWindow)
{
    if (!floatWindow) {
        return;
    }
    if (!floats_.contains(floatWindow)) {
        // Already torn down by a cleanup path in this drop; do not double-free.
        if (dragFloat_ == floatWindow) {
            dragFloat_ = nullptr;
        }
        maybeRemoveSelf();
        return;
    }
    floats_.removeAll(floatWindow);
    if (dragFloat_ == floatWindow) {
        dragFloat_ = nullptr;
    }
    if (floatIndicator_ == floatWindow) {
        floatIndicator_ = nullptr;
    }
    floatWindow->hide();
    floatWindow->deleteLater();
    maybeRemoveSelf();
}

void PanelColumn::closeFloat(PanelFloat* floatWindow)
{
    if (!floatWindow) {
        return;
    }
    PanelGroup* group = floatWindow->group();
    if (group && group->parentWidget() == floatWindow) {
        group->setParent(nullptr);
        insertGroupAt(group, groups_.size());
        // Destroy the shell before `closeGroup` can trigger `maybeRemoveSelf`,
        // which would re-home this float and leave the empty overlay behind.
        destroyFloat(floatWindow);
        closeGroup(group);
        return;
    }
    destroyFloat(floatWindow);
}

void PanelColumn::rehomeFloatsTo(PanelColumn* target)
{
    if (!target || target == this) {
        return;
    }
    for (PanelFloat* floatWindow : floats_) {
        if (!floatWindow) {
            continue;
        }
        PanelGroup* group = floatWindow->group();
        if (!group) {
            continue;
        }
        QObject::disconnect(group, nullptr, this, nullptr);
        wired_.remove(group);
        target->wireGroup(group);
        // The float's close callback captured this (now-removed) column; repoint
        // it at the target so the re-homed overlay's close button stays valid.
        floatWindow->onClose = [target, floatWindow]() { target->closeFloat(floatWindow); };
        for (QWidget* panel : group->panels()) {
            if (panel) {
                target->panelVisible_[panel->objectName()] =
                    group->isPanelVisible(panel->objectName());
            }
        }
    }
    target->floats_.append(floats_);
    floats_.clear();
}

PanelFloat* PanelColumn::groupFloatAtGlobal(const QPoint& globalPos) const
{
    for (PanelFloat* floatWindow : floats_) {
        if (!floatWindow || !floatWindow->group()) {
            continue;
        }
        if (QRect(floatWindow->mapToGlobal(QPoint(0, 0)), floatWindow->size()).contains(globalPos)) {
            return floatWindow;
        }
    }
    return nullptr;
}

PanelFloat* PanelColumn::floatColumn(const QPoint& globalTopLeft)
{
    if (columnFloat_) {
        return columnFloat_;
    }
    QWidget* host = window();
    if (!host) {
        return nullptr;
    }
    // Keep the on-screen width (never below this column's minimum) so a re-dock
    // lands in the slot the column left; the overlay carries the shared minimum
    // width and the size grip for both group and column payloads.
    const int width =
        qMax(minimumWidth(), this->width() > 0 ? this->width() : kDefaultNormalWidth);
    const int height = this->height() > 0 ? this->height() : 240;
    auto* floatWindow = new PanelFloat(host);
    floatWindow->setContent(this);
    floatWindow->resize(width, height);
    moveFloat(floatWindow, globalTopLeft);
    floatWindow->show();
    floatWindow->raise();
    columnFloat_ = floatWindow;
    return floatWindow;
}

bool PanelColumn::redockColumnFloat(int side, PanelColumn* anchor)
{
    auto* frame = qobject_cast<PicturaMainWindow*>(window());
    if (!columnFloat_ || !frame) {
        return false;
    }
    // The frame's move path reparents the column out of the overlay and into
    // the splitter; only then is the empty overlay safe to delete.
    if (!frame->movePanelColumn(this, side, anchor)) {
        return false;
    }
    destroyColumnFloat();
    return true;
}

void PanelColumn::destroyColumnFloat()
{
    if (!columnFloat_) {
        return;
    }
    PanelFloat* floatWindow = columnFloat_;
    columnFloat_ = nullptr;
    floatWindow->hide();
    floatWindow->deleteLater();
}

void PanelColumn::cancelColumnFloat()
{
    destroyColumnFloat();
}

void PanelColumn::updateColumnDrag(const QPoint& globalPos)
{
    auto* frame = qobject_cast<PicturaMainWindow*>(window());
    if (!frame) {
        return;
    }
    int side = -1;
    PanelColumn* anchor = frame->resolveColumnMoveTarget(globalPos, this, &side);
    const bool hasTarget = side >= 0;
    // M48: an unresolved move tears the whole column off into the in-window
    // overlay, which then follows the cursor until a target resolves or the drag
    // ends. This mirrors the group drag's outside-band float; no second drag
    // system.
    if (!hasTarget && !columnFloat_) {
        floatColumn(globalPos - columnGrabOffset_);
    } else if (columnFloat_) {
        moveFloat(columnFloat_, globalPos - columnGrabOffset_);
    }
    // Phase 6: a whole-column drag dims its overlay for the whole drag.
    setDragDimTarget(columnFloat_);
    // A bare workspace edge resolves no anchor; approximate the edge line on the
    // outermost visible column so the user still sees where the column lands.
    // ponytail: visual-only anchor; the commit inserts at the splitter head/tail.
    PanelColumn* indicator = anchor;
    if (!indicator && hasTarget) {
        const QList<PanelColumn*> columns = frame->panelColumns();
        if (side == 0) {
            for (PanelColumn* column : columns) {
                if (column && column->isVisible() && column != this) {
                    indicator = column;
                    break;
                }
            }
        } else {
            for (int i = columns.size() - 1; i >= 0; --i) {
                PanelColumn* column = columns.at(i);
                if (column && column->isVisible() && column != this) {
                    indicator = column;
                    break;
                }
            }
        }
    }
    if (columnDropAnchor_ && columnDropAnchor_ != indicator) {
        columnDropAnchor_->hideEdgeDropIndicator();
    }
    columnDropAnchor_ = indicator;
    columnDropSide_ = side;
    if (indicator && hasTarget) {
        indicator->showEdgeDropIndicator(side == 0 ? PanelSide::Left : PanelSide::Right);
    }
}

bool PanelColumn::finishColumnDrag(const QPoint& globalPos)
{
    setDragDimTarget(nullptr);
    auto* frame = qobject_cast<PicturaMainWindow*>(window());
    if (!frame) {
        return false;
    }
    int side = -1;
    PanelColumn* anchor = frame->resolveColumnMoveTarget(globalPos, this, &side);
    if (columnDropAnchor_) {
        columnDropAnchor_->hideEdgeDropIndicator();
    }
    columnDropAnchor_ = nullptr;
    columnDropSide_ = -1;
    if (side >= 0) {
        // A valid target re-places the column; a floating one redocks into the
        // splitter at the resolved boundary.
        return columnFloat_ ? redockColumnFloat(side, anchor)
                            : frame->movePanelColumn(this, side, anchor);
    }
    // No target: the column stays torn off in the in-window overlay instead of
    // doing nothing, and a drag that never moved floats it now.
    if (!columnFloat_) {
        return floatColumn(globalPos - columnGrabOffset_) != nullptr;
    }
    return true;
}

} // namespace pictura
