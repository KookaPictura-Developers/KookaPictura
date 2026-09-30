#pragma once

#include <QtWidgets/QWidget>

class QPaintEvent;

namespace pictura {

// A rainbow strip whose colours rotate with the hue shift, so a Hue/Saturation
// control says what it does before it is moved.
class SpectrumBar : public QWidget {
    Q_OBJECT

public:
    explicit SpectrumBar(QWidget* parent = nullptr);

    void setHueShift(int degrees);

protected:
    void paintEvent(QPaintEvent* event) override;

private:
    int hueShift_ = 0;
};

} // namespace pictura
