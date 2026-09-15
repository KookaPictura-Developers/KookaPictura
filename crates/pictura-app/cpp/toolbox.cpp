#include "toolbox.h"

#include "icons.h"
#include "panels/color_panel.h"
#include "tools.h"

#include <QtCore/QSize>
#include <QtGui/QKeySequence>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

constexpr int kSwatchSize = 22;
constexpr int kWidgetSize = 40;
constexpr int kResetSize = 12;

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
    , colors_(colors)
{
    setObjectName(QStringLiteral("toolsPanel"));

    auto* body = new QWidget(this);
    auto* layout = new QVBoxLayout(body);
    layout->setContentsMargins(2, 2, 2, 2);
    layout->setSpacing(4);

    auto* gridWidget = new QWidget(body);
    auto* grid = new QGridLayout(gridWidget);
    grid->setContentsMargins(0, 0, 0, 0);
    grid->setSpacing(2);

    auto* group = new QButtonGroup(this);
    group->setExclusive(true);

    const QList<ToolId> ids = allToolIds();
    for (int i = 0; i < ids.size(); ++i) {
        const ToolId id = ids.at(i);
        const ToolInfo& info = toolInfo(id);
        auto* button = new QToolButton(gridWidget);
        button->setIcon(icon(QStringLiteral("tool.") + toolIdName(id)));
        button->setIconSize(QSize(20, 20));
        button->setFixedSize(30, 30);
        button->setCheckable(true);
        button->setAutoRaise(true);
        if (id != ToolId::Brush && id != ToolId::Pencil) {
            button->setShortcut(QKeySequence(QString(info.shortcut)));
        }
        button->setToolTip(QStringLiteral("%1 (%2) — %3")
                               .arg(QString::fromLatin1(info.label), QString(info.shortcut),
                                    QString::fromLatin1(info.hint)));
        grid->addWidget(button, i / 2, i % 2);
        group->addButton(button);

        if (controller) {
            button->setChecked(controller->activeTool() == id);
            connect(button, &QToolButton::clicked, controller,
                    [controller, id]() { controller->setActiveTool(id); });
            connect(controller, &ToolController::activeToolChanged, button,
                    [button, id](ToolId active) { button->setChecked(active == id); });
        }
    }
    layout->addWidget(gridWidget, 0, Qt::AlignHCenter);

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
}

} // namespace pictura
