#include "panel_column.h"

#include "icons.h"
#include "panel_group.h"

#include <QtCore/QEvent>
#include <QtCore/QJsonObject>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QAction>
#include <QtGui/QCursor>
#include <QtGui/QHideEvent>
#include <QtGui/QShowEvent>
#include <QtWidgets/QBoxLayout>
#include <QtWidgets/QFrame>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QScrollArea>
#include <QtWidgets/QSplitter>
#include <QtWidgets/QTabBar>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

constexpr int kIconLabelWidth = 120;
constexpr int kIconButtonSize = 24;
constexpr int kIconPixmapSize = 16;

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
    setWindowFlags(Qt::Tool);
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
    splitter_->setHandleWidth(4);
    scroll_->setWidget(splitter_);
    layout->addWidget(scroll_, 1);

    iconStrip_ = new QWidget(this);
    iconStrip_->setObjectName(QStringLiteral("panelColumnIconStrip"));
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

    setMinimumWidth(0);
    setMinimumHeight(0);
}

void PanelColumn::addGroup(PanelGroup* group)
{
    if (!group) {
        return;
    }
    insertGroupAt(group, groups_.size());
    for (QWidget* panel : group->panels()) {
        if (panel) {
            panelVisible_[panel->objectName()] = true;
        }
    }
    if (railMode_) {
        buildIconStrip();
    }
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
        buildIconStrip();
        iconStrip_->setVisible(true);
        scroll_->setVisible(false);
    } else {
        closeIconFlyout();
        iconStrip_->setVisible(false);
        scroll_->setVisible(true);
    }
    updateColumnToggle();
    emit stateChanged();
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
    if (!splitter || splitter->count() != 2 || splitter->width() <= width) {
        // Before the first layout the splitter has no width; remember the value
        // and apply it on show.
        pendingWidth_ = width;
        return;
    }
    splitter->setSizes({splitter->width() - width, width});
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
    }
    if (railMode_) {
        buildIconStrip();
    }
}

void PanelColumn::updateColumnToggle()
{
    const QIcon target =
        icon(railMode_ ? QStringLiteral("panel.columnsTwo") : QStringLiteral("panel.columnsOne"));
    if (!target.isNull()) {
        columnToggle_->setIcon(target);
        columnToggle_->setIconSize(QSize(16, 16));
        columnToggle_->setText(QString());
    } else {
        columnToggle_->setIcon(QIcon());
        columnToggle_->setText(railMode_ ? QStringLiteral("\u00bb") : QStringLiteral("\u00ab"));
    }
    columnToggle_->setToolTip(railMode_ ? tr("Expand panels")
                                        : tr("Collapse panels to icons"));
}

void PanelColumn::clearIconStrip()
{
    stripLabels_.clear();
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
            divider->setFrameShape(QFrame::HLine);
            divider->setFrameShadow(QFrame::Plain);
            iconStripLayout_->addWidget(divider);
        }
        first = false;
        const QStringList titles = autoShowHidden_ ? group->titles() : group->visibleTitles();
        for (int i = 0; i < panels.size(); ++i) {
            QWidget* panel = panels.at(i);
            const QString title = titles.value(i);
            auto* row = new QWidget(iconStrip_);
            row->setObjectName(QStringLiteral("panelIconRow"));
            auto* rowLayout = new QHBoxLayout(row);
            rowLayout->setContentsMargins(0, 0, 0, 0);
            rowLayout->setSpacing(4);
            rowLayout->addWidget(makeIconButton(row, panel->objectName(), title,
                                                group->iconForPanel(panel->objectName())));
            auto* label = new QLabel(title, row);
            label->setObjectName(QStringLiteral("panelIconLabel"));
            label->setVisible(iconLabelsShown_);
            rowLayout->addWidget(label);
            rowLayout->addStretch(1);
            iconStripLayout_->addWidget(row);
            stripLabels_ << label;
        }
    }
    iconStripLayout_->addStretch(1);
    updateIconStripLabels();
}

void PanelColumn::updateIconStripLabels()
{
    const bool show = iconStrip_->width() >= kIconLabelWidth;
    if (show == iconLabelsShown_) {
        return;
    }
    iconLabelsShown_ = show;
    for (QLabel* label : stripLabels_) {
        if (label) {
            label->setVisible(show);
        }
    }
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
    button->setFixedSize(kIconButtonSize, kIconButtonSize);
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
    flyout_->onHidden = [this]() { restoreFlyoutPanel(); };
}

