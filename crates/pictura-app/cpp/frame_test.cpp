#include "frame_includes.h"

namespace pictura {

QString PicturaMainWindow::panelColumnSideForTest(int index) const
{    const QList<PanelColumn*> columns = panelColumns();    if (index < 0 || index >= columns.size()) {
        return QString();
    }
    return sideOf(columns.at(index)) == PanelSide::Left ? QStringLiteral("left")
                                                        : QStringLiteral("right");
}

bool PicturaMainWindow::newColumnDropForTest(const QString& panelName, const QString& side)
{
    PanelColumn* source = columnForPanel(panelName);
    if (!source || !centralWidget()) {
        return false;
    }
    const int before = columnCount();
    const QRect central(centralWidget()->mapToGlobal(QPoint(0, 0)), centralWidget()->size());
    QPoint point;
    int expected = -1;
    if (side == QStringLiteral("left")) {
        point = QPoint(central.left() + 2, central.center().y());
        expected = 0;
    } else if (side == QStringLiteral("right")) {
        point = QPoint(central.right() - 2, central.center().y());
        expected = 1;
    } else if (side == QStringLiteral("tools")) {
        if (!toolsDock_ || !toolsDock_->isVisible()) {
            return false;
        }
        point = toolsDock_->mapToGlobal(toolsDock_->rect().center());
        expected = toolsArea_ == Qt::LeftDockWidgetArea ? 0 : 1;
    } else {
        return false;
    }
    if (!source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    QCoreApplication::processEvents();
    PanelColumn* destination = columnForPanel(panelName);
    if (!dropped || !destination || destination == source) {
        return false;
    }
    if (columnCount() != before + 1) {
        return false;
    }
    const PanelSide want = expected == 0 ? PanelSide::Left : PanelSide::Right;
    return sideOf(destination) == want;
}

bool PicturaMainWindow::newColumnBesideForTest(const QString& panelName,
                                               const QString& anchorPanel)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* anchor = columnForPanel(anchorPanel);
    if (!source || !anchor || source == anchor) {
        return false;
    }
    const int before = columnCount();
    const QRect r(anchor->mapToGlobal(QPoint(0, 0)), anchor->size());
    // Just outside the anchor's left edge: the resolver treats this as a new
    // column anchored immediately before `anchor`.
    const QPoint point(r.left() - 8, r.center().y());
    if (!source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    QCoreApplication::processEvents();
    PanelColumn* destination = columnForPanel(panelName);
    if (!dropped || !destination || destination == source || columnCount() != before + 1) {
        return false;
    }
    const int destIndex = centerSplitter_->indexOf(destination);
    const int anchorIndex = centerSplitter_->indexOf(anchor);
    return destIndex >= 0 && anchorIndex >= 0 && qAbs(destIndex - anchorIndex) == 1;
}

bool PicturaMainWindow::toolboxBesideColumnForTest(const QString& side, bool dynamicAnchor)
{
    if (!toolbox_ || !panelColumn_ || !centerSplitter_) {
        return false;
    }
    PanelColumn* anchor = panelColumn_;
    if (dynamicAnchor) {
        anchor = createPanelColumn(PanelSide::Right, panelColumn_);
        if (!anchor) {
            return false;
        }
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
    }
    // The real path is a floating-Tools drag; float it so the resolve/commit
    // below is the same path the title-bar gesture drives. Re-dock first when a
    // previous beside-column placement left it in the splitter.
    if (centerSplitter_->indexOf(toolbox_) >= 0) {
        addDockWidget(Qt::LeftDockWidgetArea, toolbox_);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
    }
    if (!toolbox_->isFloating()) {
        toolbox_->setFloating(true);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
    }
    const QRect r(anchor->mapToGlobal(QPoint(0, 0)), anchor->size());
    if (!r.isValid() || r.width() <= 0) {
        return false;
    }
    const bool left = side == QStringLiteral("left");
    const QPoint point = left ? QPoint(r.left() - 8, r.center().y())
                              : QPoint(r.right() + 8, r.center().y());
    PanelColumn* resolved = nullptr;
    int resolvedSide = -1;
    if (!resolveToolboxDrop(point, &resolved, &resolvedSide) || resolved != anchor) {
        return false;
    }
    const bool indicator = anchor->dropIndicatorVisibleForTest();
    const int anchorIndex = centerSplitter_->indexOf(anchor);
    // `commitToolboxDrop` inserts before the anchor for a left drop and after it
    // for a right drop, matching `createPanelColumn`.
    const int expected = left ? anchorIndex : anchorIndex + 1;
    const bool placed = commitToolboxDrop(point) && centerSplitter_->indexOf(toolbox_) == expected;
    return indicator && placed;
}

bool PicturaMainWindow::dropIntoGroupForTest(const QString& panelName, const QString& targetPanel,
                                             int index)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* destination = columnForPanel(targetPanel);
    if (!source || !destination) {
        return false;
    }
    PanelGroup* targetGroup = destination->groupForPanel(targetPanel);
    if (!targetGroup) {
        return false;
    }
    QPoint point;
    if (destination->railMode()) {
        point = destination->stripEntryPointForTest(targetPanel, 0);
    } else {
        destination->ensureGroupVisibleForTest(targetPanel);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
        const int at = index < 0 ? targetGroup->titleCountForTest()
                                 : qMin(index, targetGroup->titleCountForTest());
        point = targetGroup->tabInsertionGlobalPointForTest(at);
    }
    if (point.isNull() || !source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    for (int i = 0; i < 4; ++i) {
        QCoreApplication::processEvents();
    }
    PanelColumn* owner = columnForPanel(panelName);
    return dropped && owner && owner->groupForPanel(panelName) == targetGroup;
}

bool PicturaMainWindow::dropBoundaryForTest(const QString& panelName, const QString& targetPanel,
                                            bool above)
{
    PanelColumn* source = columnForPanel(panelName);
    PanelColumn* destination = columnForPanel(targetPanel);
    if (!source || !destination) {
        return false;
    }
    PanelGroup* targetGroup = destination->groupForPanel(targetPanel);
    if (!targetGroup) {
        return false;
    }
    QPoint point;
    if (destination->railMode()) {
        // M44 C3: compact boundaries are the inter-group dividers / strip ends,
        // not a button edge (a button edge now inserts into its group).
        const int gi = destination->compactStripGroupIndexForTest(targetPanel);
        if (gi < 0) {
            return false;
        }
        point = destination->compactStripBoundaryPointForTest(above ? gi : gi + 1);
    } else {
        destination->ensureGroupVisibleForTest(targetPanel);
        for (int i = 0; i < 4; ++i) {
            QCoreApplication::processEvents();
        }
        const QRect groupRect(targetGroup->mapToGlobal(QPoint(0, 0)), targetGroup->size());
        // Stay clear of the tab bar so the drop resolves the group's top/bottom
        // half, not a tab insertion.
        const int barBottom = targetGroup->tabBarGlobalRect().bottom();
        const int top = qMax(groupRect.top(), barBottom + 1);
        const int height = qMax(1, groupRect.bottom() - top);
        point = QPoint(groupRect.center().x(),
                       above ? top + height / 4 : top + (height * 3) / 4);
    }
    PanelGroup* original = source->groupForPanel(panelName);
    if (point.isNull() || !source->beginTabDragForTest(panelName)) {
        return false;
    }
    source->dragToForTest(point);
    const bool dropped = source->dropForTest(point);
    for (int i = 0; i < 4; ++i) {
        QCoreApplication::processEvents();
    }
    // A panel boundary drop leaves the panel alone in a fresh group.
    PanelColumn* owner = columnForPanel(panelName);
    PanelGroup* landed = owner ? owner->groupForPanel(panelName) : nullptr;
    return dropped && landed && landed != original && landed->titleCountForTest() == 1;
}

} // namespace pictura
