#pragma once

#include <QtWidgets/QWidget>

class QMouseEvent;
class QPaintEvent;

namespace pictura {

// CS6's angle dial: a circle with a radius line, dragged to set the angle. The
// line points at the direction the light comes from, so 90° points up rather
// than down: screen y counts the other way.
class AngleDial : public QWidget {
    Q_OBJECT

public:
    explicit AngleDial(QWidget* parent = nullptr);

    // Wraps into [0, 360). A no-op when the wrapped value is unchanged.
    void setAngle(double degrees);
    double angle() const { return angle_; }

signals:
    void angleChanged(double degrees);

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;

private:
    void aim(const QPointF& pos);

    double angle_ = 0.0;
};

} // namespace pictura
