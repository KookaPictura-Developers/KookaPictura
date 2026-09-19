#include "panel_group.h"

#include "history_panel.h"
#include "layers_panel.h"

#include <QtCore/QHash>
#include <QtGui/QAction>
#include <QtWidgets/QMenu>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>

#include <functional>

namespace pictura {

namespace {
// Phase D per-panel menu table, transcribed from `docs/dev/m42-panel-menus.md`
// in the researched order with the CS6 grouping separators. `action` non-empty
// means the entry is wired to a per-panel handler (via the panel's
// `performPanelMenuAction`); empty means it ships disabled under the
// "<label> — not implemented yet" tooltip convention.
struct MenuRow {
    QString label;
    QString action;
    bool checkable = false;
    bool submenu = false;
    bool separatorBefore = false;
    int depth = 0;
};

MenuRow dis(const QString& label, bool separatorBefore = false)
{
    return {label, QString(), false, false, separatorBefore, 0};
}
MenuRow chk(const QString& label, bool separatorBefore = false)
{
    return {label, QString(), true, false, separatorBefore, 0};
}
MenuRow imp(const QString& label, const QString& action, bool separatorBefore = false)
{
    return {label, action, false, false, separatorBefore, 0};
}
MenuRow sub(const QString& label, bool separatorBefore = false)
{
    return {label, QString(), false, true, separatorBefore, 0};
}
MenuRow kid(const QString& label, const QString& action = QString())
{
    return {label, action, false, false, false, 1};
}

const QList<MenuRow>& rowsForPanel(const QString& panel)
{
    static const QHash<QString, QList<MenuRow>> table = {
        {QStringLiteral("layersPanel"),
         {
             imp(QStringLiteral("New Layer…"), QStringLiteral("newLayer")),
             dis(QStringLiteral("Copy CSS")),
             imp(QStringLiteral("Duplicate Layer/Group…"), QStringLiteral("duplicate"), true),
             imp(QStringLiteral("Delete Layer/Group"), QStringLiteral("delete")),
             imp(QStringLiteral("Delete Hidden Layers"), QStringLiteral("deleteHidden")),
             imp(QStringLiteral("New Group…"), QStringLiteral("newGroup"), true),
             dis(QStringLiteral("New Group from Layers…")),
             dis(QStringLiteral("Lock Layers…")),
             dis(QStringLiteral("Convert to Smart Object")),
             dis(QStringLiteral("Rasterize Layer")),
             imp(QStringLiteral("Group Layers"), QStringLiteral("group"), true),
             imp(QStringLiteral("Ungroup Layers"), QStringLiteral("ungroup")),
             imp(QStringLiteral("Hide Layers"), QStringLiteral("hide")),
             sub(QStringLiteral("Arrange"), true),
             kid(QStringLiteral("Bring to Front")),
             kid(QStringLiteral("Bring Forward")),
             kid(QStringLiteral("Send Backward")),
             kid(QStringLiteral("Send to Back")),
             kid(QStringLiteral("Reverse")),
             kid(QStringLiteral("Move Layer Up"), QStringLiteral("moveUp")),
             kid(QStringLiteral("Move Layer Down"), QStringLiteral("moveDown")),
             imp(QStringLiteral("Link Layers"), QStringLiteral("link"), true),
             imp(QStringLiteral("Select Linked Layers"), QStringLiteral("selectLinked")),
             imp(QStringLiteral("Merge Down"), QStringLiteral("mergeDown"), true),
             imp(QStringLiteral("Merge Visible"), QStringLiteral("mergeVisible")),
             imp(QStringLiteral("Merge Clipping Mask"), QStringLiteral("mergeClippingMask")),
             imp(QStringLiteral("Flatten Image"), QStringLiteral("flatten"), true),
             dis(QStringLiteral("Blending Options…"), true),
             dis(QStringLiteral("Animation Options")),
             imp(QStringLiteral("Panel Options…"), QStringLiteral("panelOptions"), true),
         }},
        {QStringLiteral("channelsPanel"),
         {
             dis(QStringLiteral("New Channel…")),
             dis(QStringLiteral("Duplicate Channel…")),
             dis(QStringLiteral("Delete Channel")),
             dis(QStringLiteral("New Spot Channel…"), true),
             dis(QStringLiteral("Merge Spot Channel(s)")),
             dis(QStringLiteral("Split Channels"), true),
             dis(QStringLiteral("Merge Channels…")),
             dis(QStringLiteral("Channel Options…"), true),
             dis(QStringLiteral("Panel Options…"), true),
         }},
        {QStringLiteral("pathsPanel"),
         {
             dis(QStringLiteral("New Path…")),
             dis(QStringLiteral("Duplicate Path…")),
             dis(QStringLiteral("Delete Path")),
             dis(QStringLiteral("Save Path…")),
             dis(QStringLiteral("Make Work Path…"), true),
             dis(QStringLiteral("Make Selection…")),
             dis(QStringLiteral("Fill Path…"), true),
             dis(QStringLiteral("Fill Subpath…")),
             dis(QStringLiteral("Stroke Path…")),
             dis(QStringLiteral("Stroke Subpath…")),
             dis(QStringLiteral("Clipping Path…"), true),
             dis(QStringLiteral("Panel Options…"), true),
         }},
        {QStringLiteral("colorPanel"),
         {
             sub(QStringLiteral("Sliders")),
             kid(QStringLiteral("Grayscale")),
             kid(QStringLiteral("RGB")),
             kid(QStringLiteral("HSB")),
             kid(QStringLiteral("CMYK")),
             kid(QStringLiteral("Lab")),
             kid(QStringLiteral("Web Color Sliders")),
             dis(QStringLiteral("RGB Spectrum"), true),
             dis(QStringLiteral("CMYK Spectrum")),
             dis(QStringLiteral("Grayscale Ramp")),
             dis(QStringLiteral("Current Colors")),
             chk(QStringLiteral("Make Ramp Web Safe")),
         }},
        {QStringLiteral("swatchesPanel"),
         {
             dis(QStringLiteral("New Swatch…")),
             sub(QStringLiteral("Display"), true),
             kid(QStringLiteral("Small Thumbnail")),
             kid(QStringLiteral("Large Thumbnail")),
             kid(QStringLiteral("Small List")),
             kid(QStringLiteral("Large List")),
             dis(QStringLiteral("Preset Manager…"), true),
             dis(QStringLiteral("Load Swatches…")),
             dis(QStringLiteral("Save Swatches…")),
             dis(QStringLiteral("Save Swatches For Exchange…")),
             dis(QStringLiteral("Replace Swatches…")),
             dis(QStringLiteral("Reset Swatches")),
             dis(QStringLiteral("(libraries list)"), true),
         }},
        {QStringLiteral("stylesPanel"),
         {
             dis(QStringLiteral("New Style…")),
             sub(QStringLiteral("Display"), true),
             kid(QStringLiteral("Text Only")),
             kid(QStringLiteral("Small Thumbnail")),
             kid(QStringLiteral("Large Thumbnail")),
             kid(QStringLiteral("Small List")),
             kid(QStringLiteral("Large List")),
             dis(QStringLiteral("Preset Manager…"), true),
             dis(QStringLiteral("Load Styles…")),
             dis(QStringLiteral("Save Styles…")),
             dis(QStringLiteral("Replace Styles…")),
             dis(QStringLiteral("Reset Styles")),
             dis(QStringLiteral("(libraries list)"), true),
         }},
        {QStringLiteral("navigatorPanel"),
         {
             dis(QStringLiteral("Panel Options…")),
         }},
        {QStringLiteral("histogramPanel"),
         {
             dis(QStringLiteral("Uncached Refresh")),
             chk(QStringLiteral("Compact View"), true),
             chk(QStringLiteral("Expanded View")),
             chk(QStringLiteral("All Channels View")),
             chk(QStringLiteral("Show Channels In Color"), true),
             chk(QStringLiteral("Show Statistics")),
         }},
        {QStringLiteral("infoPanel"),
         {
             dis(QStringLiteral("Panel Options…")),
             chk(QStringLiteral("Color Samplers"), true),
         }},
        {QStringLiteral("historyPanel"),
         {
             imp(QStringLiteral("Step Forward"), QStringLiteral("stepForward")),
             imp(QStringLiteral("Step Backward"), QStringLiteral("stepBackward")),
             imp(QStringLiteral("New Snapshot…"), QStringLiteral("newSnapshot"), true),
             dis(QStringLiteral("Delete")),
             dis(QStringLiteral("Clear History")),
             dis(QStringLiteral("New Document"), true),
             dis(QStringLiteral("History Options…"), true),
         }},
        {QStringLiteral("actionsPanel"),
         {
             chk(QStringLiteral("Button Mode")),
             dis(QStringLiteral("New Action…"), true),
             dis(QStringLiteral("New Set…")),
             dis(QStringLiteral("Duplicate"), true),
             dis(QStringLiteral("Delete")),
             dis(QStringLiteral("Play")),
             dis(QStringLiteral("Start Recording"), true),
             dis(QStringLiteral("Record Again…")),
             dis(QStringLiteral("Insert Menu Item…"), true),
             dis(QStringLiteral("Insert Stop…")),
             dis(QStringLiteral("Insert Path")),
             dis(QStringLiteral("Action Options…"), true),
             dis(QStringLiteral("Playback Options…")),
             chk(QStringLiteral("Allow Tool Recording"), true),
             dis(QStringLiteral("Clear All Actions"), true),
             dis(QStringLiteral("Reset Actions")),
             dis(QStringLiteral("Load Actions…"), true),
             dis(QStringLiteral("Replace Actions…")),
             dis(QStringLiteral("Save Actions…")),
             dis(QStringLiteral("(installed sets)"), true),
         }},
        {QStringLiteral("adjustmentsPanel"),
         {
             imp(QStringLiteral("Invert"), QStringLiteral("adjustment:invert")),
             imp(QStringLiteral("Posterize"), QStringLiteral("adjustment:posterize")),
             imp(QStringLiteral("Threshold"), QStringLiteral("adjustment:threshold")),
             imp(QStringLiteral("Brightness-Contrast"),
                 QStringLiteral("adjustment:brightness-contrast")),
             imp(QStringLiteral("Hue-Saturation"), QStringLiteral("adjustment:hue-saturation")),
             imp(QStringLiteral("Photo Filter"), QStringLiteral("adjustment:photo-filter")),
             imp(QStringLiteral("Gradient Map"), QStringLiteral("adjustment:gradient-map")),
             imp(QStringLiteral("Color Balance"), QStringLiteral("adjustment:color-balance")),
             dis(QStringLiteral("Add Mask by Default"), true),
             chk(QStringLiteral("Clip to Layer")),
         }},
        {QStringLiteral("propertiesPanel"),
         {
             dis(QStringLiteral("Save Preset…")),
             dis(QStringLiteral("Save Black & White Preset")),
             dis(QStringLiteral("Auto Options…"), true),
             dis(QStringLiteral("Curves Display Options…")),
             chk(QStringLiteral("Show Clipping For Black/White Points"), true),
             chk(QStringLiteral("Auto-Select Parameter")),
             chk(QStringLiteral("Auto-Select Targeted Adjustment Tool")),
             dis(QStringLiteral("Apply Mask"), true),
             dis(QStringLiteral("Delete Mask")),
             dis(QStringLiteral("Disable Mask")),
         }},
    };
    static const QList<MenuRow> empty;
    const auto it = table.find(panel);
    return it == table.end() ? empty : it.value();
}

void populatePanelMenu(QMenu* menu, const QString& panelName,
                       const std::function<void(const QString&)>& dispatch)
{
    QMenu* submenu = nullptr;
    for (const MenuRow& row : rowsForPanel(panelName)) {
        if (row.depth == 0) {
            submenu = nullptr;
            if (row.separatorBefore) {
                menu->addSeparator();
            }
            if (row.submenu) {
                submenu = menu->addMenu(row.label);
                submenu->menuAction()->setEnabled(false);
                submenu->menuAction()->setToolTip(
                    QStringLiteral("%1 — not implemented yet").arg(row.label));
                continue;
            }
            QAction* action = menu->addAction(row.label);
            action->setCheckable(row.checkable);
            if (row.action.isEmpty()) {
                action->setEnabled(false);
                action->setToolTip(
                    QStringLiteral("%1 — not implemented yet").arg(row.label));
            } else if (dispatch) {
                const QString id = row.action;
                QObject::connect(action, &QAction::triggered, menu,
                                 [dispatch, id]() { dispatch(id); });
            }
            continue;
        }
        if (!submenu) {
            continue;
        }
        QAction* action = submenu->addAction(row.label);
        action->setCheckable(row.checkable);
        if (row.action.isEmpty()) {
            action->setEnabled(false);
            action->setToolTip(QStringLiteral("%1 — not implemented yet").arg(row.label));
        } else {
            submenu->menuAction()->setEnabled(true);
            submenu->menuAction()->setToolTip(QString());
            if (dispatch) {
                const QString id = row.action;
                QObject::connect(action, &QAction::triggered, menu,
                                 [dispatch, id]() { dispatch(id); });
            }
        }
    }
}
} // namespace

void PanelGroup::updateHeaderMenu()
{
    if (!headerButton_) {
        return;
    }
    const QString panel = currentPanelName();
    const bool has = !panel.isEmpty() && panelHasMenu(panel);
    headerButton_->setVisible(has);
    if (headerCorner_) {
        // `headerButton_->isVisible()` is false while the corner container is
        // still hidden, so decide from the explicit flags, not effective state.
        headerCorner_->setVisible(has
                                  || (floatCloseButton_ && !floatCloseButton_->isHidden()));
    }
    headerButton_->setMenu(nullptr);
    delete headerMenu_;
    headerMenu_ = nullptr;
    if (!has) {
        headerButton_->setObjectName(QStringLiteral("panelWidgetMenu"));
        headerButton_->setToolTip(QString());
        return;
    }
    headerButton_->setObjectName(QStringLiteral("panelWidgetMenu_") + panel);
    headerButton_->setToolTip(tr("%1 Panel Menu").arg(titleForPanel(panel)));
    headerMenu_ = new QMenu(headerButton_);
    populatePanelMenu(headerMenu_, panel,
                      [this](const QString& id) { runPanelMenuAction(id); });
    headerButton_->setMenu(headerMenu_);
}

void PanelGroup::runPanelMenuAction(const QString& actionId)
{
    QWidget* panel = tabs_->currentWidget();
    if (!panel) {
        return;
    }
    if (auto* layers = qobject_cast<LayersPanel*>(panel)) {
        layers->performPanelMenuAction(actionId);
        return;
    }
    if (auto* history = qobject_cast<HistoryPanel*>(panel)) {
        history->performPanelMenuAction(actionId);
        return;
    }
    // The Adjustments panel is a placeholder with no view handle; its
    // adjustment kinds run through the Layers panel, which owns the view.
    if (actionId.startsWith(QStringLiteral("adjustment:"))) {
        if (QWidget* win = window()) {
            if (auto* layers = win->findChild<LayersPanel*>(QStringLiteral("layersPanel"))) {
                layers->performPanelMenuAction(actionId);
            }
        }
    }
}

bool PanelGroup::panelHasMenu(const QString& panelName)
{
    return !rowsForPanel(panelName).isEmpty();
}

QStringList PanelGroup::menuTextsForPanel(const QString& panelName)
{
    QMenu menu;
    populatePanelMenu(&menu, panelName, nullptr);
    QStringList texts;
    for (QAction* action : menu.actions()) {
        if (!action->isSeparator()) {
            texts.push_back(action->text());
        }
    }
    return texts;
}

} // namespace pictura
