#include "panel_column.h"

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
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

constexpr int kIconLabelMinWidth = 16;
constexpr int kIconButtonSize = 34;
constexpr int kIconPixmapSize = 24;
// ponytail: chosen compact-strip divider height, not a sourced CS6 metric.
constexpr int kCompactDividerHeight = 2;
// M44 C3: chosen compact-strip grip height and the "very close to a group"
// proximity band, not sourced CS6 metrics. The band is zero so the strip's own
// margins remain new-group boundaries rather than swallowing the first/last
// group.
constexpr int kCompactGripHeight = 10;
constexpr int kCompactNearBand = 0;
// ponytail: chosen constants, not sourced CS6 metrics. Normal-mode minimum is
// the widest visible group's size hint clamped between a readable floor and a
// cap that keeps the main window resizable; the iconic strip only needs its
// icon button plus margins. Tune here.
constexpr int kNormalMinWidthFloor = 180;
constexpr int kNormalMinWidthCap = 320;
constexpr int kIconStripMinWidth = 40;

const char* const kTabMenuTexts[] = {
    "Close",
    "Close Panel Group",
    "Minimize",
    "Collapse to Icons",
    "Auto-Collapse Iconic Panels",
    "Auto-Show Hidden Panels",
    "Interface Options\u2026",
};

} // namespace

PanelFlyout::PanelFlyout(QWidget* parent)
    : QWidget(parent)
{
    setWindowFlags(Qt::Popup);
}

void PanelFlyout::hideEvent(QHideEvent* event)
{
    QWidget::hideEvent(event);
    if (onHidden) {
        onHidden();
    }
}

PanelFloat::PanelFloat(QWidget* parent)
    : QWidget(parent)
{
    setObjectName(QStringLiteral("panelFloat"));
    // ponytail: in-window overlay — a plain raised child, never a top-level
    // window, so it is clipped to the main window and stays out of the task
    // list. OS-window float chrome and multi-monitor float are non-goals.
    setWindowFlags(Qt::Widget);
    setAttribute(Qt::WA_StyledBackground, true);
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(1, 1, 1, 1);
    layout->setSpacing(0);
}

void PanelFloat::setGroup(PanelGroup* group)
{
    group_ = group;
    if (group_) {
        if (auto* layout = qobject_cast<QBoxLayout*>(this->layout())) {
            layout->addWidget(group_);
        }
    }
}

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
    auto* headerLayout = new QHBoxLayout(header_);
    headerLayout->setContentsMargins(2, 2, 2, 2);
    headerLayout->setSpacing(2);
    headerLayout->addStretch(1);
    columnToggle_ = new QToolButton(header_);
    columnToggle_->setObjectName(QStringLiteral("panelColumnToggle"));
    columnToggle_->setAutoRaise(true);
    columnToggle_->setFixedSize(20, 20);
    connect(columnToggle_, &QToolButton::clicked, this, [this]() { setRailMode(!railMode_); });
    headerLayout->addWidget(columnToggle_);
    layout->addWidget(header_);
    updateColumnToggle();

    scroll_ = new QScrollArea(this);
    scroll_->setObjectName(QStringLiteral("panelColumnScroll"));
    scroll_->setWidgetResizable(true);
    scroll_->setFrameShape(QFrame::NoFrame);
    scroll_->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    splitter_ = new QSplitter(Qt::Vertical, scroll_);
    splitter_->setObjectName(QStringLiteral("panelColumnSplitter"));
    splitter_->setHandleWidth(Theme::kGroupDividerWidth);
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
    if (!group || group->titleCountForTest() > 0) {
        return;
    }
    wired_.remove(group);
    if (PanelFloat* floatWindow = floatForGroup(group)) {
        destroyFloat(floatWindow);
        return;
    }
    removeGroup(group);
    group->deleteLater();
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
        group->setVisible(true);
    }
    if (railMode_) {
        buildIconStrip();
    }
    updateMinimumWidth();
    emit stateChanged();
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
            panelVisible_[panel->objectName()] = false;
        }
    }
    group->setVisible(false);
    updateMinimumWidth();
    emit stateChanged();
}

void PanelColumn::setRailMode(bool iconic)
{
    if (railMode_ == iconic) {
        updateColumnToggle();
        return;
    }
    railMode_ = iconic;
    if (iconic) {
        if (width() > 0) {
            normalWidthBeforeIconic_ = width();
        }
        buildIconStrip();
        iconStrip_->setVisible(true);
        scroll_->setVisible(false);
        updateMinimumWidth();
        setPreferredWidth(kIconStripMinWidth);
    } else {
        closeIconFlyout();
        iconStrip_->setVisible(false);
        scroll_->setVisible(true);
        updateMinimumWidth();
        setPreferredWidth(normalWidthBeforeIconic_ > 0 ? normalWidthBeforeIconic_
                                                       : minimumWidth());
    }
    updateColumnToggle();
    emit stateChanged();
}

