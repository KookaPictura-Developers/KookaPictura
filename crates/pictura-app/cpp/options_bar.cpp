#include "options_bar.h"

#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
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
    case ToolId::Lasso:
    case ToolId::PolygonalLasso:
    case ToolId::Marquee:
    case ToolId::EllipticalMarquee:
        return buildSelectionPage(id);
    case ToolId::MagicWand:
        return buildWandPage(id);
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

    // Quick Selection has no Intersect mode in CS6.
    addModeButtons(layout, page, id != ToolId::QuickSelection);

    if (withTolerance && controller_) {
        layout->addWidget(new QLabel(QStringLiteral("Tolerance"), page));
        auto* spin = new QSpinBox(page);
        spin->setRange(0, 255);
        spin->setValue(controller_->tolerance());
        layout->addWidget(spin);
        connect(spin, &QSpinBox::valueChanged, this,
                [this](int value) { controller_->setTolerance(value); });
    }

    if (id == ToolId::QuickSelection) {
        // ponytail: the brush pop-up and Refine Edge are deferred; Sample All
        // Layers and Auto-Enhance have no engine behind them, so they are shown
        // checked but disabled rather than as silent no-ops.
        auto* sample = new QCheckBox(QStringLiteral("Sample All Layers"), page);
        sample->setChecked(true);
        sample->setEnabled(false);
        sample->setToolTip(
            QStringLiteral("Not modelled: the wand samples the visible composite."));
        layout->addWidget(sample);
        auto* enhance = new QCheckBox(QStringLiteral("Auto-Enhance"), page);
        enhance->setChecked(true);
        enhance->setEnabled(false);
        enhance->setToolTip(
            QStringLiteral("Not modelled: Quick Selection has no edge-flow refinement."));
        layout->addWidget(enhance);
    }

    return page;
}

QWidget* OptionsBar::buildWandPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(new QLabel(QString::fromLatin1(toolInfo(id).label), page));

    addModeButtons(layout, page, true);

    layout->addWidget(new QLabel(QStringLiteral("Tolerance"), page));
    auto* tolerance = new QSpinBox(page);
    tolerance->setRange(0, 255);
    tolerance->setValue(controller_ ? controller_->tolerance() : 32);
    layout->addWidget(tolerance);
    if (controller_) {
        connect(tolerance, &QSpinBox::valueChanged, this,
                [this](int value) { controller_->setTolerance(value); });
    }

    auto* contiguous = new QCheckBox(QStringLiteral("Contiguous"), page);
    contiguous->setChecked(controller_ ? controller_->contiguous() : true);
    layout->addWidget(contiguous);
    if (controller_) {
        connect(contiguous, &QCheckBox::toggled, this,
                [this](bool on) { controller_->setContiguous(on); });
    }

    // ponytail: the engine rasteriser is binary (pixel-centre) and the wand
    // samples the visible composite; both controls stay visible but disabled.
    auto* antiAlias = new QCheckBox(QStringLiteral("Anti-alias"), page);
    antiAlias->setChecked(controller_ ? controller_->antiAlias() : true);
    antiAlias->setEnabled(false);
    antiAlias->setToolTip(
        QStringLiteral("Not modelled: the selection rasteriser is binary (pixel-centre)."));
    layout->addWidget(antiAlias);
    auto* sample = new QCheckBox(QStringLiteral("Sample All Layers"), page);
    sample->setChecked(controller_ ? controller_->sampleAllLayers() : true);
    sample->setEnabled(false);
    sample->setToolTip(QStringLiteral("Not modelled: the wand samples the visible composite."));
    layout->addWidget(sample);

    return page;
}

