#include "ramp_slider.h"

#include <QtCore/QString>
#include <QtCore/QStringList>

namespace pictura {

RampSlider::RampSlider(QWidget* parent)
    : JumpSlider(Qt::Horizontal, parent)
{
}

void RampSlider::setRamp(const QList<QColor>& stops)
{
    // Rebuilding a stylesheet makes the widget recalculate its rules, and the
    // hue ramp is asked for again on every tick of a drag, so nothing happens
    // unless the colours actually moved.
    if (stops == stops_) {
        return;
    }
    stops_ = stops;

    if (stops.size() < 2) {
        setStyleSheet(QString());
        return;
    }

    QStringList gradient;
    for (int i = 0; i < stops.size(); ++i) {
        gradient << QStringLiteral("stop:%1 %2")
                        .arg(qreal(i) / (stops.size() - 1))
                        .arg(stops.at(i).name());
    }

    // Taller than the theme's 3px line, which is too thin to read a rainbow off,
    // and with the filled sub-page turned off: on a ramp there is no "how far
    // along" to shade, the colour is the information.
    setStyleSheet(QStringLiteral(
                      "QSlider::groove:horizontal {"
                      "  height: 7px; border: 1px solid #2a2a2a; border-radius: 0px;"
                      "  background: qlineargradient(x1:0, y1:0, x2:1, y2:0, %1); }"
                      "QSlider::sub-page:horizontal { background: transparent; }")
                      .arg(gradient.join(QStringLiteral(", "))));
}

} // namespace pictura