void PanelColumn::updateMinimumWidth()
{
    if (railMode_) {
        setMinimumWidth(kIconStripMinWidth);
        return;
    }
    int widest = 0;
    for (PanelGroup* group : groups_) {
        if (group && group->isVisible()) {
            // M43: reserve the per-widget corner width so the `▾` button is
            // inside the header at the minimum width even when the tab text
            // elides.
            widest = qMax(widest, group->sizeHint().width() + group->headerCornerWidthForTest());
        }
    }
    setMinimumWidth(qBound(kNormalMinWidthFloor, widest, kNormalMinWidthCap));
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
    auto* splitter = qobject_cast<QSplitter*>(parentWidget());
    const int index = splitter ? splitter->indexOf(this) : -1;
    if (!splitter || index < 0 || splitter->width() <= width) {
        // Before the first layout the splitter has no width; remember the value
        // and apply it on show.
        pendingWidth_ = width;
        return;
    }
    // Keep every other pane's size (other columns, the document tabs) and set
    // only this column. The M41 two-pane case is the same operation.
    QList<int> sizes = splitter->sizes();
    if (index >= sizes.size()) {
        pendingWidth_ = width;
        return;
    }
    sizes[index] = width;
    splitter->setSizes(sizes);
    pendingWidth_ = 0;
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

void PanelColumn::updateColumnToggle()
{
    // M44: the toggle shows the icon for the action it performs — collapse to
    // icons in normal mode (`panel.columnsTwo`, the inward double chevron) and
    // expand in iconic mode (`panel.columnsOne`).
    const QIcon target = icon(railMode_ ? QStringLiteral("panel.columnsOne")
                                        : QStringLiteral("panel.columnsTwo"));
    if (!target.isNull()) {
        columnToggle_->setIcon(target);
        columnToggle_->setIconSize(QSize(16, 16));
        columnToggle_->setText(QString());
    } else {
        columnToggle_->setIcon(QIcon());
        columnToggle_->setText(railMode_ ? QStringLiteral("\u00ab") : QStringLiteral("\u00bb"));
    }
    columnToggle_->setToolTip(railMode_ ? tr("Expand panels")
                                        : tr("Collapse panels to icons"));
}

void PanelColumn::clearIconStrip()
{
    stripLabels_.clear();
    stripEntries_.clear();
    stripGroupOrder_.clear();
    stripGroupBoxes_.clear();
    stripDividers_.clear();
    stripPressPending_ = false;
    stripDragging_ = false;
    stripDragButton_ = nullptr;
    stripGripGroup_ = nullptr;
    while (QLayoutItem* item = iconStripLayout_->takeAt(0)) {
        if (QWidget* widget = item->widget()) {
            delete widget;
        }
        delete item;
    }
}

void PanelColumn::buildIconStrip()
{
    clearIconStrip();
    bool first = true;
    for (PanelGroup* group : groups_) {
        // `Auto-Show Hidden Panels` includes the hidden panels' icons in the
        // strip; otherwise only the visible panels of visible groups appear.
        const QList<QWidget*> panels =
            autoShowHidden_ ? group->panels() : group->visiblePanels();
        if (panels.isEmpty()) {
            continue;
        }
        if (!first) {
            auto* divider = new QFrame(iconStrip_);
            divider->setObjectName(QStringLiteral("panelIconDivider"));
            divider->setAttribute(Qt::WA_StyledBackground, true);
            divider->setFrameShape(QFrame::NoFrame);
            divider->setFixedHeight(kCompactDividerHeight);
            iconStripLayout_->addWidget(divider);
            stripDividers_.append(StripDivider{group, divider});
        }
        first = false;
        // M44 C3: one shared container per group (with a drag-handle grip above
        // its icons) so the group reads as a single visual unit and the whole
        // group is draggable.
        auto* groupBox = new QWidget(iconStrip_);
        groupBox->setObjectName(QStringLiteral("panelIconGroup"));
        groupBox->setAttribute(Qt::WA_StyledBackground, true);
        groupBox->setProperty("groupObjectName", group->objectName());
        auto* groupLayout = new QVBoxLayout(groupBox);
        groupLayout->setContentsMargins(1, 1, 1, 1);
        groupLayout->setSpacing(1);
        groupLayout->addWidget(makeGroupGrip(group));
        const QStringList titles = autoShowHidden_ ? group->titles() : group->visibleTitles();
        for (int i = 0; i < panels.size(); ++i) {
            QWidget* panel = panels.at(i);
            const QString title = titles.value(i);
            auto* row = new QWidget(groupBox);
            row->setObjectName(QStringLiteral("panelIconRow"));
            auto* rowLayout = new QHBoxLayout(row);
            rowLayout->setContentsMargins(0, 0, 0, 0);
            rowLayout->setSpacing(4);
            QToolButton* iconButton = makeIconButton(
                row, panel->objectName(), title, group->iconForPanel(panel->objectName()));
            rowLayout->addWidget(iconButton);
            stripEntries_.append(StripEntry{group, panel->objectName(), iconButton});
            auto* label = new QLabel(title, row);
            label->setObjectName(QStringLiteral("panelIconLabel"));
            label->setProperty("fullTitle", title);
            label->setVisible(iconLabelsShown_);
            rowLayout->addWidget(label);
            rowLayout->addStretch(1);
            groupLayout->addWidget(row);
            stripLabels_ << label;
        }
        iconStripLayout_->addWidget(groupBox);
        stripGroupOrder_.append(group);
        stripGroupBoxes_.insert(group, groupBox);
    }
    iconStripLayout_->addStretch(1);
    updateIconStripLabels();
}

QWidget* PanelColumn::makeGroupGrip(PanelGroup* group)
{
    auto* grip = new QLabel(iconStrip_);
    grip->setObjectName(QStringLiteral("panelIconGroupGrip"));
    grip->setProperty("groupObjectName", group ? group->objectName() : QString());
    grip->setAlignment(Qt::AlignCenter);
    grip->setText(QStringLiteral("\u2022\u2022\u2022"));
    grip->setFixedHeight(kCompactGripHeight);
    grip->setCursor(Qt::SizeAllCursor);
    grip->setToolTip(tr("Drag to move this panel group"));
    grip->setAttribute(Qt::WA_StyledBackground, true);
    grip->installEventFilter(this);
    return grip;
}

QWidget* PanelColumn::stripGroupBoxFor(PanelGroup* group) const
{
    return stripGroupBoxes_.value(group, nullptr);
}

void PanelColumn::updateIconStripLabels()
{
    // M44: show the label as soon as the row has room beyond the icon button and
    // elide its text to that room, instead of hiding it until the full title
    // fits (the M42/M43 `>= 120 px` gate).
    const int available = iconStrip_->width() - kIconButtonSize - 10;
    const bool show = available >= kIconLabelMinWidth;
    QFontMetrics metrics(font());
    for (QLabel* label : stripLabels_) {
        if (!label) {
            continue;
        }
        if (show) {
            const QString full = label->property("fullTitle").toString();
            label->setText(metrics.elidedText(full, Qt::ElideRight, available));
        }
        label->setVisible(show);
    }
    iconLabelsShown_ = show;
}

QToolButton* PanelColumn::makeIconButton(QWidget* parent, const QString& objectName,
                                         const QString& title, const QIcon& icon)
{
    auto* button = new QToolButton(parent);
    button->setObjectName(QStringLiteral("panelIcon_") + objectName);
    if (!icon.isNull()) {
        button->setIcon(icon);
        button->setIconSize(QSize(kIconPixmapSize, kIconPixmapSize));
    } else {
        button->setText(title.left(1));
    }
    button->setToolTip(title);
    button->setAutoRaise(true);
    button->setCheckable(true);
    button->setChecked(activeIconName_ == objectName);
    button->setFixedSize(kIconButtonSize, kIconButtonSize);
    button->installEventFilter(this);
    connect(button, &QToolButton::clicked, this, [this, objectName, button]() {
        openIconFlyout(objectName, button->mapToGlobal(QPoint(button->width(), 0)));
    });
    return button;
}

void PanelColumn::ensureFlyout()
{
    if (flyout_) {
        return;
    }
    flyout_ = new PanelFlyout(this);
    flyout_->setObjectName(QStringLiteral("panelIconFlyout"));
    flyoutLayout_ = new QVBoxLayout(flyout_);
    flyoutLayout_->setContentsMargins(1, 1, 1, 1);
    flyoutLayout_->setSpacing(0);

    // A one-tab group look: the panel title as the header, then the content,
    // with the close chevron at the right end of the header.
    flyoutHeader_ = new QWidget(flyout_);
    flyoutHeader_->setObjectName(QStringLiteral("panelFlyoutHeader"));
    auto* headerLayout = new QHBoxLayout(flyoutHeader_);
    headerLayout->setContentsMargins(6, 2, 2, 2);
    headerLayout->setSpacing(2);
    flyoutTitle_ = new QLabel(flyoutHeader_);
    flyoutTitle_->setObjectName(QStringLiteral("panelFlyoutTitle"));
    headerLayout->addWidget(flyoutTitle_);
    headerLayout->addStretch(1);
    flyoutClose_ = new QToolButton(flyoutHeader_);
    flyoutClose_->setObjectName(QStringLiteral("panelFlyoutClose"));
    flyoutClose_->setAutoRaise(true);
    flyoutClose_->setIcon(icon(QStringLiteral("panel.closeChevron")));
    flyoutClose_->setIconSize(QSize(16, 16));
    flyoutClose_->setToolTip(tr("Close"));
    connect(flyoutClose_, &QToolButton::clicked, this, [this]() { closeIconFlyout(); });
    headerLayout->addWidget(flyoutClose_);
    flyoutLayout_->addWidget(flyoutHeader_);

    flyout_->onHidden = [this]() { restoreFlyoutPanel(); };
}

void PanelColumn::openIconFlyout(const QString& objectName, const QPoint& globalPos)
{
    closeIconFlyout();
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        return;
    }
    const QString title = group->titleForPanel(objectName);
    QWidget* panel = group->detachPanel(objectName);
    if (!panel) {
        return;
    }
    // `Auto-Show Hidden Panels`: opening a hidden panel's icon reveals it.
    if (autoShowHidden_ && !panelVisible_.value(objectName, false)) {
        panelVisible_[objectName] = true;
        group->setVisible(true);
    }
    ensureFlyout();
    flyoutPanel_ = panel;
    flyoutGroup_ = group;
    flyoutName_ = objectName;
    if (flyoutTitle_) {
        flyoutTitle_->setText(title);
    }
    flyoutLayout_->addWidget(panel);
    panel->setVisible(true);
    setActiveIcon(objectName);
    QSize size = panel->sizeHint().expandedTo(QSize(220, 180));
    if (size.width() > 480) {
        size.setWidth(480);
    }
    flyout_->resize(size);
    // M43: derive the flyout position from the button's actual geometry (not
    // `anchorRightTop ± kIconButtonSize`), so it meets the inner edge exactly
    // and never overlaps the button.
    if (QToolButton* button = flyoutButtonFor(objectName)) {
        placeFlyout(QRect(button->mapToGlobal(QPoint(0, 0)), button->size()), size);
    } else {
        placeFlyout(QRect(globalPos - QPoint(kIconButtonSize, 0),
                           QSize(kIconButtonSize, kIconButtonSize)),
                    size);
    }
    flyout_->show();
    flyout_->raise();
}

