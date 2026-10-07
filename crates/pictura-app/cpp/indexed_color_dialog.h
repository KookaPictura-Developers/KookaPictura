#pragma once

#include <QtWidgets/QCheckBox>
#include <QtWidgets/QComboBox>
#include <QtWidgets/QDialog>
#include <QtWidgets/QSpinBox>

namespace pictura {

// Image > Mode > Indexed Color: palette, color count, and dither options, with a
// Preview toggle (docs/04-image-ops/indexed-color.md). The values are the
// `convert_to_indexed` bridge's integers. Ported from photorust's
// IndexedColorDialog, keeping only the options the engine honours.
//
// ponytail: no System/Uniform/Master/Custom/Previous palettes, Forced colors,
// Transparency, Matte, or Preserve Exact Colors (the engine has none of them).
class IndexedColorDialog : public QDialog {
    Q_OBJECT

public:
    // `exactAvailable` offers the Exact palette (and selects it), which only
    // fits an image of at most 256 colors.
    explicit IndexedColorDialog(bool exactAvailable, QWidget* parent = nullptr);

    int palette() const;  // 0 Exact, 1 Web, 2/3/4 Local Perceptual/Selective/Adaptive
    int colors() const;
    int dither() const;   // 0 None, 1 Diffusion, 2 Pattern, 3 Noise
    int amount() const;
    bool preview() const;

signals:
    // Any option, including Preview, changed.
    void optionsChanged();

private:
    void syncEnabled();

    QComboBox* palette_ = nullptr;
    QSpinBox* colors_ = nullptr;
    QComboBox* dither_ = nullptr;
    QSpinBox* amount_ = nullptr;
    QCheckBox* preview_ = nullptr;
};

} // namespace pictura
