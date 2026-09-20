#include "tool_handler.h"

#include "image_view.h"
#include "selection_geometry.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"

#include <QtCore/QElapsedTimer>
#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtCore/Qt>
#include <QtGui/QPainterPath>
#include <QtGui/QPolygonF>
#include <QtWidgets/QApplication>

#include <cmath>
#include <memory>

namespace pictura {

namespace {

QRect marqueeGeometry(ToolContext& ctx, const QPointF& a, const QPointF& b,
                      Qt::KeyboardModifiers mods)
{
    return marqueeDragRect(a, b, mods, ctx.marqueeStyle(), ctx.fixedRatioWidth(),
                           ctx.fixedRatioHeight(), ctx.fixedSizeWidth(),
                           ctx.fixedSizeHeight());
}

// Rectangular and Elliptical Marquee share the drag lifecycle; `elliptical_`
// selects the ellipse preview and the ellipse raster at release.
class MarqueeToolHandler : public ToolHandler {
public:
    explicit MarqueeToolHandler(bool elliptical)
        : elliptical_(elliptical)
    {
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()));
        dragMods_ = mods;
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        anchor_ = last_ = imagePos;
        updateOverlay(ctx, imagePos);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        last_ = imagePos;
        updateOverlay(ctx, imagePos);
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        PictureView* v = ctx.view();
        const QRect rect = marqueeGeometry(ctx, anchor_, imagePos, dragMods_);
        const bool shaped = v && rect.width() > 0 && rect.height() > 0;
        const bool committed = shaped
            && (elliptical_ ? v->select_ellipse(rect.x(), rect.y(), rect.width(), rect.height(),
                                                selectionModeString(ctx.dragMode()), ctx.feather())
                            : v->select_rect(rect.x(), rect.y(), rect.width(), rect.height(),
                                             selectionModeString(ctx.dragMode()), ctx.feather()));
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSelectionPreview();
            canvas->clearDragSizeHint();
        }
        if (committed) {
            ctx.emitSelectionCommitted();
        }
    }

    Qt::KeyboardModifiers dragMods() const override { return dragMods_; }

private:
    void updateOverlay(ToolContext& ctx, const QPointF& imagePos)
    {
        ImageView* canvas = ctx.canvas();
        if (!canvas) {
            return;
        }
        const QRect rect = marqueeGeometry(ctx, anchor_, imagePos, dragMods_);
        if (rect.width() <= 0 || rect.height() <= 0) {
            canvas->clearSelectionPreview();
            return;
        }
        if (elliptical_) {
            // Preview the actual ellipse, not its bounding rectangle.
            QPainterPath path;
            path.addEllipse(QRectF(rect));
            canvas->setSelectionPreview({path.toFillPolygon()});
        } else {
            canvas->setSelectionPreview({QPolygonF(QRectF(rect))});
        }
        canvas->setDragSizeHint(
            QStringLiteral("%1 x %2").arg(rect.width()).arg(rect.height()), imagePos);
    }

    bool elliptical_;
    Qt::KeyboardModifiers dragMods_ = Qt::NoModifier;
    QPointF anchor_;
    QPointF last_;
};

// Freehand Lasso: the mask grows with each move until release.
class LassoToolHandler : public ToolHandler {
public:
    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->begin_lasso(selectionModeString(ctx.dragMode()))) {
            return true;
        }
        ctx.setDragging(true);
        ctx.setDragCommitted(false);
        last_ = imagePos;
        lassoPolygon_.clear();
        lassoPolygon_ << imagePos;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setSelectionPreview({lassoPolygon_});
        }
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        PictureView* v = ctx.view();
        if (!v) {
            return;
        }
        v->lasso_add_point(qRound(imagePos.x()), qRound(imagePos.y()));
        lassoPolygon_ << imagePos;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setSelectionPreview({lassoPolygon_});
        }
    }

    void onRelease(ToolContext& ctx, const QPointF&, Qt::KeyboardModifiers) override
    {
        if (!ctx.dragging()) {
            return;
        }
        ctx.setDragging(false);
        PictureView* v = ctx.view();
        const bool committed = v && v->end_lasso(ctx.feather());
        lassoPolygon_.clear();
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSelectionPreview();
        }
        if (committed) {
            ctx.emitSelectionCommitted();
        }
    }