void OptionsBar::addModeButtons(QHBoxLayout* layout, QWidget* page, bool withIntersect)
{
    auto* group = new QButtonGroup(page);
    group->setExclusive(true);
    for (const ModeButton& modeButton : kModes) {
        if (!withIntersect && modeButton.mode == SelectionMode::Intersect) {
            continue;
        }
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
}

QWidget* OptionsBar::buildSelectionPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(new QLabel(QString::fromLatin1(toolInfo(id).label), page));

    addModeButtons(layout, page, true);

    layout->addWidget(new QLabel(QStringLiteral("Feather"), page));
    auto* feather = new QDoubleSpinBox(page);
    feather->setRange(0.0, 250.0);
    feather->setDecimals(1);
    feather->setSuffix(QStringLiteral(" px"));
    feather->setValue(controller_ ? controller_->feather() : 0.0);
    layout->addWidget(feather);
    if (controller_) {
        connect(feather, &QDoubleSpinBox::valueChanged, this,
                [this](double value) { controller_->setFeather(value); });
    }

    const bool isMarquee = id == ToolId::Marquee || id == ToolId::EllipticalMarquee;
    if (isMarquee) {
        layout->addWidget(new QLabel(QStringLiteral("Style"), page));
        auto* styleCombo = new QComboBox(page);
        styleCombo->addItem(QStringLiteral("Normal"), int(MarqueeStyle::Normal));
        styleCombo->addItem(QStringLiteral("Fixed Ratio"), int(MarqueeStyle::FixedRatio));
        styleCombo->addItem(QStringLiteral("Fixed Size"), int(MarqueeStyle::FixedSize));
        layout->addWidget(styleCombo);

        // ponytail: 1:1 and 100x100 are inferred defaults (CS6 Help does not
        // state shipped values); the in/cm unit combo is deferred, so size is px.
        auto* ratioBox = new QWidget(page);
        auto* ratioLayout = new QHBoxLayout(ratioBox);
        ratioLayout->setContentsMargins(0, 0, 0, 0);
        auto* ratioW = new QDoubleSpinBox(ratioBox);
        ratioW->setRange(0.1, 100.0);
        ratioW->setDecimals(1);
        ratioW->setValue(controller_ ? controller_->fixedRatioWidth() : 1.0);
        auto* ratioH = new QDoubleSpinBox(ratioBox);
        ratioH->setRange(0.1, 100.0);
        ratioH->setDecimals(1);
        ratioH->setValue(controller_ ? controller_->fixedRatioHeight() : 1.0);
        ratioLayout->addWidget(ratioW);
        ratioLayout->addWidget(new QLabel(QStringLiteral(":"), ratioBox));
        ratioLayout->addWidget(ratioH);
        ratioBox->setToolTip(
            QStringLiteral("Inferred default 1:1; CS6 Help does not state shipped values."));
        layout->addWidget(ratioBox);

        auto* sizeBox = new QWidget(page);
        auto* sizeLayout = new QHBoxLayout(sizeBox);
        sizeLayout->setContentsMargins(0, 0, 0, 0);
        auto* sizeW = new QSpinBox(sizeBox);
        sizeW->setRange(1, 10000);
        sizeW->setValue(controller_ ? controller_->fixedSizeWidth() : 100);
        auto* sizeH = new QSpinBox(sizeBox);
        sizeH->setRange(1, 10000);
        sizeH->setValue(controller_ ? controller_->fixedSizeHeight() : 100);
        sizeLayout->addWidget(sizeW);
        sizeLayout->addWidget(new QLabel(QStringLiteral("x"), sizeBox));
        sizeLayout->addWidget(sizeH);
        sizeBox->setToolTip(
            QStringLiteral("Inferred default 100x100 px; non-pixel units are deferred."));
        layout->addWidget(sizeBox);

        if (controller_) {
            connect(ratioW, &QDoubleSpinBox::valueChanged, this, [this, ratioH](double v) {
                controller_->setFixedRatio(v, ratioH->value());
            });
            connect(ratioH, &QDoubleSpinBox::valueChanged, this, [this, ratioW](double v) {
                controller_->setFixedRatio(ratioW->value(), v);
            });
            connect(sizeW, &QSpinBox::valueChanged, this,
                    [this, sizeH](int v) { controller_->setFixedSize(v, sizeH->value()); });
            connect(sizeH, &QSpinBox::valueChanged, this,
                    [this, sizeW](int v) { controller_->setFixedSize(sizeW->value(), v); });
        }

        const auto applyStyleVisibility = [ratioBox, sizeBox](int style) {
            ratioBox->setVisible(style == int(MarqueeStyle::FixedRatio));
            sizeBox->setVisible(style == int(MarqueeStyle::FixedSize));
        };
        const int initialStyle =
            controller_ ? int(controller_->marqueeStyle()) : int(MarqueeStyle::Normal);
        styleCombo->setCurrentIndex(styleCombo->findData(initialStyle));
        applyStyleVisibility(initialStyle);
        if (controller_) {
            connect(styleCombo, &QComboBox::currentIndexChanged, this,
                    [this, styleCombo, applyStyleVisibility](int) {
                        const int style = styleCombo->currentData().toInt();
                        controller_->setMarqueeStyle(static_cast<MarqueeStyle>(style));
                        applyStyleVisibility(style);
                    });
        }
    }

    if (id != ToolId::Marquee) {
        // ponytail: the engine rasterises at the pixel centre (binary coverage);
        // a fractional-coverage rasteriser is new engine work, so the control is
        // visible but disabled rather than a silent no-op.
        auto* antiAlias = new QCheckBox(QStringLiteral("Anti-alias"), page);
        antiAlias->setChecked(true);
        antiAlias->setEnabled(false);
        antiAlias->setToolTip(QStringLiteral(
            "Not modelled: the selection rasteriser is binary (pixel-centre)."));
        layout->addWidget(antiAlias);
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
