#include "toolbox.h"

#include "color_picker_dialog.h"
#include "icons.h"
#include "panels/color_panel.h"
#include "tools.h"

#include <QtCore/QEvent>
#include <QtCore/QSize>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QKeySequence>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPolygon>
#include <QtGui/QScreen>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QApplication>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QSizePolicy>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

#include <algorithm>
#include <functional>

namespace pictura {

namespace {

constexpr int kSwatchSize = 22;
constexpr int kWidgetSize = 40;
constexpr int kMinWidgetSize = 20;
constexpr int kResetSize = 12;
constexpr int kSlotButtonSize = 32;
constexpr int kSlotIconSize = 22;
constexpr int kFlyoutDelayMs = 300;

// A slot button: the base tool button plus the CS6 interactions the stock
// class lacks — a custom lower-right flyout triangle, right-click and
// press-and-hold open the flyout, Alt-click cycles the group's implemented
// members.
class ToolSlotButton : public QToolButton {
public:
    explicit ToolSlotButton(QWidget* parent = nullptr)
        : QToolButton(parent)
    {
        holdTimer_.setSingleShot(true);
        holdTimer_.setInterval(kFlyoutDelayMs);
        connect(&holdTimer_, &QTimer::timeout, this, [this]() {
            held_ = true;
            if (onMenu) {
                onMenu();
            }
        });
    }

    void setHasFlyout(bool hasFlyout)
    {
        hasFlyout_ = hasFlyout;
        update();
    }
    bool hasFlyout() const { return hasFlyout_; }

    std::function<void()> onMenu;
    std::function<void()> onCycle;

protected:
    void paintEvent(QPaintEvent* event) override
    {
        QToolButton::paintEvent(event);
        if (!hasFlyout_) {
            return;
        }
        QPainter painter(this);
        painter.setRenderHint(QPainter::Antialiasing, true);
        painter.setPen(Qt::NoPen);
        painter.setBrush(palette().color(QPalette::ButtonText));
        const int size = 5;
        const int margin = 2;
        QPolygon triangle;
        triangle << QPoint(width() - margin - size, height() - margin)
                 << QPoint(width() - margin, height() - margin)
                 << QPoint(width() - margin, height() - margin - size);
        painter.drawPolygon(triangle);
    }

    void mousePressEvent(QMouseEvent* event) override
    {
        if (event->button() == Qt::RightButton) {
            if (onMenu) {
                onMenu();
            }
            event->accept();
            return;
        }
        if (event->button() == Qt::LeftButton
            && event->modifiers().testFlag(Qt::AltModifier)) {
            if (onCycle) {
                onCycle();
            }
            event->accept();
            return;
        }
        if (event->button() == Qt::LeftButton) {
            held_ = false;
            holdTimer_.start();
        }
        QToolButton::mousePressEvent(event);
    }

