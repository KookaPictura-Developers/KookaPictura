#pragma once

#include <QtCore/QList>
#include <QtWidgets/QDialog>

class QDoubleSpinBox;

namespace pictura {

// CS6 `Filter > Pictura Raw…` (a post-CS6 extension kept under the CS6
// label). The eleven PV2012 Basic controls in dialog order: temperature, tint,
// exposure, contrast, highlights, shadows, whites, blacks, clarity, vibrance,
// saturation.
class PicturaRawDialog : public QDialog {
    Q_OBJECT

public:
    explicit PicturaRawDialog(const QList<double>& initial, QWidget* parent = nullptr);

    QList<double> values() const;

    // Run the dialog modally; returns true and fills `out` on OK.
    static bool get(QWidget* parent, const QList<double>& initial, QList<double>* out);

private:
    QDoubleSpinBox* spins_[11] = {};
};

} // namespace pictura
