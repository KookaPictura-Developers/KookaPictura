// ImageView overlays for the Crop tool group: the crop box, the Perspective
// Crop quad, and the slice map. Split from image_view.cpp to keep each translation unit small.
// Ported from photorust's CanvasView::paintCropQuad / paintSlices.

#include "image_view.h"

#include <QtGui/QFontMetrics>
#include <QtGui/QPainter>
#include <QtGui/QPainterPath>

namespace pictura {

void ImageView::setCropBox(const QRectF& box)
{
    cropBox_ = box.normalized();
    update();
}

void ImageView::clearCropBox()
{
    cropBox_ = QRectF();
    update();
}

void ImageView::setPerspectiveCropQuad(const QPolygonF& quad)
{
    perspectiveQuad_ = quad.size() == 4 ? quad : QPolygonF();
    update();
}

void ImageView::clearPerspectiveCropQuad()
{
    perspectiveQuad_.clear();
    update();
}

void ImageView::setSliceOverlay(const QList<SliceOverlay>& slices, const QRectF& dragging)
{
    sliceOverlay_ = slices;
    sliceDrag_ = dragging.normalized();
    update();
}

void ImageView::clearSliceOverlay()
{
    sliceOverlay_.clear();
    sliceDrag_ = QRectF();
    update();
}

// Drawn in widget space so lines, handles, and badges keep their screen size at
// any zoom.
void ImageView::paintCropGroupOverlays(QPainter& painter)
{
    if (perspectiveQuad_.size() != 4 && sliceOverlay_.isEmpty() && sliceDrag_.isNull()
        && cropBox_.isNull()) {
        return;
    }
    const auto toWidget = [this](const QPointF& p) { return p * zoom_ + offset_; };
    painter.save();
    painter.resetTransform();
    painter.setClipping(false);

    if (!sliceOverlay_.isEmpty() || !sliceDrag_.isNull()) {
        // CS6 colours: user slices saturated blue and solid, auto slices grey
        // and dotted.
        const QColor userColor(0x2c, 0x6f, 0xd6);
        const QColor autoColor(0x8a, 0x8a, 0x8a);
        QFont badgeFont = painter.font();
        badgeFont.setPixelSize(9);
        painter.setFont(badgeFont);
        const QFontMetrics metrics(badgeFont);
        painter.setRenderHint(QPainter::Antialiasing, false);
        for (const SliceOverlay& slice : sliceOverlay_) {
            const QRectF box(toWidget(slice.rect.topLeft()), toWidget(slice.rect.bottomRight()));
            const QColor color = slice.user ? userColor : autoColor;
            painter.setPen(QPen(color, 1, slice.user ? Qt::SolidLine : Qt::DotLine));
            painter.setBrush(Qt::NoBrush);
            painter.drawRect(box);
            // The numbered badge sits inside the top-left corner, skipped on a
            // slice too small to hold it.
            const QString label = QStringLiteral("%1").arg(slice.number, 2, 10, QLatin1Char('0'));
            const QRectF badge(box.left() + 1, box.top() + 1,
                               metrics.horizontalAdvance(label) + 6, 12);
            if (box.width() >= badge.width() + 2 && box.height() >= badge.height() + 2) {
                painter.setPen(Qt::NoPen);
                painter.setBrush(color);
                painter.drawRect(badge);
                painter.setPen(Qt::white);
                painter.drawText(badge, Qt::AlignCenter, label);
            }
            if (slice.selected) {
                // CS6 marks the selected slice with orange handles.
                const QColor handle(0xf5, 0x9e, 0x0b);
                painter.setPen(QPen(handle, 1));
                painter.setBrush(handle);
                const QPointF c = box.center();
                for (const QPointF& h :
                     {box.topLeft(), QPointF(c.x(), box.top()), box.topRight(),
                      QPointF(box.right(), c.y()), box.bottomRight(), QPointF(c.x(), box.bottom()),
                      box.bottomLeft(), QPointF(box.left(), c.y())}) {
                    painter.drawRect(QRectF(h.x() - 2.5, h.y() - 2.5, 5, 5));
                }
            }
        }
        if (!sliceDrag_.isNull()) {
            painter.setPen(QPen(userColor, 1));
            painter.setBrush(Qt::NoBrush);
            painter.drawRect(QRectF(toWidget(sliceDrag_.topLeft()),
                                    toWidget(sliceDrag_.bottomRight())));
        }
    }

    if (!cropBox_.isNull()) {
        const QRectF box(toWidget(cropBox_.topLeft()), toWidget(cropBox_.bottomRight()));
        painter.setRenderHint(QPainter::Antialiasing, false);
        // The crop shield: what is about to be thrown away, dimmed.
        QPainterPath shield;
        shield.addRect(rect());
        shield.addRect(box);
        painter.setPen(Qt::NoPen);
        painter.setBrush(QColor(0, 0, 0, 150));
        painter.drawPath(shield);
        // Rule-of-thirds guides, CS6's default overlay.
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(QColor(255, 255, 255, 90), 1));
        for (int i = 1; i <= 2; ++i) {
            const double fx = box.left() + box.width() * i / 3.0;
            const double fy = box.top() + box.height() * i / 3.0;
            painter.drawLine(QPointF(fx, box.top()), QPointF(fx, box.bottom()));
            painter.drawLine(QPointF(box.left(), fy), QPointF(box.right(), fy));
        }
        painter.setPen(QPen(QColor(255, 255, 255, 220), 1));
        painter.drawRect(box);
        painter.setPen(QPen(QColor(40, 40, 40), 1));
        painter.setBrush(Qt::white);
        const QPointF c = box.center();
        for (const QPointF& h :
             {box.topLeft(), QPointF(c.x(), box.top()), box.topRight(), QPointF(box.right(), c.y()),
              box.bottomRight(), QPointF(c.x(), box.bottom()), box.bottomLeft(),
              QPointF(box.left(), c.y())}) {
            painter.drawRect(QRectF(h.x() - 3, h.y() - 3, 6, 6));
        }
    }

    if (perspectiveQuad_.size() == 4) {
        QPolygonF quad;
        for (const QPointF& p : perspectiveQuad_) {
            quad << toWidget(p);
        }
        painter.setRenderHint(QPainter::Antialiasing, true);
        // Shade everything outside the quad, as the rectangular crop does.
        QPainterPath shield;
        shield.addRect(rect());
        QPainterPath inside;
        inside.addPolygon(quad);
        inside.closeSubpath();
        painter.setPen(Qt::NoPen);
        painter.setBrush(QColor(0, 0, 0, 150));
        painter.drawPath(shield.subtracted(inside));
        // A 3x3 grid interpolated along the edges, so it follows the
        // perspective and shows how the warp will land.
        painter.setBrush(Qt::NoBrush);
        painter.setPen(QPen(QColor(255, 255, 255, 90), 1));
        for (int i = 1; i <= 2; ++i) {
            const double t = i / 3.0;
            painter.drawLine(quad[0] + (quad[1] - quad[0]) * t, quad[3] + (quad[2] - quad[3]) * t);
            painter.drawLine(quad[0] + (quad[3] - quad[0]) * t, quad[1] + (quad[2] - quad[1]) * t);
        }
        painter.setPen(QPen(QColor(255, 255, 255, 220), 1));
        painter.drawPolygon(quad);
        painter.setPen(QPen(QColor(40, 40, 40), 1));
        painter.setBrush(Qt::white);
        for (const QPointF& p : quad) {
            painter.drawRect(QRectF(p.x() - 3.5, p.y() - 3.5, 7, 7));
        }
    }
    painter.restore();
}

} // namespace pictura