void PanelColumn::openIconFlyout(const QString& objectName, const QPoint& globalPos)
{
    closeIconFlyout();
    PanelGroup* group = groupForPanel(objectName);
    if (!group) {
        return;
    }
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
    flyoutLayout_->addWidget(panel);
    panel->setVisible(true);
    QSize size = panel->sizeHint().expandedTo(QSize(220, 180));
    if (size.width() > 480) {
        size.setWidth(480);
    }
    flyout_->resize(size);
    flyout_->move(globalPos);
    flyout_->show();
    flyout_->raise();
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
    DropTarget target;
    if (!scroll_ || !scroll_->isVisible()) {
        target.valid = true;
        target.outside = true;
        return target;
    }
    QWidget* viewport = scroll_->viewport();
    if (!viewport || !viewport->rect().contains(viewport->mapFromGlobal(globalPos))) {
        target.valid = true;
        target.outside = true;
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
                return target;
            }
        }
        const QRect groupRect(group->mapToGlobal(QPoint(0, 0)), group->size());
        if (groupRect.contains(globalPos)) {
            target.valid = true;
            target.group = group;
            const int centerY = groupRect.top() + groupRect.height() / 2;
            const int base = groups_.indexOf(group);
            target.boundary = globalPos.y() < centerY ? base : base + 1;
            return target;
        }
    }
    target.valid = true;
    target.boundary = boundaryIndexForGlobalY(globalPos);
    return target;
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
    if (!indicator_ || !scroll_) {
        return;
    }
    QWidget* viewport = scroll_->viewport();
    if (!viewport) {
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
    dragPanel_ = objectName;
    dragOriginalIndex_ = groups_.indexOf(group);
    dragFloat_ = floatForGroup(group);
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
            dragFloat_->move(globalPos - dragGrabOffset_);
        }
        return;
    }
    if (dragFloat_) {
        dragFloat_->move(globalPos - dragGrabOffset_);
    }
    showIndicatorFor(dropTarget_);
}

bool PanelColumn::commitDrop()
{
    if (!dragActive_) {
        return false;
    }
    const DropTarget target = dropTarget_;
    const bool tornOffThisDrag = dragFloat_ && dragOriginalIndex_ >= 0;
    bool ok = false;
    clearIndicator();
    if (target.valid && !target.outside) {
        if (dragIsPanel_ && !tornOffThisDrag) {
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
    dragActive_ = false;
    dragIsPanel_ = false;
    dragGroup_ = nullptr;
    dragPanel_.clear();
    dragGrabOffset_ = QPoint();
    dragOriginalIndex_ = -1;
    dragFloat_ = nullptr;
    dropTarget_ = {};
    if (railMode_) {
        buildIconStrip();
    }
    emit stateChanged();
    return ok;
}

void PanelColumn::cancelDrag()
{
    clearIndicator();
    const bool tornOffThisDrag = dragFloat_ && dragOriginalIndex_ >= 0;
    if (tornOffThisDrag && dragGroup_) {
        insertGroupAt(dragGroup_, qBound(0, dragOriginalIndex_, groups_.size()));
        destroyFloat(dragFloat_);
    }
    dragActive_ = false;
    dragIsPanel_ = false;
    dragGroup_ = nullptr;
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
    const int index = groups_.indexOf(group);
    if (index >= 0) {
        groups_.removeAt(index);
        group->setParent(nullptr);
    }
    auto* floatWindow = new PanelFloat(this);
    floatWindow->setGroup(group);
    group->setVisible(true);
    QSize size = group->sizeHint();
    size = size.expandedTo(QSize(220, 120));
    if (size.width() > 520) {
        size.setWidth(520);
    }
    floatWindow->resize(size);
    floatWindow->move(globalPos - dragGrabOffset_);
    floatWindow->show();
    floatWindow->raise();
    floats_ << floatWindow;
    if (railMode_) {
        buildIconStrip();
    }
    return floatWindow;
}

void PanelColumn::destroyFloat(PanelFloat* floatWindow)
{
    if (!floatWindow) {
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
    return indicator_ && indicator_->isVisible();
}

QRect PanelColumn::dropIndicatorGeometryForTest() const
{
    return indicator_ ? indicator_->geometry() : QRect();
}

int PanelColumn::dropIndexForTest() const
{
    return dropTarget_.onTabBar ? dropTarget_.tabIndex : dropTarget_.boundary;
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
    const QPoint outside = mapToGlobal(QPoint(-40, height() / 2));
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
