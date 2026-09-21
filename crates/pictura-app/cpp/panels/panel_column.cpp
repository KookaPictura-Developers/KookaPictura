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
#include <QtGui/QResizeEvent>
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

namespace {

// M45 W8: one shared minimum width for every normal-mode widget column, so all
// columns match and none can be squeezed away. ponytail: chosen, not a sourced
// CS6 metric; tune here. The iconic strip keeps its own narrow minimum.
constexpr int kPanelMinWidth = 180;
constexpr int kPanelMaxWidth = 400;

} // namespace

int gSharedFloor = kPanelMinWidth;

PanelColumn::PanelColumn(QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("panelColumnContainer"));
    setAttribute(Qt::WA_StyledBackground, true);
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    header_ = new QWidget(this);
    header_->setObjectName(QStringLiteral("panelColumnHeader"));
    // M47: the whole column is draggable from its top header. The filter goes on
    // the header only, never the toggle child, so a toggle click still toggles.
    header_->installEventFilter(this);
    header_->setCursor(Qt::SizeAllCursor);
    header_->setToolTip(tr("Drag to move this panel column"));
    auto* headerLayout = new QHBoxLayout(header_);
    headerLayout->setContentsMargins(2, 2, 2, 2);
    headerLayout->setSpacing(2);
    headerLayout->addStretch(1);
    columnToggle_ = new QToolButton(header_);
    columnToggle_->setObjectName(QStringLiteral("panelColumnToggle"));
    columnToggle_->setAutoRaise(true);
    columnToggle_->setFixedSize(20, 20);
    connect(columnToggle_, &QToolButton::clicked, this, [this]() {
        if (toolsContent_) {
            if (toolsToggleAction_) {
                toolsToggleAction_();
            }
            updateColumnToggle();
        } else {
            setRailMode(!railMode_);
        }
    });
    headerLayout->addWidget(columnToggle_);
    layout->addWidget(header_);
    updateColumnToggle();

    scroll_ = new QScrollArea(this);
    scroll_->setObjectName(QStringLiteral("panelColumnScroll"));
    scroll_->setWidgetResizable(true);
    scroll_->setFrameShape(QFrame::NoFrame);
    // M47: the column never scrolls horizontally; the shared width floor keeps
    // content within the column instead of clipping.
    scroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    splitter_ = new QSplitter(Qt::Vertical, scroll_);
    splitter_->setObjectName(QStringLiteral("panelColumnSplitter"));
    splitter_->setHandleWidth(Theme::kGroupDividerWidth);
    splitter_->setChildrenCollapsible(false);
    scroll_->setWidget(splitter_);
    layout->addWidget(scroll_, 1);

    iconStrip_ = new QWidget(this);
    iconStrip_->setObjectName(QStringLiteral("panelColumnIconStrip"));
    iconStrip_->setAttribute(Qt::WA_StyledBackground, true);
    iconStripLayout_ = new QVBoxLayout(iconStrip_);
    iconStripLayout_->setContentsMargins(2, 2, 2, 2);
    iconStripLayout_->setSpacing(2);
    iconStrip_->installEventFilter(this);
    iconStrip_->setVisible(false);
    layout->addWidget(iconStrip_, 1);

    // The thick blue insertion line, drawn by whatever is under the drag. It
    // lives in the scroll viewport so group/tab coordinates map straight in.
    indicator_ = new QWidget(scroll_->viewport());
    indicator_->setObjectName(QStringLiteral("panelDropIndicator"));
    indicator_->setAttribute(Qt::WA_TransparentForMouseEvents);
    indicator_->setStyleSheet(QStringLiteral("background-color:#2a7fff;"));
    indicator_->hide();

    // The strip's own insertion line: the scroll viewport is hidden in iconic
    // mode, so a strip drop needs a sibling inside the icon strip.
    stripIndicator_ = new QWidget(iconStrip_);
    stripIndicator_->setObjectName(QStringLiteral("panelStripDropIndicator"));
    stripIndicator_->setAttribute(Qt::WA_TransparentForMouseEvents);
    stripIndicator_->setStyleSheet(QStringLiteral("background-color:#2a7fff;"));
    stripIndicator_->hide();

    // Phase 4: the blue region outline drawn around a whole target group for a
    // group-on-group tabify. A transparent centre keeps the group visible.
    outlineIndicator_ = new QWidget(scroll_->viewport());
    outlineIndicator_->setObjectName(QStringLiteral("panelOutlineIndicator"));
    outlineIndicator_->setAttribute(Qt::WA_TransparentForMouseEvents);
    outlineIndicator_->setAttribute(Qt::WA_StyledBackground);
    outlineIndicator_->setStyleSheet(QStringLiteral(
        "background:transparent; border:2px solid #2a7fff; border-radius:6px;"));
    outlineIndicator_->hide();

    setMinimumHeight(0);
    updateMinimumWidth();
}