QString PanelColumn::flyoutSide() const
{
    // M43: the inner side generalises to N columns from the column's own
    // layout side (right column => popup to its left, left column => to its
    // right), not from the window-edge heuristic.
    return side() == PanelSide::Right ? QStringLiteral("left") : QStringLiteral("right");
}

void PanelColumn::placeFlyout(const QRect& buttonGlobalRect, const QSize& size)
{
    if (!flyout_) {
        return;
    }
    // M43: use the button's actual global rect. For a right-hand column the
    // flyout sits flush to the left of the button (its inner edge); for a
    // left-hand column, to the right. The screen clamp bounds only the inner
    // coordinate, so it can never push the flyout across to the outer side.
    const bool innerLeft = flyoutSide() == QStringLiteral("left");
    int x = innerLeft ? buttonGlobalRect.left() - size.width()
                      : buttonGlobalRect.right() + 1;
    int y = buttonGlobalRect.top();
    QScreen* screen = window() ? window()->screen()
                               : QGuiApplication::screenAt(buttonGlobalRect.center());
    if (!screen) {
        screen = QGuiApplication::primaryScreen();
    }
    if (screen) {
        const QRect avail = screen->availableGeometry();
        x = qBound(avail.left(), x, avail.right() - size.width() + 1);
        y = qBound(avail.top(), y, avail.bottom() - size.height() + 1);
    }
    flyout_->move(x, y);
}

QToolButton* PanelColumn::flyoutButtonFor(const QString& objectName) const
{
    if (QToolButton* button = stripButtonFor(objectName)) {
        return button;
    }
    for (PanelGroup* group : groups_) {
        if (!group) {
            continue;
        }
        if (QToolButton* button =
                group->findChild<QToolButton*>(QStringLiteral("panelGroupIcon_") + objectName)) {
            return button;
        }
    }
    return nullptr;
}

void PanelColumn::setActiveIcon(const QString& objectName)
{
    activeIconName_ = objectName;
    if (!iconStrip_) {
        return;
    }
    for (QToolButton* button : iconStrip_->findChildren<QToolButton*>()) {
        if (button && button->objectName().startsWith(QStringLiteral("panelIcon_"))) {
            button->setChecked(
                button->objectName().mid(QStringLiteral("panelIcon_").size()) == objectName);
        }
    }
}

