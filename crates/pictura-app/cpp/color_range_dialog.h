#pragma once

#include <QtGui/QColor>
#include <QtWidgets/QDialog>

class QCheckBox;
class QComboBox;
class QLabel;
class QSlider;
class QSpinBox;
class QToolButton;

namespace pictura {

class PictureView;
class ToolController;

// Select > Color Range (SEL-005): pick a sampled colour, a hue band, or a tonal
// band; the preview is the coverage mask OK will select (white selected, black
// not, grey partly). Non-modal, so its eyedropper can reach the canvas: while
// it is down (Sampled Colors), a canvas click samples the composite instead of
// reaching the active tool. Ported from photorust's ColorRangeDialog.
// ponytail: no Skin Tones, Detect Faces, Localized Color Clusters, Out Of
// Gamut, plus / minus eyedroppers, Image preview, Selection Preview modes, or
// Save / Load.
class ColorRangeDialog : public QDialog {
    Q_OBJECT

public:
    ColorRangeDialog(PictureView* view, ToolController* tools, QWidget* parent = nullptr);
    ~ColorRangeDialog() override;

    // 0 Sampled Colors ... 9 Shadows (the engine's numbering).
    PictureView* view() const { return view_; }
    int range() const;
    QColor sampledColor() const { return sampled_; }
    int fuzziness() const;
    bool inverted() const;
    // Take the composite colour at `imagePos` (the canvas sampler's entry).
    void sampleAt(const QPointF& imagePos);
    // The preview's mask, as shown (for the Qt Test).
    QImage previewForTest() const { return preview_; }

protected:
    void hideEvent(QHideEvent* event) override;

private:
    void refreshSampler();
    void refreshPreview();
    void refreshEnabled();

    PictureView* view_ = nullptr;
    ToolController* tools_ = nullptr;
    QComboBox* select_ = nullptr;
    QSpinBox* fuzziness_ = nullptr;
    QSlider* fuzzinessSlider_ = nullptr;
    QCheckBox* invert_ = nullptr;
    QLabel* previewLabel_ = nullptr;
    QLabel* swatch_ = nullptr;
    QToolButton* eyedropper_ = nullptr;
    QColor sampled_{Qt::black};
    QImage preview_;
};

} // namespace pictura
