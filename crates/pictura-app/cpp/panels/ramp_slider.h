#pragma once

#include <QtCore/QList>
#include <QtGui/QColor>

#include "jump_slider.h"

namespace pictura {

// A slider whose groove is a colour ramp, and which tracks a press-drag like the
// shared JumpSlider. CS6 draws the Hue/Saturation sliders this way. The ramp is
// a stylesheet on the groove rather than custom painting so the theme, handle,
// hit testing, and disabled state stay exactly as on every other slider.
class RampSlider : public JumpSlider {
    Q_OBJECT

public:
    explicit RampSlider(QWidget* parent = nullptr);

    // Colours across the groove, evenly spaced. Fewer than two puts the ordinary
    // groove back.
    void setRamp(const QList<QColor>& stops);

private:
    QList<QColor> stops_;
};

} // namespace pictura