private:
    QPointF last_;
    QPolygonF lassoPolygon_;
};

// Polygonal Lasso: click to add vertices, close on the first vertex or a rapid
// second click near the last one, commit on Enter, discard on Escape.
class PolygonalLassoToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override { ctx_ = &ctx; }

    void onDeactivate(ToolContext& ctx) override
    {
        if (polygonInProgress_) {
            cancel(ctx);
        }
        ctx_ = nullptr;
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        PictureView* v = ctx.view();
        if (!v) {
            return true;
        }
        ctx_ = &ctx;
        const double firstDist = polygonPoints_.isEmpty()
            ? 1e9
            : std::hypot(imagePos.x() - polygonPoints_.first().x(),
                         imagePos.y() - polygonPoints_.first().y());
        const double lastDist = polygonClock_.isValid()
            ? std::hypot(imagePos.x() - lastPolygonPress_.x(),
                         imagePos.y() - lastPolygonPress_.y())
            : 1e9;
        const bool closeClick = polygonInProgress_ && firstDist <= kPolygonCloseRadius;
        const bool doubleClick = polygonInProgress_ && polygonClock_.isValid()
            && polygonClock_.elapsed() <= QApplication::doubleClickInterval()
            && lastDist <= kPolygonCloseRadius;
        if (closeClick || doubleClick) {
            close(ctx);
            return true;
        }
        if (!polygonInProgress_) {
            ctx.setDragMode(ctx.resolveSelectionMode(mods, v->has_selection()));
            if (!v->begin_lasso(selectionModeString(ctx.dragMode()))) {
                return true;
            }
            polygonInProgress_ = true;
            polygonPoints_.clear();
        }
        // ponytail: CS6's Shift 45-degree segment snap is deferred; a plain
        // click adds the vertex at the pointer.
        polygonPoints_ << imagePos;
        lastPolygonPress_ = imagePos;
        polygonClock_.restart();
        v->lasso_add_point(qRound(imagePos.x()), qRound(imagePos.y()));
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setSelectionPreview({polygonPoints_}, false, /*solid=*/true);
        }
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (polygonInProgress_) {
            if (ImageView* canvas = ctx.canvas()) {
                QPolygonF preview = polygonPoints_;
                preview << imagePos;
                canvas->setSelectionPreview({preview}, false, /*solid=*/true);
            }
        }
    }

    bool commitPolygonLasso() override
    {
        if (!polygonInProgress_) {
            return false;
        }
        if (ctx_) {
            close(*ctx_);
        }
        return true;
    }

    bool cancelPolygonLasso() override
    {
        if (!polygonInProgress_) {
            return false;
        }
        if (ctx_) {
            cancel(*ctx_);
        }
        return true;
    }

private:
    void close(ToolContext& ctx)
    {
        PictureView* v = ctx.view();
        const bool enough = polygonPoints_.size() >= 3;
        const bool committed = enough && v && v->end_lasso(ctx.feather());
        if (v && !committed) {
            v->cancel_lasso();
        }
        polygonInProgress_ = false;
        polygonPoints_.clear();
        polygonClock_.invalidate();
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSelectionPreview();
        }
        if (committed) {
            ctx.emitSelectionCommitted();
        }
    }

    void cancel(ToolContext& ctx)
    {
        polygonInProgress_ = false;
        polygonPoints_.clear();
        polygonClock_.invalidate();
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearSelectionPreview();
        }
        if (PictureView* v = ctx.view()) {
            v->cancel_lasso();
        }
    }

    QPolygonF polygonPoints_;
    QPointF lastPolygonPress_;
    QElapsedTimer polygonClock_;
    bool polygonInProgress_ = false;
    ToolContext* ctx_ = nullptr;
};

} // namespace

std::unique_ptr<ToolHandler> makeMarqueeToolHandler()
{
    return std::make_unique<MarqueeToolHandler>(false);
}

std::unique_ptr<ToolHandler> makeEllipticalMarqueeToolHandler()
{
    return std::make_unique<MarqueeToolHandler>(true);
}

std::unique_ptr<ToolHandler> makeLassoToolHandler()
{
    return std::make_unique<LassoToolHandler>();
}

std::unique_ptr<ToolHandler> makePolygonalLassoToolHandler()
{
    return std::make_unique<PolygonalLassoToolHandler>();
}

} // namespace pictura
