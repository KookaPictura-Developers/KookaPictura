#include "image_view.h"

#include "canvas_range.h"

#include <QtCore/QDebug>
#include <QtCore/QRect>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtCore/QtNumeric>
#include <QtGui/QFontMetrics>
#include <QtGui/QKeyEvent>
#include <QtGui/QMouseEvent>
#include <QtGui/QPainter>
#include <QtGui/QPixmap>
#include <QtGui/QWheelEvent>

#include <algorithm>
#include <cmath>
#include <cstring>

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
    setMouseTracking(true);
    antsTimer_ = new QTimer(this);
    antsTimer_->setInterval(120);
    connect(antsTimer_, &QTimer::timeout, this, [this]() {
        antsPhase_ = (antsPhase_ + 1) % 8;
        update();
    });
}

void ImageView::setImage(const QImage& image)
{
    image_ = image;
    userAdjusted_ = false;
    if (image_.isNull()) {
        zoom_ = 1.0;
        offset_ = QPointF();
        update();
        emit viewChanged();
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
    clampOffset();
    emit zoomChanged(zoom_);
    emit viewChanged();
    update();
}

void ImageView::centreImage()
{
    offset_ = QPointF((width() - image_.width() * zoom_) / 2.0,
                      (height() - image_.height() * zoom_) / 2.0);
    clampOffset();
}

void ImageView::clampOffset()
{
    if (image_.isNull()) {
        offset_ = QPointF();
        return;
    }
    const OffsetRange range = offsetRangeFor(
        QSizeF(image_.width(), image_.height()), zoom_, QSizeF(width(), height()));
    offset_.setX(std::clamp(offset_.x(), range.minX, range.maxX));
    offset_.setY(std::clamp(offset_.y(), range.minY, range.maxY));
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
    // Copy the region rows straight into the canvas instead of constructing a
    // QPainter on the full-resolution image: a painter on a shared buffer
    // detaches the whole image. Own the buffer once, then every later dab is an
    // in-place row copy. CompositionMode_Source semantics (replace, clipped to
    // the image rect) are reproduced by the clamped memcpy below.
    if (!image_.isDetached()) {
        image_ = image_.copy();
    }
    QImage src = region;
    if (src.format() != image_.format()) {
        src = src.convertToFormat(image_.format());
    }
    const int x0 = std::max(0, x);
    const int y0 = std::max(0, y);
    const int x1 = std::min(image_.width(), x + src.width());
    const int y1 = std::min(image_.height(), y + src.height());
    if (x1 > x0 && y1 > y0) {
        const int bpp = std::max(1, image_.depth() / 8);
        const int rowBytes = (x1 - x0) * bpp;
        const int sx = x0 - x;
        const int sy = y0 - y;
        for (int row = 0; row < y1 - y0; ++row) {
            std::memcpy(image_.scanLine(y0 + row) + x0 * bpp,
                        src.constScanLine(sy + row) + sx * bpp, rowBytes);
        }
    }
    // Patch the same region of the scaled present cache under the identical
    // pan/zoom transform, so a dab does not force a full-document rescale on the
    // next paint (O(canvas) per dab). CompositionMode_Source matches the
    // full-rebuild result: the cache is one scaled copy of `image_`.
    if (presentCacheEnabledForTest_ && presentCache_.valid && presentCache_.zoom == zoom_
        && !presentCache_.scaled.isNull()) {
        QPainter patch(&presentCache_.scaled);
        patch.setCompositionMode(QPainter::CompositionMode_Source);
        // Qt composes `world = world * new`, so S(zoom) then T(x, y) maps a
        // region point p to (p + (x, y)) * zoom and lands the region's top-left
        // at scaled cache (x*zoom, y*zoom). The reversed order mapped it to
        // (x, y), off the cache at any zoom below 1.
        patch.scale(zoom_, zoom_);
        patch.translate(x, y);
        patch.drawImage(QPointF(0.0, 0.0), region);
        // An in-place QPainter write need not bump image_.cacheKey(); keep the
        // cache key in sync with the image it now mirrors (patched region plus
        // the untouched pixels that already matched).
        presentCache_.key = image_.cacheKey();
    } else {
        presentCache_.valid = false;
    }
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
    clampOffset();
    update();
    emit viewChanged();
}

void ImageView::setOffset(const QPointF& offset)
{
    userAdjusted_ = true;
    offset_ = offset;
    clampOffset();
    update();
    emit viewChanged();
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
    clampOffset();
    emit zoomChanged(zoom_);
    emit viewChanged();
    update();
}

void ImageView::actualPixels()
{
    // 100% is an explicit user choice: keep it when the workspace scrollbars
    // appear and resize the canvas instead of refitting on the next resize.
    userAdjusted_ = true;
    zoom_ = 1.0;
    offset_ = image_.isNull()
                  ? QPointF()
                  : QPointF((width() - image_.width()) / 2.0,
                            (height() - image_.height()) / 2.0);
    clampOffset();
    emit zoomChanged(zoom_);
    emit viewChanged();
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

void ImageView::setSpacePan(bool on)
{
    if (spacePan_ == on) {
        return;
    }
    spacePan_ = on;
    if (on) {
        setPanEnabled(true);
        setCursor(Qt::OpenHandCursor);
    } else {
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

void ImageView::setSelectionContour(const QString& encoded)
{
    selectionContours_.clear();
    if (!encoded.isEmpty()) {
        const QStringList loops = encoded.split(QLatin1Char(';'), Qt::SkipEmptyParts);
        for (const QString& loop : loops) {
            QPolygonF poly;
            const QStringList points = loop.split(QLatin1Char(' '), Qt::SkipEmptyParts);
            for (const QString& point : points) {
                const int comma = point.indexOf(QLatin1Char(','));
                if (comma < 0) {
                    continue;
                }
                bool okX = false;
                bool okY = false;
                const double x = point.left(comma).toDouble(&okX);
                const double y = point.mid(comma + 1).toDouble(&okY);
                if (okX && okY) {
                    poly << QPointF(x, y);
                }
            }
            if (poly.size() >= 2) {
                selectionContours_ << poly;
            }
        }
    }
    updateAntsTimer();
    update();
}

void ImageView::clearSelectionContour()
{
    selectionContours_.clear();
    updateAntsTimer();
    update();
}

void ImageView::setSelectionEdgesVisible(bool on)
{
    selectionEdgesVisible_ = on;
    updateAntsTimer();
    update();
}

void ImageView::setSelectionPreview(const QList<QPolygonF>& loops, bool closed, bool solid)
{
    previewContours_ = loops;
    selectionPreviewClosed_ = closed;
    selectionPreviewSolid_ = solid;
    updateAntsTimer();
    update();
}

void ImageView::setSelectionPreviewOrigin(const QPointF& imagePos)
{
    previewOrigin_ = imagePos;
    previewOriginActive_ = true;
    update();
}

void ImageView::clearSelectionPreview()
{
    previewContours_.clear();
    previewOriginActive_ = false;
    selectionPreviewClosed_ = true;
    selectionPreviewSolid_ = false;
    updateAntsTimer();
    update();
}

void ImageView::updateAntsTimer()
{
    const bool run = (!selectionContours_.isEmpty() && selectionEdgesVisible_)
                     || !previewContours_.isEmpty();
    const bool active = run && isVisible();
    if (active && !antsTimer_->isActive()) {
        antsTimer_->start();
    } else if (!active && antsTimer_->isActive()) {
        antsTimer_->stop();
    }
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

void ImageView::beginTransformPreview(const QImage& base, const QImage& layer,
                                      const QPointF& layerPos, double opacity)
{
    moveBase_ = base;
    moveLayer_ = layer;
    moveLayerPos_ = layerPos;
    moveDelta_ = QPointF();
    moveOpacity_ = opacity;
    movePreviewActive_ = true;
    transformActive_ = true;
    transformScaleX_ = 1.0;
    transformScaleY_ = 1.0;
    transformAngle_ = 0.0;
    transformDx_ = 0.0;
    transformDy_ = 0.0;
    transformProjective_ = QTransform();
    transformProjectiveActive_ = false;
    transformQuad_.clear();
    update();
}

void ImageView::setTransformPreviewProjective(const QString& matrix9)
{
    const QStringList values = matrix9.split(QLatin1Char(' '), Qt::SkipEmptyParts);
    if (values.size() != 9) {
        transformProjectiveActive_ = false;
        update();
        return;
    }
    double c[9] = {0.0};
    for (int i = 0; i < 9; ++i) {
        bool ok = false;
        c[i] = values.at(i).toDouble(&ok);
        if (!ok) {
            transformProjectiveActive_ = false;
            update();
            return;
        }
    }
    transformProjective_ = QTransform(c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], c[8]);
    transformProjectiveActive_ = true;
    update();
}

void ImageView::setTransformPreview(double scaleX, double scaleY, double angleRadians, double dx,
                                    double dy)
{
    transformScaleX_ = scaleX;
    transformScaleY_ = scaleY;
    transformAngle_ = angleRadians;
    transformDx_ = dx;
    transformDy_ = dy;
    update();
}

QTransform ImageView::transformPreviewMatrix() const
{
    const double cx = moveLayerPos_.x() + moveLayer_.width() / 2.0;
    const double cy = moveLayerPos_.y() + moveLayer_.height() / 2.0;
    QTransform matrix;
    matrix.translate(transformDx_, transformDy_);
    matrix.translate(cx, cy);
    matrix.rotate(transformAngle_ * 180.0 / M_PI);
    matrix.scale(transformScaleX_, transformScaleY_);
    matrix.translate(-cx, -cy);
    return matrix;
}

void ImageView::setTransformQuad(const QString& encoded)
{
    transformQuad_.clear();
    const QStringList points = encoded.split(QLatin1Char(' '), Qt::SkipEmptyParts);
    for (const QString& point : points) {
        const int comma = point.indexOf(QLatin1Char(','));
        if (comma < 0) {
            continue;
        }
        bool okX = false;
        bool okY = false;
        const double x = point.left(comma).toDouble(&okX);
        const double y = point.mid(comma + 1).toDouble(&okY);
        if (okX && okY) {
            transformQuad_ << QPointF(x, y);
        }
    }
    update();
}

void ImageView::clearTransformPreview()
{
    transformActive_ = false;
    transformProjective_ = QTransform();
    transformProjectiveActive_ = false;
    transformQuad_.clear();
    movePreviewActive_ = false;
    moveBase_ = QImage();
    moveLayer_ = QImage();
    moveDelta_ = QPointF();
    moveOpacity_ = 1.0;
    update();
}

void ImageView::setDragSizeHint(const QString& text, const QPointF& imagePos)
{
    dragSizeText_ = text;
    dragSizeImagePos_ = imagePos;
    dragSizeActive_ = !text.isEmpty();
    update();
}

void ImageView::clearDragSizeHint()
{
    dragSizeText_.clear();
    dragSizeImagePos_ = QPointF();
    dragSizeActive_ = false;
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
    // One integer device rect for the document, rounded the way fillRect rounds,
    // so the checkerboard and the cached base share an exact boundary. Without
    // this the floor-sized present cache fell a pixel short at the right/bottom
    // edge and let the checkerboard show through an opaque canvas-sized layer.
    const QRect docDevice(qRound(docRect.left()), qRound(docRect.top()),
                          qRound(docRect.right()) - qRound(docRect.left()),
                          qRound(docRect.bottom()) - qRound(docRect.top()));
    const QRect checkerRect = docDevice.intersected(rect());
    if (!checkerRect.isEmpty()) {
        painter.setBrushOrigin(docDevice.topLeft());
        painter.fillRect(checkerRect, QBrush(transparencyTile()));
    }

    painter.translate(offset_);
    painter.scale(zoom_, zoom_);
    // Crop the document content (base, preview, overlays) to the image rect.
    // The brush ring below is drawn outside this scope so it can render past
    // the document edge while staying inside the canvas widget.
    const QRectF docClip(0.0, 0.0, image_.width(), image_.height());
    painter.save();
    painter.setClipRect(docClip);

    if (transformActive_ && !moveBase_.isNull()) {
        // Free Transform: the base is fixed; the layer is drawn under the live
        // similarity transform about its original centre.
        painter.drawImage(QPointF(0.0, 0.0), moveBase_);
        painter.setOpacity(moveOpacity_);
        painter.save();
        painter.setTransform(transformProjectiveActive_ ? transformProjective_
                                                        : transformPreviewMatrix(), true);
        painter.drawImage(moveLayerPos_, moveLayer_);
        painter.restore();
        painter.setOpacity(1.0);
        presentCacheRebuiltLastPaint_ = false;
    } else if (movePreviewActive_ && !moveBase_.isNull()) {
        // Draw the base and moving layer straight through the pan/zoom
        // transform. A move preview is transient and its sources change every
        // drag, so a scaled present cache would be rebuilt on the first frame
        // of every drag (a full-size conversion copy: 16 MP ≈ 48 ms at 4000²
        // at 100%, 64 MP ≈ 4× that at 200%). Drawing directly is a steady
        // ~3–4 ms/frame at any zoom and removes the drag-start hitch.
        painter.drawImage(QPointF(0.0, 0.0), moveBase_);
        painter.setOpacity(moveOpacity_);
        painter.drawImage(moveLayerPos_ + moveDelta_, moveLayer_);
        painter.setOpacity(1.0);
        presentCacheRebuiltLastPaint_ = false;
    } else {
        const int before = presentCache_.rebuilds;
        const QImage* base = cachedScaled(presentCache_, image_);
        if (base) {
            painter.save();
            painter.resetTransform();
            painter.setClipRect(docDevice);
            painter.drawImage(docDevice, *base);
            painter.restore();
        } else {
            painter.drawImage(QPointF(0.0, 0.0), image_);
        }
        presentCacheRebuiltLastPaint_ = presentCache_.rebuilds != before;
    }
    painter.restore();

    if (brushOutlineActive_ && brushOutlineDiameter_ > 0.0) {
        // Cosmetic pens keep each ring 1 device px independent of zoom; the
        // radius is in image pixels, so the on-screen circle scales with zoom.
        // A white ring one image px outside the black one keeps the outline
        // legible on both light and dark documents. Drawn with the full-widget
        // clip, not the document clip, so the ring stays visible past the edge.
        const double radius = brushOutlineDiameter_ / 2.0;
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0));
        painter.drawEllipse(brushOutlineImagePos_, radius + 1.0, radius + 1.0);
        painter.setPen(QPen(Qt::black, 0));
        painter.drawEllipse(brushOutlineImagePos_, radius, radius);
    }

    painter.save();
    painter.setClipRect(docClip);

    if (!overlayPolygon_.isEmpty()) {
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0, Qt::DashLine));
        painter.drawPolygon(overlayPolygon_);
    }

    if (selectionEdgesVisible_ && !selectionContours_.isEmpty()) {
        // Marching ants: a solid white base under an animated black dashed pen,
        // so the edge reads against both light and dark pixels. Width 0 = a
        // cosmetic 1-px line independent of the pan/zoom transform.
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0));
        for (const QPolygonF& loop : selectionContours_) {
            painter.drawPolygon(loop);
        }
        QPen ants(Qt::black, 0, Qt::DashLine);
        ants.setDashOffset(antsPhase_);
        painter.setPen(ants);
        for (const QPolygonF& loop : selectionContours_) {
            painter.drawPolygon(loop);
        }
    }

    if (!previewContours_.isEmpty()) {
        // Live tool rubber band, so the selection reads during the drag exactly
        // as it will once committed. An open preview (Polygonal Lasso) is a
        // polyline: no phantom closing edge until it commits. A solid preview
        // draws white then black with no dash offset.
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0));
        for (const QPolygonF& loop : previewContours_) {
            if (selectionPreviewClosed_) {
                painter.drawPolygon(loop);
            } else {
                painter.drawPolyline(loop);
            }
        }
        QPen secondPen(Qt::black, 0,
                       selectionPreviewSolid_ ? Qt::SolidLine : Qt::DashLine);
        if (!selectionPreviewSolid_) {
            secondPen.setDashOffset(antsPhase_);
        }
        painter.setPen(secondPen);
        for (const QPolygonF& loop : previewContours_) {
            if (selectionPreviewClosed_) {
                painter.drawPolygon(loop);
            } else {
                painter.drawPolyline(loop);
            }
        }
        if (previewOriginActive_) {
            // 5 screen px whatever the zoom; the painter is in image space.
            const double half = 2.5 / std::max(zoom_, 1e-6);
            painter.setPen(QPen(Qt::black, 0));
            painter.setBrush(Qt::white);
            painter.drawRect(QRectF(previewOrigin_.x() - half, previewOrigin_.y() - half,
                                    2 * half, 2 * half));
            painter.setBrush(Qt::NoBrush);
        }
    }

    if (transformActive_ && transformQuad_.size() >= 4) {
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(Qt::white, 0, Qt::SolidLine));
        painter.drawPolygon(transformQuad_);
        painter.setPen(QPen(Qt::black, 0, Qt::DashLine));
        painter.drawPolygon(transformQuad_);

        // Eight scale handles at the corners and edge midpoints.
        const QPointF tl = transformQuad_.at(0);
        const QPointF tr = transformQuad_.at(1);
        const QPointF br = transformQuad_.at(2);
        const QPointF bl = transformQuad_.at(3);
        const QPointF handles[8] = {
            tl,
            tr,
            br,
            bl,
            (tl + tr) / 2.0,
            (tr + br) / 2.0,
            (br + bl) / 2.0,
            (bl + tl) / 2.0,
        };
        const double hs = 3.5 / (zoom_ > 0.0 ? zoom_ : 1.0);
        painter.setBrush(Qt::white);
        painter.setPen(QPen(Qt::black, 0, Qt::SolidLine));
        for (const QPointF& h : handles) {
            const QRectF box(h.x() - hs, h.y() - hs, 2.0 * hs, 2.0 * hs);
            painter.drawRect(box);
        }

        // Rotate affordance: a short stem and knob below the bottom edge.
        const QPointF bottomMid = (br + bl) / 2.0;
        const double stem = 22.0 / (zoom_ > 0.0 ? zoom_ : 1.0);
        const QPointF knob(bottomMid.x(), bottomMid.y() + stem);
        painter.setPen(QPen(Qt::white, 0, Qt::SolidLine));
        painter.drawLine(bottomMid, knob);
        painter.setBrush(Qt::white);
        painter.drawEllipse(knob, hs, hs);
    }
    painter.restore();

    paintCropGroupOverlays(painter);

    if (dragSizeActive_ && !dragSizeText_.isEmpty()) {
        painter.save();
        painter.resetTransform();
        painter.setClipping(false);
        const QPointF widgetPos = dragSizeImagePos_ * zoom_ + offset_;
        QFont font = painter.font();
        font.setPointSize(10);
        painter.setFont(font);
        const QFontMetrics fm(painter.font());
        QRectF boxRect(widgetPos + QPointF(14, 14),
                       fm.boundingRect(dragSizeText_).size() + QSize(10, 10));
        if (boxRect.right() > rect().right()) {
            boxRect.moveRight(rect().right());
        }
        if (boxRect.bottom() > rect().bottom()) {
            boxRect.moveBottom(rect().bottom());
        }
        boxRect.moveLeft(std::max(boxRect.left(), qreal(rect().left())));
        boxRect.moveTop(std::max(boxRect.top(), qreal(rect().top())));
        painter.setPen(Qt::NoPen);
        painter.setBrush(QColor(0, 0, 0, 200));
        painter.drawRoundedRect(boxRect, 3, 3);
        painter.setPen(Qt::white);
        painter.drawText(boxRect, Qt::AlignCenter, dragSizeText_);
        painter.restore();
    }

    if (pressTrace_.armed) {
        pressTrace_.armed = false;
        const qint64 totalNs = pressTrace_.clock.nsecsElapsed();
        const bool force = qEnvironmentVariableIsSet("PICTURA_PRESS_TRACE");
        if (force || totalNs > 16000000) {
            const qint64 handled = pressTrace_.handledNs < 0 ? 0 : pressTrace_.handledNs;
            if (pressTrace_.firstMoveNs < 0) {
                qWarning("[press-trace] no-move press_handler=%.1fms total=%.1fms",
                         handled / 1e6, totalNs / 1e6);
            } else {
                const double osPressToMove = double(pressTrace_.firstMoveHwMs - pressTrace_.pressHwMs);
                qWarning("[press-trace] press_handler=%.1fms os_press_to_move=%.0fms "
                         "after_handler_to_move=%.1fms move_to_paint=%.1fms total=%.1fms",
                         handled / 1e6, osPressToMove,
                         (pressTrace_.firstMoveNs - handled) / 1e6,
                         (totalNs - pressTrace_.firstMoveNs) / 1e6, totalNs / 1e6);
            }
        }
    }
}

