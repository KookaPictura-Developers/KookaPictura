#include "curve_widget.h"

#include <QtCore/QRectF>
#include <QtGui/QColor>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPaintEvent>

#include <algorithm>
#include <cmath>

namespace pictura {
namespace {

// Monotone cubic spline (Fritsch-Carlson), evaluated at each integer input.
void splineInterpolate(const QVector<QPointF>& pts, float out[256])
{
    const int n = pts.size();
    if (n == 0) {
        for (int i = 0; i < 256; ++i) {
            out[i] = i / 255.0f;
        }
        return;
    }
    if (n == 1) {
        for (int i = 0; i < 256; ++i) {
            out[i] = qBound(0.0f, static_cast<float>(pts[0].y()), 1.0f);
        }
        return;
    }

    QVector<double> dx(n - 1), dy(n - 1), m(n);
    for (int i = 0; i < n - 1; ++i) {
        dx[i] = pts[i + 1].x() - pts[i].x();
        dy[i] = pts[i + 1].y() - pts[i].y();
    }
    QVector<double> slopes(n - 1);
    for (int i = 0; i < n - 1; ++i) {
        slopes[i] = dx[i] > 1e-9 ? dy[i] / dx[i] : 0.0;
    }

    m[0] = slopes[0];
    m[n - 1] = slopes[n - 2];
    for (int i = 1; i < n - 1; ++i) {
        m[i] = (slopes[i - 1] + slopes[i]) * 0.5;
    }

    for (int i = 0; i < n - 1; ++i) {
        if (std::abs(slopes[i]) < 1e-9) {
            m[i] = 0;
            m[i + 1] = 0;
        } else {
            double a = m[i] / slopes[i];
            double b = m[i + 1] / slopes[i];
            double h = std::hypot(a, b);
            if (h > 3.0) {
                double t = 3.0 / h;
                m[i] = t * a * slopes[i];
                m[i + 1] = t * b * slopes[i];
            }
        }
    }

    for (int ix = 0; ix < 256; ++ix) {
        double x = ix / 255.0;
        if (x <= pts[0].x()) {
            out[ix] = qBound(0.0f, static_cast<float>(pts[0].y()), 1.0f);
            continue;
        }
        if (x >= pts[n - 1].x()) {
            out[ix] = qBound(0.0f, static_cast<float>(pts[n - 1].y()), 1.0f);
            continue;
        }
        int seg = 0;
        for (int i = n - 2; i >= 0; --i) {
            if (x >= pts[i].x()) {
                seg = i;
                break;
            }
        }
        double h = dx[seg];
        if (h < 1e-12) {
            out[ix] = qBound(0.0f, static_cast<float>(pts[seg].y()), 1.0f);
            continue;
        }
        double t = (x - pts[seg].x()) / h;
        double t2 = t * t, t3 = t2 * t;
        double h00 = 2 * t3 - 3 * t2 + 1;
        double h10 = t3 - 2 * t2 + t;
        double h01 = -2 * t3 + 3 * t2;
        double h11 = t3 - t2;
        double val = h00 * pts[seg].y() + h10 * h * m[seg] + h01 * pts[seg + 1].y()
                     + h11 * h * m[seg + 1];
        out[ix] = qBound(0.0f, static_cast<float>(val), 1.0f);
    }
}

} // namespace

CurveWidget::CurveWidget(QWidget* parent)
    : QWidget(parent)
{
    setFixedSize(kSize + 2, kSize + 2);
    resetCurve();
}

void CurveWidget::resetCurve()
{
    points_.clear();
    points_.append(QPointF(0.0, 0.0));
    points_.append(QPointF(1.0, 1.0));
    interpolate();
    update();
}

void CurveWidget::setPoints(const QVector<QPointF>& pts)
{
    points_ = pts;
    interpolate();
    update();
    emit curveChanged();
}

void CurveWidget::setHistogram(const QImage& img, int channel)
{
    std::fill(std::begin(histo_), std::end(histo_), 0);
    const QImage src = img.convertToFormat(QImage::Format_ARGB32);
    for (int y = 0; y < src.height(); ++y) {
        const auto* line = reinterpret_cast<const QRgb*>(src.constScanLine(y));
        for (int x = 0; x < src.width(); ++x) {
            const QRgb px = line[x];
            int val = 0;
            switch (channel) {
            case 0:
                val = qGray(px);
                break;
            case 1:
                val = qRed(px);
                break;
            case 2:
                val = qGreen(px);
                break;
            case 3:
                val = qBlue(px);
                break;
            }
            histo_[val]++;
        }
    }
    histoPeak_ = 1;
    for (int i = 0; i < 256; ++i) {
        histoPeak_ = qMax(histoPeak_, histo_[i]);
    }
    update();
}

void CurveWidget::buildLut(uint8_t lut[256]) const
{
    for (int i = 0; i < 256; ++i) {
        lut[i] = static_cast<uint8_t>(qBound(0.0f, curve_[i] * 255.0f + 0.5f, 255.0f));
    }
}

void CurveWidget::interpolate()
{
    std::sort(points_.begin(), points_.end(),
              [](const QPointF& a, const QPointF& b) { return a.x() < b.x(); });
    splineInterpolate(points_, curve_);
}

QPointF CurveWidget::toWidget(QPointF p) const
{
    return QPointF(1 + p.x() * kSize, 1 + (1.0 - p.y()) * kSize);
}

QPointF CurveWidget::fromWidget(QPointF p) const
{
    return QPointF((p.x() - 1) / kSize, 1.0 - (p.y() - 1) / kSize);
}

void CurveWidget::paintEvent(QPaintEvent*)
{
    QPainter p(this);
    p.setRenderHint(QPainter::Antialiasing);

    p.fillRect(rect(), Qt::white);

    if (showHisto_) {
        p.setPen(Qt::NoPen);
        p.setBrush(QColor(220, 220, 220));
        for (int i = 0; i < 256; ++i) {
            int barH = static_cast<int>(
                static_cast<double>(histo_[i]) / histoPeak_ * kSize);
            if (barH > 0) {
                p.drawRect(1 + i, 1 + kSize - barH, 1, barH);
            }
        }
    }

    p.setPen(QPen(QColor(200, 200, 200), 1));
    for (int i = 1; i < 4; ++i) {
        int pos = 1 + i * kSize / 4;
        p.drawLine(pos, 1, pos, 1 + kSize);
        p.drawLine(1, pos, 1 + kSize, pos);
    }

    if (showBaseline_) {
        p.setPen(QPen(QColor(180, 180, 180), 1, Qt::DashLine));
        p.drawLine(1, 1 + kSize, 1 + kSize, 1);
    }

    p.setPen(QPen(Qt::black, 1.5));
    for (int i = 0; i < 255; ++i) {
        QPointF a = toWidget(QPointF(i / 255.0, curve_[i]));
        QPointF b = toWidget(QPointF((i + 1) / 255.0, curve_[i + 1]));
        p.drawLine(a, b);
    }

    p.setPen(QPen(Qt::black, 1));
    for (const auto& pt : points_) {
        QPointF w = toWidget(pt);
        p.setBrush(Qt::white);
        p.drawEllipse(w, 4, 4);
    }

    p.setPen(QPen(QColor(150, 150, 150), 1));
    p.setBrush(Qt::NoBrush);
    p.drawRect(QRectF(0.5, 0.5, kSize + 1, kSize + 1));
}

void CurveWidget::mousePressEvent(QMouseEvent* event)
{
    QPointF pos = fromWidget(event->position());
    dragging_ = -1;

    for (int i = 0; i < points_.size(); ++i) {
        QPointF w = toWidget(points_[i]);
        if ((event->position() - w).manhattanLength() < 10) {
            dragging_ = i;
            return;
        }
    }

    pos.setX(qBound(0.0, pos.x(), 1.0));
    pos.setY(qBound(0.0, pos.y(), 1.0));
    points_.append(pos);
    std::sort(points_.begin(), points_.end(),
              [](const QPointF& a, const QPointF& b) { return a.x() < b.x(); });
    for (int i = 0; i < points_.size(); ++i) {
        if (points_[i] == pos) {
            dragging_ = i;
            break;
        }
    }
    interpolate();
    update();
    emit curveChanged();
}

void CurveWidget::mouseMoveEvent(QMouseEvent* event)
{
    if (dragging_ < 0) {
        return;
    }
    QPointF pos = fromWidget(event->position());
    pos.setX(qBound(0.0, pos.x(), 1.0));
    pos.setY(qBound(0.0, pos.y(), 1.0));

    if (dragging_ == 0) {
        pos.setX(0.0);
    } else if (dragging_ == points_.size() - 1) {
        pos.setX(1.0);
    }

    points_[dragging_] = pos;
    interpolate();
    for (int i = 0; i < points_.size(); ++i) {
        if (points_[i] == pos) {
            dragging_ = i;
            break;
        }
    }
    update();
    emit curveChanged();
}

void CurveWidget::mouseReleaseEvent(QMouseEvent* event)
{
    if (dragging_ >= 0) {
        QPointF pos = fromWidget(event->position());
        if (dragging_ > 0 && dragging_ < points_.size() - 1) {
            if (pos.x() < -0.05 || pos.x() > 1.05 || pos.y() < -0.05
                || pos.y() > 1.05) {
                points_.removeAt(dragging_);
                interpolate();
                update();
                emit curveChanged();
            }
        }
    }
    dragging_ = -1;
}

} // namespace pictura