    void mouseReleaseEvent(QMouseEvent* event) override
    {
        holdTimer_.stop();
        if (held_) {
            held_ = false;
            event->accept();
            return;
        }
        QToolButton::mouseReleaseEvent(event);
    }

private:
    QTimer holdTimer_;
    bool held_ = false;
    bool hasFlyout_ = false;
};

QString enabledTooltip(const ToolInfo& info)
{
    return QStringLiteral("%1 (%2) — %3")
        .arg(QString::fromLatin1(info.label), QString(info.shortcut),
             QString::fromLatin1(info.hint));
}

QString disabledTooltip(const ToolInfo& info)
{
    return QStringLiteral("%1 — not implemented yet").arg(QString::fromLatin1(info.label));
}

int groupMemberCount(int group)
{
    int count = 0;
    for (ToolId id : allToolIds()) {
        if (toolInfo(id).group == group) {
            ++count;
        }
    }
    return count;
}

bool groupHasImplemented(int group)
{
    for (ToolId id : allToolIds()) {
        if (toolInfo(id).group == group && toolImplemented(id)) {
            return true;
        }
    }
    return false;
}

} // namespace

ForegroundBackgroundWidget::ForegroundBackgroundWidget(ColorState* state, QWidget* parent)
    : QWidget(parent)
    , state_(state)
{
    setMinimumSize(kMinWidgetSize, kMinWidgetSize);
    setToolTip(tr("Foreground / background colors — click a swatch to make it active"));
    if (state_) {
        connect(state_, &ColorState::foregroundChanged, this, [this](const QColor&) { update(); });
        connect(state_, &ColorState::backgroundChanged, this, [this](const QColor&) { update(); });
        connect(state_, &ColorState::activeChanged, this, [this](bool) { update(); });
    }
}

void ForegroundBackgroundWidget::setSide(int side)
{
    // ponytail: square only; a height-for-width layout is overkill for a pair of
    // swatches. Clamped so the two squares stay readable at the smallest column.
    const int clamped = std::clamp(side, kMinWidgetSize, kWidgetSize);
    if (clamped == width() && clamped == height()) {
        return;
    }
    setFixedSize(clamped, clamped);
}

int ForegroundBackgroundWidget::swatchSize() const
{
    return std::clamp(width() * 55 / 100, 12, kSwatchSize);
}

int ForegroundBackgroundWidget::resetSize() const
{
    return std::clamp(width() * 30 / 100, 7, kResetSize);
}

QRect ForegroundBackgroundWidget::foregroundRect() const
{
    const int size = swatchSize();
    return QRect(1, 1, size, size);
}

QRect ForegroundBackgroundWidget::backgroundRect() const
{
    const int size = swatchSize();
    return QRect(width() - size - 1, height() - size - 1, size, size);
}

QRect ForegroundBackgroundWidget::resetRect() const
{
    const int size = resetSize();
    return QRect(2, height() - size - 2, size, size);
}

QRect ForegroundBackgroundWidget::swapRect() const
{
    const int size = resetSize();
    return QRect(width() - size - 2, 2, size, size);
}

void ForegroundBackgroundWidget::swapForegroundBackground()
{
    if (state_) {
        const QColor fg = state_->foreground();
        state_->setForeground(state_->background());
        state_->setBackground(fg);
    }
    ++swapCount_;
    update();
}

QColor ForegroundBackgroundWidget::foregroundForTest() const
{
    return state_ ? state_->foreground() : QColor(Qt::black);
}

QColor ForegroundBackgroundWidget::backgroundForTest() const
{
    return state_ ? state_->background() : QColor(Qt::white);
}

void ForegroundBackgroundWidget::resetColors()
{
    if (state_) {
        state_->setForeground(Qt::black);
        state_->setBackground(Qt::white);
    }
    emit reset();
}

void ForegroundBackgroundWidget::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    painter.fillRect(rect(), palette().window());

    const QColor fg = state_ ? state_->foreground() : QColor(Qt::black);
    const QColor bg = state_ ? state_->background() : QColor(Qt::white);
    const bool fgActive = !state_ || state_->foregroundActive();

    auto drawSwatch = [&painter](const QRect& r, const QColor& color, bool active) {
        painter.fillRect(r, color);
        const QColor border =
            active ? (color.lightness() > 128 ? QColor(20, 20, 20) : QColor(240, 240, 240))
                   : QColor(90, 90, 90);
        painter.setPen(QPen(border, active ? 2 : 1));
        painter.drawRect(r);
    };

    drawSwatch(backgroundRect(), bg, !fgActive);
    drawSwatch(foregroundRect(), fg, fgActive);

    const QRect reset = resetRect();
    painter.fillRect(reset, QColor(Qt::white));
    painter.setPen(QColor(40, 40, 40));
    painter.drawRect(reset.adjusted(0, 0, -1, -1));
    painter.fillRect(QRect(reset.topLeft(), QSize(reset.width() / 2, reset.height() / 2)),
                     QColor(Qt::black));