void PanelColumn::closeIconFlyout()
{
    if (flyout_ && flyout_->isVisible()) {
        flyout_->hide();
    } else {
        restoreFlyoutPanel();
    }
}

void PanelColumn::restoreFlyoutPanel()
{
    if (!flyoutPanel_ || restoringFlyout_) {
        return;
    }
    restoringFlyout_ = true;
    QWidget* panel = flyoutPanel_;
    PanelGroup* group = flyoutGroup_;
    const QString name = flyoutName_;
    flyoutPanel_ = nullptr;
    flyoutGroup_ = nullptr;
    flyoutName_.clear();
    if (flyoutLayout_) {
        flyoutLayout_->removeWidget(panel);
    }
    if (group) {
        group->attachPanel(name);
    }
    restoringFlyout_ = false;
    setActiveIcon(QString());
    // `Auto-Collapse Iconic Panels`: once an icon flyout closes, return the
    // column to the iconic strip rather than leaving it expanded.
    if (autoCollapseIconic_ && !railMode_) {
        setRailMode(true);
    }
}

void PanelColumn::showTabMenu(PanelGroup* group, const QPoint& globalPos)
{
    menuGroup_ = group;
    QMenu* menu = buildTabMenu(group);
    menu->setAttribute(Qt::WA_DeleteOnClose);
    menu->popup(globalPos);
}

QMenu* PanelColumn::buildTabMenu(PanelGroup* group)
{
    auto* menu = new QMenu(this);
    menu->setObjectName(QStringLiteral("panelTabMenu"));

    QAction* close = menu->addAction(tr("Close"));
    connect(close, &QAction::triggered, this, [this, group]() {
        if (!group) {
            return;
        }
        const QString name = group->currentPanelName();
        if (!name.isEmpty()) {
            showPanel(name, false);
        }
    });

    QAction* closeGroupAction = menu->addAction(tr("Close Panel Group"));
    connect(closeGroupAction, &QAction::triggered, this,
            [this, group]() { closeGroup(group); });

    QAction* minimize = menu->addAction(tr("Minimize"));
    connect(minimize, &QAction::triggered, this, [this, group]() {
        group->setMinimized(!group->isMinimized());
        emit stateChanged();
    });

    QAction* collapse = menu->addAction(tr("Collapse to Icons"));
    connect(collapse, &QAction::triggered, this, [this, group]() {
        group->setCollapsedToIcons(!group->isCollapsedToIcons());
        emit stateChanged();
    });

    menu->addSeparator();

    QAction* autoCollapse = menu->addAction(tr("Auto-Collapse Iconic Panels"));
    autoCollapse->setCheckable(true);
    autoCollapse->setChecked(autoCollapseIconic_);
    connect(autoCollapse, &QAction::triggered, this, [this](bool on) {
        setAutoCollapseIconic(on);
    });

    QAction* autoShow = menu->addAction(tr("Auto-Show Hidden Panels"));
    autoShow->setCheckable(true);
    autoShow->setChecked(autoShowHidden_);
    connect(autoShow, &QAction::triggered, this, [this](bool on) { setAutoShowHidden(on); });

    menu->addSeparator();

    QAction* options = menu->addAction(tr("Interface Options\u2026"));
    connect(options, &QAction::triggered, this, [this]() { emit interfaceOptionsRequested(); });

    return menu;
}

