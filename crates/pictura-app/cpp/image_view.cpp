#include "image_view.h"

#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPixmap>
#include <QtGui/QWheelEvent>

#include <algorithm>
#include <cmath>

namespace pictura {

namespace {
constexpr double kMinZoom = 0.01;
constexpr double kMaxZoom = 32.0;
// ponytail: cap the whole-document present cache at ~64 MP (256 MB); beyond it
// painting falls back to the transform path rather than risking an OOM.
constexpr qint64 kMaxCachePixels = 64ll * 1024 * 1024;

// 2x2-cell tile reused for every transparency fill. Built lazily on the GUI
// thread the first time a document is painted.
const QPixmap& transparencyTile()
{
    static const QPixmap tile = [] {
        const int cell = ImageView::transparencyCellSize();
        QPixmap pm(2 * cell, 2 * cell);
        pm.fill(ImageView::transparencyColorA());
        QPainter p(&pm);
        p.fillRect(cell, 0, cell, cell, ImageView::transparencyColorB());
        p.fillRect(0, cell, cell, cell, ImageView::transparencyColorB());
        return pm;
    }();
    return tile;
}
} // namespace

int ImageView::transparencyCellSize()
{
    return 8;
}

QColor ImageView::transparencyColorA()
{
    return QColor(255, 255, 255);
}

QColor ImageView::transparencyColorB()
{
    return QColor(204, 204, 204);
}

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

void ImageView::blitRegion(const QImage& region, int x, int y)
{
    if (region.isNull() || region.width() <= 0 || region.height() <= 0 || image_.isNull()) {
        return;
    }
    QPainter painter(&image_);
    // Source mode overwrites the destination pixels; QPainter clips the draw to
    // the image rect, so a region straddling the document edge cannot spill.
    painter.setCompositionMode(QPainter::CompositionMode_Source);
    painter.drawImage(QPoint(x, y), region);
    painter.end();
    // An in-place QPainter write does not reliably bump image_.cacheKey(), so
    // invalidate the scaled present cache explicitly; the next paint rebuilds it
    // through the same transform as a direct draw.
    presentCache_.valid = false;
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

void ImageView::setPresentCacheEnabledForTest(bool enabled)
{
    presentCacheEnabledForTest_ = enabled;
}

const QImage* ImageView::cachedScaled(PresentCache& cache, const QImage& source)
{
    if (!presentCacheEnabledForTest_ || source.isNull()) {
        return nullptr;
    }
    const qint64 targetW = qint64(source.width() * zoom_);
    const qint64 targetH = qint64(source.height() * zoom_);
    if (targetW <= 0 || targetH <= 0 || targetW * targetH > kMaxCachePixels) {
        return nullptr;
    }
    if (cache.valid && cache.key == source.cacheKey() && cache.zoom == zoom_) {
        return &cache.scaled;
    }
    // Build through the same painter transform the direct path uses so the
    // presented pixels match it exactly; QImage::scaled would resample with a
    // different filter.
    cache.scaled = QImage(int(targetW), int(targetH), QImage::Format_ARGB32_Premultiplied);
    cache.scaled.fill(Qt::transparent);
    {
        QPainter builder(&cache.scaled);
        builder.scale(zoom_, zoom_);
        builder.setClipRect(QRectF(0.0, 0.0, source.width(), source.height()));
        builder.drawImage(QPointF(0.0, 0.0), source);
    }
    cache.key = source.cacheKey();
    cache.zoom = zoom_;
    cache.valid = true;
    ++cache.rebuilds;
    return &cache.scaled;
}

void ImageView::paintEvent(QPaintEvent*)
{
    QPainter painter(this);
    painter.fillRect(rect(), canvasColor_);
    if (image_.isNull()) {
        return;
    }

    // Checkerboard in screen space, anchored to the document origin and
    // clipped to the document rect so it never spills onto the canvas.
    const QRectF docRect(offset_, QSizeF(image_.width() * zoom_, image_.height() * zoom_));
    const QRectF checkerRect = docRect.intersected(QRectF(rect()));
    if (!checkerRect.isEmpty()) {
        painter.setBrushOrigin(docRect.topLeft().toPoint());
        painter.fillRect(checkerRect, QBrush(transparencyTile()));
    }

    painter.translate(offset_);
    painter.scale(zoom_, zoom_);
    // Crop the base, the move-preview layer, and the overlay to the document.
    painter.setClipRect(QRectF(0.0, 0.0, image_.width(), image_.height()));

    if (movePreviewActive_ && !moveBase_.isNull()) {
        const QImage* base = cachedScaled(moveBaseCache_, moveBase_);
        const QImage* layer = cachedScaled(moveLayerCache_, moveLayer_);
        if (base && layer) {
            painter.save();
            painter.resetTransform();
            painter.translate(offset_);
            painter.drawImage(QPointF(0.0, 0.0), *base);
            painter.setOpacity(moveOpacity_);
            painter.drawImage((moveLayerPos_ + moveDelta_) * zoom_, *layer);
            painter.setOpacity(1.0);
            painter.restore();
        } else {
            painter.drawImage(QPointF(0.0, 0.0), moveBase_);
            painter.setOpacity(moveOpacity_);
            painter.drawImage(moveLayerPos_ + moveDelta_, moveLayer_);
            painter.setOpacity(1.0);
        }
        presentCacheRebuiltLastPaint_ = false;
    } else {
        const int before = presentCache_.rebuilds;
        const QImage* base = cachedScaled(presentCache_, image_);
        if (base) {
            painter.save();
            painter.resetTransform();
            painter.translate(offset_);
            painter.drawImage(QPointF(0.0, 0.0), *base);
            painter.restore();
        } else {
            painter.drawImage(QPointF(0.0, 0.0), image_);
        }
        presentCacheRebuiltLastPaint_ = presentCache_.rebuilds != before;
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
