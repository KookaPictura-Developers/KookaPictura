#include "exposure_dialog.h"

#include "icons.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QGridLayout>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QSlider>
#include <QtWidgets/QToolButton>
#include <QtWidgets/QVBoxLayout>

#include <cmath>

namespace pictura {

namespace {

struct Field {
    const char* label;
    const char* key;
    double min;
    double max;
    int decimals;
    const char* eyedropper;
};

constexpr Field kFields[] = {
    {"Exposure:", "exposure", -20.0, 20.0, 2, "Sample in image to set black point"},
    {"Offset:", "offset", -0.5, 0.5, 4, "Sample in image to set gray point"},
    {"Gamma Correction:", "gamma", 0.01, 9.99, 2, "Sample in image to set white point"},
};

struct Preset {
    const char* name;
    double exposure;
};

constexpr Preset kPresets[] = {
    {"Default", 0.0}, {"Minus 1.0", -1.0}, {"Minus 2.0", -2.0}, {"Plus 1.0", 1.0},
    {"Plus 2.0", 2.0},
};
const QString kCustom = QStringLiteral("Custom");

} // namespace

ExposureDialog::ExposureDialog(PictureView* view, const QByteArray& block, const QRect& visible,
                               QWidget* parent)
    : AdjustmentDialog(view, block, visible, QStringLiteral("Exposure"), parent)
{
    auto* outer = new QHBoxLayout(this);
    outer->setContentsMargins(12, 12, 12, 12);
    outer->setSpacing(16);
    auto* left = new QVBoxLayout;

    auto* presetRow = new QHBoxLayout;
    presetRow->addWidget(new QLabel(QStringLiteral("Preset:"), this));
    preset_ = new QComboBox(this);
    preset_->setObjectName(QStringLiteral("exposurePreset"));
    for (const Preset& preset : kPresets) {
        preset_->addItem(QLatin1String(preset.name));
    }
    preset_->insertSeparator(1);
    preset_->addItem(kCustom);
    presetRow->addWidget(preset_, 1);
    left->addLayout(presetRow);
    left->addSpacing(10);

    auto* grid = new QGridLayout;
    grid->setColumnStretch(1, 1);
    grid->setHorizontalSpacing(8);
    grid->setVerticalSpacing(8);
    for (int row = 0; row < int(std::size(kFields)); ++row) {
        const Field& field = kFields[row];
        const QString key = QLatin1String(field.key);
        const double scale = std::pow(10.0, field.decimals);
        auto* label = new QLabel(QLatin1String(field.label), this);
        label->setAlignment(Qt::AlignRight | Qt::AlignVCenter);
        auto* slider = new QSlider(Qt::Horizontal, this);
        slider->setObjectName(key + QStringLiteral("Slider"));
        slider->setRange(qRound(field.min * scale), qRound(field.max * scale));
        auto* spin = new QDoubleSpinBox(this);
        spin->setObjectName(key);
        spin->setRange(field.min, field.max);
        spin->setDecimals(field.decimals);
        spin->setSingleStep(1.0 / scale);
        spin->setFixedWidth(kFieldWidth);
        spin->setKeyboardTracking(false);
        spin->setValue(param(key, key == QLatin1String("gamma") ? 1.0 : 0.0));
        slider->setValue(qRound(spin->value() * scale));
        auto* eyedropper = new QToolButton(this);
        eyedropper->setIcon(icon(QStringLiteral("tool.eyedropper")));
        eyedropper->setToolTip(QLatin1String(field.eyedropper));
        eyedropper->setAutoRaise(true);
        eyedropper->setEnabled(false);
        grid->addWidget(label, row, 0);
        grid->addWidget(slider, row, 1);
        grid->addWidget(spin, row, 2);
        grid->addWidget(eyedropper, row, 3);

        connect(slider, &QSlider::valueChanged, spin,
                [spin, scale](int v) { spin->setValue(v / scale); });
        connect(spin, &QDoubleSpinBox::valueChanged, this, [this, slider, key, scale](double v) {
            const QSignalBlocker block(slider);
            slider->setValue(qRound(v * scale));
            setParam(key, v);
            markCustom();
        });
    }
    left->addLayout(grid);
    left->addStretch(1);
    outer->addLayout(left, 1);

    QVBoxLayout* buttons = buttonColumn();
    buttons->addSpacing(10);
    buttons->addWidget(previewCheck());
    buttons->addStretch(1);
    outer->addLayout(buttons);

    connect(preset_, &QComboBox::activated, this, &ExposureDialog::applyPreset);
    setMinimumWidth(500);
}

void ExposureDialog::applyPreset(int index)
{
    const QString name = preset_->itemText(index);
    for (const Preset& preset : kPresets) {
        if (name == QLatin1String(preset.name)) {
            applying_ = true;
            findChild<QDoubleSpinBox*>(QStringLiteral("exposure"))->setValue(preset.exposure);
            findChild<QDoubleSpinBox*>(QStringLiteral("offset"))->setValue(0.0);
            findChild<QDoubleSpinBox*>(QStringLiteral("gamma"))->setValue(1.0);
            applying_ = false;
            return;
        }
    }
}

void ExposureDialog::markCustom()
{
    if (!applying_) {
        const QSignalBlocker block(preset_);
        preset_->setCurrentIndex(preset_->findText(kCustom));
    }
}

QWidget* ExposureDialog::controlForTest(const QString& key) const
{
    if (key == QLatin1String("preset")) {
        return preset_;
    }
    return findChild<QDoubleSpinBox*>(key);
}

} // namespace pictura
