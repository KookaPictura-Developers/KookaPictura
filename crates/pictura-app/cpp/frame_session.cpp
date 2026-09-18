#include "frame.h"

#include "commands.h"
#include "dialogs.h"
#include "icons.h"
#include "image_view.h"
#include "new_document_dialog.h"
#include "options_bar.h"
#include "panels/color_panel.h"
#include "panels/histogram_panel.h"
#include "panels/history_panel.h"
#include "panels/info_panel.h"
#include "panels/layers_panel.h"
#include "panels/navigator_panel.h"
#include "panels/panel_column.h"
#include "panels/panel_group.h"
#include "panels/placeholder_panel.h"
#include "panels/swatches_panel.h"
#include "preferences_dialog.h"
#include "session.h"
#include "theme.h"
#include "toolbox.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QFileInfo>
#include <QtCore/QCoreApplication>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonObject>
#include <QtCore/QRect>
#include <QtCore/QSignalBlocker>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QActionGroup>
#include <QtGui/QCloseEvent>
#include <QtGui/QColor>
#include <QtGui/QKeyEvent>
#include <QtGui/QKeySequence>
#include <QtGui/QShortcut>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDockWidget>
#include <QtWidgets/QFileDialog>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QMessageBox>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QStatusBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>
#include <QtWidgets/QWidget>

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
    // v6: the ordered per-column layout, in central-splitter order.
    QJsonArray columns;
    int order = 0;
    for (PanelColumn* column : panelColumns()) {
        QJsonObject entry;
        entry.insert(QStringLiteral("side"),
                     sideOf(column) == PanelSide::Left ? QStringLiteral("left")
                                                       : QStringLiteral("right"));
        entry.insert(QStringLiteral("order"), order++);
        entry.insert(QStringLiteral("groups"), column->savePanelState());
        columns.append(entry);
    }
    state.panelColumns = columns;
    state.schemaVersion = 6;
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

    auto buildColumn = [this](const QJsonObject& entry, PanelSide side) {
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
    };

    if (entries.isEmpty()) {
        // No v6 layout (or an explicit empty one): keep the legacy behaviour.
        panelColumn_->restorePanelState(state.panelGroups);
    } else {
        // Left columns are inserted at the splitter head, so adopt them
        // outermost-first to preserve their left-to-right order.
        for (int k = leftIndices.size() - 1; k >= 0; --k) {
            buildColumn(entries.at(leftIndices.at(k)), PanelSide::Left);
        }
        for (int i = 0; i < rightIndices.size(); ++i) {
            if (rightIndices.at(i) != primary) {
                buildColumn(entries.at(rightIndices.at(i)), PanelSide::Right);
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
    if (!iconic) {
        panelColumn_->setPreferredWidth(state.railWidth);
    }
    restoringPanelSession_ = false;
}

} // namespace pictura
