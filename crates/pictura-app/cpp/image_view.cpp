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
    userAdjusted_ = false;
    if (image_.isNull()) {
        zoom_ = 1.0;
        offset_ = QPointF();
        update();
        return;
    }
    applyInitialView();
}

void ImageView::applyInitialView()
{
    if (image_.isNull()) {
        return;
    }
    if (image_.width() > width() || image_.height() > height()) {
        const double fit = std::min(double(width()) / image_.width(),
                                    double(height()) / image_.height())
                           * 0.95;
        zoom_ = std::clamp(fit, kMinZoom, kMaxZoom);
    } else {
        zoom_ = 1.0;
    }
    centreImage();
    emit zoomChanged(zoom_);
    update();
}

void ImageView::centreImage()
{
    offset_ = QPointF((width() - image_.width() * zoom_) / 2.0,
                      (height() - image_.height() * zoom_) / 2.0);
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
    userAdjusted_ = true;
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
    userAdjusted_ = false;
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
    userAdjusted_ = false;
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

void ImageView::setPanEnabled(bool enabled)
{
    panEnabled_ = enabled;
    if (!enabled) {
        panning_ = false;
    }
}

void ImageView::setOverlayPolygon(const QPolygonF& polygon)
{
    overlayPolygon_ = polygon;
    update();
}

void ImageView::clearOverlay()
{
    overlayPolygon_.clear();
    update();
}

void ImageView::beginMovePreview(const QImage& base, const QImage& layer, const QPointF& layerPos,
                                 double opacity)
{
    moveBase_ = base;
    moveLayer_ = layer;
    moveLayerPos_ = layerPos;
    moveDelta_ = QPointF();
    moveOpacity_ = opacity;
    movePreviewActive_ = true;
    update();
}

void ImageView::setMovePreviewDelta(const QPointF& delta)
{
    moveDelta_ = delta;
    update();
}

void ImageView::endMovePreview()
{
    movePreviewActive_ = false;
    moveBase_ = QImage();
    moveLayer_ = QImage();
    moveDelta_ = QPointF();
    moveOpacity_ = 1.0;
    update();
}

QPointF ImageView::widgetToImage(const QPointF& widgetPos) const
{
    return (widgetPos - offset_) / zoom_;
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
    if (movePreviewActive_ && !moveBase_.isNull()) {
        painter.drawImage(QPointF(0.0, 0.0), moveBase_);
        painter.setOpacity(moveOpacity_);
        painter.drawImage(moveLayerPos_ + moveDelta_, moveLayer_);
        painter.setOpacity(1.0);
    } else {
        painter.drawImage(QPointF(0.0, 0.0), image_);
    }

    if (!overlayPolygon_.isEmpty()) {
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0, Qt::DashLine));
        painter.drawPolygon(overlayPolygon_);
    }
}

void ImageView::wheelEvent(QWheelEvent* event)
{
    zoomAt(event->position(), event->angleDelta().y());
    event->accept();
}

void ImageView::mousePressEvent(QMouseEvent* event)
{
    if (event->button() == Qt::MiddleButton) {
        panning_ = true;
        userAdjusted_ = true;
        last_ = event->position();
        setCursor(Qt::ClosedHandCursor);
    } else if (panEnabled_ && event->button() == Qt::LeftButton) {
        panning_ = true;
        userAdjusted_ = true;
        last_ = event->position();
    } else {
        emit mousePressed(widgetToImage(event->position()), event->button(),
                          event->modifiers());
    }
    QWidget::mousePressEvent(event);
}

void ImageView::mouseMoveEvent(QMouseEvent* event)
{
    const Qt::MouseButtons buttons = event->buttons();
    if (panning_ && (buttons & (Qt::MiddleButton | Qt::LeftButton))) {
        panBy(event->position() - last_);
        last_ = event->position();
    } else {
        emit mouseMoved(widgetToImage(event->position()));
    }
    QWidget::mouseMoveEvent(event);
}

void ImageView::mouseReleaseEvent(QMouseEvent* event)
{
    if (event->button() == Qt::MiddleButton || event->button() == Qt::LeftButton) {
        panning_ = false;
    }
    if (event->button() == Qt::MiddleButton) {
        unsetCursor();
    }
    emit mouseReleased(widgetToImage(event->position()));
    QWidget::mouseReleaseEvent(event);
}

void ImageView::resizeEvent(QResizeEvent* event)
{
    if (!userAdjusted_ && !image_.isNull()) {
        applyInitialView();
    }
    QWidget::resizeEvent(event);
}

void ImageView::setZoom(double zoom, const QPointF& anchor)
{
    userAdjusted_ = true;
    const double target = std::clamp(zoom, kMinZoom, kMaxZoom);
    const double factor = (zoom_ > 0.0) ? target / zoom_ : 1.0;
    offset_ = anchor - (anchor - offset_) * factor;
    zoom_ = target;
    emit zoomChanged(zoom_);
    update();
}

} // namespace pictura
