#include "new_document_dialog.h"

#include "panels/numeric_field.h"

#include <QtCore/QSize>
#include <QtGui/QStandardItemModel>

#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QFormLayout>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>

namespace pictura {

namespace {

NumericFieldConfig dimensionConfig(const QString& objectName)
{
    NumericFieldConfig config;
    config.minimum = 1;
    config.maximum = 30000;
    config.decimals = 0;
    config.objectName = objectName;
    return config;
}

} // namespace

NewDocumentDialog::NewDocumentDialog(QWidget* parent)
    : QDialog(parent)
{
    setWindowTitle(QStringLiteral("New Document"));

    nameEdit_ = new QLineEdit(QStringLiteral("Untitled"), this);

    presetCombo_ = new QComboBox(this);
    presetCombo_->addItem(QStringLiteral("Default Photoshop Size"), QSize(1000, 1000));
    presetCombo_->addItem(QStringLiteral("Custom"), QSize());

    widthSpin_ = new NumericField(QString(), dimensionConfig(QStringLiteral("newDocWidth")), this);
    heightSpin_ =
        new NumericField(QString(), dimensionConfig(QStringLiteral("newDocHeight")), this);
    widthSpin_->setValue(1000);
    heightSpin_->setValue(1000);

    modeCombo_ = new QComboBox(this);
    modeCombo_->addItem(QStringLiteral("RGB Color"), QStringLiteral("rgb"));
    modeCombo_->addItem(QStringLiteral("Grayscale"), QStringLiteral("grayscale"));

    // ponytail: only 8-bit Grayscale/RGB exist because the codec cannot write
    // deeper or other color modes; add the real modes once the codec supports them.
    depthCombo_ = new QComboBox(this);
    depthCombo_->addItem(QStringLiteral("8 Bits/Channel"), 8);
    depthCombo_->addItem(QStringLiteral("16 Bits/Channel"), 16);
    depthCombo_->addItem(QStringLiteral("32 Bits/Channel"), 32);
    auto* depthModel = qobject_cast<QStandardItemModel*>(depthCombo_->model());
    for (int i = 1; i < depthCombo_->count(); ++i) {
        depthCombo_->setItemData(
            i, QStringLiteral("Only 8-bit channels are supported."), Qt::ToolTipRole);
        if (depthModel) {
            if (QStandardItem* item = depthModel->item(i)) {
                item->setFlags(item->flags() & ~Qt::ItemIsEnabled);
            }
        }
    }

    backgroundCombo_ = new QComboBox(this);
    backgroundCombo_->addItem(QStringLiteral("White"), QStringLiteral("white"));
    backgroundCombo_->addItem(QStringLiteral("Transparent"), QStringLiteral("transparent"));

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    connect(buttons, &QDialogButtonBox::accepted, this, &NewDocumentDialog::accept);
    connect(buttons, &QDialogButtonBox::rejected, this, &NewDocumentDialog::reject);

    auto* form = new QFormLayout(this);
    form->addRow(QStringLiteral("Name:"), nameEdit_);
    form->addRow(QStringLiteral("Preset:"), presetCombo_);
    form->addRow(QStringLiteral("Width:"), widthSpin_);
    form->addRow(QStringLiteral("Height:"), heightSpin_);
    form->addRow(QStringLiteral("Color Mode:"), modeCombo_);
    form->addRow(QStringLiteral("Bit Depth:"), depthCombo_);
    form->addRow(QStringLiteral("Background:"), backgroundCombo_);
    form->addRow(buttons);

    connect(presetCombo_, &QComboBox::currentIndexChanged, this, [this](int) {
        const QSize size = presetCombo_->currentData().toSize();
        if (size.isValid()) {
            widthSpin_->setValue(size.width());
            heightSpin_->setValue(size.height());
        }
    });
}

NewDocumentSpec NewDocumentDialog::spec() const
{
    NewDocumentSpec result;
    result.name = nameEdit_->text();
    result.width = qRound(widthSpin_->value());
    result.height = qRound(heightSpin_->value());
    result.mode = modeCombo_->currentData().toString();
    result.depth = depthCombo_->currentData().toInt();
    result.background = backgroundCombo_->currentData().toString();
    return result;
}

bool NewDocumentDialog::get(QWidget* parent, NewDocumentSpec* out)
{
    NewDocumentDialog dialog(parent);
    if (dialog.exec() != QDialog::Accepted) {
        return false;
    }
    *out = dialog.spec();
    return true;
}

} // namespace pictura