void ImageView::wheelEvent(QWheelEvent* event)
{
    const QPoint delta = event->angleDelta();
    const Qt::KeyboardModifiers mods = event->modifiers();
    // Shared wheel precedence: a horizontal side-wheel pans horizontally;
    // Ctrl+Alt pans vertically; Alt (without Ctrl) pans horizontally; otherwise
    // the vertical wheel zooms at the cursor, with Shift doubling the step.
    if (delta.x() != 0) {
        panBy(QPointF(delta.x(), 0.0));
    } else if (mods.testFlag(Qt::ControlModifier) && mods.testFlag(Qt::AltModifier)) {
        panBy(QPointF(0.0, delta.y()));
    } else if (mods.testFlag(Qt::AltModifier)) {
        panBy(QPointF(delta.y(), 0.0));
    } else {
        const int step = (delta.y() != 0 && mods.testFlag(Qt::ShiftModifier)) ? delta.y() * 2
                                                                              : delta.y();
        zoomAt(event->position(), step);
    }
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
        if (spacePan_) {
            setCursor(Qt::ClosedHandCursor);
        }
    } else {
        pressTrace_.clock.restart();
        pressTrace_.pressHwMs = event->timestamp();
        pressTrace_.handledNs = -1;
        pressTrace_.firstMoveNs = -1;
        pressTrace_.firstMoveHwMs = -1;
        pressTrace_.armed = true;
        emit mousePressed(widgetToImage(event->position()), event->button(),
                          event->modifiers());
        pressTrace_.handledNs = pressTrace_.clock.nsecsElapsed();
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
        if (pressTrace_.armed && pressTrace_.firstMoveNs < 0) {
            pressTrace_.firstMoveNs = pressTrace_.clock.nsecsElapsed();
            pressTrace_.firstMoveHwMs = event->timestamp();
        }
        emit mouseMoved(widgetToImage(event->position()));
    }
    QWidget::mouseMoveEvent(event);
}

