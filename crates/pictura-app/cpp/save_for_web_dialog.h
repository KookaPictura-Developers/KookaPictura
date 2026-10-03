#pragma once

#include <QtCore/QByteArray>
#include <QtCore/QString>
#include <QtGui/QImage>
#include <QtWidgets/QDialog>

class QCheckBox;
class QComboBox;
class QLabel;
class QSpinBox;
class QStackedWidget;
class QTabBar;

namespace pictura {

class PictureView;

// File > Save for Web & Devices: CS6's web-optimisation dialog. Preview tabs
// (Original / Optimized / 2-Up; 4-Up listed, disabled), a preset, the format
// (GIF, JPEG, PNG-8, PNG-24, WBMP) and its settings, Image Size, and the
// optimised file's size and 56.6 Kbps download time. Save writes the
// optimised bytes; Done keeps the settings for the next time and closes.
// Ported from photorust's shell/src/dialogs/SaveForWebDialog.cpp.
//
// ponytail: no 4-Up, slices / HTML output, colour-table editing, Lossy, Web
// Snap, JPEG Blur, metadata choice, or PNG interlacing; the settings are kept
// for the session only.
class SaveForWebDialog : public QDialog {
    Q_OBJECT

public:
    enum Format { Gif, Jpeg, Png8, Png24, Wbmp };

    explicit SaveForWebDialog(const QImage& image, QWidget* parent = nullptr);

    // The optimised file for the current settings, and its extension.
    QByteArray encoded() const;
    QString suffix() const;
    // The image Image Size makes, before optimisation.
    QImage sized() const;
    // Write `encoded()` to `path`; false on a write failure.
    bool saveTo(const QString& path) const;

private:
    void buildUi();
    void applyPreset(int index);
    void refresh();
    void remember() const;
    void restore();
    QImage indexedImage(const QImage& rgba, QByteArray* gif) const;

    QImage original_;
    QTabBar* tabs_ = nullptr;
    QLabel* originalView_ = nullptr;
    QLabel* optimizedView_ = nullptr;
    QLabel* originalInfo_ = nullptr;
    QLabel* optimizedInfo_ = nullptr;
    QComboBox* preset_ = nullptr;
    QComboBox* format_ = nullptr;
    QStackedWidget* settings_ = nullptr;
    // GIF / PNG-8
    QComboBox* reduction_ = nullptr;
    QSpinBox* colors_ = nullptr;
    QComboBox* dither_ = nullptr;
    QSpinBox* ditherAmount_ = nullptr;
    QCheckBox* transparency_ = nullptr;
    QCheckBox* interlaced_ = nullptr;
    QComboBox* matte_ = nullptr;
    // JPEG
    QComboBox* qualityPreset_ = nullptr;
    QSpinBox* quality_ = nullptr;
    QCheckBox* progressive_ = nullptr;
    QCheckBox* optimized_ = nullptr;
    // PNG-24
    QCheckBox* pngTransparency_ = nullptr;
    // WBMP
    QComboBox* wbmpDither_ = nullptr;
    QSpinBox* wbmpAmount_ = nullptr;
    // Image Size
    QSpinBox* width_ = nullptr;
    QSpinBox* height_ = nullptr;
    QSpinBox* percent_ = nullptr;
    QCheckBox* constrain_ = nullptr;
    QComboBox* resample_ = nullptr;
};

// Run the dialog on `view`'s flattened image and, on Save, write the file the
// user names. False when cancelled, without a document, or on a write failure.
bool saveForWebFromView(QWidget* parent, PictureView* view, const QString& documentName);

} // namespace pictura
