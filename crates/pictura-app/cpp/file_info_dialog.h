#pragma once

#include <QtCore/QHash>
#include <QtCore/QString>
#include <QtCore/QStringList>
#include <QtWidgets/QDialog>

#include <functional>

class QComboBox;
class QLineEdit;
class QListWidget;
class QPlainTextEdit;
class QStackedWidget;

namespace pictura {

// CS6 File Info: a category list (Camera Data / Description / IPTC / Raw Data)
// over a stacked pane. Camera Data, Description, and Raw Data are read-only; the
// IPTC core fields are editable and the caller applies `edits()` on OK.
class FileInfoDialog : public QDialog {
    Q_OBJECT

public:
    static const QString kCameraData;
    static const QString kDescription;
    static const QString kIptc;
    static const QString kRawData;

    // `xmpRows` rows are the parsed XMP properties as `"Label\tValue"`, shown
    // read-only under Description. `iptcEditFields` rows are
    // `"record:dataset\tLabel\tValue"`. `iptcOtherRows` rows are `"Label\tValue"`
    // for IPTC records outside the editable core set; they are shown read-only.
    FileInfoDialog(const QStringList& exifRows, const QStringList& xmpRows,
                   const QStringList& iptcEditFields, const QStringList& iptcOtherRows,
                   const QString& xmp, QWidget* parent = nullptr);

    // The edited core fields as `"record:dataset\tValue"` rows, in field order.
    QStringList edits() const;

    // Metadata-template controls. The frame sets these; the buttons stay on the
    // dialog and pass the current merge mode to `onApplyTemplate`.
    std::function<void()> onExportTemplate;
    std::function<void(int)> onApplyTemplate;

    // Merge mode the Apply button passes: 0 Append, 1 Replace,
    // 2 KeepOriginalReplaceMatching.
    int templateMode() const;

    // Test hooks.
    QStringList categoriesForTest() const;
    QStringList rowsForTest(const QString& category) const;
    QString xmpForTest() const;
    QString fieldForTest(const QString& id) const;
    void setFieldForTest(const QString& id, const QString& value);
    bool hasTemplateControlsForTest() const;
    int mergeModeForTest() const;
    void setMergeModeForTest(int mode);

private:
    QWidget* buildIptcPage(const QStringList& editFields, const QStringList& otherRows);

    QStringList exifRows_;
    QStringList xmpRows_;
    QStringList iptcOtherRows_;
    QString xmp_;
    QStringList fieldIds_;
    QListWidget* categoryList_ = nullptr;
    QStackedWidget* stack_ = nullptr;
    QPlainTextEdit* xmpEdit_ = nullptr;
    QComboBox* templateMode_ = nullptr;
    QHash<QString, QLineEdit*> fields_;
    QHash<QString, QString> initial_;
};

} // namespace pictura
