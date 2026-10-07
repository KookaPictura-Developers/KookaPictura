#pragma once

#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>

namespace pictura {

// Image > Mode > Bitmap: how 8-bit gray is reduced to black and white
// (docs/04-image-ops/image-modes.md). `method()` is the `convert_to_bitmap`
// bridge's integer.
//
// ponytail: no Output resolution (the conversion does not resample), and
// Halftone Screen / Custom Pattern are absent.
class BitmapModeDialog : public QDialog {
    Q_OBJECT

public:
    explicit BitmapModeDialog(QWidget* parent = nullptr);

    int method() const;  // 0 50% Threshold, 1 Pattern Dither, 2 Diffusion Dither

private:
    QComboBox* method_ = nullptr;
};

} // namespace pictura
