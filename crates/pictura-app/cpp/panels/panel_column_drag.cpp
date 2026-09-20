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

bool PanelColumn::eventFilter(QObject* watched, QEvent* event)
{
    // M47: drag the whole column from its top header onto any side of another
    // column (or a workspace edge). The toggle child is not filtered.
    if (watched == header_) {
        const QEvent::Type type = event->type();
        if (type == QEvent::MouseButtonPress) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            if (mouse->button() == Qt::LeftButton) {
                columnPressPending_ = true;
                columnDragging_ = false;
                columnPressGlobal_ = mouse->globalPosition().toPoint();
            }
        } else if (type == QEvent::MouseMove) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            const QPoint globalPos = mouse->globalPosition().toPoint();
            if (columnPressPending_ && !columnDragging_
                && (globalPos - columnPressGlobal_).manhattanLength()
                       >= QApplication::startDragDistance()) {
                columnPressPending_ = false;
                columnDragging_ = true;
            }
            if (columnDragging_) {
                updateColumnDrag(globalPos);
                return true;
            }
        } else if (type == QEvent::MouseButtonRelease) {
            columnPressPending_ = false;
            if (columnDragging_) {
                columnDragging_ = false;
                auto* mouse = static_cast<QMouseEvent*>(event);
                return finishColumnDrag(mouse->globalPosition().toPoint());
            }
        }
    }
    if (watched == iconStrip_ && event->type() == QEvent::Resize) {
        updateIconStripLabels();
    }
    // M44 C3: the group drag-handle grip drags the whole group through the same
    // begin/update/commit path as a panel button.
    if (auto* grip = qobject_cast<QWidget*>(watched);
        grip && grip->objectName() == QStringLiteral("panelIconGroupGrip")) {
        const QEvent::Type type = event->type();
        if (type == QEvent::MouseButtonPress) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            if (mouse->button() == Qt::LeftButton) {
                stripGripGroup_ =
                    findGroupByName(grip->property("groupObjectName").toString());
                stripPressPending_ = true;
                stripDragging_ = false;
                stripDragButton_ = nullptr;
                stripPressGlobal_ = mouse->globalPosition().toPoint();
            }
        } else if (type == QEvent::MouseMove) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            const QPoint globalPos = mouse->globalPosition().toPoint();
            if (stripPressPending_ && !stripDragging_ && stripGripGroup_
                && (globalPos - stripPressGlobal_).manhattanLength()
                       >= QApplication::startDragDistance()) {
                stripPressPending_ = false;
                stripDragging_ = true;
                beginGroupDrag(stripGripGroup_, globalPos);
            }
            if (stripDragging_ && stripGripGroup_) {
                updateDrag(globalPos);
                return true;
            }
        } else if (type == QEvent::MouseButtonRelease) {
            stripPressPending_ = false;
            if (stripDragging_ && stripGripGroup_) {
                stripDragging_ = false;
                stripGripGroup_ = nullptr;
                auto* mouse = static_cast<QMouseEvent*>(event);
                const QPoint globalPos = mouse->globalPosition().toPoint();
                QMetaObject::invokeMethod(
                    this, [this, globalPos]() { updateDrag(globalPos); commitDrop(); },
                    Qt::QueuedConnection);
                return true;
            }
            stripGripGroup_ = nullptr;
        }
    }
    auto* button = qobject_cast<QToolButton*>(watched);
    if (button && button->objectName().startsWith(QStringLiteral("panelIcon_"))) {
        if (event->type() == QEvent::MouseButtonPress) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            if (mouse->button() == Qt::LeftButton) {
                stripPressPending_ = true;
                stripDragging_ = false;
                stripDragButton_ = button;
                stripPressGlobal_ = mouse->globalPosition().toPoint();
            }
        } else if (event->type() == QEvent::MouseMove) {
            auto* mouse = static_cast<QMouseEvent*>(event);
            const QPoint globalPos = mouse->globalPosition().toPoint();
            if (stripPressPending_ && !stripDragging_
                && (globalPos - stripPressGlobal_).manhattanLength()
                       >= QApplication::startDragDistance()) {
                stripPressPending_ = false;
                stripDragging_ = true;
                const QString name =
                    button->objectName().mid(QStringLiteral("panelIcon_").size());
                beginPanelDrag(groupForPanel(name), name, globalPos);
            }
            if (stripDragging_) {
                updateDrag(globalPos);
                return true;
            }
        } else if (event->type() == QEvent::MouseButtonRelease) {
            stripPressPending_ = false;
            if (stripDragging_) {
                stripDragging_ = false;
                stripDragButton_ = nullptr;
                auto* mouse = static_cast<QMouseEvent*>(event);
                const QPoint globalPos = mouse->globalPosition().toPoint();
                // The commit rebuilds (and deletes) the strip; run it after this
                // event returns instead of inside the button's handler.
                QMetaObject::invokeMethod(
                    this, [this, globalPos]() { updateDrag(globalPos); commitDrop(); },
                    Qt::QueuedConnection);
                return true;
            }
        }
    }
    return QWidget::eventFilter(watched, event);
}

