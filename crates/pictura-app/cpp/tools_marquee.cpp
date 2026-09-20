// Marquee/lasso geometry and overlay helpers for ToolController. Split from
// tools.cpp to keep each translation unit under the size cap.

#include "tools.h"

#include "icons.h"
#include "image_view.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtGui/QCursor>
#include <QtGui/QGuiApplication>
#include <QtGui/QPainterPath>

#include <algorithm>
#include <cmath>

namespace pictura {

bool topmostPixelLocked(PictureView* view)
{
    if (!view) {
        return false;
    }
    const int index = view->topmost_pixel_layer_index();
    return index >= 0 && (view->layer_lock(index) & 0x02) != 0;
}

void ToolController::refreshCursor()
{
    refreshCursor(QGuiApplication::queryKeyboardModifiers());
}

void ToolController::refreshCursor(Qt::KeyboardModifiers mods)
{
    if (!canvas_) {
        return;
    }
    PictureView* hoverView = view();
    const bool ctrlPreview = isSelectionTool(active_) && hoverView
        && mods.testFlag(Qt::ControlModifier) && hoverView->has_selection();
    // A combine modifier (Shift/Alt) hands the cursor back to the tool's
    // add/remove assets; a gesture follows the mode captured at press.
    if (hoverCursorId(active_, mods, movingSelection_ || cursorOverSelection_, ctrlPreview)
        == QStringLiteral("cursor.moveSelection")) {
        const QCursor c = cursor(QStringLiteral("cursor.moveSelection"), 2, 2);
        if (!c.pixmap().isNull()) {
            return canvas_->setCursor(c);
        }
    }
    if (isSelectionTool(active_) && dragging_ && !movingSelection_) {
        const ToolInfo& info = toolInfo(active_);
        const QCursor c = cursor(dragCursorId(), info.hotspotX, info.hotspotY);
        if (!c.pixmap().isNull()) {
            return canvas_->setCursor(c);
        }
    }
    if (active_ == ToolId::Brush || active_ == ToolId::Pencil) {
        // The transient Alt eyedropper wins over the blank paint cursor and the
        // invisible/locked refusal (tool-framework cursor precedence).
        if (mods.testFlag(Qt::AltModifier)) {
            const ToolInfo& info = toolInfo(ToolId::Eyedropper);
            const QCursor c =
                cursor(toolCursorId(ToolId::Eyedropper, mods), info.hotspotX, info.hotspotY);
            return canvas_->setCursor(c.pixmap().isNull() ? QCursor(info.cursor) : c);
        }
        if (hoverView
            && (topmostPixelLocked(hoverView) || !hoverView->active_layer_visible())) {
            return canvas_->setCursor(Qt::ForbiddenCursor);
        }
        // The drawn brush-size ring is the pointer affordance; hide the OS
        // cursor rather than scaling a pixmap with the brush.
        return canvas_->setCursor(Qt::BlankCursor);
    }
    const ToolInfo& info = toolInfo(active_);
    const QCursor toolCursor = cursor(toolCursorId(active_, mods), info.hotspotX, info.hotspotY);
    canvas_->setCursor(toolCursor.pixmap().isNull() ? QCursor(info.cursor) : toolCursor);
}

namespace {

// The synthetic modifier set that selects a combine mode's cursor asset, so a
// locked drag mode can reuse `toolCursorId` while live keys are ignored.
Qt::KeyboardModifiers modsForSelectionMode(SelectionMode mode)
{
    switch (mode) {
    case SelectionMode::Add:
        return Qt::ShiftModifier;
    case SelectionMode::Subtract:
        return Qt::AltModifier;
    case SelectionMode::Intersect:
        return Qt::ShiftModifier | Qt::AltModifier;
    case SelectionMode::New:
        break;
    }
    return Qt::NoModifier;
}

} // namespace

QString ToolController::hoverCursorId(ToolId id, Qt::KeyboardModifiers mods, bool overSelection,
                                      bool ctrlPreview)
{
    if (isSelectionTool(id) && !mods.testFlag(Qt::ShiftModifier)
        && !mods.testFlag(Qt::AltModifier) && (overSelection || ctrlPreview)) {
        return QStringLiteral("cursor.moveSelection");
    }
    return toolCursorId(id, mods);
}

QString ToolController::dragCursorId() const
{
    return toolCursorId(active_, modsForSelectionMode(dragMode_));
}

QString ToolController::cursorIdForModifiersForTest(ToolId id, int mods) const
{
    return toolCursorId(id, Qt::KeyboardModifiers(mods));
}

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
    const QRect rect = marqueeDragRect(anchor_, imagePos, dragMods_);
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

void ToolController::updateBrushOutline(const QPointF& imagePos)
{
    if (!canvas_) {
        return;
    }
    if (active_ == ToolId::Brush || active_ == ToolId::Pencil) {
        canvas_->setBrushOutline(brushSize_, imagePos);
    } else {
        canvas_->clearBrushOutline();
    }
}

} // namespace pictura
