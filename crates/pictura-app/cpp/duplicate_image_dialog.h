#pragma once

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QLineEdit>

namespace pictura {

// Photoshop's Image > Duplicate: names the copy and says whether it takes the
// whole layer stack or just the flattened image. Ported from photorust's
// DuplicateImageDialog.
class DuplicateImageDialog : public QDialog {
    Q_OBJECT

public:
    DuplicateImageDialog(const QString& sourceName, const QString& suggestedName,
                         QWidget* parent = nullptr);

    QString copyName() const;
    bool mergedOnly() const;

private:
    QLineEdit* name_ = nullptr;
    QCheckBox* merged_ = nullptr;
};

} // namespace pictura
