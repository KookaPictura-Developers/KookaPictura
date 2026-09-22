#include "file_info_dialog.h"

#include <QtWidgets/QAbstractItemView>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QHBoxLayout>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QListWidget>
#include <QtWidgets/QPlainTextEdit>
#include <QtWidgets/QStackedWidget>
#include <QtWidgets/QTableWidget>
#include <QtWidgets/QTableWidgetItem>
#include <QtWidgets/QVBoxLayout>

namespace pictura {

const QString FileInfoDialog::kCameraData = QStringLiteral("Camera Data");
const QString FileInfoDialog::kIptc = QStringLiteral("IPTC");
const QString FileInfoDialog::kRawData = QStringLiteral("Raw Data");

namespace {

// A read-only two-column view of one category's `"Label\tValue"` rows.
QTableWidget* makeRowTable(const QStringList& rows, QWidget* parent)
{
    auto* table = new QTableWidget(rows.size(), 2, parent);
    table->setEditTriggers(QAbstractItemView::NoEditTriggers);
    table->setSelectionMode(QAbstractItemView::NoSelection);
    table->verticalHeader()->setVisible(false);
    table->horizontalHeader()->setStretchLastSection(true);
    table->setHorizontalHeaderLabels({QObject::tr("Field"), QObject::tr("Value")});
    for (int i = 0; i < rows.size(); ++i) {
        const QStringList parts = rows[i].split(QLatin1Char('\t'));
        table->setItem(i, 0, new QTableWidgetItem(parts.value(0)));
        table->setItem(i, 1, new QTableWidgetItem(parts.value(1)));
    }
    table->resizeColumnsToContents();
    return table;
}

} // namespace

FileInfoDialog::FileInfoDialog(const QStringList& exifRows, const QStringList& iptcRows,
                               const QString& xmp, QWidget* parent)
    : QDialog(parent), exifRows_(exifRows), iptcRows_(iptcRows), xmp_(xmp)
{
    setObjectName(QStringLiteral("fileInfoDialog"));
    setWindowTitle(tr("File Info"));
    setMinimumSize(560, 360);

    categoryList_ = new QListWidget(this);
    categoryList_->setObjectName(QStringLiteral("fileInfoCategories"));
    categoryList_->setFixedWidth(150);
    for (const QString& name : {kCameraData, kIptc, kRawData}) {
        new QListWidgetItem(name, categoryList_);
    }

    stack_ = new QStackedWidget(this);
    stack_->setObjectName(QStringLiteral("fileInfoStack"));
    stack_->addWidget(makeRowTable(exifRows_, stack_));
    stack_->addWidget(makeRowTable(iptcRows_, stack_));
    xmpEdit_ = new QPlainTextEdit(xmp_, stack_);
    xmpEdit_->setObjectName(QStringLiteral("fileInfoXmp"));
    xmpEdit_->setReadOnly(true);
    stack_->addWidget(xmpEdit_);

    connect(categoryList_, &QListWidget::currentRowChanged, stack_,
            &QStackedWidget::setCurrentIndex);
    categoryList_->setCurrentRow(0);

    auto* buttons = new QDialogButtonBox(QDialogButtonBox::Close, this);
    connect(buttons, &QDialogButtonBox::rejected, this, &FileInfoDialog::reject);

    auto* top = new QHBoxLayout();
    top->addWidget(categoryList_);
    top->addWidget(stack_, 1);
    auto* outer = new QVBoxLayout(this);
    outer->addLayout(top, 1);
    outer->addWidget(buttons);
}

QStringList FileInfoDialog::categoriesForTest() const
{
    QStringList out;
    for (int i = 0; i < categoryList_->count(); ++i) {
        out.append(categoryList_->item(i)->text());
    }
    return out;
}

QStringList FileInfoDialog::rowsForTest(const QString& category) const
{
    if (category == kCameraData) {
        return exifRows_;
    }
    if (category == kIptc) {
        return iptcRows_;
    }
    return {};
}

QString FileInfoDialog::xmpForTest() const
{
    return xmpEdit_->toPlainText();
}

} // namespace pictura
