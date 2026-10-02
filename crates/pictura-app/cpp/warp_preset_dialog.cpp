#include "warp_preset_dialog.h"
#include "dialogs.h"

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QHBoxLayout>

namespace pictura {

namespace {

// Display names and stable ids in CS6 pop-up order. The ids are the contract
// with `pictura_render::WarpStyle::from_id`; keep the two lists in step.
struct StyleEntry {
    const char* name;
    const char* id;
};
constexpr StyleEntry kStyles[] = {
    {"None", "warpNone"},
    {"Custom", "warpCustom"},
    {"Arc", "warpArc"},
    {"Arc Lower", "warpArcLower"},
    {"Arc Upper", "warpArcUpper"},
    {"Arch", "warpArch"},
    {"Bulge", "warpBulge"},
    {"Shell Lower", "warpShellLower"},
    {"Shell Upper", "warpShellUpper"},
    {"Flag", "warpFlag"},
    {"Wave", "warpWave"},
    {"Fish", "warpFish"},
    {"Rise", "warpRise"},
    {"Fisheye", "warpFisheye"},
    {"Inflate", "warpInflate"},
    {"Squeeze", "warpSqueeze"},
    {"Twist", "warpTwist"},
};

} // namespace

WarpPresetDialog::WarpPresetDialog(QWidget* parent)
    : QDialog(parent)
{
    setWindowTitle(QStringLiteral("Warp"));

    styleCombo_ = new QComboBox(this);
    for (const StyleEntry& entry : kStyles) {
        styleCombo_->addItem(QString::fromLatin1(entry.name), QString::fromLatin1(entry.id));
    }
    styleCombo_->setCurrentIndex(2); // Arc

    // Bend/H/V are disabled for None/Custom (TOOL-001), so they do nothing until
    // a preset is selected.
    bendSpin_ = new QDoubleSpinBox(this);
    bendSpin_->setRange(-100.0, 100.0);
    bendSpin_->setDecimals(0);
    bendSpin_->setValue(50.0);

    xSpin_ = new QDoubleSpinBox(this);
    xSpin_->setRange(-100.0, 100.0);
    xSpin_->setDecimals(0);

    ySpin_ = new QDoubleSpinBox(this);
    ySpin_->setRange(-100.0, 100.0);
    ySpin_->setDecimals(0);

    verticalCheck_ = new QCheckBox(QStringLiteral("Vertical orientation"), this);

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    connect(buttons, &QDialogButtonBox::accepted, this, &WarpPresetDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &WarpPresetDialog::reject);
    connect(styleCombo_, &QComboBox::currentIndexChanged, this,
            [this](int) { update_enabled(); });

    auto* distortion = new QHBoxLayout;
    distortion->addWidget(xSpin_);
    distortion->addWidget(ySpin_);

    auto* form = new QFormLayout(this);
    form->addRow(QStringLiteral("Warp Style:"), styleCombo_);
    form->addRow(QStringLiteral("Bend:"), bendSpin_);
    form->addRow(QStringLiteral("X / Y distortion:"), distortion);
    form->addRow(QString(), verticalCheck_);
    form->addRow(buttons);

    update_enabled();
}

void WarpPresetDialog::update_enabled()
{
    const QString id = styleCombo_->currentData().toString();
    const bool preset = id != QStringLiteral("warpNone") && id != QStringLiteral("warpCustom");
    bendSpin_->setEnabled(preset);
    xSpin_->setEnabled(preset);
    ySpin_->setEnabled(preset);
    verticalCheck_->setEnabled(preset);
}

WarpPresetSpec WarpPresetDialog::spec() const
{
    WarpPresetSpec result;
    result.style_id = styleCombo_->currentData().toString();
    result.bend = bendSpin_->value();
    result.distort_x = xSpin_->value();
    result.distort_y = ySpin_->value();
    result.vertical = verticalCheck_->isChecked();
    return result;
}

bool WarpPresetDialog::get(QWidget* parent, WarpPresetSpec* out)
{
    WarpPresetDialog dialog(parent);
    if (runDialog(dialog, parent) != QDialog::Accepted) {
        return false;
    }
    *out = dialog.spec();
    return true;
}

} // namespace pictura
