#pragma once

#include <QtCore/QString>
#include <QtWidgets/QDialog>

class QComboBox;
class QLineEdit;

namespace pictura {

class NumericField;

// The values collected by the New Document dialog.
struct NewDocumentSpec {
    QString name;
    int width = 1000;
    int height = 1000;
    QString mode = QStringLiteral("rgb");          // "rgb" | "grayscale"
    int depth = 8;                                 // only 8 is supported
    QString background = QStringLiteral("white");  // "white" | "transparent"
};

class NewDocumentDialog : public QDialog {
    Q_OBJECT

public:
    explicit NewDocumentDialog(QWidget* parent = nullptr);

    NewDocumentSpec spec() const;

    // Run the dialog modally; returns true and fills `out` on OK.
    static bool get(QWidget* parent, NewDocumentSpec* out);

private:
    QLineEdit* nameEdit_ = nullptr;
    NumericField* widthSpin_ = nullptr;
    NumericField* heightSpin_ = nullptr;
    QComboBox* presetCombo_ = nullptr;
    QComboBox* modeCombo_ = nullptr;
    QComboBox* depthCombo_ = nullptr;
    QComboBox* backgroundCombo_ = nullptr;
};

} // namespace pictura