PanelSide PanelColumn::side() const
{
    auto* splitter = qobject_cast<QSplitter*>(parentWidget());
    if (!splitter) {
        return PanelSide::Right;
    }
    const int self = splitter->indexOf(const_cast<PanelColumn*>(this));
    for (int i = 0; i < splitter->count(); ++i) {
        QWidget* pane = splitter->widget(i);
        if (pane && pane->objectName() == QStringLiteral("documentTabs")) {
            return self < i ? PanelSide::Left : PanelSide::Right;
        }
    }
    // The document tabs are the first pane when no left column exists.
    return self == 0 ? PanelSide::Left : PanelSide::Right;
}

void PanelColumn::addGroup(PanelGroup* group)
{
    if (!group) {
        return;
    }
    // A group adopted from another column may still be wired to it; drop any
    // stale wiring to this column so `insertGroupAt` re-wires it cleanly.
    QObject::disconnect(group, nullptr, this, nullptr);
    wired_.remove(group);
    insertGroupAt(group, groups_.size());
    for (QWidget* panel : group->panels()) {
        if (panel) {
            panelVisible_[panel->objectName()] = true;
        }
    }
    if (railMode_) {
        buildIconStrip();
    }
    updateMinimumWidth();
}

void PanelColumn::adoptGroup(PanelGroup* group)
{
    if (!group) {
        return;
    }
    // M45 W4: take a group from another column. Its own tab visibility carries
    // the state (a group closed via `closeGroup` has its tabs hidden), so
    // `insertGroupAt` restores exactly the right visibility.
    QObject::disconnect(group, nullptr, this, nullptr);
    wired_.remove(group);
    insertGroupAt(group, groups_.size());
    for (QWidget* panel : group->panels()) {
        if (panel) {
            panelVisible_[panel->objectName()] = group->isPanelVisible(panel->objectName());
        }
    }
    if (railMode_) {
        buildIconStrip();
    }
    updateMinimumWidth();
}

void PanelColumn::wireGroup(PanelGroup* group)
{
    if (!group || wired_.contains(group)) {
        return;
    }
    wired_.insert(group);
    connect(group, &PanelGroup::panelActivated, this,
            [this](const QString& objectName, const QPoint& globalPos) {
                openIconFlyout(objectName, globalPos);
            });
    connect(group, &PanelGroup::tabContextMenuRequested, this,
            [this, group](const QPoint& globalPos) { showTabMenu(group, globalPos); });
    connect(group, &PanelGroup::tabDragStarted, this,
            [this, group](const QString& objectName, const QPoint& globalPos) {
                beginPanelDrag(group, objectName, globalPos);
            });
    connect(group, &PanelGroup::groupDragStarted, this,
            [this, group](const QPoint& globalPos) { beginGroupDrag(group, globalPos); });
    connect(group, &PanelGroup::dragMoved, this,
            [this](const QPoint& globalPos) { updateDrag(globalPos); });
    connect(group, &PanelGroup::dragFinished, this, [this](const QPoint& globalPos) {
        updateDrag(globalPos);
        commitDrop();
    });
    connect(group, &PanelGroup::dragCanceled, this, [this]() { cancelDrag(); });
    // M47: the per-widget menu's Close / Close Group reuse this column's own
    // close paths, the same ones `buildTabMenu` drives.
    connect(group, &PanelGroup::closePanelRequested, this, [this](const QString& objectName) {
        if (!objectName.isEmpty()) {
            showPanel(objectName, false);
        }
    });
    connect(group, &PanelGroup::closeGroupRequested, this,
            [this, group]() { closeGroup(group); });
}

