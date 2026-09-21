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
    // Dropping over the Tools dock allocates a column on the dock's side. M47:
    // skip this when the toolbox is a central-splitter pane; the column grammar
    // owns the drop there.
    if (toolsDock_ && toolsDock_->isVisible()
        && (!centerSplitter_ || centerSplitter_->indexOf(toolsDock_) < 0)) {
        const QRect dockRect(toolsDock_->mapToGlobal(QPoint(0, 0)), toolsDock_->size());
        if (dockRect.contains(globalPos)) {
            switch (toolsArea_) {
            case Qt::RightDockWidgetArea:
            case Qt::BottomDockWidgetArea:
                return 1;
            default:
                return 0;
            }
        }
    }
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
    if (from < 0) {
        return false;
    }
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
    if (from < target) {
        --target;
    }
    target = qBound(0, target, count - 1);
    if (target == from) {
        return false;
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

bool PicturaMainWindow::resolveToolboxDrop(const QPoint& globalPos, PanelColumn** anchor,
                                           int* side)
{
    if (anchor) {
        *anchor = nullptr;
    }
    if (side) {
        *side = -1;
    }
    PanelColumn* resolved = nullptr;
    int resolvedSide = -1;
    if (toolbox_ && centerSplitter_) {
        resolved = columnEdgeAnchorAt(globalPos, nullptr, &resolvedSide);
    }
    // M47 D9: the Tools pane itself is a valid target; anchor its nearest
    // neighbouring column so the indicator and the landing slot agree.
    if (!resolved && toolbox_ && centerSplitter_) {
        const int toolsIndex = centerSplitter_->indexOf(toolbox_);
        const QRect toolsRect(toolbox_->mapToGlobal(QPoint(0, 0)), toolbox_->size());
        if (toolsIndex >= 0 && toolsRect.contains(globalPos)) {
            PanelColumn* left = nullptr;
            PanelColumn* right = nullptr;
            for (int i = 0; i < centerSplitter_->count(); ++i) {
                auto* column = qobject_cast<PanelColumn*>(centerSplitter_->widget(i));
                if (!column || !column->isVisible()) {
                    continue;
                }
                if (i < toolsIndex) {
                    left = column;
                } else if (i > toolsIndex && !right) {
                    right = column;
                }
            }
            if (left) {
                resolved = left;
                resolvedSide = 1;
            } else if (right) {
                resolved = right;
                resolvedSide = 0;
            }
        }
    }
    // The workspace outer band is a DOCK target, not a pane: a release there
    // docks the panel to the left/right dock area. Show the dock preview as a
    // visual-only edge line on the outermost visible column (mirroring
    // PanelColumn::updateColumnDrag) and decline the pane resolve so the release
    // takes the dock path. Checked before the column passes so an edge-adjacent
    // column cannot swallow the band.
    if (!resolved && newColumnSideAt(globalPos) >= 0) {
        const int wsSide = newColumnSideAt(globalPos);
        PanelColumn* preview = nullptr;
        const QList<PanelColumn*> columns = panelColumns();
        if (wsSide == 0) {
            for (PanelColumn* column : columns) {
                if (column && column->isVisible()) {
                    preview = column;
                    break;
                }
            }
        } else {
            for (int i = columns.size() - 1; i >= 0; --i) {
                PanelColumn* column = columns.at(i);
                if (column && column->isVisible()) {
                    preview = column;
                    break;
                }
            }
        }
        if (toolboxDropAnchor_ && toolboxDropAnchor_ != preview) {
            toolboxDropAnchor_->hideEdgeDropIndicator();
        }
        toolboxDropAnchor_ = preview;
        if (preview) {
            preview->showEdgeDropIndicator(wsSide == 0 ? PanelSide::Left : PanelSide::Right);
        }
        return false;
    }
    if (!resolved) {
        if (PanelColumn* column = columnAtGlobal(globalPos)) {
            const QRect r(column->mapToGlobal(QPoint(0, 0)), column->size());
            if (r.isValid() && r.width() > 0) {
                resolved = column;
                resolvedSide = globalPos.x() < r.center().x() ? 0 : 1;
            }
        }
    }
    // A release with no column under the pointer and outside the outer band
    // declines here; the caller's fall-through then floats the panel at the
    // cursor, so a drop over the empty workspace is no longer snapped to the
    // nearest column.
    // Drop the previous boundary's line when the pointer moves off it.
    if (toolboxDropAnchor_ && toolboxDropAnchor_ != resolved) {
        toolboxDropAnchor_->hideEdgeDropIndicator();
        toolboxDropAnchor_ = nullptr;
    }
    if (!resolved) {
        return false;
    }
    resolved->showEdgeDropIndicator(resolvedSide == 0 ? PanelSide::Left : PanelSide::Right);
    toolboxDropAnchor_ = resolved;
    if (anchor) {
        *anchor = resolved;
    }
    if (side) {
        *side = resolvedSide;
    }
    return true;
}

bool PicturaMainWindow::commitToolboxDrop(const QPoint& globalPos)
{
    int side = -1;
    PanelColumn* anchor = nullptr;
    const bool resolved = resolveToolboxDrop(globalPos, &anchor, &side);
    if (anchor) {
        anchor->hideEdgeDropIndicator();
    }
    // The outer-band preview is a visual-only anchor (the resolve declines, so
    // anchor/side stay null/-1); always clear it so a declined commit leaves no
    // line on screen.
    if (toolboxDropAnchor_) {
        toolboxDropAnchor_->hideEdgeDropIndicator();
        toolboxDropAnchor_ = nullptr;
    }
    if (!resolved || !centerSplitter_ || side < 0) {
        return false;
    }
    // A bare workspace edge (no visible column) hosts the panel at the splitter
    // head/tail; otherwise it lands immediately before/after its anchor column.
    int insertAt = side == 0 ? 0 : centerSplitter_->count();
    if (anchor) {
        const int anchorIndex = centerSplitter_->indexOf(anchor);
        if (anchorIndex < 0) {
            return false;
        }
        insertAt = side == 0 ? anchorIndex : anchorIndex + 1;
    }
    // M45 T3 honest limit: a QDockWidget cannot sit *between* two columns, so the
    // panel is re-hosted as a fixed-width pane at that central-splitter boundary.
    removeDockWidget(toolbox_);
    toolbox_->hide();
    centerSplitter_->insertWidget(insertAt, toolbox_);
    toolbox_->setSplitterPane(true);
    reapplyColumnStretch();
    toolbox_->show();
    return centerSplitter_->indexOf(toolbox_) == insertAt;
}

bool PicturaMainWindow::dockToolbox(int side)
{
    if (!toolbox_) {
        return false;
    }
    // A splitter pane is not in the dock layout; detach it before re-docking.
    if (centerSplitter_ && centerSplitter_->indexOf(toolbox_) >= 0) {
        toolbox_->setParent(nullptr);
        toolbox_->setSplitterPane(false);
    }
    const Qt::DockWidgetArea area =
        side == 0 ? Qt::LeftDockWidgetArea : Qt::RightDockWidgetArea;
    toolbox_->setFloating(false);
    addDockWidget(area, toolbox_);
    toolsArea_ = area;
    toolbox_->show();
    ensureToolsNotTabified();
    return true;
}

bool PicturaMainWindow::floatToolboxAt(const QPoint& globalPos)
{
    if (!toolbox_) {
        return false;
    }
    // A splitter pane (and a detached one) is not a managed dock; re-attach it to
    // the frame so the float stays wired to the Window menu and the session.
    bool detached = false;
    if (centerSplitter_ && centerSplitter_->indexOf(toolbox_) >= 0) {
        toolbox_->setParent(nullptr);
        toolbox_->setSplitterPane(false);
        detached = true;
    }
    if (detached || !toolbox_->isFloating()) {
        addDockWidget(toolsArea_, toolbox_);
        toolbox_->setFloating(true);
    }
    toolbox_->move(globalPos - toolbox_->titleDragOffset());
    toolbox_->show();
    toolbox_->raise();
    return true;
}

void PicturaMainWindow::clearDynamicColumns()
{
    // Move every drop-created column's groups back into the primary column and
    // delete the column, without the `removeColumnIfEmpty` save side effect.
    const QList<PanelColumn*> columns = panelColumns();
    bool removedAny = false;
    for (PanelColumn* column : columns) {
        if (!column || column == panelColumn_) {
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
