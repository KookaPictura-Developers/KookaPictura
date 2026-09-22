#pragma once

#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

class QComboBox;

namespace pictura {

// The built-in-profile chooser for Edit > Assign Profile… / Convert to
// Profile… . The bridge index is the combo item's data: 0 sRGB, 1 Adobe RGB,
// 2 Pro Photo RGB. Convert offers the three destinations; Assign also offers
// "Don't Color Manage" (the sRGB working space).
// ponytail: built-in profiles only; add installed-profile discovery / .icc
// loading when a Color Settings policy needs it.
class ProfileDialog : public QDialog {
    Q_OBJECT

public:
    explicit ProfileDialog(bool convert, QWidget* parent = nullptr);

    int profileIndex() const;

    // Test hooks.
    QStringList choicesForTest() const;
    void setChoiceForTest(int index);

private:
    QComboBox* combo_ = nullptr;
};

} // namespace pictura
