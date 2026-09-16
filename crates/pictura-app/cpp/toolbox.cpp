#include "toolbox.h"

#include "icons.h"
#include "panels/color_panel.h"
#include "tools.h"

#include <QtCore/QSize>
#include <QtCore/QTimer>
#include <QtGui/QAction>
#include <QtGui/QKeySequence>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QMenu>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

#include <functional>

namespace pictura {

namespace {

constexpr int kSwatchSize = 22;
constexpr int kWidgetSize = 40;
constexpr int kResetSize = 12;

// A slot button: the base tool button plus the CS6 interactions the stock
// class lacks — right-click and press-and-hold open the flyout, Alt-click
// cycles the group's implemented members.
class ToolSlotButton : public QToolButton {
public:
    explicit ToolSlotButton(QWidget* parent = nullptr)
        : QToolButton(parent)
    {
        holdTimer_.setSingleShot(true);
        holdTimer_.setInterval(350);
        connect(&holdTimer_, &QTimer::timeout, this, [this]() {
            held_ = true;
            if (onMenu) {
                onMenu();
            }
        });
    }

    std::function<void()> onMenu;
    std::function<void()> onCycle;

protected:
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

} // namespace

ForegroundBackgroundWidget::ForegroundBackgroundWidget(ColorState* state, QWidget* parent)
    : QWidget(parent)
    , state_(state)
{
    setFixedSize(kWidgetSize, kWidgetSize);
    setToolTip(tr("Foreground / background colors — click a swatch to make it active"));
    if (state_) {
        connect(state_, &ColorState::foregroundChanged, this, [this](const QColor&) { update(); });
        connect(state_, &ColorState::backgroundChanged, this, [this](const QColor&) { update(); });
        connect(state_, &ColorState::activeChanged, this, [this](bool) { update(); });
    }
}

QRect ForegroundBackgroundWidget::foregroundRect() const
{
    return QRect(1, 1, kSwatchSize, kSwatchSize);
}

QRect ForegroundBackgroundWidget::backgroundRect() const
{
    return QRect(kWidgetSize - kSwatchSize - 1, kWidgetSize - kSwatchSize - 1, kSwatchSize,
                 kSwatchSize);
}

QRect ForegroundBackgroundWidget::resetRect() const
{
    return QRect(2, kWidgetSize - kResetSize - 2, kResetSize, kResetSize);
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
    if (foregroundRect().contains(pos)) {
        state_->setForegroundActive(true);
    } else if (backgroundRect().contains(pos)) {
        state_->setForegroundActive(false);
    } else {
        event->ignore();
        return;
    }
    emit clicked();
}

Toolbox::Toolbox(ToolController* controller, ColorState* colors, QWidget* parent)
    : QDockWidget(QStringLiteral("Tools"), parent)
    , controller_(controller)
    , colors_(colors)
{
    setObjectName(QStringLiteral("toolsPanel"));

    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);
    layout->setContentsMargins(2, 2, 2, 2);
    layout->setSpacing(4);

    auto* columnWidget = new QWidget(body);
    auto* column = new QVBoxLayout(columnWidget);
    column->setContentsMargins(0, 0, 0, 0);
    column->setSpacing(1);

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
        auto* button = new ToolSlotButton(columnWidget);
        button->setIconSize(QSize(20, 20));
        button->setFixedSize(30, 30);
        button->setCheckable(true);
        button->setAutoRaise(true);
        button->setEnabled(anyImplemented);
        if (members.size() > 1) {
            button->setPopupMode(QToolButton::MenuButtonPopup);
            auto* menu = new QMenu(button);
            for (ToolId member : members) {
                const ToolInfo& info = toolInfo(member);
                QAction* action =
                    menu->addAction(icon(QStringLiteral("tool.") + toolIdName(member)),
                                    QString::fromLatin1(info.label));
                action->setData(int(member));
                action->setEnabled(info.implemented);
                action->setToolTip(info.implemented ? enabledTooltip(info)
                                                    : disabledTooltip(info));
            }
            button->setMenu(menu);
            connect(menu, &QMenu::triggered, this, [this, g](QAction* action) {
                selectMember(g, ToolId(action->data().toInt()));
            });
        }

        button->onMenu = [button]() { button->showMenu(); };
        button->onCycle = [this, g]() { cycleGroup(g); };

        slotButtons_ << button;
        group->addButton(button);
        column->addWidget(button, 0, Qt::AlignHCenter);

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
    layout->addWidget(columnWidget, 0, Qt::AlignHCenter);

    layout->addWidget(new ForegroundBackgroundWidget(colors, body), 0, Qt::AlignHCenter);

    auto* screenMode = new QToolButton(body);
    screenMode->setObjectName(QStringLiteral("screenModeButton"));
    screenMode->setFixedSize(30, 30);
    screenMode->setAutoRaise(true);
    const QIcon screenModeIcon = icon(QStringLiteral("view.screenMode.full"));
    if (!screenModeIcon.isNull()) {
        screenMode->setIcon(screenModeIcon);
        screenMode->setIconSize(QSize(20, 20));
    } else {
        screenMode->setText(QStringLiteral("Screen Mode"));
    }
    screenMode->setToolTip(tr("Screen Mode"));
    connect(screenMode, &QToolButton::clicked, this, [this]() { emit screenModeRequested(); });
    layout->addWidget(screenMode, 0, Qt::AlignHCenter);

    layout->addStretch(1);
    setWidget(body);
    setMinimumWidth(66);

    if (controller_) {
        connect(controller_, &ToolController::activeToolChanged, this, [this](ToolId id) {
            const int g = toolInfo(id).group;
            currentByGroup_[g] = id;
            refreshSlot(g);
        });
    }
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
    if (info.implemented) {
        button->setToolTip(enabledTooltip(info));
        if (id != ToolId::Brush && id != ToolId::Pencil && !info.shortcut.isNull()) {
            button->setShortcut(QKeySequence(QString(info.shortcut)));
        } else {
            button->setShortcut(QKeySequence());
        }
    } else {
        button->setToolTip(disabledTooltip(info));
        button->setShortcut(QKeySequence());
    }
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
