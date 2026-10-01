// The options bars of the Pen and Freeform Pen tools.
// Part of OptionsBar; split from options_bar.cpp along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

// The Pen bar: Auto Add/Delete (on) and Rubber Band (off; CS6 keeps it in a
// pop-up). The Freeform Pen bar: Curve Fit (2 px).
// ponytail: no Shape / Path / Pixels mode, path operations, or the Freeform
// Pen's Magnetic option; the tools always draw the Work Path.
QWidget* OptionsBar::buildPenPage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    const PenOptions initial = controller_ ? controller_->penOptions() : PenOptions{};
    const auto update = [this](auto edit) {
        if (controller_) {
            PenOptions o = controller_->penOptions();
            edit(o);
            controller_->setPenOptions(o);
        }
    };
    if (id == ToolId::Pen) {
        auto* autoAdd = new QCheckBox(QStringLiteral("Auto Add/Delete"), page);
        autoAdd->setObjectName(QStringLiteral("optionsPenAutoAddDelete"));
        autoAdd->setChecked(initial.autoAddDelete);
        layout->addWidget(autoAdd);
        connect(autoAdd, &QCheckBox::toggled, this,
                [update](bool on) { update([on](PenOptions& o) { o.autoAddDelete = on; }); });
        auto* rubber = new QCheckBox(QStringLiteral("Rubber Band"), page);
        rubber->setObjectName(QStringLiteral("optionsPenRubberBand"));
        rubber->setChecked(initial.rubberBand);
        layout->addWidget(rubber);
        connect(rubber, &QCheckBox::toggled, this,
                [update](bool on) { update([on](PenOptions& o) { o.rubberBand = on; }); });
    } else {
        auto* fit = new NumericField(
            QStringLiteral("Curve Fit:"),
            numericConfig(0.5, 10, 0.5, 1, QStringLiteral(" px"), false,
                          QStringLiteral("optionsFreeformPenCurveFit")),
            page);
        fit->setValue(initial.curveFit);
        layout->addWidget(fit);
        connect(fit, &NumericField::valueChanged, this,
                [update](double v) { update([v](PenOptions& o) { o.curveFit = v; }); });
    }
    layout->addStretch(1);
    return page;
}

} // namespace pictura
