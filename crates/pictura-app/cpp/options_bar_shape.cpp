// The options bar of the shape tools (Rectangle, Rounded Rectangle, Ellipse,
// Polygon). Part of OptionsBar; split from options_bar.cpp along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include <QtCore/QSignalBlocker>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

// Mode (Shape / Path / Pixels), shared by the four tools; the Rounded
// Rectangle adds Radius (10 px) and the Polygon Sides (5).
QWidget* OptionsBar::buildShapePage(ToolId id)
{
    auto* page = new QWidget(stack_);
    auto* layout = new QHBoxLayout(page);
    layout->setContentsMargins(4, 2, 4, 2);
    layout->addWidget(toolButton(id, page));

    const ShapeOptions initial = controller_ ? controller_->shapeOptions() : ShapeOptions{};
    const auto update = [this](auto edit) {
        if (controller_) {
            ShapeOptions o = controller_->shapeOptions();
            edit(o);
            controller_->setShapeOptions(o);
        }
    };

    auto* mode = new QComboBox(page);
    mode->setObjectName(QStringLiteral("optionsShapeMode"));
    mode->setToolTip(QStringLiteral("Pick tool mode"));
    mode->addItems({QStringLiteral("Shape"), QStringLiteral("Path"), QStringLiteral("Pixels")});
    mode->setCurrentIndex(initial.mode);
    layout->addWidget(mode);
    // Each tool has its own page, so the other pages' Mode follows this one.
    connect(mode, &QComboBox::currentIndexChanged, this, [this, update, mode](int i) {
        for (auto* other : stack_->findChildren<QComboBox*>(mode->objectName())) {
            const QSignalBlocker block(other);
            other->setCurrentIndex(i);
        }
        update([i](ShapeOptions& o) { o.mode = i; });
    });

    if (id == ToolId::RoundedRectangle) {
        auto* radius = new NumericField(
            QStringLiteral("Radius:"),
            numericConfig(0, 1000, 1, 0, QStringLiteral(" px"), true,
                          QStringLiteral("optionsShapeRadius")),
            page);
        radius->setValue(initial.radius);
        layout->addWidget(radius);
        connect(radius, &NumericField::valueChanged, this,
                [update](double v) { update([v](ShapeOptions& o) { o.radius = v; }); });
    } else if (id == ToolId::Polygon) {
        auto* sides = new NumericField(
            QStringLiteral("Sides:"),
            numericConfig(3, 100, 1, 0, QString(), true, QStringLiteral("optionsShapeSides")),
            page);
        sides->setValue(initial.sides);
        layout->addWidget(sides);
        connect(sides, &NumericField::valueChanged, this,
                [update](double v) { update([v](ShapeOptions& o) { o.sides = qRound(v); }); });
    }
    layout->addStretch(1);
    return page;
}

} // namespace pictura