// --- Phase C: drag, drop, the blue line, tear-off and re-dock ---------------

PanelColumn::DropTarget PanelColumn::resolveDrop(const QPoint& globalPos) const
{
    // M43: the workspace-edge new-column band is resolved before the local
    // column grammar, so a drop at an outer edge always means a new column on
    // that side. Compact strips are exempt (they sit on the edge), so the
    // iconic pass runs first.
    if (railMode_) {
        DropTarget iconic;
        if (resolveIconicDrop(globalPos, iconic)) {
            iconic.owner = const_cast<PanelColumn*>(this);
            return iconic;
        }
    }
    if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
        const int side = frame->newColumnSideAt(globalPos);
        if (side >= 0) {
            DropTarget target;
            target.valid = true;
            // The new column lands at the workspace end on `side`; the line is
            // drawn by the column actually on that side, not the drag source.
            const QList<PanelColumn*> columns = frame->panelColumns();
            PanelColumn* edgeColumn = nullptr;
            if (side == 0) {
                for (PanelColumn* column : columns) {
                    if (column && column->isVisible()
                        && frame->sideOf(column) == PanelSide::Left) {
                        edgeColumn = column;
                        break;
                    }
                }
            } else {
                for (int i = columns.size() - 1; i >= 0; --i) {
                    PanelColumn* column = columns.at(i);
                    if (column && column->isVisible()
                        && frame->sideOf(column) == PanelSide::Right) {
                        edgeColumn = column;
                        break;
                    }
                }
            }
            target.owner = edgeColumn ? edgeColumn : const_cast<PanelColumn*>(this);
            target.kind = side == 0 ? DropKind::NewColumnLeft : DropKind::NewColumnRight;
            return target;
        }
        // M44 W5: a drop beside another column allocates the new column adjacent
        // to that column, so a widget column can dock on any side of another.
        int anchorSide = -1;
        if (PanelColumn* anchor = frame->columnEdgeAnchorAt(globalPos, this, &anchorSide)) {
            DropTarget target;
            target.valid = true;
            target.anchorColumn = anchor;
            // M45 W1: the new column lands beside the anchor, so the edge line
            // is drawn in the anchor column, not the drag's source column.
            target.owner = anchor;
            target.kind = anchorSide == 0 ? DropKind::NewColumnLeft : DropKind::NewColumnRight;
            return target;
        }
    }
    DropTarget target = resolveLocalDrop(globalPos);
    target.owner = const_cast<PanelColumn*>(this);
    if (target.outside) {
        // A point inside another column may still land there; that column's own
        // grammar decides the valid non-outside target.
        if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
            if (PanelColumn* other = frame->columnAtGlobal(globalPos)) {
                if (other != this) {
                    DropTarget delegated = other->resolveLocalDrop(globalPos);
                    if (delegated.valid && !delegated.outside
                        && delegated.kind != DropKind::OnStrip) {
                        delegated.owner = other;
                        return delegated;
                    }
                }
            }
        }
    }
    return target;
}

