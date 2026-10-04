#include "character_panel.h"

#include "color_picker_dialog.h"
#include "options_bar.h"
#include "tools.h"

#include <QtCore/QLocale>
#include <QtCore/QSignalBlocker>
#include <QtGui/QDoubleValidator>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFontComboBox>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

namespace {

QString sizeText(double size)
{
    return QString::number(size, 'g', 6);
}

// A control CS6 has but the type model does not: shown for shape, disabled.
template <typename T>
T* unmodelled(T* widget, const QString& tip)
{
    widget->setEnabled(false);
    widget->setToolTip(tip + QStringLiteral(" — not implemented yet"));
    return widget;
}

} // namespace

CharacterPanel::CharacterPanel(QWidget* parent)
    : QWidget(parent)
{
    auto* root = new QVBoxLayout(this);
    root->setContentsMargins(6, 6, 6, 6);
    root->setSpacing(5);

    family_ = new QFontComboBox(this);
    family_->setObjectName(QStringLiteral("characterFamily"));
    family_->setToolTip(QStringLiteral("Set the font family"));
    // A font combo asks for a very wide minimum; the panel column must not scroll.
    family_->setSizeAdjustPolicy(QComboBox::AdjustToMinimumContentsLengthWithIcon);
    family_->setMinimumContentsLength(10);
    root->addWidget(family_);

    auto* style = new QComboBox(this);
    style->addItem(QStringLiteral("Regular"));
    root->addWidget(unmodelled(style, QStringLiteral("Font style")));

    auto* grid = new QGridLayout();
    grid->setSpacing(4);
    grid->setContentsMargins(0, 4, 0, 0);
    // Fields may shrink below their hint so the panel never widens the column.
    const auto addField = [&](int row, int col, const QString& label, QWidget* field) {
        field->setMinimumWidth(48);
        grid->addWidget(new QLabel(label, this), row, col * 2);
        grid->addWidget(field, row, col * 2 + 1);
    };

    size_ = new QComboBox(this);
    size_->setObjectName(QStringLiteral("characterSize"));
    size_->setToolTip(QStringLiteral("Set the font size (px)"));
    size_->setEditable(true);
    size_->setInsertPolicy(QComboBox::NoInsert);
    auto* validator = new QDoubleValidator(1.0, 1296.0, 1, size_);
    validator->setNotation(QDoubleValidator::StandardNotation);
    size_->setValidator(validator);
    for (const int px : typeSizes()) {
        size_->addItem(QString::number(px));
    }
    addField(0, 0, QStringLiteral("Size:"), size_);

    auto* leading = new QComboBox(this);
    leading->addItem(QStringLiteral("(Auto)"));
    addField(0, 1, QStringLiteral("Leading:"), unmodelled(leading, QStringLiteral("Leading")));

    auto* kerning = new QComboBox(this);
    kerning->addItem(QStringLiteral("Metrics"));
    addField(1, 0, QStringLiteral("Kerning:"), unmodelled(kerning, QStringLiteral("Kerning")));

    auto* tracking = new QSpinBox(this);
    tracking->setRange(-1000, 10000);
    addField(1, 1, QStringLiteral("Tracking:"), unmodelled(tracking, QStringLiteral("Tracking")));

    auto* vScale = new QSpinBox(this);
    vScale->setRange(0, 1000);
    vScale->setValue(100);
    vScale->setSuffix(QStringLiteral("%"));
    addField(2, 0, QStringLiteral("Vert. Scale:"),
             unmodelled(vScale, QStringLiteral("Vertical scale")));

    auto* hScale = new QSpinBox(this);
    hScale->setRange(0, 1000);
    hScale->setValue(100);
    hScale->setSuffix(QStringLiteral("%"));
    addField(2, 1, QStringLiteral("Horiz. Scale:"),
             unmodelled(hScale, QStringLiteral("Horizontal scale")));

    auto* baseline = new QDoubleSpinBox(this);
    baseline->setRange(-1000, 1000);
    baseline->setSuffix(QStringLiteral(" pt"));
    addField(3, 0, QStringLiteral("Baseline:"),
             unmodelled(baseline, QStringLiteral("Baseline shift")));

    color_ = new QToolButton(this);
    color_->setObjectName(QStringLiteral("characterColor"));
    color_->setToolTip(QStringLiteral("Set the text color"));
    color_->setAutoRaise(true);
    color_->setIconSize(QSize(16, 16));
    addField(3, 1, QStringLiteral("Color:"), color_);

    antialias_ = new QComboBox(this);
    antialias_->setObjectName(QStringLiteral("characterAntialias"));
    antialias_->setToolTip(QStringLiteral("Set the anti-aliasing method"));
    antialias_->addItems({QStringLiteral("None"), QStringLiteral("Sharp")});
    addField(4, 0, QStringLiteral("Anti-alias:"), antialias_);

    grid->setColumnStretch(1, 1);
    grid->setColumnStretch(3, 1);
    root->addLayout(grid);
    root->addStretch(1);
}

void CharacterPanel::setController(ToolController* controller)
{
    controller_ = controller;
    if (!controller_) {
        return;
    }
    const auto update = [this](auto edit) {
        TypeOptions o = controller_->typeOptions();
        edit(o);
        controller_->setTypeOptions(o);
    };
    connect(controller_, &ToolController::typeOptionsChanged, this, &CharacterPanel::refresh);
    connect(family_, &QFontComboBox::currentFontChanged, this, [update](const QFont& font) {
        update([&font](TypeOptions& o) { o.family = font.family(); });
    });
    // As on the options bar, a size applies on a pick or Enter, not per keystroke.
    connect(size_, &QComboBox::activated, this, &CharacterPanel::applySize);
    connect(size_->lineEdit(), &QLineEdit::editingFinished, this, &CharacterPanel::applySize);
    // The swatch sets the foreground, which the text colour follows.
    connect(color_, &QToolButton::clicked, this, [this]() {
        const QColor picked = ColorPickerDialog::getColor(controller_->typeOptions().color, this,
                                                          QStringLiteral("Text Color"));
        if (picked.isValid()) {
            controller_->setForeground(picked);
        }
    });
    connect(antialias_, &QComboBox::currentIndexChanged, this,
            [update](int i) { update([i](TypeOptions& o) { o.antialias = i == 1; }); });
    refresh();
}

void CharacterPanel::refresh()
{
    if (!controller_) {
        return;
    }
    const TypeOptions o = controller_->typeOptions();
    const QSignalBlocker blockFamily(family_);
    const QSignalBlocker blockAntialias(antialias_);
    family_->setCurrentFont(QFont(o.family));
    size_->setEditText(sizeText(o.size));
    color_->setIcon(typeSwatchIcon(o.color));
    antialias_->setCurrentIndex(o.antialias ? 1 : 0);
}

void CharacterPanel::applySize()
{
    if (!controller_) {
        return;
    }
    bool ok = false;
    const double v = QLocale::c().toDouble(size_->currentText(), &ok);
    if (!ok || v < 1.0 || v > 1296.0) {
        size_->setEditText(sizeText(controller_->typeOptions().size));
        return;
    }
    if (v != controller_->typeOptions().size) {
        TypeOptions o = controller_->typeOptions();
        o.size = v;
        controller_->setTypeOptions(o);
    }
}

} // namespace pictura
