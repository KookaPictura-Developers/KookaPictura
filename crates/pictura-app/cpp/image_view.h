#pragma once

#include <QtCore/QPointF>
#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtWidgets/QWidget>

class QMouseEvent;
class QPaintEvent;
class QWheelEvent;

namespace pictura {

// The central document canvas: paints a QImage under a pan/zoom transform over
// a plain canvas colour. GPU compositing is out of scope (CPU image for now).
class ImageView : public QWidget {
    Q_OBJECT

public:
    explicit ImageView(QWidget* parent = nullptr);

    // Replace the image and reset the view (zoom 1.0, centred).
    void setImage(const QImage& image);

    // Replace the image but keep the current zoom and pan.
    void replaceImage(const QImage& image);

    const QImage& image() const { return image_; }

    // Zoom about a cursor position so the point under the cursor stays put.
    void zoomAt(const QPointF& cursor, int angleDelta);
    void panBy(const QPointF& delta);

    void zoomIn();
    void zoomOut();
    void fitOnScreen();
    void actualPixels();

    double zoom() const { return zoom_; }
    QPointF offset() const { return offset_; }

    void setCanvasColor(const QColor& color);
    QColor canvasColor() const { return canvasColor_; }

signals:
    void zoomChanged(double zoom);

protected:
    void paintEvent(QPaintEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;

private:
    void setZoom(double zoom, const QPointF& anchor);

    QImage image_;
    QColor canvasColor_{Qt::darkGray};
    double zoom_ = 1.0;
    QPointF offset_;
    QPointF last_;
};

} // namespace pictura