PanelColumn::DropTarget PanelColumn::resolveLocalDrop(const QPoint& globalPos) const
{
    DropTarget target;
    if (railMode_) {
        if (resolveIconicDrop(globalPos, target)) {
            return target;
        }
        target.valid = true;
        target.outside = true;
        target.kind = DropKind::Outside;
        return target;
    }
    if (!scroll_ || !scroll_->isVisible()) {
        target.valid = true;
        target.outside = true;
        target.kind = DropKind::Outside;
        return target;
    }
    QWidget* viewport = scroll_->viewport();
    if (!viewport || !viewport->rect().contains(viewport->mapFromGlobal(globalPos))) {
        target.valid = true;
        target.outside = true;
        target.kind = DropKind::Outside;
        return target;
    }
    for (PanelGroup* group : groups_) {
        if (!group || !group->isVisible()) {
            continue;
        }
        const QRect bar = group->tabBarGlobalRect();
        if (bar.contains(globalPos)) {
            const int index = group->tabInsertionIndexAt(globalPos);
            if (index >= 0) {
                target.valid = true;
                target.onTabBar = true;
                target.group = group;
                target.tabIndex = index;
                target.boundary = boundaryIndexForGlobalY(globalPos);
                target.kind = group == dragGroup_ ? DropKind::Reorder : DropKind::IntoGroup;
                return target;
            }
        }
        const QRect groupRect(group->mapToGlobal(QPoint(0, 0)), group->size());
        if (groupRect.contains(globalPos)) {
            target.valid = true;
            target.group = group;
            const int centerY = groupRect.top() + groupRect.height() / 2;
            const int base = groups_.indexOf(group);
            if (globalPos.y() < centerY) {
                target.kind = DropKind::AboveGroup;
                target.boundary = base;
            } else {
                target.kind = DropKind::BelowGroup;
                target.boundary = base + 1;
            }
            return target;
        }
    }
    // The column background between groups is a boundary insert.
    target.valid = true;
    target.kind = DropKind::AboveGroup;
    target.boundary = boundaryIndexForGlobalY(globalPos);
    return target;
}

bool PanelColumn::resolveIconicDrop(const QPoint& globalPos, DropTarget& target) const
{
    if (!iconStrip_ || !iconStrip_->isVisible() || stripEntries_.isEmpty()) {
        return false;
    }
    if (!iconStrip_->rect().contains(iconStrip_->mapFromGlobal(globalPos))) {
        return false;
    }
    // M44 C3 proximity rule, in order:
    //   1. a direct hit on an icon inserts into that group at that place;
    //   2. an inter-group divider creates a new group at that boundary;
    //   3. within (or very close to) a group's container inserts into it;
    //   4. beyond the top/bottom group creates a new group at that end.

    // 1. A hit on a strip button's interior inserts into the group that owns it.
    for (int i = 0; i < stripEntries_.size(); ++i) {
        const StripEntry& entry = stripEntries_.at(i);
        QToolButton* button = entry.button;
        if (!button || !entry.group) {
            continue;
        }
        const QRect row(button->mapToGlobal(QPoint(0, 0)), button->size());
        if (row.adjusted(0, 2, 0, -2).contains(globalPos)) {
            target.valid = true;
            target.group = entry.group;
            target.stripIndex = i;
            if (dragIsPanel_) {
                target.kind = DropKind::IntoGroup;
                target.onTabBar = true;
                target.tabIndex = entry.group->indexOfPanel(entry.name);
            } else {
                const int gi = groups_.indexOf(entry.group);
                const bool below = globalPos.y() > row.center().y();
                target.kind = below ? DropKind::BelowGroup : DropKind::AboveGroup;
                target.boundary = below ? gi + 1 : gi;
            }
            return true;
        }
    }

    // 2. The divider between two groups is a new-group boundary.
    for (const StripDivider& divider : stripDividers_) {
        if (!divider.widget || !divider.after) {
            continue;
        }
        const QRect r(divider.widget->mapToGlobal(QPoint(0, 0)), divider.widget->size());
        if (r.adjusted(0, -kCompactNearBand - 1, 0, kCompactNearBand + 1).contains(globalPos)) {
            const int after = groups_.indexOf(divider.after);
            target.valid = true;
            target.group = divider.after;
            target.boundary = after >= 0 ? after : groups_.size();
            target.kind = DropKind::AboveGroup;
            target.stripIndex = stripInsertionIndexAt(globalPos);
            return true;
        }
    }

    // M47: a hit on a group's grip creates a new group directly above it.
    for (PanelGroup* group : groups_) {
        if (!group) {
            continue;
        }
        QWidget* box = stripGroupBoxFor(group);
        if (!box) {
            continue;
        }
        QWidget* grip = box->findChild<QWidget*>(QStringLiteral("panelIconGroupGrip"));
        if (!grip) {
            continue;
        }
        const QRect gripRect(grip->mapToGlobal(QPoint(0, 0)), grip->size());
        if (gripRect.contains(globalPos)) {
            target.valid = true;
            target.group = group;
            target.kind = DropKind::AboveGroup;
            target.boundary = groups_.indexOf(group);
            target.stripIndex = stripInsertionIndexAt(globalPos);
            return true;
        }
    }

    // 3. Within (or very close to) a group container.
    for (int gi = 0; gi < groups_.size(); ++gi) {
        PanelGroup* group = groups_.at(gi);
        if (!group) {
            continue;
        }
        QWidget* box = stripGroupBoxFor(group);
        if (!box) {
            continue;
        }
        const QRect r(box->mapToGlobal(QPoint(0, 0)), box->size());
        if (r.adjusted(0, -kCompactNearBand, 0, kCompactNearBand).contains(globalPos)) {
            target.valid = true;
            target.group = group;
            target.stripIndex = stripInsertionIndexAt(globalPos);
            if (dragIsPanel_) {
                target.kind = DropKind::OnStrip;
                target.onStrip = true;
            } else {
                const bool below = globalPos.y() > r.center().y();
                target.kind = below ? DropKind::BelowGroup : DropKind::AboveGroup;
                target.boundary = below ? gi + 1 : gi;
            }
            return true;
        }
    }

    // 4. Between groups, above the top, or below the bottom: a new group there.
    int boundary = groups_.size();
    for (int gi = 0; gi < groups_.size(); ++gi) {
        QWidget* box = stripGroupBoxFor(groups_.at(gi));
        if (!box) {
            continue;
        }
        const QRect r(box->mapToGlobal(QPoint(0, 0)), box->size());
        if (globalPos.y() < r.center().y()) {
            boundary = gi;
            break;
        }
    }
    target.valid = true;
    target.boundary = boundary;
    target.kind = DropKind::AboveGroup;
    target.stripIndex = stripInsertionIndexAt(globalPos);
    return true;
}