    // CS6 double-headed swap arrow, top-right of the swatch area.
    const QRect swap = swapRect();
    const int midY = swap.center().y();
    const int left = swap.left() + 1;
    const int right = swap.right() - 1;
    painter.setPen(QPen(palette().color(QPalette::ButtonText), 1));
    painter.drawLine(left, midY, right, midY);
    painter.drawLine(left, midY, left + 3, midY - 3);
    painter.drawLine(left, midY, left + 3, midY + 3);
    painter.drawLine(right, midY, right - 3, midY - 3);
    painter.drawLine(right, midY, right - 3, midY + 3);
}

void ForegroundBackgroundWidget::mousePressEvent(QMouseEvent* event)
{
    if (event->button() != Qt::LeftButton || !state_) {
        QWidget::mousePressEvent(event);
        return;
    }
    const QPoint pos = event->position().toPoint();
    if (resetRect().contains(pos)) {
        resetColors();
        return;
    }
    if (swapRect().contains(pos)) {
        swapForegroundBackground();
        return;
    }
    if (foregroundRect().contains(pos)) {
        state_->setForegroundActive(true);
        const QColor picked = ColorPickerDialog::getColor(state_->foreground(), this,
                                                          tr("Foreground Color"));
        if (picked.isValid()) {
            state_->setForeground(picked);
        }
    } else if (backgroundRect().contains(pos)) {
        state_->setForegroundActive(false);
        const QColor picked = ColorPickerDialog::getColor(state_->background(), this,
                                                          tr("Background Color"));
        if (picked.isValid()) {
            state_->setBackground(picked);
        }
    } else {
        event->ignore();
        return;
    }
    emit clicked();
}

Toolbox::Toolbox(ToolController* controller, ColorState* colors, QWidget* parent)
    : QWidget(parent)
    , controller_(controller)
    , colors_(colors)
{
    setObjectName(QStringLiteral("toolsPanel"));
    setAttribute(Qt::WA_StyledBackground, true);

    auto* layout = new QVBoxLayout(this);
    bodyLayout_ = layout;
    // Track the body's content exactly so a 2->1 column change lowers the
    // content's minimum width instead of leaving the two-column floor behind.
    layout->setSizeConstraint(QLayout::SetMinimumSize);
    layout->setContentsMargins(2, 2, 2, 2);
    layout->setSpacing(4);

    gridWidget_ = new QWidget(this);
    grid_ = new QGridLayout(gridWidget_);
    grid_->setContentsMargins(0, 0, 0, 0);
    grid_->setSpacing(1);

    auto* group = new QButtonGroup(this);
    group->setExclusive(true);

    for (int g = 1; g <= 23; ++g) {
        QList<ToolId> members;
        bool anyImplemented = false;
        for (ToolId id : allToolIds()) {
            if (toolInfo(id).group == g) {
                members << id;
                anyImplemented = anyImplemented || toolImplemented(id);
            }
        }
        if (members.isEmpty()) {
            continue;
        }

        currentByGroup_[g] = groupCurrentTool(g);
        auto* button = new ToolSlotButton(gridWidget_);
        button->setIconSize(QSize(kSlotIconSize, kSlotIconSize));
        button->setFixedSize(kSlotButtonSize, kSlotButtonSize);
        button->setCheckable(true);
        button->setAutoRaise(true);
        button->setEnabled(anyImplemented);
        button->setHasFlyout(members.size() > 1);

        QMenu* menu = nullptr;
        if (members.size() > 1) {
            menu = new QMenu(button);
            for (ToolId member : members) {
                const ToolInfo& info = toolInfo(member);
                QAction* action =
                    menu->addAction(icon(QStringLiteral("tool.") + toolIdName(member)),
                                    QString::fromLatin1(info.label));
                action->setData(int(member));
                action->setEnabled(info.implemented);
                action->setToolTip(info.implemented ? enabledTooltip(info)
                                                    : disabledTooltip(info));
                if (!info.shortcut.isNull()) {
                    action->setShortcut(QKeySequence(QString(info.shortcut)));
                    action->setShortcutVisibleInContextMenu(true);
                    action->setShortcutContext(Qt::WidgetWithChildrenShortcut);
                }
            }
            connect(menu, &QMenu::triggered, this, [this, g](QAction* action) {
                selectMember(g, ToolId(action->data().toInt()));
            });
        }
        slotMenus_ << menu;

        button->onMenu = [this, g]() { showSlotMenu(g); };
        button->onCycle = [this, g]() { cycleGroup(g); };

        slotButtons_ << button;
        group->addButton(button);

        refreshSlot(g);
        if (controller_) {
            connect(button, &QToolButton::clicked, this, [this, g]() {
                const ToolId current = groupCurrentTool(g);
                if (toolImplemented(current)) {
                    controller_->setActiveTool(current);
                }
            });
        }
    }
    reflow();

    layout->addWidget(gridWidget_, 0, Qt::AlignHCenter);

    fgbg_ = new ForegroundBackgroundWidget(colors, this);
    layout->addWidget(fgbg_, 0, Qt::AlignHCenter);

    screenMode_ = new QToolButton(this);
    screenMode_->setObjectName(QStringLiteral("screenModeButton"));
    screenMode_->setFixedSize(kSlotButtonSize, kSlotButtonSize);
    screenMode_->setAutoRaise(true);
    const QIcon screenModeIcon = icon(QStringLiteral("view.screenMode.full"));
    if (!screenModeIcon.isNull()) {
        screenMode_->setIcon(screenModeIcon);
        screenMode_->setIconSize(QSize(kSlotIconSize, kSlotIconSize));
    } else {
        screenMode_->setText(QStringLiteral("Screen Mode"));
    }
    screenMode_->setToolTip(tr("Screen Mode"));
    connect(screenMode_, &QToolButton::clicked, this,
            [this]() { emit screenModeRequested(); });
    layout->addWidget(screenMode_, 0, Qt::AlignHCenter);

    layout->addStretch(1);
    updateContentMetrics();

    if (controller_) {
        connect(controller_, &ToolController::activeToolChanged, this, [this](ToolId id) {
            const int g = toolInfo(id).group;
            currentByGroup_[g] = id;
            refreshSlot(g);
        });
    }
}

