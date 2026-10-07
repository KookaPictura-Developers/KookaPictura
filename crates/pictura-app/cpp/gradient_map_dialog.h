#pragma once

#include "adjustment_dialog.h"
#include "gradient_editor_dialog.h"

#include <functional>

namespace pictura {

// Image > Adjustments > Gradient Map in CS6's layout: the Gradient Used for
// Grayscale Mapping sample (click it for the Gradient Editor; its arrow lists
// the presets), Gradient Options (Dither, Reverse), and OK / Cancel / Preview
// down the right. The canvas previews each Gradient Editor edit as it is made;
// cancelling the editor puts the previous gradient back. Ported from
// photorust's GradientMapDialog.
// ponytail: Dither is stored in the block but the map is not dithered.
class GradientMapDialog : public AdjustmentDialog {
    Q_OBJECT

public:
    GradientMapDialog(PictureView* view, const QByteArray& block, const QRect& visible,
                      QWidget* parent = nullptr);

    QWidget* controlForTest(const QString& key) const override;
    const GradientSpec& spec() const { return spec_; }
    void setSpec(const GradientSpec& spec);
    // Open the Gradient Editor on the current gradient, previewing its edits.
    // `editor` (tests) sees the dialog before it runs.
    void openEditor(const std::function<void(GradientEditorDialog&)>& editor = {});

private:
    void rebuild();
    void paintSample();

    GradientSpec spec_;
    QColor foreground_;
    QColor background_;
    QToolButton* sample_ = nullptr;
    QToolButton* presets_ = nullptr;
    QCheckBox* dither_ = nullptr;
    QCheckBox* reverse_ = nullptr;
};

} // namespace pictura
