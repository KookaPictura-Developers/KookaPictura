#include "layer_new_dialog.h"

#include "panels/layers_panel_internal.h"
#include "panels/numeric_field.h"

#include <QtCore/QSet>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QLineEdit>

namespace pictura {

namespace {

// The seven modes whose neutral color is undefined (docs/05-layers/blend-modes.md).
// The engine's `pictura_render::neutral_color` returns None for the same set and
// is the authority for the pixels actually written.
const QSet<QString>& modesWithoutNeutral()
{
    static const QSet<QString> keys = {
        QStringLiteral("norm"), QStringLiteral("diss"), QStringLiteral("hMix"),
        QStringLiteral("hue "), QStringLiteral("sat "), QStringLiteral("colr"),
        QStringLiteral("lum ")};
    return keys;
}

} // namespace

LayerNewDialog::LayerNewDialog(bool group, QWidget* parent)
    : QDialog(parent)
    , group_(group)
{
    setWindowTitle(group ? QStringLiteral("New Group") : QStringLiteral("New Layer"));

    nameEdit_ = new QLineEdit(this);

    colorCombo_ = new QComboBox(this);
    colorCombo_->addItem(QStringLiteral("None"), 0);
    colorCombo_->addItem(QStringLiteral("Red"), 1);
    colorCombo_->addItem(QStringLiteral("Orange"), 2);
    colorCombo_->addItem(QStringLiteral("Yellow"), 3);
    colorCombo_->addItem(QStringLiteral("Green"), 4);
    colorCombo_->addItem(QStringLiteral("Blue"), 5);
    colorCombo_->addItem(QStringLiteral("Violet"), 6);
    colorCombo_->addItem(QStringLiteral("Gray"), 7);

    modeCombo_ = new QComboBox(this);
    for (const BlendEntry& entry : kBlends) {
        modeCombo_->addItem(QString::fromLatin1(entry.name), QString::fromLatin1(entry.key));
    }

    NumericFieldConfig opacityConfig;
    opacityConfig.minimum = 0;
    opacityConfig.maximum = 255;
    opacityConfig.decimals = 0;
    opacityConfig.objectName = QStringLiteral("layerNewOpacity");
    opacitySpin_ = new NumericField(QString(), opacityConfig, this);
    opacitySpin_->setValue(255);

    if (!group_) {
        neutralCheck_ = new QCheckBox(QStringLiteral("Fill With Mode-Neutral Color"), this);
        clippingCheck_ =
            new QCheckBox(QStringLiteral("Use Previous Layer to Create Clipping Mask"), this);
    }

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    connect(buttons, &QDialogButtonBox::accepted, this, &LayerNewDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &LayerNewDialog::reject);

    auto* form = new QFormLayout(this);
    form->addRow(QStringLiteral("Name:"), nameEdit_);
    form->addRow(QStringLiteral("Color:"), colorCombo_);
    form->addRow(QStringLiteral("Mode:"), modeCombo_);
    form->addRow(QStringLiteral("Opacity:"), opacitySpin_);
    if (neutralCheck_) {
        form->addRow(neutralCheck_);
    }
    if (clippingCheck_) {
        form->addRow(clippingCheck_);
    }
    form->addRow(buttons);

    connect(modeCombo_, &QComboBox::currentIndexChanged, this,
            [this](int) { syncNeutralForMode(); });
    syncNeutralForMode();
}

void LayerNewDialog::syncNeutralForMode()
{
    if (!neutralCheck_) {
        return;
    }
    const bool available = !modesWithoutNeutral().contains(modeCombo_->currentData().toString());
    neutralCheck_->setEnabled(available);
    if (!available) {
        neutralCheck_->setChecked(false);
    }
}

LayerNewSpec LayerNewDialog::spec() const
{
    LayerNewSpec result;
    result.name = nameEdit_->text();
    result.color = colorCombo_->currentData().toInt();
    result.blend = modeCombo_->currentData().toString();
    result.opacity = qRound(opacitySpin_->value());
    result.neutralFill = neutralCheck_ && neutralCheck_->isChecked();
    result.clipping = clippingCheck_ && clippingCheck_->isChecked();
    return result;
}

bool LayerNewDialog::get(bool group, QWidget* parent, LayerNewSpec* out)
{
    LayerNewDialog dialog(group, parent);
    if (dialog.exec() != QDialog::Accepted) {
        return false;
    }
    *out = dialog.spec();
    return true;
}

bool LayerNewDialog::neutralEnabledForTest() const
{
    return neutralCheck_ && neutralCheck_->isEnabled();
}

bool LayerNewDialog::clippingVisibleForTest() const
{
    return clippingCheck_ != nullptr;
}

void LayerNewDialog::setModeForTest(const QString& blendKey)
{
    const int index = modeCombo_->findData(blendKey);
    if (index >= 0) {
        modeCombo_->setCurrentIndex(index);
    }
}

} // namespace pictura
