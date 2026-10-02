#pragma once

#include <QtCore/QPointF>
#include <QtCore/QVector>
#include <QtGui/QImage>
#include <QtWidgets/QWidget>

#include <cstdint>

class QMouseEvent;
class QPaintEvent;

namespace pictura {

// The Curves editor: a histogram, a 4x4 grid, a baseline diagonal, and the
// monotone cubic-spline curve through its control points. Dragging a control
// point re-interpolates and emits `curveChanged`; dragging a non-endpoint off
// the face removes it.
class CurveWidget : public QWidget {
    Q_OBJECT

public:
    explicit CurveWidget(QWidget* parent = nullptr);

    void setHistogram(const QImage& img, int channel);
    void setShowHistogram(bool v)
    {
        showHisto_ = v;
        update();
    }
    void setShowBaseline(bool v)
    {
        showBaseline_ = v;
        update();
    }

    void resetCurve();
    void setPoints(const QVector<QPointF>& pts);
    QVector<QPointF> points() const { return points_; }

    void buildLut(uint8_t lut[256]) const;

signals:
    void curveChanged();

protected:
    void paintEvent(QPaintEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;

private:
    void interpolate();
    QPointF toWidget(QPointF p) const;
    QPointF fromWidget(QPointF p) const;

    QVector<QPointF> points_;
    float curve_[256]{};
    int dragging_ = -1;
    int histo_[256]{};
    int histoPeak_ = 1;
    bool showHisto_ = true;
    bool showBaseline_ = true;
    static constexpr int kSize = 256;
};

} // namespace pictura
