#pragma once

#include <QtCore/QString>
#include <QtWidgets/QDialog>

class QCheckBox;
class QComboBox;
class QLineEdit;
class QSpinBox;

namespace pictura {

// The values collected by the New Layer / New Group dialog.
struct LayerNewSpec {
    QString name;
    int color = 0;  // ColorLabel byte, 0 (none) … 7 (gray)
    QString blend = QStringLiteral("norm");
    int opacity = 255;  // stored opacity byte, 0..255
    bool neutralFill = false;
    bool clipping = false;
};

// Modal dialog mirroring CS6's New Layer / New Group options (design D6).
// Groups omit the clipping and mode-neutral-fill options entirely.
class LayerNewDialog : public QDialog {
    Q_OBJECT

public:
    explicit LayerNewDialog(bool group, QWidget* parent = nullptr);

    LayerNewSpec spec() const;

    // Run the dialog modally; returns true and fills `out` on OK.
    static bool get(bool group, QWidget* parent, LayerNewSpec* out);

    // Self-test hooks: inspect or drive the dialog without showing it.
    bool neutralEnabledForTest() const;
    bool clippingVisibleForTest() const;
    void setModeForTest(const QString& blendKey);

private:
    void syncNeutralForMode();

    bool group_ = false;
    QLineEdit* nameEdit_ = nullptr;
    QComboBox* colorCombo_ = nullptr;
    QComboBox* modeCombo_ = nullptr;
    QSpinBox* opacitySpin_ = nullptr;
    QCheckBox* neutralCheck_ = nullptr;
    QCheckBox* clippingCheck_ = nullptr;
};

} // namespace pictura
