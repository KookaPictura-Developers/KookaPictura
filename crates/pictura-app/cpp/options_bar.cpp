#include "options_bar.h"

#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
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
    case ToolId::Brush:
    case ToolId::Pencil:
        return buildPaintPage(id);
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

QWidget* OptionsBar::buildPaintPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(new QLabel(QString::fromLatin1(toolInfo(id).label), page));

    auto addSpin = [&](const QString& label, int lo, int hi, int value,
                       void (ToolController::*setter)(int)) {
        layout->addWidget(new QLabel(label, page));
        auto* spin = new QSpinBox(page);
        spin->setRange(lo, hi);
        spin->setValue(value);
        layout->addWidget(spin);
        if (controller_) {
            connect(spin, &QSpinBox::valueChanged, this,
                    [this, setter](int v) { (controller_->*setter)(v); });
        }
    };

    addSpin(QStringLiteral("Size"), 1, 5000, controller_ ? controller_->brushSize() : 12,
            &ToolController::setBrushSize);
    addSpin(QStringLiteral("Hardness"), 0, 100,
            controller_ ? controller_->brushHardness() : 100, &ToolController::setBrushHardness);
    addSpin(QStringLiteral("Opacity"), 0, 100,
            controller_ ? controller_->brushOpacity() : 100, &ToolController::setBrushOpacity);
    addSpin(QStringLiteral("Flow"), 0, 100, controller_ ? controller_->brushFlow() : 100,
            &ToolController::setBrushFlow);

    layout->addWidget(new QLabel(QStringLiteral("Mode"), page));
    auto* combo = new QComboBox(page);
    combo->addItem(QStringLiteral("Normal"), QStringLiteral("normal"));
    combo->addItem(QStringLiteral("Dissolve"), QStringLiteral("dissolve"));
    combo->addItem(QStringLiteral("Behind"), QStringLiteral("behind"));
    combo->addItem(QStringLiteral("Clear"), QStringLiteral("clear"));
    if (controller_) {
        combo->setCurrentIndex(combo->findData(controller_->brushMode()));
        connect(combo, &QComboBox::currentIndexChanged, this, [this, combo](int) {
            controller_->setBrushMode(combo->currentData().toString());
        });
    }
    layout->addWidget(combo);

    if (id == ToolId::Pencil) {
        auto* check = new QCheckBox(QStringLiteral("Auto Erase"), page);
        if (controller_) {
            check->setChecked(controller_->autoErase());
            connect(check, &QCheckBox::toggled, this,
                    [this](bool on) { controller_->setAutoErase(on); });
        }
        layout->addWidget(check);
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