void ImageView::mouseReleaseEvent(QMouseEvent* event)
{
    if (event->button() == Qt::MiddleButton || event->button() == Qt::LeftButton) {
        const bool wasPanning = panning_;
        panning_ = false;
        if (wasPanning && spacePan_ && event->button() == Qt::LeftButton) {
            setCursor(Qt::OpenHandCursor);
        }
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
    } else {
        clampOffset();
        emit viewChanged();
    }
    QWidget::resizeEvent(event);
}

void ImageView::leaveEvent(QEvent* event)
{
    clearBrushOutline();
    QWidget::leaveEvent(event);
}

void ImageView::showEvent(QShowEvent* event)
{
    QWidget::showEvent(event);
    updateAntsTimer();
}

void ImageView::keyPressEvent(QKeyEvent* event)
{
    if (transformActive_) {
        if (event->key() == Qt::Key_Return || event->key() == Qt::Key_Enter) {
            emit transformCommitRequested();
            event->accept();
            return;
        }
        if (event->key() == Qt::Key_Escape) {
            emit transformCancelRequested();
            event->accept();
            return;
        }
    }
    QWidget::keyPressEvent(event);
}

void ImageView::hideEvent(QHideEvent* event)
{
    QWidget::hideEvent(event);
    updateAntsTimer();
}

void ImageView::setZoom(double zoom, const QPointF& anchor)
{
    userAdjusted_ = true;
    const double target = std::clamp(zoom, kMinZoom, kMaxZoom);
    const double factor = (zoom_ > 0.0) ? target / zoom_ : 1.0;
    offset_ = anchor - (anchor - offset_) * factor;
    zoom_ = target;
    clampOffset();
    emit zoomChanged(zoom_);
    emit viewChanged();
    update();
}

void ImageView::setBrushOutline(double diameter, const QPointF& imagePos)
{
    if (diameter <= 0.0) {
        clearBrushOutline();
        return;
    }
    brushOutlineActive_ = true;
    brushOutlineDiameter_ = diameter;
    brushOutlineImagePos_ = imagePos;
    update();
}

void ImageView::clearBrushOutline()
{
    if (!brushOutlineActive_) {
        return;
    }
    brushOutlineActive_ = false;
    update();
}

} // namespace pictura
