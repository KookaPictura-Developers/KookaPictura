#pragma once

#include "adjustment_dialog.h"

namespace pictura {

// Image > Adjustments > Black & White in the newer single-column layout: the
// six colour rows (percent fields over black-to-colour ramp sliders), Tint
// with its colour swatch, Default (the six back to CS6's defaults), Preview,
// and Cancel / OK. Ported from photorust's BlackWhiteDialog.
// ponytail: no Preset menu, Auto, or on-canvas scrubby targeting.
class BlackWhiteDialog : public AdjustmentDialog {
    Q_OBJECT

public:
    BlackWhiteDialog(PictureView* view, const QByteArray& block, const QRect& visible,
                     QWidget* parent = nullptr);

    QWidget* controlForTest(const QString& key) const override;
};

} // namespace pictura