int PanelColumn::stripInsertionIndexAt(const QPoint& globalPos) const
{
    if (!iconStrip_ || !iconStrip_->isVisible()) {
        return -1;
    }
    if (!iconStrip_->rect().contains(iconStrip_->mapFromGlobal(globalPos))) {
        return -1;
    }
    for (int i = 0; i < stripEntries_.size(); ++i) {
        QToolButton* button = stripEntries_.at(i).button;
        if (!button) {
            continue;
        }
        const int centerY = button->mapToGlobal(QPoint(0, button->height() / 2)).y();
        if (globalPos.y() < centerY) {
            return i;
        }
    }
    return stripEntries_.size();
}

int PanelColumn::boundaryIndexForGlobalY(const QPoint& globalPos) const
{
    int boundary = groups_.size();
    for (int i = 0; i < groups_.size(); ++i) {
        PanelGroup* group = groups_.at(i);
        if (!group || !group->isVisible()) {
            continue;
        }
        const QPoint top = group->mapToGlobal(QPoint(0, 0));
        const int centerY = top.y() + group->height() / 2;
        if (globalPos.y() < centerY) {
            boundary = i;
            break;
        }
    }
    return boundary;
}

void PanelColumn::showIndicatorFor(const DropTarget& target)
{
    const bool newColumn =
        target.kind == DropKind::NewColumnLeft || target.kind == DropKind::NewColumnRight;
    const bool compactKind =
        railMode_
        && (target.kind == DropKind::IntoGroup || target.kind == DropKind::AboveGroup
            || target.kind == DropKind::BelowGroup)
        && target.stripIndex >= 0;
    if (target.onStrip || compactKind) {
        if (!stripIndicator_ || stripEntries_.isEmpty()) {
            clearIndicator();
            return;
        }
        // M45 C2: a whole-group drag draws at the group container's insertion
        // boundary — above its drag-handle grip — not at an icon button, so the
        // line sits where the drop inserts rather than inside the group.
        if (!dragIsPanel_) {
            QWidget* box = nullptr;
            bool below = false;
            if (target.group) {
                box = stripGroupBoxFor(target.group);
                below = target.kind == DropKind::BelowGroup;
            }
            if (!box) {
                const int boundary = qBound(0, target.boundary, groups_.size());
                if (boundary < groups_.size()) {
                    box = stripGroupBoxFor(groups_.at(boundary));
                } else if (!groups_.isEmpty()) {
                    box = stripGroupBoxFor(groups_.last());
                    below = true;
                }
            }
            if (box) {
                const QPoint origin = box->mapTo(iconStrip_, QPoint(0, 0));
                const int y = below ? origin.y() + box->height() + 1 : origin.y() - 1;
                stripIndicator_->setGeometry(QRect(0, y, iconStrip_->width(), 3));
                stripIndicator_->show();
                stripIndicator_->raise();
                if (indicator_) {
                    indicator_->hide();
                }
                return;
            }
        }
        const int index = qBound(0, target.stripIndex, stripEntries_.size());
        QToolButton* anchor = index < stripEntries_.size() ? stripEntries_.at(index).button
                                                           : stripEntries_.last().button;
        if (!anchor) {
            clearIndicator();
            return;
        }
        const QPoint origin = anchor->mapTo(iconStrip_, QPoint(0, 0));
        const int y = index < stripEntries_.size() ? origin.y() - 1
                                                   : origin.y() + anchor->height() + 1;
        stripIndicator_->setGeometry(QRect(0, y, iconStrip_->width(), 3));
        stripIndicator_->show();
        stripIndicator_->raise();
        if (indicator_) {
            indicator_->hide();
        }
        return;
    }
    if (stripIndicator_) {
        stripIndicator_->hide();
    }
    if (!indicator_ || !scroll_) {
        return;
    }
    QWidget* viewport = scroll_->viewport();
    if (!viewport) {
        return;
    }
    if (newColumn) {
        // A full-height mark at the workspace edge for a new-column candidate.
        const int x = target.kind == DropKind::NewColumnLeft ? 0 : qMax(0, viewport->width() - 3);
        indicator_->setGeometry(QRect(x, 0, 3, viewport->height()));
        indicator_->show();
        indicator_->raise();
        return;
    }
    if (target.onTabBar && target.group) {
        QTabBar* bar = target.group->tabBar();
        if (!bar) {
            clearIndicator();
            return;
        }
        const int x = target.group->tabInsertionX(target.tabIndex);
        const QPoint origin = bar->mapTo(viewport, QPoint(x, 0));
        indicator_->setGeometry(QRect(origin.x() - 1, origin.y(), 3, bar->height()));
    } else {
        const QList<PanelGroup*> visible = visibleGroups();
        if (visible.isEmpty()) {
            clearIndicator();
            return;
        }
        auto topOf = [viewport](PanelGroup* group) {
            return group->mapTo(viewport, QPoint(0, 0)).y();
        };
        int prev = -1;
        int next = -1;
        for (int i = 0; i < groups_.size(); ++i) {
            PanelGroup* group = groups_.at(i);
            if (!group || !group->isVisible()) {
                continue;
            }
            if (i < target.boundary) {
                prev = i;
            } else if (next < 0) {
                next = i;
            }
        }
        int y = 0;
        if (prev >= 0 && next >= 0) {
            const int bottom = topOf(groups_.at(prev)) + groups_.at(prev)->height();
            y = (bottom + topOf(groups_.at(next))) / 2;
        } else if (next >= 0) {
            y = topOf(groups_.at(next)) - 1;
        } else if (prev >= 0) {
            y = topOf(groups_.at(prev)) + groups_.at(prev)->height() + 1;
        }
        y = qBound(0, y, qMax(0, viewport->height() - 3));
        indicator_->setGeometry(QRect(0, y, viewport->width(), 3));
    }
    indicator_->show();
    indicator_->raise();
}

