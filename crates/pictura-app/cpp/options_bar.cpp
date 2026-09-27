#include "options_bar.h"

#include "icons.h"
#include "panels/numeric_field.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/annotations.cxxqt.h"

#include <QtGui/QAction>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QMenu>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

namespace {

struct ModeButton {
    SelectionMode mode;
    const char* label;
    const char* icon;
};

const ModeButton kModes[] = {
    {SelectionMode::New, "New selection", "select.mode.new"},
    {SelectionMode::Add, "Add to selection", "select.mode.add"},
    {SelectionMode::Subtract, "Subtract from selection", "select.mode.subtract"},
    {SelectionMode::Intersect, "Intersect with selection", "select.mode.intersect"},
};

NumericFieldConfig numericConfig(double lo, double hi, double step, int decimals,
                                 const QString& suffix, bool popup, const QString& name)
{
    NumericFieldConfig config;
    config.minimum = lo;
    config.maximum = hi;
    config.step = step;
    config.decimals = decimals;
    config.suffix = suffix;
    config.popup = popup;
    config.objectName = name;
    return config;
}

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
    case ToolId::MagneticLasso:
    case ToolId::Marquee:
    case ToolId::EllipticalMarquee:
        return buildSelectionPage(id);
    case ToolId::MagicWand:
        return buildWandPage(id);
    case ToolId::Crop:
        return buildCropPage(id);
    case ToolId::ColorSampler:
    case ToolId::Ruler:
    case ToolId::Note:
        return buildAnnotationPage(id);
    case ToolId::QuickSelection:
        return buildCombinePage(id, true);
    case ToolId::Brush:
    case ToolId::Pencil:
        return buildPaintPage(id);
    default: {
        auto* page = new QWidget(stack_);
        auto* layout = new QHBoxLayout(page);
        layout->setContentsMargins(4, 2, 4, 2);
        layout->addWidget(toolButton(id, page));
        layout->addStretch(1);
        return page;
    }
    }
}

QWidget* OptionsBar::buildCombinePage(ToolId id, bool withTolerance)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    // Quick Selection has no Intersect mode in CS6.
    addModeButtons(layout, page, id != ToolId::QuickSelection);

    if (withTolerance && controller_) {
        auto* spin = new NumericField(QStringLiteral("Tolerance"),
                                      numericConfig(0, 255, 1, 0, QString(), true,
                                                    QStringLiteral("optionsTolerance")),
                                      page);
        spin->setValue(controller_->tolerance());
        layout->addWidget(spin);
        connect(spin, &NumericField::valueChanged, this,
                [this](double value) { controller_->setTolerance(qRound(value)); });
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

    layout->addStretch(1);
    return page;
}

