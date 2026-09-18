#include "panel_group.h"

#include "history_panel.h"
#include "icons.h"
#include "layers_panel.h"

#include <QtCore/QEvent>
#include <QtCore/QHash>
#include <QtGui/QAction>
#include <QtGui/QContextMenuEvent>
#include <QtGui/QMouseEvent>
#include <QtWidgets/QApplication>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

#include <functional>

namespace pictura {

namespace {
constexpr int kIconButtonSize = 22;
constexpr int kIconPixmapSize = 16;
constexpr int kHeaderButtonSize = 18;

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
             dis(QStringLiteral("Delete Hidden Layers")),
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
             dis(QStringLiteral("Link Layers"), true),
             dis(QStringLiteral("Select Linked Layers")),
             dis(QStringLiteral("Merge Down"), true),
             dis(QStringLiteral("Merge Visible")),
             dis(QStringLiteral("Merge Clipping Mask")),
             dis(QStringLiteral("Flatten Image"), true),
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

PanelGroup::PanelGroup(QWidget* parent)
    : QWidget(parent)
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    tabs_ = new QTabWidget(this);
    tabs_->setObjectName(QStringLiteral("panelGroupTabs"));
    tabs_->setTabPosition(QTabWidget::North);
    tabs_->setDocumentMode(true);
    tabs_->tabBar()->installEventFilter(this);
    // M43: name the panel-group tab bar so the scoped theme can make the active
    // tab use the pane `${base}` colour without touching the document tabs. The
    // tab bar elides instead of forcing width, and does not expand, so the
    // corner `▾` button keeps its place at the column minimum width.
    tabs_->tabBar()->setObjectName(QStringLiteral("panelTabBar"));
    tabs_->tabBar()->setElideMode(Qt::ElideRight);
    tabs_->tabBar()->setExpanding(false);
    layout->addWidget(tabs_);

    headerCorner_ = new QWidget(tabs_);
    auto* cornerLayout = new QHBoxLayout(headerCorner_);
    cornerLayout->setContentsMargins(0, 0, 0, 0);
    cornerLayout->setSpacing(0);

    headerButton_ = new QToolButton(headerCorner_);
    headerButton_->setObjectName(QStringLiteral("panelWidgetMenu"));
    headerButton_->setText(QStringLiteral("\u25BE"));
    headerButton_->setAutoRaise(true);
    headerButton_->setPopupMode(QToolButton::InstantPopup);
    headerButton_->setFixedSize(kHeaderButtonSize, kHeaderButtonSize);
    headerButton_->setVisible(false);
    cornerLayout->addWidget(headerButton_);

    floatCloseButton_ = new QToolButton(headerCorner_);
    floatCloseButton_->setObjectName(QStringLiteral("panelFloatClose"));
    floatCloseButton_->setAutoRaise(true);
    floatCloseButton_->setFixedSize(kHeaderButtonSize, kHeaderButtonSize);
    floatCloseButton_->setToolTip(tr("Close"));
    floatCloseButton_->setIcon(icon(QStringLiteral("panel.close")));
    floatCloseButton_->setVisible(false);
    cornerLayout->addWidget(floatCloseButton_);

    tabs_->setCornerWidget(headerCorner_, Qt::TopRightCorner);
    connect(tabs_, &QTabWidget::currentChanged, this, [this]() { updateHeaderMenu(); });