void Toolbox::setColumns(int columns)
{
    columns = columns == 2 ? 2 : 1;
    if (columns == columns_) {
        return;
    }
    columns_ = columns;
    reflow();
    updateContentMetrics();
    emit columnsChanged(columns_);
}

int Toolbox::contentWidth(int columns) const
{
    const int slot = kSlotButtonSize;
    const int spacing = grid_ ? grid_->spacing() : 0;
    const int gridWidth = columns * slot + (columns > 1 ? (columns - 1) * spacing : 0);
    const int margins = bodyLayout_
        ? bodyLayout_->contentsMargins().left() + bodyLayout_->contentsMargins().right()
        : 0;
    return gridWidth + margins;
}

void Toolbox::updateContentMetrics()
{
    // The content's minimum width follows the tool grid; the hosting PanelColumn
    // tracks it so the column hugs the grid plus the fg/bg control.
    const int content = contentWidth(columns_);
    setMinimumWidth(content);
    if (fgbg_) {
        fgbg_->setSide(content - (bodyLayout_ ? bodyLayout_->contentsMargins().left()
                                                   + bodyLayout_->contentsMargins().right()
                                               : 0));
    }
    if (QWidget* body = this) {
        body->updateGeometry();
    }
}

int Toolbox::contentWidthForTest() const
{
    return contentWidth(columns_);
}

int Toolbox::foregroundBackgroundWidthForTest() const
{
    return fgbg_ ? fgbg_->width() : 0;
}

void Toolbox::swapForegroundBackground()
{
    if (fgbg_) {
        fgbg_->swapForegroundBackground();
    }
}

void Toolbox::resetForegroundBackground()
{
    if (fgbg_) {
        fgbg_->resetColors();
    }
}