void PanelColumn::insertGroupAt(PanelGroup* group, int index)
{
    if (!group) {
        return;
    }
    index = qBound(0, index, groups_.size());
    groups_.insert(index, group);
    group->setParent(nullptr);
    splitter_->insertWidget(index, group);
    splitter_->setStretchFactor(index, 1);
    group->setFloating(false);
    group->setVisible(!group->visibleTitles().isEmpty());
    wireGroup(group);
}

bool PanelColumn::removeGroup(PanelGroup* group)
{
    const int index = groups_.indexOf(group);
    if (index < 0) {
        return false;
    }
    groups_.removeAt(index);
    group->setParent(nullptr);
    return true;
}

PanelGroup* PanelColumn::takeGroup(const QString& groupObjectName)
{
    PanelGroup* group = findGroupByName(groupObjectName);
    if (!group) {
        return nullptr;
    }
    const int index = groups_.indexOf(group);
    if (index < 0) {
        return nullptr;
    }
    groups_.removeAt(index);
    group->setParent(nullptr);
    // Leave the group unwired here; the adopting column re-wires it.
    QObject::disconnect(group, nullptr, this, nullptr);
    wired_.remove(group);
    return group;
}

void PanelColumn::cleanupEmptyGroup(PanelGroup* group)
{
    if (!group) {
        return;
    }
    if (group->titleCountForTest() > 0) {
        // M47: a group whose tabs are all hidden is empty on screen; hide it
        // but keep it alive (and float ownership intact) so a later show works.
        if (group->visibleTitles().isEmpty()) {
            group->setVisible(false);
            maybeRemoveSelf();
        }
        return;
    }
    wired_.remove(group);
    if (PanelFloat* floatWindow = floatForGroup(group)) {
        destroyFloat(floatWindow);
        maybeRemoveSelf();
        return;
    }
    removeGroup(group);
    group->deleteLater();
    maybeRemoveSelf();
}

PanelFloat* PanelColumn::floatForGroup(PanelGroup* group) const
{
    for (PanelFloat* floatWindow : floats_) {
        if (floatWindow && floatWindow->group() == group) {
            return floatWindow;
        }
    }
    return nullptr;
}

PanelGroup* PanelColumn::findGroupByName(const QString& objectName) const
{
    for (PanelGroup* group : groups_) {
        if (group && group->objectName() == objectName) {
            return group;
        }
    }
    return nullptr;
}

QList<PanelGroup*> PanelColumn::visibleGroups() const
{
    QList<PanelGroup*> out;
    for (PanelGroup* group : groups_) {
        if (group && group->isVisible()) {
            out << group;
        }
    }
    return out;
}

PanelGroup* PanelColumn::groupForPanel(const QString& objectName) const
{
    for (PanelGroup* group : groups_) {
        if (group && group->containsPanel(objectName)) {
            return group;
        }
    }
    return nullptr;
}

bool PanelColumn::showPanel(const QString& objectName, bool visible)
{
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        panelVisible_[objectName] = visible;
        return true;
    }
    if (!group->setPanelVisible(objectName, visible)) {
        return false;
    }
    panelVisible_[objectName] = visible;
    if (visible) {
        show();
        group->setVisible(true);
    }
    if (railMode_) {
        buildIconStrip();
    }
    updateMinimumWidth();
    emit stateChanged();
    if (!visible) {
        maybeRemoveSelf();
    }
    return true;
}

bool PanelColumn::isPanelVisible(const QString& objectName) const
{
    return panelVisible_.value(objectName, false);
}

