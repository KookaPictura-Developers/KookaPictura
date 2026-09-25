#pragma once

#include <QtCore/QString>
#include <QtWidgets/QDialog>

class QCheckBox;
class QComboBox;
class QDoubleSpinBox;

namespace pictura {

// The values collected by the Warp preset dialog.
struct WarpPresetSpec {
    QString style_id = QStringLiteral("warpNone");
    double bend = 0.0;
    double distort_x = 0.0;
    double distort_y = 0.0;
    bool vertical = false;
};

// CS6 `Edit > Transform > Warp` as a one-shot preset apply: a Warp Style combo
// plus Bend and X/Y distortion. The interactive cage is not shipped, so this is
// the whole surface (see the `transform-warp-presets` change).
class WarpPresetDialog : public QDialog {
    Q_OBJECT

public:
    explicit WarpPresetDialog(QWidget* parent = nullptr);

    WarpPresetSpec spec() const;

    // Run the dialog modally; returns true and fills `out` on OK.
    static bool get(QWidget* parent, WarpPresetSpec* out);

private:
    void update_enabled();

    QComboBox* styleCombo_ = nullptr;
    QDoubleSpinBox* bendSpin_ = nullptr;
    QDoubleSpinBox* xSpin_ = nullptr;
    QDoubleSpinBox* ySpin_ = nullptr;
    QCheckBox* verticalCheck_ = nullptr;
};

} // namespace pictura