    iconRow_ = new QWidget(this);
    iconRow_->setObjectName(QStringLiteral("panelGroupIconRow"));
    iconRowLayout_ = new QHBoxLayout(iconRow_);
    iconRowLayout_->setContentsMargins(2, 2, 2, 2);
    iconRowLayout_->setSpacing(2);
    iconRowLayout_->addStretch(1);
    iconRow_->setVisible(false);
    layout->addWidget(iconRow_);
}

void PanelGroup::addPanel(QWidget* panel, const QString& title, const QIcon& icon)
{
    if (!panel) {
        return;
    }
    const int index = tabs_->addTab(panel, icon, title);
    tabs_->setTabToolTip(index, title);
    if (tabs_->count() == 1) {
        setObjectName(QStringLiteral("panelGroup_") + panel->objectName());
    }
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
    updateHeaderMenu();
}

QStringList PanelGroup::titles() const
{
    QStringList out;
    for (int i = 0; i < tabs_->count(); ++i) {
        out << tabs_->tabText(i);
    }
    return out;
}

void PanelGroup::setPanelOrder(const QStringList& order)
{
    const int count = tabs_->count();
    QList<QWidget*> panels;
    QStringList titles;
    QList<QIcon> icons;
    for (int i = 0; i < count; ++i) {
        panels << tabs_->widget(i);
        titles << tabs_->tabText(i);
        icons << tabs_->tabIcon(i);
    }
    QList<int> sequence;
    for (const QString& name : order) {
        for (int i = 0; i < panels.size(); ++i) {
            if (panels.at(i) && panels.at(i)->objectName() == name && !sequence.contains(i)) {
                sequence << i;
                break;
            }
        }
    }
    for (int i = 0; i < panels.size(); ++i) {
        if (!sequence.contains(i)) {
            sequence << i;
        }
    }
    // QTabBar::moveTab reorders the tab bar but not QTabWidget's page stack, so
    // re-insert the pages to keep `panels()` and `titles()` in the same order.
    for (int i = tabs_->count() - 1; i >= 0; --i) {
        tabs_->removeTab(i);
    }
    for (int index : sequence) {
        tabs_->addTab(panels.at(index), icons.at(index), titles.at(index));
    }
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
}

bool PanelGroup::isPanelVisible(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->isTabVisible(i);
        }
    }
    return false;
}

QStringList PanelGroup::visibleTitles() const
{
    QStringList out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            out << tabs_->tabText(i);
        }
    }
    return out;
}

QList<QWidget*> PanelGroup::panels() const
{
    QList<QWidget*> out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (QWidget* panel = tabs_->widget(i)) {
            out << panel;
        }
    }
    return out;
}

QList<QWidget*> PanelGroup::visiblePanels() const
{
    QList<QWidget*> out;
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            if (QWidget* panel = tabs_->widget(i)) {
                out << panel;
            }
        }
    }
    return out;
}

bool PanelGroup::containsPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return true;
        }
    }
    return false;
}

bool PanelGroup::removePanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return true;
        }
    }
    return false;
}

bool PanelGroup::setPanelVisible(const QString& objectName, bool visible)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->setTabVisible(i, visible);
            if (visible) {
                tabs_->setCurrentIndex(i);
            }
            setVisible(!visibleTitles().isEmpty());
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return true;
        }
    }
    return false;
}

QString PanelGroup::currentPanelName() const
{
    QWidget* panel = tabs_->currentWidget();
    return panel ? panel->objectName() : QString();
}

bool PanelGroup::setCurrentPanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            tabs_->setCurrentIndex(i);
            return true;
        }
    }
    return false;
}

void PanelGroup::setCurrentToFirstVisible()
{
    for (int i = 0; i < tabs_->count(); ++i) {
        if (tabs_->isTabVisible(i)) {
            tabs_->setCurrentIndex(i);
            return;
        }
    }
}

QString PanelGroup::titleForPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->tabText(i);
        }
    }
    return QString();
}

QIcon PanelGroup::iconForPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return tabs_->tabIcon(i);
        }
    }
    return QIcon();
}

QWidget* PanelGroup::takePanel(const QString& objectName, QString* title, QIcon* icon, int* index)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            if (title) {
                *title = tabs_->tabText(i);
            }
            if (icon) {
                *icon = tabs_->tabIcon(i);
            }
            if (index) {
                *index = i;
            }
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return panel;
        }
    }
    return nullptr;
}

void PanelGroup::insertPanel(QWidget* panel, const QString& title, const QIcon& icon, int index)
{
    if (!panel) {
        return;
    }
    index = qBound(0, index, tabs_->count());
    const int actual = tabs_->insertTab(index, panel, icon, title);
    tabs_->setTabToolTip(actual, title);
    tabs_->setCurrentIndex(actual);
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
}

QTabBar* PanelGroup::tabBar() const
{
    return tabs_ ? tabs_->tabBar() : nullptr;
}

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

