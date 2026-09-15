#include "options_bar.h"

#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

namespace {

struct ModeButton {
    SelectionMode mode;
    const char* label;
};

const ModeButton kModes[] = {
    {SelectionMode::New, "New"},
    {SelectionMode::Add, "Add"},
    {SelectionMode::Subtract, "Subtract"},
    {SelectionMode::Intersect, "Intersect"},
};

} // namespace

OptionsBar::OptionsBar(ToolController* controller, QWidget* parent)
    : QToolBar(QStringLiteral("Options"), parent)
    , controller_(controller)
{
    setObjectName(QStringLiteral("optionsBar"));

    stack_ = new QStackedWidget(this);
    // Page order follows allToolIds(), which matches ToolId's integer order, so
    // the stack index is the enum value.
    for (ToolId id : allToolIds()) {
        stack_->addWidget(buildPage(id));
    }
    addWidget(stack_);

    if (controller_) {
        showTool(controller_->activeTool());
    }
}

QWidget* OptionsBar::buildPage(ToolId id)
{
    switch (id) {
    case ToolId::Marquee:
    case ToolId::Lasso:
        return buildCombinePage(id, false);
    case ToolId::QuickSelection:
        return buildCombinePage(id, true);
    default: {
        auto* page = new QWidget(stack_);
        auto* layout = new QHBoxLayout(page);
        layout->setContentsMargins(4, 2, 4, 2);
        layout->addWidget(new QLabel(QString::fromLatin1(toolInfo(id).label), page));
        return page;
    }
    }
}

QWidget* OptionsBar::buildCombinePage(ToolId id, bool withTolerance)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(new QLabel(QString::fromLatin1(toolInfo(id).label), page));

    auto* group = new QButtonGroup(page);
    group->setExclusive(true);
    for (const ModeButton& modeButton : kModes) {
        auto* button = new QToolButton(page);
        button->setText(QString::fromLatin1(modeButton.label));
        button->setCheckable(true);
        button->setChecked(controller_ && controller_->combineMode() == modeButton.mode);
        group->addButton(button);
        layout->addWidget(button);
        if (controller_) {
            connect(button, &QToolButton::clicked, this,
                    [this, modeButton]() { controller_->setCombineMode(modeButton.mode); });
        }
    }

    if (withTolerance && controller_) {
        layout->addWidget(new QLabel(QStringLiteral("Tolerance"), page));
        auto* spin = new QSpinBox(page);
        spin->setRange(0, 255);
        spin->setValue(controller_->tolerance());
        layout->addWidget(spin);
        connect(spin, &QSpinBox::valueChanged, this,
                [this](int value) { controller_->setTolerance(value); });
    }

    return page;
}

void OptionsBar::showTool(ToolId id)
{
    if (stack_) {
        stack_->setCurrentIndex(static_cast<int>(id));
    }
}

} // namespace pictura