void PanelColumn::closeGroup(PanelGroup* group)
{
    if (!group) {
        return;
    }
    for (QWidget* panel : group->panels()) {
        if (panel) {
            const QString name = panel->objectName();
            // M45 W4: hide each tab too, so the group stays closed if it is
            // rehomed to another column when this dynamic column is removed.
            group->setPanelVisible(name, false);
            panelVisible_[name] = false;
        }
    }
    group->setVisible(false);
    updateMinimumWidth();
    emit stateChanged();
    maybeRemoveSelf();
}

void PanelColumn::maybeRemoveSelf()
{
    // M45 W4: the frame owns the "is this dynamic column empty" test; every
    // path that can empty this column funnels through here.
    if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
        frame->removeColumnIfEmpty(this);
    }
}

void PanelColumn::setToolsContent(QWidget* content, std::function<int()> columnsState,
                                  std::function<void()> onToggleRequested)
{
    if (!content || toolsContent_) {
        return;
    }
    toolsContent_ = content;
    toolsColumnsState_ = std::move(columnsState);
    toolsToggleAction_ = std::move(onToggleRequested);
    // No group stack and no rail: host the plain content in the column body.
    scroll_->setVisible(false);
    iconStrip_->setVisible(false);
    if (auto* box = qobject_cast<QBoxLayout*>(layout())) {
        box->addWidget(content, 1);
    }
    content->setVisible(true);
    updateMinimumWidth();
    updateColumnToggle();
}

void PanelColumn::refreshToolsWidth()
{
    if (!toolsContent_) {
        return;
    }
    updateMinimumWidth();
    // A splitter pane keeps its explicit size until told otherwise; resize this
    // pane to its content width when a 1<->2 column flip changed that width.
    if (auto* splitter = qobject_cast<QSplitter*>(parentWidget())) {
        const int index = splitter->indexOf(this);
        if (index >= 0 && splitter->width() > 0) {
            QList<int> sizes = splitter->sizes();
            if (index < sizes.size()) {
                sizes[index] = minimumWidth();
                splitter->setSizes(sizes);
            }
        }
    }
}

void PanelColumn::updateMinimumWidth()
{
    if (toolsContent_) {
        // D2: the tools column tracks its content width (slot grid + fg/bg),
        // not the widget-column floor.
        const int want = toolsContent_->minimumWidth() > 0 ? toolsContent_->minimumWidth()
                                                           : toolsContent_->sizeHint().width();
        setMinimumWidth(qMax(kIconStripMinWidth, want));
        return;
    }
    if (railMode_) {
        setMinimumWidth(kIconStripMinWidth);
        return;
    }
    // M47: derive a shared floor from this column's real content (never from a
    // sibling splitter), clamped so all normal columns converge and none can
    // grow past the max.
    int need = kPanelMinWidth;
    if (splitter_) {
        need = qMax(need, splitter_->minimumSizeHint().width());
    }
    if (scroll_) {
        need += scroll_->verticalScrollBar()->sizeHint().width() + 2 * scroll_->frameWidth();
    }
    gSharedFloor = qBound(kPanelMinWidth, qMax(gSharedFloor, need), kPanelMaxWidth);
    setMinimumWidth(gSharedFloor);
}

void PanelColumn::refreshSharedFloor(PicturaMainWindow* frame)
{
    gSharedFloor = kPanelMinWidth;
    if (!frame) {
        return;
    }
    for (PanelColumn* column : frame->panelColumns()) {
        column->updateMinimumWidth();
    }
}

void PanelColumn::setAutoCollapseIconic(bool on)
{
    if (autoCollapseIconic_ == on) {
        return;
    }
    autoCollapseIconic_ = on;
    emit stateChanged();
}

void PanelColumn::setAutoShowHidden(bool on)
{
    if (autoShowHidden_ == on) {
        return;
    }
    autoShowHidden_ = on;
    if (railMode_) {
        buildIconStrip();
    }
    emit stateChanged();
}

