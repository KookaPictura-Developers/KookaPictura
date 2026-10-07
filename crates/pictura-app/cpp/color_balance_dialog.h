#pragma once

#include "adjustment_dialog.h"

class QButtonGroup;

namespace pictura {

// Image > Adjustments > Color Balance in the newer single-column layout: Tone
// radios (Shadows, Midtones, Highlights), the Cyan · Red, Magenta · Green, and
// Yellow · Blue rows over colour-ramp sliders, Preserve Luminosity, Preview,
// and Cancel / OK. Each tone edits its own triple. Ported from photorust's
// ColorBalanceDialog.
class ColorBalanceDialog : public AdjustmentDialog {
    Q_OBJECT

public:
    ColorBalanceDialog(PictureView* view, const QByteArray& block, const QRect& visible,
                       QWidget* parent = nullptr);

    QWidget* controlForTest(const QString& key) const override;

protected:
    QString paramKey(const QString& key) const override;

private:
    void loadTone();

    QButtonGroup* tone_ = nullptr;
};

} // namespace pictura
