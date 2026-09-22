#pragma once

#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

class QComboBox;
class QLabel;

namespace pictura {

// The Color Settings dialog: shows the fixed sRGB RGB working space and edits
// the incoming-profile policy. The combo item data is the persistence code:
// 0 Preserve Embedded Profiles, 1 Convert to Working RGB, 2 Off.
// ponytail: sRGB working space only; no .csf file or CMYK/Gray policies.
class ColorSettingsDialog : public QDialog {
    Q_OBJECT

public:
    explicit ColorSettingsDialog(int currentPolicy, QWidget* parent = nullptr);

    int policyCode() const;

    // Test hooks.
    QStringList policyChoicesForTest() const;
    QString workingSpaceForTest() const;
    void setPolicyForTest(int index);

private:
    QComboBox* combo_ = nullptr;
    QLabel* workingSpace_ = nullptr;
};

} // namespace pictura