void PanelColumn::setPreferredWidth(int width)
{
    if (width <= 0) {
        return;
    }
    // A normal column is never wider than `kMaxNormalWidth`; an iconic strip
    // keeps its own narrow floor. This is both the layout bound and the guard
    // against a stale oversized store expanding the column across the workspace.
    const int floor = railMode_ ? kIconStripMinWidth : kPanelMinWidth;
    const int target = qBound(floor, width, kMaxNormalWidth);
    auto* splitter = qobject_cast<QSplitter*>(parentWidget());
    const int index = splitter ? splitter->indexOf(this) : -1;
    if (!splitter || index < 0 || splitter->width() <= 0) {
        // Before the first layout the splitter has no width; remember the value
        // and apply it on show.
        pendingWidth_ = target;
        return;
    }
    QList<int> sizes = splitter->sizes();
    if (index >= sizes.size()) {
        pendingWidth_ = target;
        return;
    }
    // Keep every other pane's size (other columns, the document tabs) and set
    // only this column. The M41 two-pane case is the same operation.
    sizes[index] = target;
    splitter->setSizes(sizes);
    pendingWidth_ = 0;
}

void PanelColumn::setRestoredWidth(int width)
{
    if (width <= 0) {
        return;
    }
    if (railMode_) {
        normalWidthBeforeIconic_ = qBound(kMinNormalWidth, width, kMaxNormalWidth);
        return;
    }
    setPreferredWidth(width);
}

void PanelColumn::showEvent(QShowEvent* event)
{
    QWidget::showEvent(event);
    if (pendingWidth_ > 0 && !railMode_) {
        const int width = pendingWidth_;
        pendingWidth_ = 0;
        setPreferredWidth(width);
    }
}

void PanelColumn::resizeEvent(QResizeEvent* event)
{
    QWidget::resizeEvent(event);
    // A normal-mode resize lands the flip's widening; the live width is now the
    // real normal width, so `persistedWidth` may use it again.
    if (!railMode_ && width() > 0) {
        widthFlipPending_ = false;
    }
}

QJsonArray PanelColumn::savePanelState() const
{
    QJsonArray groups;
    for (PanelGroup* group : groups_) {
        if (!group) {
            continue;
        }
        QJsonObject entry;
        entry.insert(QStringLiteral("name"), group->objectName());
        QJsonArray order;
        QJsonArray visible;
        for (QWidget* panel : group->panels()) {
            if (!panel) {
                continue;
            }
            order.append(panel->objectName());
            if (group->isPanelVisible(panel->objectName())) {
                visible.append(panel->objectName());
            }
        }
        entry.insert(QStringLiteral("order"), order);
        entry.insert(QStringLiteral("visible"), visible);
        entry.insert(QStringLiteral("minimized"), group->isMinimized());
        entry.insert(QStringLiteral("collapsed"), group->isCollapsedToIcons());
        groups.append(entry);
    }
    return groups;
}

void PanelColumn::restorePanelState(const QJsonArray& state)
{
    for (const QJsonValue& value : state) {
        const QJsonObject entry = value.toObject();
        PanelGroup* group = findGroupByName(entry.value(QStringLiteral("name")).toString());
        const QJsonArray order = entry.value(QStringLiteral("order")).toArray();
        if (!group && !order.isEmpty()) {
            group = groupForPanel(order.first().toString());
        }
        if (!group) {
            continue;
        }
        QStringList orderList;
        for (const QJsonValue& name : order) {
            orderList << name.toString();
        }
        if (!orderList.isEmpty()) {
            group->setPanelOrder(orderList);
        }
        QStringList visibleList;
        for (const QJsonValue& name : entry.value(QStringLiteral("visible")).toArray()) {
            visibleList << name.toString();
        }
        for (QWidget* panel : group->panels()) {
            if (!panel) {
                continue;
            }
            const bool visible = visibleList.contains(panel->objectName());
            group->setPanelVisible(panel->objectName(), visible);
            panelVisible_[panel->objectName()] = visible;
        }
        group->setMinimized(entry.value(QStringLiteral("minimized")).toBool(false));
        group->setCollapsedToIcons(entry.value(QStringLiteral("collapsed")).toBool(false));
        // M44: `setPanelVisible` makes each shown panel current, so the last
        // visible panel would win; restore the first visible as the active tab.
        group->setCurrentToFirstVisible();
    }
    if (railMode_) {
        buildIconStrip();
    }
    updateMinimumWidth();
}

} // namespace pictura