int PanelGroup::indexOfPanel(const QString& objectName) const
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            return i;
        }
    }
    return -1;
}

QRect PanelGroup::tabBarGlobalRect() const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return QRect();
    }
    return QRect(bar->mapToGlobal(QPoint(0, 0)), bar->size());
}

int PanelGroup::tabInsertionIndexAt(const QPoint& globalPos) const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return -1;
    }
    const QPoint pos = bar->mapFromGlobal(globalPos);
    if (!bar->rect().contains(pos)) {
        return -1;
    }
    for (int i = 0; i < bar->count(); ++i) {
        if (pos.x() < bar->tabRect(i).center().x()) {
            return i;
        }
    }
    return bar->count();
}

int PanelGroup::tabInsertionX(int index) const
{
    QTabBar* bar = tabBar();
    if (!bar || bar->count() == 0) {
        return 0;
    }
    const int count = bar->count();
    index = qBound(0, index, count);
    // ponytail: hidden QTabBar tabs have empty rects, so scan for the first
    // visible tab instead of trusting tabRect(index).
    for (int i = index; i < count; ++i) {
        const QRect rect = bar->tabRect(i);
        if (rect.isValid() && !rect.isEmpty()) {
            return rect.left();
        }
    }
    for (int i = qMin(index, count) - 1; i >= 0; --i) {
        const QRect rect = bar->tabRect(i);
        if (rect.isValid() && !rect.isEmpty()) {
            return rect.right() + 1;
        }
    }
    return bar->rect().right() + 1;
}

QPoint PanelGroup::tabInsertionGlobalPointForTest(int index) const
{
    QTabBar* bar = tabBar();
    if (!bar) {
        return QPoint();
    }
    return bar->mapToGlobal(QPoint(tabInsertionX(index), bar->height() / 2));
}

void PanelGroup::setMinimized(bool minimized)
{
    if (minimized_ == minimized) {
        return;
    }
    minimized_ = minimized;
    applyMinimize();
}

void PanelGroup::applyMinimize()
{
    if (minimized_) {
        if (collapsedToIcons_) {
            setCollapsedToIcons(false);
        }
        savedMaxHeight_ = tabs_->maximumHeight();
        savedGroupMaxHeight_ = maximumHeight();
        const int barHeight = tabs_->tabBar() ? tabs_->tabBar()->sizeHint().height() : 24;
        savedTabsMinHeight_ = tabs_->minimumHeight();
        savedMinHeight_ = minimumHeight();
        tabs_->setMinimumHeight(barHeight);
        setMinimumHeight(barHeight);
        // M45 W5: clamp the group itself, not only its tab widget, so the
        // splitter gives it just the tab-bar height.
        tabs_->setMaximumHeight(barHeight);
        setMaximumHeight(barHeight);
        setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Fixed);
    } else {
        tabs_->setMinimumHeight(savedTabsMinHeight_);
        setMinimumHeight(savedMinHeight_);
        tabs_->setMaximumHeight(savedMaxHeight_);
        setMaximumHeight(savedGroupMaxHeight_);
        setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Expanding);
    }
    updateGeometry();
}

void PanelGroup::setCollapsedToIcons(bool collapsed)
{
    if (collapsedToIcons_ == collapsed) {
        return;
    }
    collapsedToIcons_ = collapsed;
    if (collapsed) {
        if (minimized_) {
            minimized_ = false;
            applyMinimize();
        }
        rebuildIconRow();
    }
    tabs_->setVisible(!collapsed);
    iconRow_->setVisible(collapsed);
}

void PanelGroup::rebuildIconRow()
{
    while (QLayoutItem* item = iconRowLayout_->takeAt(0)) {
        if (QWidget* widget = item->widget()) {
            delete widget;
        }
        delete item;
    }
    for (int i = 0; i < tabs_->count(); ++i) {
        if (!tabs_->isTabVisible(i)) {
            continue;
        }
        QWidget* panel = tabs_->widget(i);
        if (!panel) {
            continue;
        }
        iconRowLayout_->addWidget(
            makeIconButton(tabs_->tabIcon(i), tabs_->tabText(i), panel->objectName()));
    }
    iconRowLayout_->addStretch(1);
}