void Toolbox::reflow()
{
    while (QLayoutItem* item = grid_->takeAt(0)) {
        delete item;
    }
    for (int i = 0; i < slotButtons_.size(); ++i) {
        if (columns_ == 2) {
            grid_->addWidget(slotButtons_.at(i), i / 2, i % 2, Qt::AlignHCenter);
        } else {
            grid_->addWidget(slotButtons_.at(i), i, 0, Qt::AlignHCenter);
        }
    }
}

void Toolbox::showSlotMenu(int group)
{
    QMenu* menu = slotMenuForTest(group);
    if (!menu || group < 1 || group > slotButtons_.size()) {
        return;
    }
    QToolButton* button = slotButtons_.at(group - 1);
    const QSize hint = menu->sizeHint();
    QPoint pos = button->mapToGlobal(QPoint(0, button->height()));
    if (QScreen* screen = button->screen()) {
        const QRect avail = screen->availableGeometry();
        if (pos.x() + hint.width() > avail.right()) {
            pos.setX(avail.right() - hint.width());
        }
        if (pos.x() < avail.left()) {
            pos.setX(avail.left());
        }
        if (pos.y() + hint.height() > avail.bottom()) {
            pos.setY(avail.bottom() - hint.height());
        }
    }
    menu->popup(pos);
}

QMenu* Toolbox::slotMenuForTest(int group) const
{
    if (group < 1 || group > slotMenus_.size()) {
        return nullptr;
    }
    return slotMenus_.at(group - 1);
}

QList<QAction*> Toolbox::slotMenuActionsForTest(int group) const
{
    QMenu* menu = slotMenuForTest(group);
    return menu ? menu->actions() : QList<QAction*>();
}

bool Toolbox::hasFlyoutTriangleForTest(int group) const
{
    if (group < 1 || group > slotButtons_.size()) {
        return false;
    }
    return static_cast<ToolSlotButton*>(slotButtons_.at(group - 1))->hasFlyout();
}

void Toolbox::openSlotFlyoutForTest(int group)
{
    showSlotMenu(group);
}

bool Toolbox::handleToolKey(const QChar& key, bool shift)
{
    const int group = toolGroupForKey(key);
    if (group < 1 || !groupHasImplemented(group)) {
        return false;
    }
    if (shift || !shiftKeyForToolSwitch_) {
        cycleGroup(group);
        return true;
    }
    const ToolId current = groupCurrentTool(group);
    if (!toolImplemented(current)) {
        return false;
    }
    selectMember(group, current);
    return true;
}

ToolId Toolbox::groupCurrentTool(int group) const
{
    if (currentByGroup_.contains(group)) {
        return currentByGroup_.value(group);
    }
    ToolId first;
    bool have = false;
    for (ToolId id : allToolIds()) {
        if (toolInfo(id).group != group) {
            continue;
        }
        if (!have) {
            first = id;
            have = true;
        }
        if (toolImplemented(id)) {
            return id;
        }
    }
    return first;
}

void Toolbox::refreshSlot(int group)
{
    if (group < 1 || group > slotButtons_.size()) {
        return;
    }
    QToolButton* button = slotButtons_.at(group - 1);
    const ToolId id = groupCurrentTool(group);
    const ToolInfo& info = toolInfo(id);
    button->setIcon(icon(QStringLiteral("tool.") + toolIdName(id)));
    button->setToolTip(info.implemented ? enabledTooltip(info) : disabledTooltip(info));
    button->setChecked(controller_ && controller_->activeTool() == id);
}

void Toolbox::selectMember(int group, ToolId id)
{
    currentByGroup_[group] = id;
    refreshSlot(group);
    if (controller_ && toolImplemented(id)) {
        controller_->setActiveTool(id);
    }
}

void Toolbox::cycleGroup(int group)
{
    QList<ToolId> enabled;
    for (ToolId id : allToolIds()) {
        if (toolInfo(id).group == group && toolImplemented(id)) {
            enabled << id;
        }
    }
    if (enabled.size() < 2) {
        return;
    }
    const int index = enabled.indexOf(groupCurrentTool(group));
    selectMember(group, enabled.at((index + 1) % enabled.size()));
}

} // namespace pictura
