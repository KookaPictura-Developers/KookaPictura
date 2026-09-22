#pragma once

#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

class QListWidget;
class QPlainTextEdit;
class QStackedWidget;

namespace pictura {

// Read-only CS6 File Info surface: a category list (Camera Data / IPTC / Raw
// Data) over a stacked pane. The values come from the active document's decoded
// EXIF/IPTC metadata and its raw XMP packet; nothing here is editable.
class FileInfoDialog : public QDialog {
    Q_OBJECT

public:
    static const QString kCameraData;
    static const QString kIptc;
    static const QString kRawData;

    FileInfoDialog(const QStringList& exifRows, const QStringList& iptcRows,
                   const QString& xmp, QWidget* parent = nullptr);

    // Test hooks.
    QStringList categoriesForTest() const;
    QStringList rowsForTest(const QString& category) const;
    QString xmpForTest() const;

private:
    QStringList exifRows_;
    QStringList iptcRows_;
    QString xmp_;
    QListWidget* categoryList_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QPlainTextEdit* xmpEdit_ = nullptr;
};

} // namespace pictura