void PanelColumn::clearIndicator()
{
    if (indicator_) {
        indicator_->hide();
    }
    if (stripIndicator_) {
        stripIndicator_->hide();
    }
}

void PanelColumn::beginPanelDrag(PanelGroup* group, const QString& objectName,
                                 const QPoint& globalPos)
{
    if (!group) {
        return;
    }
    dragActive_ = true;
    dragIsPanel_ = true;
    dragGroup_ = group;
    dragSourceGroup_ = group;
    dragPanel_ = objectName;
    dragOriginalIndex_ = groups_.indexOf(group);
    // M43: a panel drag never reuses the source float. Leaving the column
    // builds a fresh one-panel float (createFloat), so clipping one tab out of
    // a float moves only that panel.
    dragFloat_ = nullptr;
    dragGrabOffset_ = globalPos - group->mapToGlobal(QPoint(0, 0));
    dropTarget_ = {};
    if (indicatorOwner_ && indicatorOwner_ != this) {
        indicatorOwner_->clearIndicator();
    }
    indicatorOwner_ = nullptr;
    clearIndicator();
}

void PanelColumn::beginGroupDrag(PanelGroup* group, const QPoint& globalPos)
{
    if (!group) {
        return;
    }
    dragActive_ = true;
    dragIsPanel_ = false;
    dragGroup_ = group;
    dragSourceGroup_ = group;
    dragPanel_.clear();
    dragOriginalIndex_ = groups_.indexOf(group);
    dragFloat_ = floatForGroup(group);
    dragGrabOffset_ = globalPos - group->mapToGlobal(QPoint(0, 0));
    dropTarget_ = {};
    if (indicatorOwner_ && indicatorOwner_ != this) {
        indicatorOwner_->clearIndicator();
    }
    indicatorOwner_ = nullptr;
    clearIndicator();
}

