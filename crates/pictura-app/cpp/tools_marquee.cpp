// Marquee/lasso geometry and overlay helpers for ToolController. Split from
// tools.cpp to keep each translation unit under the size cap.

#include "tools.h"

#include "image_view.h"

#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtGui/QGuiApplication>
#include <QtGui/QPainterPath>

#include <algorithm>
#include <cmath>

namespace pictura {

SelectionMode ToolController::selectionModeForModifiers(SelectionMode base,
                                                        Qt::KeyboardModifiers mods,
                                                        bool hasExistingSelection)
{
    const bool shift = mods.testFlag(Qt::ShiftModifier);
    const bool alt = mods.testFlag(Qt::AltModifier);
    if (!shift && !alt) {
        return base;
    }
    if (!hasExistingSelection) {
        return SelectionMode::New;
    }
    if (shift && alt) {
        return SelectionMode::Intersect;
    }
    if (shift) {
        return SelectionMode::Add;
    }
    return SelectionMode::Subtract;
}

QRect ToolController::marqueeRectForTest(const QPointF& a, const QPointF& b, int mods) const
{
    return marqueeDragRect(a, b, Qt::KeyboardModifiers(mods));
}

// Style constrains the drag geometry before rasterisation: Normal follows the
// drag; Fixed Ratio keeps the entered width:height; Fixed Size is centred on
// the mousedown (the shape-selection spec's wording).
QRect ToolController::marqueeDragRect(const QPointF& a, const QPointF& b,
                                      Qt::KeyboardModifiers mods) const
{
    if (marqueeStyle_ == MarqueeStyle::FixedSize) {
        const int w = fixedSizeW_;
        const int h = fixedSizeH_;
        return QRect(qRound(a.x()) - w / 2, qRound(a.y()) - h / 2, w, h);
    }
    if (marqueeStyle_ == MarqueeStyle::FixedRatio) {
        const double ratio = fixedRatioW_ / fixedRatioH_;
        const double dx = b.x() - a.x();
        const double dy = b.y() - a.y();
        double w = std::abs(dx);
        double h = std::abs(dy);
        if (ratio > 0.0) {
            if (h <= 0.0 || w / ratio >= h) {
                h = w / ratio;
            } else {
                w = h * ratio;
            }
        }
        const int left = dx >= 0.0 ? qRound(a.x()) : qRound(a.x() - w);
        const int top = dy >= 0.0 ? qRound(a.y()) : qRound(a.y() - h);
        return QRect(left, top, qRound(w), qRound(h));
    }

    double dx = b.x() - a.x();
    double dy = b.y() - a.y();
    if (mods.testFlag(Qt::ShiftModifier)) {
        const double m = std::max(std::abs(dx), std::abs(dy));
        dx = std::copysign(m, dx);
        dy = std::copysign(m, dy);
    }
    const QPointF end = a + QPointF(dx, dy);
    if (mods.testFlag(Qt::AltModifier)) {
        return QRect(qRound(a.x() - std::abs(dx)), qRound(a.y() - std::abs(dy)),
                     qRound(2 * std::abs(dx)), qRound(2 * std::abs(dy)));
    }
    return dragRect(a, end);
}

QRect ToolController::dragRect(const QPointF& a, const QPointF& b)
{
    const QRectF rect = QRectF(a, b).normalized();
    return QRect(qRound(rect.left()), qRound(rect.top()), qRound(rect.width()),
                 qRound(rect.height()));
}

void ToolController::updateDragOverlay(const QPointF& imagePos)
{
    if (!canvas_) {
        return;
    }
    canvas_->setOverlayPolygon(QPolygonF(QRectF(dragRect(anchor_, imagePos))));
}

void ToolController::updateMarqueeOverlay(const QPointF& imagePos)
{
    if (!canvas_) {
        return;
    }
    const Qt::KeyboardModifiers mods = QGuiApplication::queryKeyboardModifiers();
    const QRect rect = marqueeDragRect(anchor_, imagePos, mods);
    if (rect.width() <= 0 || rect.height() <= 0) {
        canvas_->clearSelectionPreview();
        return;
    }
    if (active_ == ToolId::EllipticalMarquee) {
        // Preview the actual ellipse, not its bounding rectangle.
        QPainterPath path;
        path.addEllipse(QRectF(rect));
        canvas_->setSelectionPreview({path.toFillPolygon()});
    } else {
        canvas_->setSelectionPreview({QPolygonF(QRectF(rect))});
    }
    canvas_->setDragSizeHint(
        QStringLiteral("%1 x %2").arg(rect.width()).arg(rect.height()), imagePos);
}

} // namespace pictura
