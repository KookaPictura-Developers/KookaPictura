#pragma once

#include <QtWidgets/QDialog>

class QDoubleSpinBox;

namespace pictura {

// The values collected by the HDR Conversion dialog.
struct HdrConversionSpec {
    double exposure_ev = 0.0;
    double gamma = 1.0;
};

// CS6 `Image > Mode > 32 -> 16/8` HDR Conversion. Only the documented
// Exposure & Gamma method is implemented, so there is no Method combo.
class HdrConversionDialog : public QDialog {
    Q_OBJECT

public:
    explicit HdrConversionDialog(QWidget* parent = nullptr);

    HdrConversionSpec spec() const;

    // Run the dialog modally; returns true and fills `out` on OK.
    static bool get(QWidget* parent, HdrConversionSpec* out);

private:
    QDoubleSpinBox* exposureSpin_ = nullptr;
    QDoubleSpinBox* gammaSpin_ = nullptr;
};

} // namespace pictura
