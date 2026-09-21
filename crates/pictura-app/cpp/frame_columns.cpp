#include "frame_includes.h"

namespace pictura {

void PicturaMainWindow::wirePanelColumn(PanelColumn* column)
{
    if (!column) {
        return;
    }
    connect(column, &PanelColumn::stateChanged, this, [this]() { saveSession(); });
    connect(column, &PanelColumn::interfaceOptionsRequested, this,
            [this]() { showPreferences(PreferencesDialog::kInterface); });
}

void PicturaMainWindow::reapplyColumnStretch()
{
    if (!centerSplitter_ || !tabs_) {
        return;
    }
    for (int i = 0; i < centerSplitter_->count(); ++i) {
        centerSplitter_->setStretchFactor(i, centerSplitter_->widget(i) == tabs_ ? 1 : 0);
    }
    // The Tools column is fixed-size; a live handle beside it would offer a drag
    // that can only resize the toolbar. Handles between two widget panes stay.
    for (int i = 0; i + 1 < centerSplitter_->count(); ++i) {
        if (QSplitterHandle* handle = centerSplitter_->handle(i)) {
            const bool besideTools = centerSplitter_->widget(i) == toolsColumn_
                                     || centerSplitter_->widget(i + 1) == toolsColumn_;
            handle->setEnabled(!besideTools);
        }
    }
}

QList<PanelColumn*> PicturaMainWindow::panelColumns() const
{
    QList<PanelColumn*> out;
    if (!centerSplitter_) {
        return out;
    }
    for (int i = 0; i < centerSplitter_->count(); ++i) {
        if (auto* column = qobject_cast<PanelColumn*>(centerSplitter_->widget(i))) {
            out << column;
        }
    }
    return out;
}

int PicturaMainWindow::columnCount() const
{
    return panelColumns().size();
}

PanelSide PicturaMainWindow::sideOf(const PanelColumn* column) const
{
    if (!column || !centerSplitter_ || !tabs_) {
        return PanelSide::Right;
    }
    const int tabsIndex = centerSplitter_->indexOf(tabs_);
    const int index = centerSplitter_->indexOf(const_cast<PanelColumn*>(column));
    return index >= 0 && index < tabsIndex ? PanelSide::Left : PanelSide::Right;
}

PanelColumn* PicturaMainWindow::createPanelColumn(PanelSide side, PanelColumn* anchor)
{
    if (!centerSplitter_) {
        return nullptr;
    }
    auto* column = new PanelColumn(this);
    wirePanelColumn(column);
    int insertAt;
    const int anchorIndex = anchor ? centerSplitter_->indexOf(anchor) : -1;
    if (anchorIndex >= 0) {
        // M44 W5: place the new column immediately before/after its anchor.
        insertAt = side == PanelSide::Left ? anchorIndex : anchorIndex + 1;
    } else {
        insertAt = side == PanelSide::Left ? 0 : centerSplitter_->count();
    }
    centerSplitter_->insertWidget(insertAt, column);
    reapplyColumnStretch();
    column->setVisible(!panelsHidden_);
    PanelColumn::refreshSharedFloor(this);
    return column;
}

void PicturaMainWindow::removeColumnIfEmpty(PanelColumn* column)
{
    if (!column) {
        return;
    }
    // D1: the tools column hosts one plain content child and is never empty.
    if (column->isToolsColumn()) {
        return;
    }
    // A whole column torn off into an in-window overlay is not a splitter pane
    // but is still owned by this frame; never delete it out from under its
    // float (the overlay would be left as a ghost).
    if (column->isColumnFloating()) {
        return;
    }
    // M45 W4/M46: a column is empty when no group has visible content. This
    // must not depend on `group->isVisible()`: in rail mode the scroll host is
    // hidden and a popped group is reparented into the flyout, both of which
    // make an ancestor hidden while the group still holds visible panels.
    // M47: emptiness is checked before any float guard, so a torn-off last group
    // does not keep its empty source column alive.
    for (PanelGroup* group : column->groups()) {
        if (group && !group->visibleTitles().isEmpty()) {
            return;
        }
    }
    // M46: the primary column keeps its identity but hides when empty; a hidden
    // splitter child takes no space and `PanelColumn::showPanel` re-shows it.
    // M47: it survives even with a live float, so the floats stay wired.
    if (column == panelColumn_) {
        column->hide();
        saveSession();
        return;
    }
    // M47: a dynamic column may still own a live float; rewire it to the primary
    // before the column is destroyed so the float stays re-dockable.
    column->rehomeFloatsTo(panelColumn_);
    // Rehome any still-live (hidden) groups so their panel widgets survive for
    // a later Window-menu show; then close the now-empty column.
    const QList<PanelGroup*> remaining = column->groups();
    for (PanelGroup* group : remaining) {
        if (!group) {
            continue;
        }
        if (PanelGroup* taken = column->takeGroup(group->objectName())) {
            panelColumn_->adoptGroup(taken);
        }
    }
    column->hide();
    column->setParent(nullptr);
    PanelColumn::refreshSharedFloor(this);
    column->deleteLater();
    reapplyColumnStretch();
    saveSession();
}

PanelColumn* PicturaMainWindow::columnForPanel(const QString& objectName) const
{
    for (PanelColumn* column : panelColumns()) {
        if (column->groupForPanel(objectName)) {
            return column;
        }
    }
    return nullptr;
}

PanelColumn* PicturaMainWindow::columnAtGlobal(const QPoint& globalPos) const
{
    for (PanelColumn* column : panelColumns()) {
        if (column->isVisible()
            && QRect(column->mapToGlobal(QPoint(0, 0)), column->size()).contains(globalPos)) {
            return column;
        }
    }
    return nullptr;
}

int PicturaMainWindow::newColumnSideAt(const QPoint& globalPos) const
{
    QWidget* central = centralWidget();
    if (!central) {
        return -1;
    }
    // ponytail: chosen constant band at the central area's outer edges, not a
    // sourced CS6 metric. A screenshot can retune it.
    constexpr int kNewColumnMargin = 28;
    const int left = central->mapToGlobal(QPoint(0, 0)).x();
    const int right = left + central->width();
    if (globalPos.x() < left + kNewColumnMargin) {
        return 0;
    }
    if (globalPos.x() > right - kNewColumnMargin) {
        return 1;
    }
    return -1;
}

PanelColumn* PicturaMainWindow::columnEdgeAnchorAt(const QPoint& globalPos,
                                                   const PanelColumn* exclude,
                                                   int* side) const
{
    if (side) {
        *side = -1;
    }
    // ponytail: chosen proximity band beside a column, not a sourced CS6 metric.
    // M45 W8: strictly outside the column (inside = 0), so a rightmost/leftmost
    // tab insertion inside a narrow, scrolled column is not mistaken for a
    // new-column anchor.
    constexpr int kEdgeBand = 26;
    constexpr int kEdgeInside = 0;
    for (PanelColumn* column : panelColumns()) {
        if (!column || column == exclude || !column->isVisible()) {
            continue;
        }
        const QRect r(column->mapToGlobal(QPoint(0, 0)), column->size());
        if (!r.isValid() || r.width() <= kEdgeBand + kEdgeInside) {
            continue;
        }
        if (globalPos.y() < r.top() || globalPos.y() > r.bottom()) {
            continue;
        }
        if (globalPos.x() >= r.left() - kEdgeBand && globalPos.x() <= r.left() + kEdgeInside) {
            if (side) {
                *side = 0;
            }
            return column;
        }
        if (globalPos.x() >= r.right() - kEdgeInside && globalPos.x() <= r.right() + kEdgeBand) {
            if (side) {
                *side = 1;
            }
            return column;
        }
    }
    return nullptr;
}

PanelColumn* PicturaMainWindow::resolveColumnMoveTarget(const QPoint& globalPos,
                                                        const PanelColumn* exclude,
                                                        int* side) const
{
    if (side) {
        *side = -1;
    }
    // 1. A band beside an existing column.
    int edgeSide = -1;
    if (PanelColumn* anchor = columnEdgeAnchorAt(globalPos, exclude, &edgeSide)) {
        if (side) {
            *side = edgeSide;
        }
        return anchor;
    }
    // 2. The column under the pointer, split by which half holds it.
    if (PanelColumn* column = columnAtGlobal(globalPos)) {
        if (column != exclude) {
            const QRect r(column->mapToGlobal(QPoint(0, 0)), column->size());
            if (side) {
                *side = r.isValid() && globalPos.x() >= r.center().x() ? 1 : 0;
            }
            return column;
        }
    }
    // 3. A workspace edge: a bare edge target. The anchor stays null and only
    // `side` is set, so the column lands at the splitter head/tail — to the left
    // of even a Tools pane hosted at index 0, or to the right of the last pane.
    const int wsSide = newColumnSideAt(globalPos);
    if (wsSide >= 0) {
        if (side) {
            *side = wsSide;
        }
    }
    return nullptr;
}

bool PicturaMainWindow::movePanelColumn(PanelColumn* column, int side, PanelColumn* anchor)
{
    if (!column || !centerSplitter_) {
        return false;
    }
    // `side` is 0 (left) or 1 (right); -1 means the drag resolved no target, so
    // leave the column where it is rather than snapping it to an edge.
    if (side != 0 && side != 1) {
        return false;
    }
    const int from = centerSplitter_->indexOf(column);
    const int count = centerSplitter_->count();
    int target = -1;
    if (anchor && anchor != column) {
        const int anchorIndex = centerSplitter_->indexOf(anchor);
        if (anchorIndex < 0) {
            return false;
        }
        target = side == 0 ? anchorIndex : anchorIndex + 1;
    } else {
        // A bare workspace edge: land at the extreme end of the splitter.
        target = side == 0 ? 0 : count;
    }
    if (from >= 0) {
        if (from < target) {
            --target;
        }
        target = qBound(0, target, count - 1);
        if (target == from) {
            return false;
        }
    } else {
        // A floated column is not a splitter pane, so nothing is removed first;
        // the target index needs no adjustment and may sit at the end.
        target = qBound(0, target, count);
    }
    column->setParent(nullptr);
    centerSplitter_->insertWidget(target, column);
    // `setParent(nullptr)` hid the widget; re-show it as the splitter pane it is.
    column->setVisible(!panelsHidden_);
    reapplyColumnStretch();
    PanelColumn::refreshSharedFloor(this);
    saveSession();
    return true;
}

void PicturaMainWindow::clearDynamicColumns()
{
    // Move every drop-created column's groups back into the primary column and
    // delete the column, without the `removeColumnIfEmpty` save side effect.
    const QList<PanelColumn*> columns = panelColumns();
    bool removedAny = false;
    for (PanelColumn* column : columns) {
        if (!column || column == panelColumn_ || column->isToolsColumn()) {
            continue;
        }
        const QList<PanelGroup*> groups = column->groups();
        for (PanelGroup* group : groups) {
            if (!group) {
                continue;
            }
            if (PanelGroup* taken = column->takeGroup(group->objectName())) {
                panelColumn_->addGroup(taken);
            }
        }
        // M47: live floats must not be orphaned when the column is deleted.
        column->rehomeFloatsTo(panelColumn_);
        column->hide();
        column->setParent(nullptr);
        column->deleteLater();
        removedAny = true;
    }
    reapplyColumnStretch();
    if (removedAny) {
        PanelColumn::refreshSharedFloor(this);
    }
}

} // namespace pictura
