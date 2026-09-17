#include "panel_column.h"

#include "icons.h"
#include "panel_group.h"

#include <QtCore/QEvent>
#include <QtCore/QJsonObject>
#include <QtCore/QMetaObject>
#include <QtCore/QRect>
#include <QtCore/QSize>
#include <QtGui/QAction>
#include <QtGui/QCursor>
#include <QtGui/QGuiApplication>
#include <QtGui/QHideEvent>
#include <QtGui/QMouseEvent>
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

constexpr int kIconLabelWidth = 120;
constexpr int kIconButtonSize = 30;
constexpr int kIconPixmapSize = 20;
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
    setStyleSheet(QStringLiteral(
        "#panelFloat{background:#3a3a3a;border:1px solid #555;}"));
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
            widest = qMax(widest, group->sizeHint().width());
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
    updateMinimumWidth();
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
    stripEntries_.clear();
    stripPressPending_ = false;
    stripDragging_ = false;
    stripDragButton_ = nullptr;
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
            QToolButton* iconButton = makeIconButton(
                row, panel->objectName(), title, group->iconForPanel(panel->objectName()));
            rowLayout->addWidget(iconButton);
            stripEntries_.append(StripEntry{group, panel->objectName(), iconButton});
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
    flyout_->setStyleSheet(QStringLiteral(
        "#panelIconFlyout{background:#3a3a3a;border:1px solid #555;}"
        "#panelFlyoutHeader{background:#4a4a4a;border-bottom:1px solid #555;}"));
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
    placeFlyout(globalPos, size);
    flyout_->show();
    flyout_->raise();
}

QString PanelColumn::flyoutSide() const
{
    QWidget* win = window();
    if (!win) {
        return QStringLiteral("left");
    }
    const int colLeft = mapToGlobal(QPoint(0, 0)).x();
    const int colRight = mapToGlobal(QPoint(width(), 0)).x();
    const int winLeft = win->mapToGlobal(QPoint(0, 0)).x();
    const int winRight = win->mapToGlobal(QPoint(win->width(), 0)).x();
    // A column hugging the window's right edge opens its flyouts to the left.
    return (winRight - colRight) <= (colLeft - winLeft) ? QStringLiteral("left")
                                                        : QStringLiteral("right");
}

void PanelColumn::placeFlyout(const QPoint& anchorRightTop, const QSize& size)
{
    if (!flyout_) {
        return;
    }
    // `anchorRightTop` is the icon button's top-right corner, as passed by the
    // click handlers. Derive the icon rect so the popup meets the inner edge.
    const QRect iconRect(anchorRightTop.x() - kIconButtonSize, anchorRightTop.y(),
                         kIconButtonSize, kIconButtonSize);
    int x = flyoutSide() == QStringLiteral("left") ? iconRect.left() - size.width()
                                                   : iconRect.right() + 1;
    int y = iconRect.top();
    QScreen* screen = window() ? window()->screen() : QGuiApplication::screenAt(anchorRightTop);
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
    if (railMode_) {
        // Iconic mode shows only the strip, so a drop is either an in-strip
        // reorder or a tear-off; the group stack is handled after the column
        // expands back to normal.
        const int stripIndex = stripInsertionIndexAt(globalPos);
        if (stripIndex >= 0) {
            target.valid = true;
            target.onStrip = true;
            target.stripIndex = stripIndex;
            return target;
        }
        target.valid = true;
        target.outside = true;
        return target;
    }
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
    if (target.onStrip) {
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
    const bool tornOffThisDrag = dragFloat_ && dragOriginalIndex_ >= 0;
    bool ok = false;
    clearIndicator();
    if (target.valid && !target.outside) {
        if (target.onStrip) {
            ok = applyStripDrop(dragGroup_, dragPanel_, target.stripIndex);
        } else if (dragIsPanel_ && !tornOffThisDrag) {
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
    // The overlay parents to the main window (its central area is the clamp
    // rect) so it is clipped to the window; it must not parent to the
    // `centerSplitter`, which would absorb it as a splitter pane.
    QWidget* host = window();
    if (!host) {
        host = this;
    }
    auto* floatWindow = new PanelFloat(host);
    floatWindow->setGroup(group);
    group->setVisible(true);
    QSize size = group->sizeHint();
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
