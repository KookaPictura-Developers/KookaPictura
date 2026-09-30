// The options bars of the retouch tools: Blur, Sharpen, Smudge, and Dodge.
// Part of OptionsBar; split from options_bar.cpp along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

// CS6's Blur, Sharpen, and Smudge bars: the brush tip, Mode (the cut-down list
// a tool working on its own pixels offers), Strength (50 %), and Sample All
// Layers; Sharpen adds Protect Detail and Smudge Finger Painting. Object names
// carry the tool's label (`optionsSharpenMode`, …).
QWidget* OptionsBar::buildRetouchPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    addBrushTipFields(layout, page);

    const QString tool = QString::fromLatin1(toolInfo(id).label);
    const auto name = [&tool](const char* field) {
        return QStringLiteral("options%1%2").arg(tool, QLatin1String(field));
    };
    const RetouchOptions initial = controller_ ? controller_->retouchOptions(id) : RetouchOptions{};
    const auto update = [this, id](auto edit) {
        if (controller_) {
            RetouchOptions o = controller_->retouchOptions(id);
            edit(o);
            controller_->setRetouchOptions(id, o);
        }
    };
    layout->addWidget(new QLabel(QStringLiteral("Mode:"), page));
    auto* mode = new QComboBox(page);
    mode->setObjectName(name("Mode"));
    mode->addItems({QStringLiteral("Normal"), QStringLiteral("Darken"), QStringLiteral("Lighten"),
                    QStringLiteral("Hue"), QStringLiteral("Saturation"), QStringLiteral("Color"),
                    QStringLiteral("Luminosity")});
    mode->setCurrentIndex(initial.mode);
    layout->addWidget(mode);
    connect(mode, &QComboBox::currentIndexChanged, this,
            [update](int i) { update([i](RetouchOptions& o) { o.mode = i; }); });

    auto* strength = new NumericField(
        QStringLiteral("Strength:"),
        numericConfig(1, 100, 1, 0, QStringLiteral("%"), true, name("Strength")), page);
    strength->setValue(initial.strength);
    layout->addWidget(strength);
    connect(strength, &NumericField::valueChanged, this, [update](double v) {
        update([v](RetouchOptions& o) { o.strength = qRound(v); });
    });

    const auto addToggle = [&](const QString& label, const char* field, bool on) {
        auto* box = new QCheckBox(label, page);
        box->setObjectName(name(field));
        box->setChecked(on);
        layout->addWidget(box);
        return box;
    };
    auto* sampleAll = addToggle(QStringLiteral("Sample All Layers"), "SampleAll",
                                initial.sampleAllLayers);
    connect(sampleAll, &QCheckBox::toggled, this,
            [update](bool on) { update([on](RetouchOptions& o) { o.sampleAllLayers = on; }); });
    if (id == ToolId::Sharpen) {
        auto* protect =
            addToggle(QStringLiteral("Protect Detail"), "ProtectDetail", initial.protectDetail);
        connect(protect, &QCheckBox::toggled, this,
                [update](bool on) { update([on](RetouchOptions& o) { o.protectDetail = on; }); });
    } else if (id == ToolId::Smudge) {
        auto* finger =
            addToggle(QStringLiteral("Finger Painting"), "FingerPainting", initial.fingerPainting);
        connect(finger, &QCheckBox::toggled, this,
                [update](bool on) { update([on](RetouchOptions& o) { o.fingerPainting = on; }); });
    }

    layout->addStretch(1);
    return page;
}

// CS6's Dodge bar: the brush tip, Range (Midtones), Exposure (50 %), and
// Protect Tones.
// ponytail: no Airbrush toggle.
QWidget* OptionsBar::buildDodgePage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));
    addBrushTipFields(layout, page);

    const ToneOptions initial = controller_ ? controller_->toneOptions() : ToneOptions{};
    const auto update = [this](auto edit) {
        if (controller_) {
            ToneOptions o = controller_->toneOptions();
            edit(o);
            controller_->setToneOptions(o);
        }
    };
    layout->addWidget(new QLabel(QStringLiteral("Range:"), page));
    auto* range = new QComboBox(page);
    range->setObjectName(QStringLiteral("optionsDodgeRange"));
    range->addItems(
        {QStringLiteral("Shadows"), QStringLiteral("Midtones"), QStringLiteral("Highlights")});
    range->setCurrentIndex(initial.range);
    layout->addWidget(range);
    connect(range, &QComboBox::currentIndexChanged, this,
            [update](int i) { update([i](ToneOptions& o) { o.range = i; }); });

    auto* exposure = new NumericField(
        QStringLiteral("Exposure:"),
        numericConfig(1, 100, 1, 0, QStringLiteral("%"), true,
                      QStringLiteral("optionsDodgeExposure")),
        page);
    exposure->setValue(initial.exposure);
    layout->addWidget(exposure);
    connect(exposure, &NumericField::valueChanged, this, [update](double v) {
        update([v](ToneOptions& o) { o.exposure = qRound(v); });
    });

    auto* protect = new QCheckBox(QStringLiteral("Protect Tones"), page);
    protect->setObjectName(QStringLiteral("optionsDodgeProtectTones"));
    protect->setChecked(initial.protectTones);
    layout->addWidget(protect);
    connect(protect, &QCheckBox::toggled, this,
            [update](bool on) { update([on](ToneOptions& o) { o.protectTones = on; }); });

    layout->addStretch(1);
    return page;
}

} // namespace pictura
