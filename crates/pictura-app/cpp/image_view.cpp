#include "image_view.h"

#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QWheelEvent>

#include <algorithm>
#include <cmath>

namespace pictura {

namespace {
constexpr double kMinZoom = 0.01;
constexpr double kMaxZoom = 32.0;
} // namespace

ImageView::ImageView(QWidget* parent)
    : QWidget(parent)
{
    setMinimumSize(320, 240);
    setFocusPolicy(Qt::StrongFocus);
}

void ImageView::setImage(const QImage& image)
{
    image_ = image;
    zoom_ = 1.0;
    offset_ = QPointF();
    update();
}

void ImageView::replaceImage(const QImage& image)
{
    image_ = image;
    update();
}

void ImageView::zoomAt(const QPointF& cursor, int angleDelta)
{
    const double factor = std::pow(1.0015, angleDelta);
    setZoom(zoom_ * factor, cursor);
}

void ImageView::panBy(const QPointF& delta)
{
    offset_ += delta;
    update();
}

void ImageView::zoomIn()
{
    setZoom(zoom_ * 1.2, QPointF(width() / 2.0, height() / 2.0));
}

void ImageView::zoomOut()
{
    setZoom(zoom_ / 1.2, QPointF(width() / 2.0, height() / 2.0));
}

void ImageView::fitOnScreen()
{
    if (image_.isNull()) {
        return;
    }
    const double fit = std::min(double(width()) / image_.width(),
                                double(height()) / image_.height())
                       * 0.95;
    zoom_ = std::clamp(fit, kMinZoom, kMaxZoom);
    offset_ = QPointF((width() - image_.width() * zoom_) / 2.0,
                      (height() - image_.height() * zoom_) / 2.0);
    emit zoomChanged(zoom_);
    update();
}

void ImageView::actualPixels()
{
    zoom_ = 1.0;
    offset_ = image_.isNull()
                  ? QPointF()
                  : QPointF((width() - image_.width()) / 2.0,
                            (height() - image_.height()) / 2.0);
    emit zoomChanged(zoom_);
    update();
}

void ImageView::setCanvasColor(const QColor& color)
{
    canvasColor_ = color;
    update();
}

void ImageView::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    painter.fillRect(rect(), canvasColor_);
    if (image_.isNull()) {
        return;
    }
    painter.translate(offset_);
    painter.scale(zoom_, zoom_);
    painter.drawImage(QPointF(0.0, 0.0), image_);
}

void ImageView::wheelEvent(QWheelEvent* event)
{
    zoomAt(event->position(), event->angleDelta().y());
    event->accept();
}

void ImageView::mousePressEvent(QMouseEvent* event)
{
    if (event->button() == Qt::LeftButton) {
        last_ = event->position();
    }
    QWidget::mousePressEvent(event);
}

void ImageView::mouseMoveEvent(QMouseEvent* event)
{
    if (event->buttons() & Qt::LeftButton) {
        panBy(event->position() - last_);
        last_ = event->position();
    }
    QWidget::mouseMoveEvent(event);
}

void ImageView::setZoom(double zoom, const QPointF& anchor)
{
    const double target = std::clamp(zoom, kMinZoom, kMaxZoom);
    const double factor = (zoom_ > 0.0) ? target / zoom_ : 1.0;
    offset_ = anchor - (anchor - offset_) * factor;
    zoom_ = target;
    emit zoomChanged(zoom_);
    update();
}

} // namespace pictura