QToolButton* PanelGroup::makeIconButton(const QIcon& icon, const QString& title,
                                        const QString& objectName)
{
    auto* button = new QToolButton(iconRow_);
    button->setObjectName(QStringLiteral("panelGroupIcon_") + objectName);
    if (!icon.isNull()) {
        button->setIcon(icon);
        button->setIconSize(QSize(kIconPixmapSize, kIconPixmapSize));
    } else {
        button->setText(title.left(1));
    }
    button->setToolTip(title);
    button->setAutoRaise(true);
    button->setFixedSize(kIconButtonSize, kIconButtonSize);
    connect(button, &QToolButton::clicked, this, [this, objectName, button]() {
        emit panelActivated(objectName, button->mapToGlobal(QPoint(button->width(), 0)));
    });
    return button;
}

QWidget* PanelGroup::detachPanel(const QString& objectName)
{
    for (int i = 0; i < tabs_->count(); ++i) {
        QWidget* panel = tabs_->widget(i);
        if (panel && panel->objectName() == objectName) {
            detached_.panel = panel;
            detached_.title = tabs_->tabText(i);
            detached_.icon = tabs_->tabIcon(i);
            detached_.index = i;
            tabs_->removeTab(i);
            if (collapsedToIcons_) {
                rebuildIconRow();
            }
            return panel;
        }
    }
    return nullptr;
}

bool PanelGroup::attachPanel(const QString& objectName)
{
    if (!detached_.panel || detached_.panel->objectName() != objectName) {
        return false;
    }
    const int index = qBound(0, detached_.index, tabs_->count());
    tabs_->insertTab(index, detached_.panel, detached_.icon, detached_.title);
    tabs_->setTabToolTip(index, detached_.title);
    detached_ = {};
    if (collapsedToIcons_) {
        rebuildIconRow();
    }
    return true;
}

bool PanelGroup::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == tabs_->tabBar()) {
        const QEvent::Type type = event->type();
        if (type == QEvent::ContextMenu) {
            auto* menuEvent = static_cast<QContextMenuEvent*>(event);
            emit tabContextMenuRequested(menuEvent->globalPos());
            return true;
        }
        if (type == QEvent::MouseButtonPress) {
            auto* mouseEvent = static_cast<QMouseEvent*>(event);
            if (mouseEvent->button() == Qt::LeftButton) {
                pressPending_ = true;
                dragging_ = false;
                pressGlobal_ = mouseEvent->globalPosition().toPoint();
                const int index = tabs_->tabBar()->tabAt(mouseEvent->position().toPoint());
                QWidget* panel = index >= 0 ? tabs_->widget(index) : nullptr;
                pressedPanel_ = panel ? panel->objectName() : QString();
            }
        } else if (type == QEvent::MouseMove) {
            auto* mouseEvent = static_cast<QMouseEvent*>(event);
            const QPoint globalPos = mouseEvent->globalPosition().toPoint();
            if (pressPending_ && !dragging_
                && (globalPos - pressGlobal_).manhattanLength()
                       >= QApplication::startDragDistance()) {
                pressPending_ = false;
                dragging_ = true;
                if (pressedPanel_.isEmpty()) {
                    emit groupDragStarted(globalPos);
                } else {
                    emit tabDragStarted(pressedPanel_, globalPos);
                }
            }
            if (dragging_) {
                emit dragMoved(globalPos);
                return true;
            }
        } else if (type == QEvent::MouseButtonRelease) {
            pressPending_ = false;
            if (dragging_) {
                dragging_ = false;
                auto* mouseEvent = static_cast<QMouseEvent*>(event);
                emit dragFinished(mouseEvent->globalPosition().toPoint());
                pressedPanel_.clear();
                return true;
            }
            pressedPanel_.clear();
        }
    }
    return QWidget::eventFilter(watched, event);
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

void PanelGroup::setFloating(bool on)
{
    if (floatCloseButton_) {
        floatCloseButton_->setVisible(on);
    }
    updateHeaderMenu();
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