QWidget* OptionsBar::buildWandPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    addModeButtons(layout, page, true);

    auto* tolerance = new NumericField(QStringLiteral("Tolerance"),
                                       numericConfig(0, 255, 1, 0, QString(), true,
                                                     QStringLiteral("optionsTolerance")),
                                       page);
    tolerance->setValue(controller_ ? controller_->tolerance() : 32);
    layout->addWidget(tolerance);
    if (controller_) {
        connect(tolerance, &NumericField::valueChanged, this,
                [this](double value) { controller_->setTolerance(qRound(value)); });
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

    layout->addStretch(1);
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
        button->setIcon(icon(QString::fromLatin1(modeButton.icon)));
        button->setIconSize(QSize(18, 18));
        button->setToolButtonStyle(Qt::ToolButtonIconOnly);
        button->setToolTip(QString::fromLatin1(modeButton.label));
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
    layout->addWidget(toolButton(id, page));

    addModeButtons(layout, page, true);

    auto* feather = new NumericField(
        QStringLiteral("Feather"),
        numericConfig(0.0, 250.0, 1.0, 1, QStringLiteral("px"), false,
                      QStringLiteral("optionsFeather")),
        page);
    feather->setValue(controller_ ? controller_->feather() : 0.0);
    layout->addWidget(feather);
    if (controller_) {
        connect(feather, &NumericField::valueChanged, this,
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
        auto* ratioW = new NumericField(QString(),
                                        numericConfig(0.1, 100.0, 0.1, 1, QString(), true,
                                                      QStringLiteral("optionsRatioW")),
                                        ratioBox);
        ratioW->setValue(controller_ ? controller_->fixedRatioWidth() : 1.0);
        auto* ratioH = new NumericField(QString(),
                                        numericConfig(0.1, 100.0, 0.1, 1, QString(), true,
                                                      QStringLiteral("optionsRatioH")),
                                        ratioBox);
        ratioH->setValue(controller_ ? controller_->fixedRatioHeight() : 1.0);
        ratioLayout->addWidget(ratioW);
        ratioLayout->addWidget(new QLabel(QStringLiteral(":"), ratioBox));
        ratioLayout->addWidget(ratioH);
        ratioBox->setToolTip(QStringLiteral("Inferred default 1:1."));
        layout->addWidget(ratioBox);

        auto* sizeBox = new QWidget(page);
        auto* sizeLayout = new QHBoxLayout(sizeBox);
        sizeLayout->setContentsMargins(0, 0, 0, 0);
        auto* sizeW = new NumericField(QString(),
                                       numericConfig(1, 10000, 1, 0, QString(), true,
                                                     QStringLiteral("optionsSizeW")),
                                       sizeBox);
        sizeW->setValue(controller_ ? controller_->fixedSizeWidth() : 100);
        auto* sizeH = new NumericField(QString(),
                                       numericConfig(1, 10000, 1, 0, QString(), true,
                                                     QStringLiteral("optionsSizeH")),
                                       sizeBox);
        sizeH->setValue(controller_ ? controller_->fixedSizeHeight() : 100);
        sizeLayout->addWidget(sizeW);
        sizeLayout->addWidget(new QLabel(QStringLiteral("x"), sizeBox));
        sizeLayout->addWidget(sizeH);
        sizeBox->setToolTip(
            QStringLiteral("Inferred default 100x100 px; non-pixel units are deferred."));
        layout->addWidget(sizeBox);

        if (controller_) {
            connect(ratioW, &NumericField::valueChanged, this, [this, ratioH](double v) {
                controller_->setFixedRatio(v, ratioH->value());
            });
            connect(ratioH, &NumericField::valueChanged, this, [this, ratioW](double v) {
                controller_->setFixedRatio(ratioW->value(), v);
            });
            connect(sizeW, &NumericField::valueChanged, this, [this, sizeH](double v) {
                controller_->setFixedSize(qRound(v), qRound(sizeH->value()));
            });
            connect(sizeH, &NumericField::valueChanged, this, [this, sizeW](double v) {
                controller_->setFixedSize(qRound(sizeW->value()), qRound(v));
            });
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

    if (id == ToolId::MagneticLasso) {
        addMagneticFields(layout, page);
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

    layout->addStretch(1);
    return page;
}

void OptionsBar::addMagneticFields(QHBoxLayout* layout, QWidget* page)
{
    auto* width = new NumericField(QStringLiteral("Width"),
                                   numericConfig(1, 256, 1, 0, QStringLiteral("px"), true,
                                                 QStringLiteral("optionsMagneticWidth")),
                                   page);
    auto* contrast = new NumericField(QStringLiteral("Contrast"),
                                      numericConfig(1, 100, 1, 0, QStringLiteral("%"), true,
                                                    QStringLiteral("optionsMagneticContrast")),
                                      page);
    auto* frequency = new NumericField(QStringLiteral("Frequency"),
                                       numericConfig(0, 100, 1, 0, QString(), true,
                                                     QStringLiteral("optionsMagneticFrequency")),
                                       page);
    layout->addWidget(width);
    layout->addWidget(contrast);
    layout->addWidget(frequency);
    if (controller_) {
        width->setValue(controller_->magneticWidth());
        contrast->setValue(controller_->magneticContrast());
        frequency->setValue(controller_->magneticFrequency());
        connect(width, &NumericField::valueChanged, this,
                [this](double v) { controller_->setMagneticWidth(qRound(v)); });
        connect(contrast, &NumericField::valueChanged, this,
                [this](double v) { controller_->setMagneticContrast(qRound(v)); });
        connect(frequency, &NumericField::valueChanged, this,
                [this](double v) { controller_->setMagneticFrequency(qRound(v)); });
        // `[` / `]` change the width from the keyboard.
        connect(controller_, &ToolController::magneticWidthChanged, width,
                [width](int v) { width->setValue(v); });
    }
    // ponytail: pen pressure is not read, so Stylus Pressure is shown off and
    // disabled rather than as a silent no-op.
    auto* pressure = new QCheckBox(QStringLiteral("Stylus Pressure"), page);
    pressure->setEnabled(false);
    pressure->setToolTip(QStringLiteral("Not modelled: tablet pressure is not read."));
    layout->addWidget(pressure);
}

// CS6's Crop bar, as photorust ports it: an aspect-ratio preset, Delete
// Cropped Pixels, and a cancel/commit pair (Esc / Enter do the same).
QWidget* OptionsBar::buildCropPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    struct Preset {
        const char* label;
        double ratio;
    };
    const Preset presets[] = {
        {"Unconstrained", 0.0},    {"1 : 1 (Square)", 1.0}, {"4 : 5 (8:10)", 4.0 / 5.0},
        {"5 : 7", 5.0 / 7.0},      {"2 : 3 (4:6)", 2.0 / 3.0}, {"16 : 9", 16.0 / 9.0},
    };
    auto* ratio = new QComboBox(page);
    ratio->setObjectName(QStringLiteral("optionsCropRatio"));
    ratio->setToolTip(QStringLiteral("Lock the crop box to an aspect ratio"));
    for (const Preset& preset : presets) {
        ratio->addItem(QString::fromLatin1(preset.label), preset.ratio);
    }
    layout->addWidget(ratio);

    auto* deletePixels = new QCheckBox(QStringLiteral("Delete Cropped Pixels"), page);
    deletePixels->setObjectName(QStringLiteral("optionsCropDelete"));
    deletePixels->setChecked(controller_ ? controller_->cropDeletePixels() : true);
    deletePixels->setToolTip(QStringLiteral(
        "Discard the pixels outside the crop rather than keeping them beyond the canvas edge"));
    layout->addWidget(deletePixels);

    auto* cancel = new QToolButton(page);
    cancel->setObjectName(QStringLiteral("optionsCropCancel"));
    cancel->setText(QStringLiteral("\u2718"));
    cancel->setToolTip(QStringLiteral("Cancel the crop (Esc)"));
    layout->addWidget(cancel);
    auto* commit = new QToolButton(page);
    commit->setObjectName(QStringLiteral("optionsCropCommit"));
    commit->setText(QStringLiteral("\u2713"));
    commit->setToolTip(QStringLiteral("Apply the crop (Enter)"));
    layout->addWidget(commit);

    if (controller_) {
        connect(ratio, &QComboBox::currentIndexChanged, this,
                [this, ratio](int index) { controller_->setCropRatio(ratio->itemData(index).toDouble()); });
        connect(deletePixels, &QCheckBox::toggled, this,
                [this](bool on) { controller_->setCropDeletePixels(on); });
        connect(cancel, &QToolButton::clicked, this, [this]() { controller_->cancelCrop(); });
        connect(commit, &QToolButton::clicked, this, [this]() { controller_->commitCrop(); });
    }
    layout->addStretch(1);
    return page;
}

// The annotation tools' bars, as photorust ports them: the Ruler's X, Y, W, H,
// A, D1 readout (pixels and degrees), then Clear for every tool.
// ponytail: no Sample Size for samplers, Straighten for the Ruler, or Author
// and Color for notes yet.
QWidget* OptionsBar::buildAnnotationPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    if (id == ToolId::Ruler) {
        auto* readout = new QLabel(page);
        readout->setObjectName(QStringLiteral("optionsRulerReadout"));
        readout->setTextInteractionFlags(Qt::TextSelectableByMouse);
        layout->addWidget(readout);
        const auto update = [this, readout]() {
            PictureView* v = controller_ ? controller_->view() : nullptr;
            const ::rust::Vec<double> m = v ? ruler_measurement(*v) : ::rust::Vec<double>();
            QStringList fields;
            const char* names[] = {"X", "Y", "W", "H", "A", "D1"};
            for (int i = 0; i < 6; ++i) {
                const QString value = m.size() == 6
                    ? QString::number(m[i], 'f', 1) + (i == 4 ? QStringLiteral("°") : QString())
                    : QString();
                fields << QStringLiteral("%1: %2").arg(QLatin1String(names[i]), value);
            }
            readout->setText(fields.join(QStringLiteral("   ")));
        };
        update();
        if (controller_) {
            connect(controller_, &ToolController::rulerChanged, readout, update);
        }
    }

    auto* clear = new QToolButton(page);
    clear->setObjectName(QStringLiteral("optionsAnnotationClear"));
    clear->setText(id == ToolId::Note ? QStringLiteral("Clear All") : QStringLiteral("Clear"));
    clear->setToolTip(id == ToolId::ColorSampler ? QStringLiteral("Delete every color sampler")
                      : id == ToolId::Note       ? QStringLiteral("Delete every note")
                                                 : QStringLiteral("Remove the measuring line"));
    layout->addWidget(clear);
    if (controller_) {
        connect(clear, &QToolButton::clicked, this, [this]() { controller_->clearAnnotations(); });
    }
    layout->addStretch(1);
    return page;
}

QWidget* OptionsBar::buildPaintPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    auto addField = [&](const QString& label, const QString& name, const QString& suffix, int lo,
                        int hi, int value, void (ToolController::*setter)(int)) {
        auto* field =
            new NumericField(label, numericConfig(lo, hi, 1, 0, suffix, true, name), page);
        field->setValue(value);
        layout->addWidget(field);
        if (controller_) {
            connect(field, &NumericField::valueChanged, this,
                    [this, setter](double v) { (controller_->*setter)(qRound(v)); });
            if (setter == &ToolController::setBrushSize) {
                connect(controller_, &ToolController::brushSizeChanged, field,
                        [field](int size) { field->setValue(size); });
            }
        }
    };

    addField(QStringLiteral("Size"), QStringLiteral("optionsBrushSize"), QString(), 1, 5000,
             controller_ ? controller_->brushSize() : 12, &ToolController::setBrushSize);
    addField(QStringLiteral("Hardness"), QStringLiteral("optionsBrushHardness"),
             QStringLiteral("%"), 0, 100, controller_ ? controller_->brushHardness() : 100,
             &ToolController::setBrushHardness);
    addField(QStringLiteral("Opacity"), QStringLiteral("optionsBrushOpacity"),
             QStringLiteral("%"), 0, 100, controller_ ? controller_->brushOpacity() : 100,
             &ToolController::setBrushOpacity);
    addField(QStringLiteral("Flow"), QStringLiteral("optionsBrushFlow"), QStringLiteral("%"), 0,
             100, controller_ ? controller_->brushFlow() : 100, &ToolController::setBrushFlow);

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

    layout->addStretch(1);
    return page;
}

QToolButton* OptionsBar::toolButton(ToolId id, QWidget* parent)
{
    auto* button = new QToolButton(parent);
    button->setIcon(icon(QStringLiteral("tool.") + toolIdName(id)));
    button->setIconSize(QSize(18, 18));
    button->setToolButtonStyle(Qt::ToolButtonIconOnly);
    button->setToolTip(QString::fromLatin1(toolInfo(id).label));
    auto* menu = new QMenu(button);
    QAction* presets = menu->addAction(QStringLiteral("Presets (coming soon)"));
    presets->setEnabled(false);
    button->setMenu(menu);
    button->setPopupMode(QToolButton::InstantPopup);
    return button;
}

void OptionsBar::showTool(ToolId id)
{
    if (!stack_) {
        return;
    }
    stack_->setCurrentIndex(static_cast<int>(id));
    // A stacked widget reserves the widest page's minimum; ignore the hidden
    // pages so only the active tool's options set the bar's (and window's)
    // minimum width.
    for (int i = 0; i < stack_->count(); ++i) {
        stack_->widget(i)->setSizePolicy(i == stack_->currentIndex() ? QSizePolicy::Preferred
                                                                     : QSizePolicy::Ignored,
                                         QSizePolicy::Preferred);
    }
    stack_->updateGeometry();
}

} // namespace pictura
