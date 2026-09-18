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
    column->setDynamic(true);
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
    if (!resolved) {
        if (PanelColumn* column = columnAtGlobal(globalPos)) {
            const QRect r(column->mapToGlobal(QPoint(0, 0)), column->size());
            if (r.isValid() && r.width() > 0) {
                resolved = column;
                resolvedSide = globalPos.x() < r.center().x() ? 0 : 1;
            }
        }
    }
    // A pointer over no column (e.g. the document tabs) keeps the panel in its
    // current state rather than forcing it into the splitter.
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
    if (!resolveToolboxDrop(globalPos, &anchor, &side) || !anchor || !centerSplitter_) {
        return false;
    }
    anchor->hideEdgeDropIndicator();
    toolboxDropAnchor_ = nullptr;
    const int anchorIndex = centerSplitter_->indexOf(anchor);
    if (anchorIndex < 0) {
        return false;
    }
    const int insertAt = side == 0 ? anchorIndex : anchorIndex + 1;
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

void PicturaMainWindow::clearDynamicColumns()
{
    // Move every drop-created column's groups back into the primary column and
    // delete the column, without the `removeColumnIfEmpty` save side effect.
    const QList<PanelColumn*> columns = panelColumns();
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
    }
    reapplyColumnStretch();
}

} // namespace pictura