void PanelColumn::updateDrag(const QPoint& globalPos)
{
    if (!dragActive_) {
        return;
    }
    dropTarget_ = resolveDrop(globalPos);
    // M45 W1/W2: render the resolved target in the column that owns it, so a
    // cross-column drop draws its line in the target column.
    PanelColumn* owner = dropTarget_.owner ? dropTarget_.owner : this;
    if (indicatorOwner_ && indicatorOwner_ != owner) {
        indicatorOwner_->clearIndicator();
    }
    indicatorOwner_ = owner;
    if (dropTarget_.outside) {
        owner->clearIndicator();
        indicatorOwner_ = nullptr;
        if (!dragFloat_ && dragGroup_) {
            dragFloat_ = createFloat(dragGroup_, globalPos);
        } else if (dragFloat_) {
            moveFloat(dragFloat_, globalPos - dragGrabOffset_);
        }
        return;
    }
    if (dragFloat_) {
        moveFloat(dragFloat_, globalPos - dragGrabOffset_);
    }
    owner->showIndicatorFor(dropTarget_);
}

bool PanelColumn::commitDrop()
{
    if (!dragActive_) {
        return false;
    }
    const DropTarget target = dropTarget_;
    PanelGroup* source = dragSourceGroup_;
    bool ok = false;
    if (indicatorOwner_ && indicatorOwner_ != this) {
        indicatorOwner_->clearIndicator();
    }
    indicatorOwner_ = nullptr;
    clearIndicator();
    if (target.valid && !target.outside) {
        if (target.kind == DropKind::NewColumnLeft || target.kind == DropKind::NewColumnRight) {
            ok = applyNewColumnDrop(target.kind == DropKind::NewColumnLeft ? PanelSide::Left
                                                                          : PanelSide::Right,
                                    target.anchorColumn);
        } else if (target.onStrip) {
            ok = applyStripDrop(dragGroup_, dragPanel_, target.stripIndex);
        } else if (dragIsPanel_) {
            // A tab drag always moves one panel: from the source tab stack when
            // it never left, or from a one-panel float when it did.
            ok = applyPanelDrop(dragGroup_, dragPanel_, target);
        } else {
            ok = applyGroupDrop(dragGroup_, target);
        }
        if (dragFloat_) {
            destroyFloat(dragFloat_);
        }
        if (ok && target.owner && target.owner != this && target.owner->railMode()) {
            target.owner->buildIconStrip();
        }
    } else {
        ok = dragFloat_ != nullptr;
    }
    // M44 W4: the panel's source group was kept alive for the whole drag; clean
    // it up once the panel has landed elsewhere (it is empty when the drag took
    // its only tab).
    if (source && source != dragGroup_ && source->visibleTitles().isEmpty()) {
        cleanupEmptyGroup(source);
    }
    dragActive_ = false;
    dragIsPanel_ = false;
    dragGroup_ = nullptr;
    dragSourceGroup_ = nullptr;
    dragPanel_.clear();
    dragGrabOffset_ = QPoint();
    dragOriginalIndex_ = -1;
    dragFloat_ = nullptr;
    dropTarget_ = {};
    if (railMode_) {
        buildIconStrip();
    }
    emit stateChanged();
    // M45 W4: a dynamic column whose last group just left is torn down; every
    // path funnels through the frame's one cleanup entry point.
    maybeRemoveSelf();
    return ok;
}

