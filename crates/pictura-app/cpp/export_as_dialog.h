#pragma once

#include <QtCore/QString>
#include <QtWidgets/QDialog>

class QComboBox;
class QLabel;
class QSlider;
class QSpinBox;

namespace pictura {

class PictureView;

// Format/quality/scale chooser for Export As. Format is one of PNG, JPG, TIF,
// WEBP, BMP; quality is enabled only for the lossy formats; scale is a percent.
class ExportAsDialog : public QDialog {
    Q_OBJECT

public:
    explicit ExportAsDialog(const QString& initialFormat, QWidget* parent = nullptr);

    QString format() const;
    int quality() const;
    int scale() const;

private:
    void updateQualityEnabled();

    QComboBox* format_ = nullptr;
    QSlider* quality_ = nullptr;
    QLabel* qualityLabel_ = nullptr;
    QSpinBox* scale_ = nullptr;
};

// Run the options dialog then a file dialog and write the view's flattened
// composite through `PictureView::export_image`. False when cancelled, when
// there is no document, or when the encode fails.
bool exportAsFromView(QWidget* parent, PictureView* view);

// Write `<source stem>.png` beside the view's file, prompting once when the
// document is untitled. False when cancelled or the encode fails.
bool quickExportPngFromView(QWidget* parent, PictureView* view);

} // namespace pictura
