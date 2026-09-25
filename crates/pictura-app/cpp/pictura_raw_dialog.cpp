#include "pictura_raw_dialog.h"

#include <QtCore/QString>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QDoubleSpinBox>
#include <QtWidgets/QFormLayout>

namespace pictura {

namespace {

struct ControlSpec {
    const char* label;
    double minimum;
    double maximum;
    int decimals;
    double step;
};

// ponytail: only Temperature ±100 (JPEG scale), Exposure ±5, and the other nine
// ±100 are documented; the decimals/steps are inferred, and the spec's raw
// 2,000–50,000 K temperature scale does not apply to a JPEG/TIFF filter.
const ControlSpec kControls[11] = {
    {"Temperature:", -100.0, 100.0, 0, 1.0},
    {"Tint:", -100.0, 100.0, 0, 1.0},
    {"Exposure:", -5.0, 5.0, 2, 0.05},
    {"Contrast:", -100.0, 100.0, 0, 1.0},
    {"Highlights:", -100.0, 100.0, 0, 1.0},
    {"Shadows:", -100.0, 100.0, 0, 1.0},
    {"Whites:", -100.0, 100.0, 0, 1.0},
    {"Blacks:", -100.0, 100.0, 0, 1.0},
    {"Clarity:", -100.0, 100.0, 0, 1.0},
    {"Vibrance:", -100.0, 100.0, 0, 1.0},
    {"Saturation:", -100.0, 100.0, 0, 1.0},
};

} // namespace

PicturaRawDialog::PicturaRawDialog(const QList<double>& initial, QWidget* parent)
    : QDialog(parent)
{
    setWindowTitle(QStringLiteral("Pictura Raw"));

    auto* form = new QFormLayout(this);
    for (int i = 0; i < 11; ++i) {
        auto* spin = new QDoubleSpinBox(this);
        spin->setRange(kControls[i].minimum, kControls[i].maximum);
        spin->setDecimals(kControls[i].decimals);
        spin->setSingleStep(kControls[i].step);
        spin->setValue(i < initial.size() ? initial.at(i) : 0.0);
        spins_[i] = spin;
        form->addRow(QString::fromLatin1(kControls[i].label), spin);
    }

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    connect(buttons, &QDialogButtonBox::accepted, this, &PicturaRawDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &PicturaRawDialog::reject);
    form->addRow(buttons);
}

QList<double> PicturaRawDialog::values() const
{
    QList<double> result;
    for (const QDoubleSpinBox* spin : spins_) {
        result.append(spin->value());
    }
    return result;
}

bool PicturaRawDialog::get(QWidget* parent, const QList<double>& initial, QList<double>* out)
{
    PicturaRawDialog dialog(initial, parent);
    if (dialog.exec() != QDialog::Accepted) {
        return false;
    }
    *out = dialog.values();
    return true;
}

} // namespace pictura