void PanelColumn::cancelDrag()
{
    if (indicatorOwner_ && indicatorOwner_ != this) {
        indicatorOwner_->clearIndicator();
    }
    indicatorOwner_ = nullptr;
    clearIndicator();
    PanelGroup* source = dragSourceGroup_;
    const bool tornOffThisDrag = dragFloat_ && dragOriginalIndex_ >= 0;
    if (tornOffThisDrag && dragGroup_) {
        insertGroupAt(dragGroup_, qBound(0, dragOriginalIndex_, groups_.size()));
        destroyFloat(dragFloat_);
    }
    // M44 W4: release the source group held for the drag's lifetime.
    if (source && source != dragGroup_ && source->visibleTitles().isEmpty()) {
        cleanupEmptyGroup(source);
    }
    dragActive_ = false;
    dragIsPanel_ = false;
    dragGroup_ = nullptr;
    dragSourceGroup_ = nullptr;
    dragPanel_.clear();
    dragGrabOffset_ = QPoint();
    dragOriginalIndex_ = -1;
    dragFloat_ = nullptr;
    dropTarget_ = {};
    if (railMode_) {
        buildIconStrip();
    }
    emit stateChanged();
    maybeRemoveSelf();
}

bool PanelColumn::applyPanelDrop(PanelGroup* source, const QString& name,
                                 const DropTarget& target)
{
    if (!source) {
        return false;
    }
    QString title;
    QIcon iconValue;
    int sourceIndex = -1;
    QWidget* panel = source->takePanel(name, &title, &iconValue, &sourceIndex);
    if (!panel) {
        return false;
    }
    if (target.onTabBar && target.group) {
        int index = qBound(0, target.tabIndex, target.group->titleCountForTest());
        if (target.group == source && sourceIndex >= 0 && sourceIndex < index) {
            --index;
        }
        target.group->insertPanel(panel, title, iconValue, index);
        (target.owner ? target.owner : this)->panelVisible_[name] = true;
    } else {
        // A body/boundary drop may resolve into another column; build the new
        // group in the target column so the commit matches the drawn line.
        PanelColumn* owner = target.owner ? target.owner : this;
        auto* group = new PanelGroup(owner);
        group->addPanel(panel, title, iconValue);
        owner->insertGroupAt(group, qBound(0, target.boundary, owner->groups_.size()));
        owner->panelVisible_[name] = true;
    }
    cleanupEmptyGroup(source);
    return true;
}

bool PanelColumn::applyStripDrop(PanelGroup* source, const QString& name, int stripIndex)
{
    if (!source) {
        return false;
    }
    // Map the flat strip insertion index to a target group and its local index.
    PanelGroup* targetGroup = nullptr;
    int localIndex = 0;
    const int count = stripEntries_.size();
    if (count == 0) {
        targetGroup = source;
        localIndex = qMax(0, source->indexOfPanel(name));
    } else if (stripIndex < count) {
        const int index = qBound(0, stripIndex, count);
        targetGroup = stripEntries_.at(index).group;
        for (int i = 0; i < index; ++i) {
            if (stripEntries_.at(i).group == targetGroup) {
                ++localIndex;
            }
        }
    } else {
        targetGroup = stripEntries_.last().group;
        for (const StripEntry& entry : stripEntries_) {
            if (entry.group == targetGroup) {
                ++localIndex;
            }
        }
    }
    if (!targetGroup) {
        targetGroup = source;
    }
    if (targetGroup == source) {
        QStringList order;
        for (QWidget* panel : source->panels()) {
            if (panel) {
                order << panel->objectName();
            }
        }
        const int from = order.indexOf(name);
        if (from < 0) {
            return false;
        }
        int target = qBound(0, localIndex, order.size() - 1);
        if (from < target) {
            --target;
        }
        order.move(from, target);
        source->setPanelOrder(order);
        return true;
    }
    QString title;
    QIcon iconValue;
    int sourceIndex = -1;
    QWidget* panel = source->takePanel(name, &title, &iconValue, &sourceIndex);
    if (!panel) {
        return false;
    }
    targetGroup->insertPanel(panel, title, iconValue,
                             qBound(0, localIndex, targetGroup->titleCountForTest()));
    cleanupEmptyGroup(source);
    return true;
}