bool PanelColumn::eventFilter(QObject* watched, QEvent* event)
{
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

QStringList PanelColumn::groupTitlesForTest() const
{
    QStringList out;
    for (PanelGroup* group : groups_) {
        if (!group) {
            continue;
        }
        const QStringList visible = group->visibleTitles();
        if (!visible.isEmpty()) {
            out << visible.join(QStringLiteral(" | "));
        }
    }
    return out;
}

QString PanelColumn::groupOfForTest(const QString& objectName) const
{
    PanelGroup* group = groupForPanel(objectName);
    return group ? group->objectName() : QString();
}

bool PanelColumn::scrollableForTest() const
{
    return scroll_ != nullptr;
}

int PanelColumn::minimumWidthForTest() const
{
    return minimumWidth();
}

bool PanelColumn::iconStripVisibleForTest() const
{
    return iconStrip_ && iconStrip_->isVisible();
}

int PanelColumn::dividerCountForTest() const
{
    return iconStrip_
        ? static_cast<int>(
              iconStrip_->findChildren<QFrame*>(QStringLiteral("panelIconDivider")).size())
        : 0;
}

int PanelColumn::groupDividerWidthForTest() const
{
    return splitter_ ? splitter_->handleWidth() : 0;
}

QString PanelColumn::stripLabelTextForTest(const QString& objectName) const
{
    for (int i = 0; i < stripEntries_.size() && i < stripLabels_.size(); ++i) {
        if (stripEntries_.at(i).name == objectName && stripLabels_.at(i)) {
            return stripLabels_.at(i)->text();
        }
    }
    return QString();
}

bool PanelColumn::stripLabelVisibleForTest(const QString& objectName) const
{
    for (int i = 0; i < stripEntries_.size() && i < stripLabels_.size(); ++i) {
        if (stripEntries_.at(i).name == objectName && stripLabels_.at(i)) {
            return stripLabels_.at(i)->isVisible();
        }
    }
    return false;
}

void PanelColumn::setIconStripWidthForTest(int width)
{
    if (!iconStrip_) {
        return;
    }
    iconStrip_->resize(qMax(0, width), iconStrip_->height());
    updateIconStripLabels();
}

bool PanelColumn::openIconFlyoutForTest(const QString& objectName)
{
    if (!groupForPanel(objectName)) {
        return false;
    }
    QToolButton* button =
        iconStrip_ ? iconStrip_->findChild<QToolButton*>(QStringLiteral("panelIcon_") + objectName)
                   : nullptr;
    if (!button) {
        for (PanelGroup* group : groups_) {
            button = group->findChild<QToolButton*>(QStringLiteral("panelGroupIcon_") + objectName);
            if (button) {
                break;
            }
        }
    }
    const QPoint pos =
        button ? button->mapToGlobal(QPoint(button->width(), 0)) : QCursor::pos();
    openIconFlyout(objectName, pos);
    return true;
}

bool PanelColumn::iconFlyoutVisibleForTest() const
{
    return flyout_ && flyout_->isVisible();
}

QStringList PanelColumn::tabMenuActionsForTest() const
{
    QStringList out;
    for (const char* text : kTabMenuTexts) {
        out << QString::fromUtf8(text);
    }
    return out;
}

bool PanelColumn::triggerTabMenuForTest(const QString& text)
{
    PanelGroup* group = menuGroup_;
    if (!group && !groups_.isEmpty()) {
        group = groups_.first();
    }
    if (!group) {
        return false;
    }
    menuGroup_ = group;
    QMenu* menu = buildTabMenu(group);
    bool found = false;
    for (QAction* action : menu->actions()) {
        if (action->text() == text) {
            action->trigger();
            found = true;
            break;
        }
    }
    delete menu;
    return found;
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
            return iconic;
        }
    }
    if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
        const int side = frame->newColumnSideAt(globalPos);
        if (side >= 0) {
            DropTarget target;
            target.valid = true;
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
            target.kind = anchorSide == 0 ? DropKind::NewColumnLeft : DropKind::NewColumnRight;
            return target;
        }
    }
    DropTarget target = resolveLocalDrop(globalPos);
    if (target.outside) {
        // A point inside another column may still regroup a panel there. Only
        // the tab-insert kind is safe to apply cross-column: the group lives in
        // the other column's stack, so the panel is inserted into it directly.
        if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
            if (PanelColumn* other = frame->columnAtGlobal(globalPos)) {
                if (other != this) {
                    DropTarget delegated = other->resolveLocalDrop(globalPos);
                    if (delegated.valid && delegated.kind == DropKind::IntoGroup) {
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
    clearIndicator();
}

void PanelColumn::updateDrag(const QPoint& globalPos)
{
    if (!dragActive_) {
        return;
    }
    dropTarget_ = resolveDrop(globalPos);
    if (dropTarget_.outside) {
        clearIndicator();
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
    showIndicatorFor(dropTarget_);
}

bool PanelColumn::commitDrop()
{
    if (!dragActive_) {
        return false;
    }
    const DropTarget target = dropTarget_;
    PanelGroup* source = dragSourceGroup_;
    bool ok = false;
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
    } else {
        ok = dragFloat_ != nullptr;
    }
    // M44 W4: the panel's source group was kept alive for the whole drag; clean
    // it up once the panel has landed elsewhere (it is empty when the drag took
    // its only tab).
    if (source && source != dragGroup_ && source->titleCountForTest() == 0) {
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
    // A dynamic column whose last group just left is torn down; a committed
    // drop is the only place this happens.
    if (auto* frame = qobject_cast<PicturaMainWindow*>(window())) {
        frame->removeColumnIfEmpty(this);
    }
    return ok;
}

void PanelColumn::cancelDrag()
{
    clearIndicator();
    PanelGroup* source = dragSourceGroup_;
    const bool tornOffThisDrag = dragFloat_ && dragOriginalIndex_ >= 0;
    if (tornOffThisDrag && dragGroup_) {
        insertGroupAt(dragGroup_, qBound(0, dragOriginalIndex_, groups_.size()));
        destroyFloat(dragFloat_);
    }
    // M44 W4: release the source group held for the drag's lifetime.
    if (source && source != dragGroup_ && source->titleCountForTest() == 0) {
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
    } else {
        auto* group = new PanelGroup(this);
        group->addPanel(panel, title, iconValue);
        insertGroupAt(group, qBound(0, target.boundary, groups_.size()));
        panelVisible_[name] = true;
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
    if (dragIsPanel_ && !dragPanel_.isEmpty()) {
        QString title;
        QIcon iconValue;
        int panelIndex = -1;
        QWidget* panel = source->takePanel(dragPanel_, &title, &iconValue, &panelIndex);
        if (!panel) {
            return false;
        }
        payload = new PanelGroup(this);
        payload->addPanel(panel, title, iconValue);
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
        if (index >= 0) {
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
    int boundary = target.boundary;
    if (target.onTabBar && target.group) {
        const int index = groups_.indexOf(target.group);
        boundary = index >= 0 ? index : groups_.size();
    }
    if (boundary < 0) {
        boundary = groups_.size();
    }
    const int current = groups_.indexOf(group);
    if (current >= 0) {
        if (boundary > current) {
            --boundary;
        }
        if (boundary == current) {
            return true;
        }
        groups_.removeAt(current);
        group->setParent(nullptr);
    }
    insertGroupAt(group, qBound(0, boundary, groups_.size()));
    return true;
}

PanelFloat* PanelColumn::createFloat(PanelGroup* group, const QPoint& globalPos)
{
    if (!group) {
        return nullptr;
    }
    PanelGroup* hosted = group;
    if (dragIsPanel_ && !dragPanel_.isEmpty()) {
        // M43: a tab drag floats a one-panel group holding only the dragged
        // panel; the source stack (docked group or old float) keeps the rest.
        QString title;
        QIcon iconValue;
        int index = -1;
        QWidget* panel = group->takePanel(dragPanel_, &title, &iconValue, &index);
        if (!panel) {
            return nullptr;
        }
        hosted = new PanelGroup(this);
        hosted->addPanel(panel, title, iconValue);
        panelVisible_[dragPanel_] = true;
        wireGroup(hosted);
        // M44 W4: keep the (possibly now empty) source group alive so its tab
        // bar keeps the implicit mouse grab until release; commitDrop/cancelDrag
        // clean it up once the drag ends.
    }
    const int index = groups_.indexOf(hosted);
    if (index >= 0) {
        groups_.removeAt(index);
        hosted->setParent(nullptr);
    }
    // The overlay parents to the main window (its central area is the clamp
    // rect) so it is clipped to the window; it must not parent to the
    // `centerSplitter`, which would absorb it as a splitter pane.
    QWidget* host = window();
    if (!host) {
        host = this;
    }
    auto* floatWindow = new PanelFloat(host);
    floatWindow->setGroup(hosted);
    hosted->setVisible(true);
    QSize size = hosted->sizeHint();
    size = size.expandedTo(QSize(220, 120));
    if (size.width() > 520) {
        size.setWidth(520);
    }
    floatWindow->resize(size);
    moveFloat(floatWindow, globalPos - dragGrabOffset_);
    floatWindow->show();
    floatWindow->raise();
    floats_ << floatWindow;
    if (railMode_) {
        buildIconStrip();
    }
    // Re-dock routes through the float's group, so commitDrop takes the panel
    // out of the one-panel float (or re-inserts the whole group).
    dragGroup_ = hosted;
    return floatWindow;
}

QRect PanelColumn::floatBounds(QWidget* host) const
{
    if (!host) {
        return QRect();
    }
    // Keep the overlay in the central area (below the menu/tool bars, above the
    // status bar and around the docks); fall back to the whole window.
    if (auto* mainWindow = qobject_cast<QMainWindow*>(host)) {
        if (QWidget* central = mainWindow->centralWidget()) {
            return QRect(central->mapTo(host, QPoint(0, 0)), central->size());
        }
    }
    return host->rect();
}

void PanelColumn::moveFloat(PanelFloat* floatWindow, const QPoint& globalTopLeft)
{
    if (!floatWindow) {
        return;
    }
    QWidget* host = floatWindow->parentWidget();
    if (!host) {
        return;
    }
    const QRect bounds = floatBounds(host);
    const QPoint local = host->mapFromGlobal(globalTopLeft);
    const int maxX = qMax(bounds.left(), bounds.right() - floatWindow->width() + 1);
    const int maxY = qMax(bounds.top(), bounds.bottom() - floatWindow->height() + 1);
    floatWindow->move(qBound(bounds.left(), local.x(), maxX),
                      qBound(bounds.top(), local.y(), maxY));
}

void PanelColumn::destroyFloat(PanelFloat* floatWindow)
{
    if (!floatWindow) {
        return;
    }
    if (!floats_.contains(floatWindow)) {
        // Already torn down by a cleanup path in this drop; do not double-free.
        if (dragFloat_ == floatWindow) {
            dragFloat_ = nullptr;
        }
        return;
    }
    floats_.removeAll(floatWindow);
    if (dragFloat_ == floatWindow) {
        dragFloat_ = nullptr;
    }
    floatWindow->hide();
    floatWindow->deleteLater();
}

bool PanelColumn::beginTabDragForTest(const QString& objectName)
{
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        for (PanelFloat* floatWindow : floats_) {
            PanelGroup* candidate = floatWindow ? floatWindow->group() : nullptr;
            if (candidate && candidate->containsPanel(objectName)) {
                group = candidate;
                break;
            }
        }
    }
    if (!group) {
        return false;
    }
    beginPanelDrag(group, objectName, group->mapToGlobal(QPoint(0, 0)));
    return true;
}

bool PanelColumn::beginGroupDragForTest(const QString& panelName)
{
    PanelGroup* group = groupForPanel(panelName);
    if (!group) {
        for (PanelFloat* floatWindow : floats_) {
            PanelGroup* candidate = floatWindow ? floatWindow->group() : nullptr;
            if (candidate && candidate->containsPanel(panelName)) {
                group = candidate;
                break;
            }
        }
    }
    if (!group) {
        return false;
    }
    beginGroupDrag(group, group->mapToGlobal(QPoint(0, 0)));
    return true;
}

void PanelColumn::dragToForTest(const QPoint& globalPos)
{
    updateDrag(globalPos);
}

bool PanelColumn::dropForTest(const QPoint& globalPos)
{
    if (!dragActive_) {
        return false;
    }
    updateDrag(globalPos);
    return commitDrop();
}

void PanelColumn::cancelDragForTest()
{
    cancelDrag();
}

bool PanelColumn::dropIndicatorVisibleForTest() const
{
    return (indicator_ && indicator_->isVisible())
           || (stripIndicator_ && stripIndicator_->isVisible());
}

QRect PanelColumn::dropIndicatorGeometryForTest() const
{
    if (stripIndicator_ && stripIndicator_->isVisible()) {
        return stripIndicator_->geometry();
    }
    return indicator_ ? indicator_->geometry() : QRect();
}

int PanelColumn::dropIndexForTest() const
{
    return dropTarget_.onTabBar ? dropTarget_.tabIndex : dropTarget_.boundary;
}

QStringList PanelColumn::stripOrderForTest() const
{
    QStringList out;
    for (const StripEntry& entry : stripEntries_) {
        out << entry.name;
    }
    return out;
}

QToolButton* PanelColumn::stripButtonFor(const QString& objectName) const
{
    if (!iconStrip_) {
        return nullptr;
    }
    return iconStrip_
        ->findChild<QToolButton*>(QStringLiteral("panelIcon_") + objectName);
}

bool PanelColumn::beginStripDragForTest(const QString& objectName)
{
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        return false;
    }
    QToolButton* button = stripButtonFor(objectName);
    const QPoint pos = button ? button->mapToGlobal(QPoint(0, 0))
                              : mapToGlobal(QPoint(qMax(1, width() / 2), 0));
    beginPanelDrag(group, objectName, pos);
    return true;
}

QPoint PanelColumn::stripInsertionPointForTest(int index) const
{
    if (!iconStrip_ || stripEntries_.isEmpty()) {
        return iconStrip_ ? iconStrip_->mapToGlobal(QPoint(iconStrip_->width() / 2, 0)) : QPoint();
    }
    const int clamped = qBound(0, index, stripEntries_.size());
    QToolButton* anchor = clamped < stripEntries_.size() ? stripEntries_.at(clamped).button
                                                         : stripEntries_.last().button;
    if (!anchor) {
        return QPoint();
    }
    const QPoint topLeft = anchor->mapToGlobal(QPoint(0, 0));
    const int y = clamped < stripEntries_.size() ? topLeft.y()
                                                 : topLeft.y() + anchor->height();
    return QPoint(topLeft.x() + anchor->width() / 2, y);
}

int PanelColumn::stripDropIndexForTest() const
{
    return dropTarget_.onStrip ? dropTarget_.stripIndex : -1;
}

QPoint PanelColumn::stripEntryPointForTest(const QString& panelName, int where) const
{
    for (const StripEntry& entry : stripEntries_) {
        if (entry.name != panelName || !entry.button) {
            continue;
        }
        const QRect row(entry.button->mapToGlobal(QPoint(0, 0)), entry.button->size());
        switch (where) {
        case 1:
            return QPoint(row.center().x(), row.top());
        case 2:
            return QPoint(row.center().x(), row.bottom());
        default:
            return row.center();
        }
    }
    return QPoint();
}

bool PanelColumn::dropStripOnGroupForTest(const QString& objectName, const QString& targetPanel)
{
    PanelGroup* dest = groupForPanel(targetPanel);
    if (!dest) {
        return false;
    }
    setRailMode(true);
    if (!beginStripDragForTest(objectName)) {
        setRailMode(false);
        return false;
    }
    // Reveal the normal-mode group stack underneath the active strip drag, then
    // resolve the drop through the existing group/tab-bar target path.
    setRailMode(false);
    ensureGroupVisibleForTest(targetPanel);
    QCoreApplication::processEvents();
    const QPoint target = dest->tabInsertionGlobalPointForTest(0);
    dragToForTest(target);
    const bool dropped = dropForTest(target);
    return dropped && groupForPanel(objectName) == dest;
}

QString PanelColumn::flyoutSideForTest() const
{
    return flyoutSide();
}

QRect PanelColumn::iconFlyoutGeometryForTest() const
{
    if (!flyout_ || !flyout_->isVisible()) {
        return QRect();
    }
    return QRect(flyout_->mapToGlobal(QPoint(0, 0)), flyout_->size());
}

QString PanelColumn::flyoutHeaderTitleForTest() const
{
    return flyoutTitle_ ? flyoutTitle_->text() : QString();
}

bool PanelColumn::flyoutHeaderCloseForTest() const
{
    return flyoutClose_ && flyoutClose_->parentWidget() == flyoutHeader_;
}

bool PanelColumn::triggerFlyoutCloseForTest()
{
    if (!flyoutClose_) {
        return false;
    }
    flyoutClose_->click();
    QCoreApplication::processEvents();
    return flyout_ ? !flyout_->isVisible() : true;
}

int PanelColumn::floatCountForTest() const
{
    return floats_.size();
}

QStringList PanelColumn::floatPanelNamesForTest(int index) const
{
    QStringList out;
    if (index < 0 || index >= floats_.size()) {
        return out;
    }
    PanelFloat* floatWindow = floats_.at(index);
    PanelGroup* group = floatWindow ? floatWindow->group() : nullptr;
    if (group) {
        for (QWidget* panel : group->panels()) {
            if (panel) {
                out << panel->objectName();
            }
        }
    }
    return out;
}

bool PanelColumn::floatIsWindowForTest(int index) const
{
    if (index < 0 || index >= floats_.size()) {
        return false;
    }
    return floats_.at(index)->isWindow();
}

QRect PanelColumn::floatGeometryForTest(int index) const
{
    if (index < 0 || index >= floats_.size()) {
        return QRect();
    }
    PanelFloat* floatWindow = floats_.at(index);
    return QRect(floatWindow->mapToGlobal(QPoint(0, 0)), floatWindow->size());
}

QRect PanelColumn::floatHostRectForTest() const
{
    QWidget* host = window();
    if (!host) {
        return QRect();
    }
    const QRect bounds = floatBounds(host);
    return QRect(host->mapToGlobal(bounds.topLeft()), bounds.size());
}

bool PanelColumn::floatClampedForTest(int index, const QPoint& globalTopLeft)
{
    if (index < 0 || index >= floats_.size()) {
        return false;
    }
    moveFloat(floats_.at(index), globalTopLeft);
    QCoreApplication::processEvents();
    return floatHostRectForTest().contains(floatGeometryForTest(index));
}

bool PanelColumn::tearOffForTest(const QString& groupName)
{
    PanelGroup* group = findGroupByName(groupName);
    if (!group) {
        group = groupForPanel(groupName);
    }
    if (!group) {
        return false;
    }
    beginGroupDrag(group, group->mapToGlobal(QPoint(0, 0)));
    const bool leftColumn = side() == PanelSide::Left;
    const QPoint outside = leftColumn ? mapToGlobal(QPoint(width() + 40, height() / 2))
                                      : mapToGlobal(QPoint(-40, height() / 2));
    dragToForTest(outside);
    return dropForTest(outside);
}

bool PanelColumn::tearOffPanelForTest(const QString& objectName)
{
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        for (PanelFloat* floatWindow : floats_) {
            PanelGroup* candidate = floatWindow ? floatWindow->group() : nullptr;
            if (candidate && candidate->containsPanel(objectName)) {
                group = candidate;
                break;
            }
        }
    }
    if (!group) {
        return false;
    }
    beginPanelDrag(group, objectName, group->mapToGlobal(QPoint(0, 0)));
    const bool leftColumn = side() == PanelSide::Left;
    const QPoint outside = leftColumn ? mapToGlobal(QPoint(width() + 40, height() / 2))
                                      : mapToGlobal(QPoint(-40, height() / 2));
    dragToForTest(outside);
    return dropForTest(outside);
}

bool PanelColumn::redockForTest(int floatIndex, int boundaryIndex)
{
    if (floatIndex < 0 || floatIndex >= floats_.size()) {
        return false;
    }
    PanelFloat* floatWindow = floats_.at(floatIndex);
    PanelGroup* group = floatWindow ? floatWindow->group() : nullptr;
    if (!group) {
        return false;
    }
    beginGroupDrag(group, group->mapToGlobal(QPoint(4, 4)));
    const QPoint target = boundaryPointForTest(boundaryIndex);
    dragToForTest(target);
    const bool dropped = dropForTest(target);
    return dropped && !floatForGroup(group);
}

bool PanelColumn::ensureGroupVisibleForTest(const QString& panelName)
{
    PanelGroup* group = groupForPanel(panelName);
    if (!group || !scroll_) {
        return false;
    }
    scroll_->ensureWidgetVisible(group, 0, 12);
    return true;
}

bool PanelColumn::dragSourceGroupAliveForTest() const
{
    return dragSourceGroup_ && groups_.contains(dragSourceGroup_);
}

QWidget* PanelColumn::dragHandleForTest(const QString& groupName) const
{
    PanelGroup* group = findGroupByName(groupName);
    if (!group) {
        return nullptr;
    }
    QWidget* box = stripGroupBoxFor(group);
    return box ? box->findChild<QWidget*>(QStringLiteral("panelIconGroupGrip")) : nullptr;
}

QStringList PanelColumn::compactStripGroupOrderForTest() const
{
    QStringList out;
    for (PanelGroup* group : stripGroupOrder_) {
        out << (group ? group->objectName() : QString());
    }
    return out;
}

int PanelColumn::compactStripGroupIndexForTest(const QString& panelName) const
{
    PanelGroup* owner = groupForPanel(panelName);
    return owner ? stripGroupOrder_.indexOf(owner) : -1;
}

QPoint PanelColumn::compactStripBoundaryPointForTest(int boundary) const
{
    if (!iconStrip_ || stripGroupOrder_.isEmpty()) {
        return QPoint();
    }
    auto centerX = [this](PanelGroup* group) {
        QWidget* box = stripGroupBoxFor(group);
        return box ? box->mapToGlobal(QPoint(box->width() / 2, 0)).x()
                   : iconStrip_->mapToGlobal(QPoint(iconStrip_->width() / 2, 0)).x();
    };
    auto topOf = [this](PanelGroup* group) {
        QWidget* box = stripGroupBoxFor(group);
        return box ? box->mapToGlobal(QPoint(0, 0)).y() : 0;
    };
    auto bottomOf = [this](PanelGroup* group) {
        QWidget* box = stripGroupBoxFor(group);
        return box ? box->mapToGlobal(QPoint(0, box->height())).y() : 0;
    };
    boundary = qBound(0, boundary, stripGroupOrder_.size());
    if (boundary <= 0) {
        PanelGroup* first = stripGroupOrder_.first();
        return QPoint(centerX(first), topOf(first) - 1);
    }
    if (boundary >= stripGroupOrder_.size()) {
        PanelGroup* last = stripGroupOrder_.last();
        return QPoint(centerX(last), bottomOf(last) + 1);
    }
    const int dividerIndex = boundary - 1;
    if (dividerIndex >= 0 && dividerIndex < stripDividers_.size()) {
        QWidget* divider = stripDividers_.at(dividerIndex).widget;
        if (divider) {
            return divider->mapToGlobal(QPoint(divider->width() / 2, divider->height() / 2));
        }
    }
    return QPoint();
}

bool PanelColumn::beginGroupGripDragForTest(const QString& groupName)
{
    PanelGroup* group = findGroupByName(groupName);
    QWidget* grip = dragHandleForTest(groupName);
    if (!group || !grip) {
        return false;
    }
    beginGroupDrag(group, grip->mapToGlobal(QPoint(grip->width() / 2, grip->height() / 2)));
    return true;
}

bool PanelColumn::popupStyleParityForTest(const QString& objectName) const
{
    if (!flyout_ || !flyout_->isVisible() || flyoutName_ != objectName || !flyoutPanel_) {
        return false;
    }
    // M44 C1/S2: no per-host inline override; the same scoped selectors style the
    // docked group, the float, and the flyout.
    if (!flyout_->styleSheet().isEmpty()) {
        return false;
    }
    const QString sheet = qApp->styleSheet();
    const QColor windowColor = qApp->palette().color(QPalette::Window);
    const QString windowHex = windowColor.name(QColor::HexRgb);
    const QString borderHex = windowColor.darker(135).name(QColor::HexRgb);
    const QString width = QString::number(Theme::kPanelBorderWidth);
    const QString container = QStringLiteral("background: ") + windowHex + QStringLiteral("; border: ")
        + width + QStringLiteral("px solid ") + borderHex;
    const bool sameContainer =
        sheet.contains(QStringLiteral("QWidget#panelIconFlyout { ") + container + QStringLiteral("; }"))
        && sheet.contains(QStringLiteral("QWidget#panelFloat { ") + container + QStringLiteral("; }"));
    const bool samePane = sheet.contains(
        QStringLiteral("QTabWidget#panelGroupTabs::pane { border: 0; background: ") + windowHex
        + QStringLiteral("; }"));
    const bool sameHeader = sheet.contains(
        QStringLiteral("QWidget#panelFlyoutHeader { background: ") + windowHex);
    const bool hosted = flyoutPanel_->parentWidget() == flyout_;
    return sameContainer && samePane && sameHeader && hosted;
}

QToolButton* PanelColumn::widgetMenuButtonForTest(const QString& groupObjectName) const
{
    PanelGroup* group = findGroupByName(groupObjectName);
    return group ? group->headerMenuButtonForTest() : nullptr;
}

QStringList PanelColumn::widgetMenuTextsForTest(const QString& panelName) const
{
    return PanelGroup::menuTextsForPanel(panelName);
}

bool PanelColumn::widgetMenuHasCloseForTest(const QString& panelName) const
{
    const QStringList texts = PanelGroup::menuTextsForPanel(panelName);
    return !texts.contains(QStringLiteral("Close"))
           && !texts.contains(QStringLiteral("Close Panel Group"));
}

bool PanelColumn::triggerWidgetMenuForTest(const QString& panelName, const QString& text)
{
    PanelGroup* group = groupForPanel(panelName);
    if (!group) {
        return false;
    }
    if (group->currentPanelName() != panelName) {
        showPanel(panelName, true);
        QCoreApplication::processEvents();
    }
    return group->triggerPanelMenuForTest(text);
}

QPoint PanelColumn::boundaryPointForTest(int boundary) const
{
    const QList<PanelGroup*> visible = visibleGroups();
    if (visible.isEmpty()) {
        return mapToGlobal(QPoint(4, height() / 2));
    }
    QWidget* viewport = scroll_ ? scroll_->viewport() : nullptr;
    if (!viewport) {
        return QPoint();
    }
    auto topY = [viewport](PanelGroup* group) {
        return group->mapTo(viewport, QPoint(0, 0)).y();
    };
    auto bottomY = [&topY](PanelGroup* group) { return topY(group) + group->height(); };
    int y = 0;
    if (boundary <= 0) {
        y = topY(visible.first());
    } else if (boundary >= visible.size()) {
        y = bottomY(visible.last());
    } else {
        y = (bottomY(visible.at(boundary - 1)) + topY(visible.at(boundary))) / 2;
    }
    y = qBound(0, y, viewport->height() - 1);
    return viewport->mapToGlobal(QPoint(qMax(1, viewport->width() / 2), y));
}

} // namespace pictura
