#include "frame_includes.h"

namespace pictura {

void PicturaMainWindow::setPanelsHidden(bool hidden)
{
    panelsHidden_ = hidden;
    if (toolsDock_) {
        toolsDock_->setVisible(!hidden);
    }
    for (PanelColumn* column : panelColumns()) {
        column->setVisible(!hidden);
    }
}

void PicturaMainWindow::retargetDock()
{
    PictureView* view = activeView();
    ImageView* canvas = canvasAt(activeDocumentIndex());

    if (layersPanel_) {
        layersPanel_->setView(view);
        layersPanel_->refresh();
    }
    if (historyPanel_) {
        historyPanel_->setView(view);
        historyPanel_->refresh();
    }
    if (navigatorPanel_) {
        navigatorPanel_->setCanvas(canvas);
        navigatorPanel_->refresh();
    }
    if (infoPanel_) {
        infoPanel_->setView(view);
        infoPanel_->refresh();
    }
    if (histogramPanel_) {
        histogramPanel_->setView(view);
        histogramPanel_->refresh();
    }
}

void PicturaMainWindow::refreshPanels()
{
    retargetDock();
}

void PicturaMainWindow::saveSession()
{
    // A session restore rebuilds the columns from the store; re-saving mid
    // rebuild would persist a partial layout.
    if (restoringPanelSession_) {
        return;
    }
    SessionState state = pictura::loadSession();
    state.layout = saveState();
    state.layoutRevision = kLayoutRevision;
    state.brightnessLevel = brightnessLevel_;
    state.gpuCompute = gpuCompute_;
    state.toolsColumns = toolbox_ ? toolbox_->columns() : 1;
    state.useShiftKeyForToolSwitch = useShiftKeyForToolSwitch_;
    if (panelColumn_) {
        const bool iconic = panelColumn_->railMode();
        state.panelRailMode = iconic ? QStringLiteral("iconic") : QStringLiteral("normal");
        // Keep the last normal-mode width; an iconic column has no width of its
        // own, so the loaded value survives unchanged.
        if (!iconic && panelColumn_->width() > 0) {
            state.railWidth = panelColumn_->width();
        }
        state.autoCollapseIconic = panelColumn_->autoCollapseIconic();
        state.autoShowHidden = panelColumn_->autoShowHidden();
        // Legacy flat mirror of the primary column kept for older stores.
        state.panelGroups = panelColumn_->savePanelState();
    }
    // v7: the ordered per-column layout, in central-splitter order, each with
    // its normal-mode width.
    QJsonArray columns;
    int order = 0;
    for (PanelColumn* column : panelColumns()) {
        QJsonObject entry;
        entry.insert(QStringLiteral("side"),
                     sideOf(column) == PanelSide::Left ? QStringLiteral("left")
                                                       : QStringLiteral("right"));
        entry.insert(QStringLiteral("order"), order++);
        entry.insert(QStringLiteral("width"), column->persistedWidth());
        entry.insert(QStringLiteral("groups"), column->savePanelState());
        columns.append(entry);
    }
    state.panelColumns = columns;
    state.schemaVersion = 7;
    state.recent = recent_;
    pictura::saveSession(state);
}

bool PicturaMainWindow::restoreStoredLayout(const QByteArray& layout, int revision)
{
    if (layout.isEmpty() || revision != kLayoutRevision) {
        return false;
    }
    return restoreState(layout);
}

void PicturaMainWindow::applyPanelSession(const SessionState& state)
{
    if (!panelColumn_) {
        return;
    }
    restoringPanelSession_ = true;
    clearDynamicColumns();

    // Order the stored columns by `order` (a stable selection sort: the list is
    // at most a handful of entries).
    QList<QJsonObject> entries;
    for (const QJsonValue& value : state.panelColumns) {
        if (value.isObject()) {
            entries.append(value.toObject());
        }
    }
    for (int i = 0; i < entries.size(); ++i) {
        for (int j = i + 1; j < entries.size(); ++j) {
            if (entries.at(j).value(QStringLiteral("order")).toInt()
                < entries.at(i).value(QStringLiteral("order")).toInt()) {
                entries.swapItemsAt(i, j);
            }
        }
    }

    // The primary column is the first stored right-hand column; its groups stay
    // in `panelColumn_`. Every other stored column adopts its groups from the
    // current column stack. A column whose groups are all unknown is skipped.
    int primary = -1;
    QList<int> leftIndices;
    QList<int> rightIndices;
    for (int i = 0; i < entries.size(); ++i) {
        if (entries.at(i).value(QStringLiteral("side")).toString()
            == QStringLiteral("left")) {
            leftIndices.append(i);
        } else {
            rightIndices.append(i);
        }
    }
    if (!rightIndices.isEmpty()) {
        primary = rightIndices.first();
    }

    // The width to restore for stored column `i`: its own v7 width, else the
    // legacy top-level `railWidth` for the primary column, else the default.
    auto restoredWidth = [&state, primary, &entries](int i) {
        const int stored = entries.at(i).value(QStringLiteral("width")).toInt(0);
        if (stored > 0) {
            return stored;
        }
        if (i == primary && state.railWidth > 0) {
            return state.railWidth;
        }
        return PanelColumn::kDefaultNormalWidth;
    };
    QList<QPair<PanelColumn*, int>> restoredWidths;

    auto buildColumn = [this, &entries, &restoredWidth, &restoredWidths](int entryIndex,
                                                                         PanelSide side) {
        const QJsonObject entry = entries.at(entryIndex);
        const QJsonArray groups = entry.value(QStringLiteral("groups")).toArray();
        QList<PanelGroup*> moved;
        for (const QJsonValue& value : groups) {
            const QString name =
                value.toObject().value(QStringLiteral("name")).toString();
            if (PanelGroup* group = panelColumn_->takeGroup(name)) {
                moved.append(group);
            }
        }
        if (moved.isEmpty()) {
            return;
        }
        PanelColumn* column = createPanelColumn(side);
        if (!column) {
            for (PanelGroup* group : moved) {
                panelColumn_->addGroup(group);
            }
            return;
        }
        for (PanelGroup* group : moved) {
            column->addGroup(group);
        }
        column->restorePanelState(groups);
        column->setRestoredWidth(restoredWidth(entryIndex));
        restoredWidths.append({column, restoredWidth(entryIndex)});
    };

    if (entries.isEmpty()) {
        // No v6 layout (or an explicit empty one): keep the legacy behaviour.
        panelColumn_->restorePanelState(state.panelGroups);
    } else {
        // Left columns are inserted at the splitter head, so adopt them
        // outermost-first to preserve their left-to-right order.
        for (int k = leftIndices.size() - 1; k >= 0; --k) {
            buildColumn(leftIndices.at(k), PanelSide::Left);
        }
        for (int i = 0; i < rightIndices.size(); ++i) {
            if (rightIndices.at(i) != primary) {
                buildColumn(rightIndices.at(i), PanelSide::Right);
            }
        }
        if (primary >= 0) {
            panelColumn_->restorePanelState(
                entries.at(primary).value(QStringLiteral("groups")).toArray());
        }
    }

    const bool iconic = state.panelRailMode == QStringLiteral("iconic");
    for (PanelColumn* column : panelColumns()) {
        column->setAutoCollapseIconic(state.autoCollapseIconic);
        column->setAutoShowHidden(state.autoShowHidden);
        column->setRailMode(iconic);
    }
    // The primary column's width is stored in `panelColumn_`, not a dynamic one,
    // so it is not in `restoredWidths` yet. `setRailMode` above resets an iconic
    // column's remembered width, so seed it after the mode change. A legacy
    // store with no per-column layout keeps its old no-op when `railWidth` is 0.
    const int primaryWidth = primary >= 0 ? restoredWidth(primary) : state.railWidth;
    if (primaryWidth > 0) {
        panelColumn_->setRestoredWidth(primaryWidth);
        restoredWidths.append({panelColumn_, primaryWidth});
    }
    restoringPanelSession_ = false;
    // Before the first layout the splitter has no width, so `setPreferredWidth`
    // only remembers the value; re-apply once the event loop has laid out.
    QTimer::singleShot(0, this, [restoredWidths]() {
        for (const QPair<PanelColumn*, int>& entry : restoredWidths) {
            if (entry.first) {
                entry.first->setRestoredWidth(entry.second);
            }
        }
    });
}

} // namespace pictura