bool PanelColumn::applyNewColumnDrop(PanelSide side, PanelColumn* anchor)
{
    auto* frame = qobject_cast<PicturaMainWindow*>(window());
    if (!frame || !dragGroup_) {
        return false;
    }
    PanelGroup* source = dragGroup_;
    PanelGroup* payload = source;
    QWidget* takenPanel = nullptr;
    QString takenTitle;
    QIcon takenIcon;
    int takenIndex = -1;
    if (dragIsPanel_ && !dragPanel_.isEmpty()) {
        takenPanel = source->takePanel(dragPanel_, &takenTitle, &takenIcon, &takenIndex);
        if (!takenPanel) {
            return false;
        }
        payload = new PanelGroup(this);
        payload->addPanel(takenPanel, takenTitle, takenIcon);
        panelVisible_[dragPanel_] = true;
    }
    // A whole group is detached from this column's stack; a floating group is
    // already parented to its overlay and is simply adopted by the new column.
    // A single-panel drag leaves its source group in place (it keeps the rest).
    int index = -1;
    if (!dragIsPanel_) {
        // The group stops routing its drags here; the new column wires it.
        QObject::disconnect(source, nullptr, this, nullptr);
        index = groups_.indexOf(source);
        if (index >= 0) {
            groups_.removeAt(index);
            source->setParent(nullptr);
        }
    }
    PanelColumn* destination = frame->createPanelColumn(side, anchor);
    if (!destination) {
        if (takenPanel) {
            // No column to adopt the panel: put it back in the stack it was
            // lifted from so a failed drop neither orphans nor loses it. The
            // source group is the original one (createFloat may have swapped
            // dragGroup_ for a transient one-panel float that commitDrop tears
            // down after this returns).
            PanelGroup* home = dragSourceGroup_ ? dragSourceGroup_ : source;
            home->insertPanel(takenPanel, takenTitle, takenIcon,
                              qBound(0, takenIndex, home->titleCountForTest()));
            payload->deleteLater();
        } else if (index >= 0) {
            insertGroupAt(source, qBound(0, dragOriginalIndex_, groups_.size()));
        }
        return false;
    }
    destination->addGroup(payload);
    if (dragIsPanel_) {
        cleanupEmptyGroup(source);
    }
    return true;
}

bool PanelColumn::applyGroupDrop(PanelGroup* group, const DropTarget& target)
{
    if (!group) {
        return false;
    }
    // A whole-group drop can resolve into another column; move the group there
    // so the commit matches the drawn line.
    PanelColumn* owner = target.owner ? target.owner : this;
    int boundary = target.boundary;
    if (target.onTabBar && target.group) {
        const int index = owner->groups_.indexOf(target.group);
        boundary = index >= 0 ? index : owner->groups_.size();
    }
    if (boundary < 0) {
        boundary = owner->groups_.size();
    }
    const int current = groups_.indexOf(group);
    if (current >= 0) {
        if (owner == this) {
            if (boundary > current) {
                --boundary;
            }
            if (boundary == current) {
                return true;
            }
        }
        groups_.removeAt(current);
        group->setParent(nullptr);
    }
    if (owner != this) {
        // The group stops routing its drags through the source column and the
        // destination wires it. `wired_` must be cleared or `wireGroup`'s
        // early-return would leave a later move back dead.
        QObject::disconnect(group, nullptr, this, nullptr);
        wired_.remove(group);
        for (QWidget* panel : group->panels()) {
            if (panel) {
                owner->panelVisible_[panel->objectName()] =
                    group->isPanelVisible(panel->objectName());
            }
        }
    }
    owner->insertGroupAt(group, qBound(0, boundary, owner->groups_.size()));
    return true;
}

void PanelColumn::showEdgeDropIndicator(PanelSide side)
{
    // M45 T3: reuse the M43/M44 new-column `DropTarget` and the single
    // `#2a7fff` indicator; no second indicator system.
    DropTarget target;
    target.valid = true;
    target.anchorColumn = this;
    target.kind = side == PanelSide::Left ? DropKind::NewColumnLeft
                                          : DropKind::NewColumnRight;
    showIndicatorFor(target);
}

void PanelColumn::hideEdgeDropIndicator()
{
    clearIndicator();
}

void PanelColumn::updateColumnDrag(const QPoint& globalPos)
{
    auto* frame = qobject_cast<PicturaMainWindow*>(window());
    if (!frame) {
        return;
    }
    int side = -1;
    PanelColumn* anchor = frame->resolveColumnMoveTarget(globalPos, this, &side);
    if (columnDropAnchor_ && columnDropAnchor_ != anchor) {
        columnDropAnchor_->hideEdgeDropIndicator();
    }
    columnDropAnchor_ = anchor;
    columnDropSide_ = side;
    if (anchor) {
        anchor->showEdgeDropIndicator(side == 0 ? PanelSide::Left : PanelSide::Right);
    }
}

bool PanelColumn::finishColumnDrag(const QPoint& globalPos)
{
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
    return frame->movePanelColumn(this, side, anchor);
}

} // namespace pictura
