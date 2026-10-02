// ImageView overlay for the Type tools: the text being typed and its caret,
// or for the Type Mask tools the red quick-mask tint with the letters cut out.

#include "image_view.h"

#include <QtGui/QPainter>
#include <QtGui/QRegion>

namespace pictura {

void ImageView::setTypeOverlay(const TypeOverlay& overlay)
{
    typeOverlay_ = overlay;
    update();
}

void ImageView::clearTypeOverlay()
{
    if (!typeOverlay_.active) {
        return;
    }
    typeOverlay_ = TypeOverlay();
    update();
}

void ImageView::paintTypeOverlay(QPainter& painter)
{
    const TypeOverlay& o = typeOverlay_;
    if (!o.active) {
        return;
    }
    const QTransform toWidget = QTransform::fromScale(zoom_, zoom_)
        * QTransform::fromTranslate(offset_.x(), offset_.y());
    painter.save();
    painter.resetTransform();
    painter.setClipping(false);
    painter.setTransform(toWidget);
    const QRect textRect(o.topLeft, o.image.size());
    if (o.mask) {
        QRegion tinted(QRect(QPoint(0, 0), o.canvas));
        if (!o.image.isNull()) {
            tinted -= textRect;
        }
        painter.setClipRegion(tinted);
        painter.fillRect(QRect(QPoint(0, 0), o.canvas), QColor(255, 0, 0, 128));
        painter.setClipping(false);
    }
    if (!o.image.isNull()) {
        painter.drawImage(o.topLeft, o.image);
    }
    painter.setPen(Qt::NoPen);
    painter.setBrush(o.mask ? QColor(255, 255, 255, 110) : QColor(51, 153, 255, 110));
    for (const QPolygonF& quad : o.selection) {
        painter.drawPolygon(quad);
    }
    // The caret stays one device pixel wide at any zoom.
    painter.setRenderHint(QPainter::Antialiasing, false);
    painter.setPen(QPen(o.mask ? Qt::white : Qt::black, 0));
    painter.drawLine(o.caret);
    painter.restore();
}

} // namespace pictura
