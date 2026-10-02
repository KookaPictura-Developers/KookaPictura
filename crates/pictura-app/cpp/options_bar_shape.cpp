// The options bar of the shape tools (Rectangle, Rounded Rectangle, Ellipse,
// Polygon, Line, Custom Shape). Part of OptionsBar; split from options_bar.cpp
// along the page seam.

#include "options_bar.h"

#include "panels/numeric_field.h"

#include "pictura_app/src/cxxqt_object/shapes.cxxqt.h"

#include <QtCore/QSignalBlocker>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QMenu>
#include <QtWidgets/QWidgetAction>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QToolButton>

namespace pictura {

// Mode (Shape / Path / Pixels), shared by the six tools; the Rounded
// Rectangle adds Radius (10 px), the Polygon Sides (5), the Line Weight (1 px)
// and an Arrowheads pop-up (CS6 keeps it under the geometry gear), and the
// Custom Shape its shape picker.
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
    } else if (id == ToolId::Line) {
        auto* weight = new NumericField(
            QStringLiteral("Weight:"),
            numericConfig(1, 1000, 1, 0, QStringLiteral(" px"), true,
                          QStringLiteral("optionsShapeWeight")),
            page);
        weight->setValue(initial.weight);
        layout->addWidget(weight);
        connect(weight, &NumericField::valueChanged, this,
                [update](double v) { update([v](ShapeOptions& o) { o.weight = v; }); });
        layout->addWidget(buildArrowheadsButton(page, initial, update));
    } else if (id == ToolId::CustomShape) {
        auto* picker = new QComboBox(page);
        picker->setObjectName(QStringLiteral("optionsShapeCustom"));
        picker->setToolTip(QStringLiteral("Click to open Custom Shape picker"));
        picker->setIconSize(QSize(24, 24));
        for (int i = 0; i < shape_custom_count(); ++i) {
            picker->addItem(QIcon(QPixmap::fromImage(shape_custom_preview(i, 48))),
                            shape_custom_name(i));
        }
        picker->setCurrentIndex(initial.custom);
        layout->addWidget(picker);
        connect(picker, &QComboBox::currentIndexChanged, this,
                [update](int i) { update([i](ShapeOptions& o) { o.custom = i; }); });
    }
    layout->addStretch(1);
    return page;
}

// The Line's arrowheads: Start / End and their Width, Length (both % of the
// weight), and Concavity, in a pop-up under one button.
QToolButton* OptionsBar::buildArrowheadsButton(
    QWidget* page, const ShapeOptions& initial,
    const std::function<void(const std::function<void(ShapeOptions&)>&)>& update)
{
    auto* button = new QToolButton(page);
    button->setObjectName(QStringLiteral("optionsShapeArrowheads"));
    button->setText(QStringLiteral("Arrowheads"));
    button->setPopupMode(QToolButton::InstantPopup);
    auto* menu = new QMenu(button);
    auto* form = new QWidget(menu);
    auto* layout = new QFormLayout(form);
    auto* start = new QCheckBox(QStringLiteral("Start"), form);
    start->setObjectName(QStringLiteral("optionsShapeArrowStart"));
    start->setChecked(initial.arrowStart);
    auto* end = new QCheckBox(QStringLiteral("End"), form);
    end->setObjectName(QStringLiteral("optionsShapeArrowEnd"));
    end->setChecked(initial.arrowEnd);
    layout->addRow(start);
    layout->addRow(end);
    connect(start, &QCheckBox::toggled, this,
            [update](bool on) { update([on](ShapeOptions& o) { o.arrowStart = on; }); });
    connect(end, &QCheckBox::toggled, this,
            [update](bool on) { update([on](ShapeOptions& o) { o.arrowEnd = on; }); });
    const auto field = [&](const QString& label, double lo, double hi, double value,
                           const QString& name, double ShapeOptions::*member) {
        auto* f = new NumericField(label, numericConfig(lo, hi, 1, 0, QStringLiteral("%"), true, name),
                                   form);
        f->setValue(value);
        layout->addRow(f);
        connect(f, &NumericField::valueChanged, this, [update, member](double v) {
            update([v, member](ShapeOptions& o) { o.*member = v; });
        });
    };
    field(QStringLiteral("Width:"), 10, 1000, initial.arrowWidth,
          QStringLiteral("optionsShapeArrowWidth"), &ShapeOptions::arrowWidth);
    field(QStringLiteral("Length:"), 10, 5000, initial.arrowLength,
          QStringLiteral("optionsShapeArrowLength"), &ShapeOptions::arrowLength);
    field(QStringLiteral("Concavity:"), -50, 50, initial.arrowConcavity,
          QStringLiteral("optionsShapeArrowConcavity"), &ShapeOptions::arrowConcavity);
    auto* action = new QWidgetAction(menu);
    action->setDefaultWidget(form);
    menu->addAction(action);
    button->setMenu(menu);
    return button;
}

} // namespace pictura
