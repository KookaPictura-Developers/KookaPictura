#include "tool_handler.h"

#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/crop_group.cxxqt.h"

#include <QtCore/QElapsedTimer>
#include <QtGui/QPolygonF>
#include <QtWidgets/QApplication>

#include <algorithm>
#include <cmath>
#include <memory>

namespace pictura {

namespace {

// Perspective Crop: drag out a box, then pull its corners onto the edges of
// something that should be rectangular; dragging inside moves the whole quad.
// Enter or a double-click inside commits (warping the document so the quad
// becomes the canvas); Escape discards it. Ported from photorust's CanvasView.
class PerspectiveCropToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override
    {
        ctx_ = &ctx;
        showQuad(ctx);
    }

    void onDeactivate(ToolContext& ctx) override
    {
        quad_.clear();
        gesture_ = Gesture::None;
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearPerspectiveCropQuad();
        }
        ctx_ = nullptr;
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        ctx_ = &ctx;
        corner_ = cornerAt(ctx, imagePos);
        // A double-click is a second press on the same spot inside the quad,
        // never on a corner handle, so grabbing a corner right after drawing
        // the box does not commit.
        const bool doubleClick = quad_.size() == 4 && corner_ < 0 && clock_.isValid()
            && clock_.elapsed() <= QApplication::doubleClickInterval()
            && screenDistance(ctx, imagePos, lastPress_) <= 4.0
            && quad_.containsPoint(imagePos, Qt::OddEvenFill);
        lastPress_ = imagePos;
        if (doubleClick) {
            clock_.invalidate();
            commit(ctx);
            return true;
        }
        clock_.restart();
        start_ = imagePos;
        startQuad_ = quad_;
        if (corner_ >= 0) {
            gesture_ = Gesture::Corner;
        } else if (quad_.size() == 4 && quad_.containsPoint(imagePos, Qt::OddEvenFill)) {
            gesture_ = Gesture::Move;
        } else {
            const QString refusal = perspective_crop_refusal_reason(*v);
            if (!refusal.isEmpty()) {
                ctx.refused(refusal);
                return true;
            }
            gesture_ = Gesture::New;
            quad_ = QPolygonF({imagePos, imagePos, imagePos, imagePos});
        }
        showQuad(ctx);
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        const QPointF delta = imagePos - start_;
        switch (gesture_) {
        case Gesture::None:
            return;
        case Gesture::New: {
            const QRectF box = QRectF(start_, imagePos).normalized();
            quad_ = QPolygonF({box.topLeft(), box.topRight(), box.bottomRight(), box.bottomLeft()});
            break;
        }
        case Gesture::Corner:
            quad_[corner_] = startQuad_.at(corner_) + delta;
            break;
        case Gesture::Move:
            quad_ = startQuad_.translated(delta);
            break;
        }
        showQuad(ctx);
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (gesture_ == Gesture::None) {
            return;
        }
        onMove(ctx, imagePos, mods);
        // A click that drew no box leaves nothing staged.
        if (gesture_ == Gesture::New) {
            const QRectF box = QRectF(start_, imagePos).normalized();
            if (box.width() < 1.0 || box.height() < 1.0) {
                quad_.clear();
            }
        }
        gesture_ = Gesture::None;
        showQuad(ctx);
    }

    bool commitCrop() override { return ctx_ && commit(*ctx_); }

    bool hasPendingCrop() const override { return quad_.size() == 4; }

    // Escape discards the staged quad.
    bool cancelPolygonLasso() override
    {
        if (quad_.size() != 4) {
            return false;
        }
        quad_.clear();
        if (ctx_) {
            showQuad(*ctx_);
        }
        return true;
    }

private:
    enum class Gesture { None, New, Corner, Move };

    bool commit(ToolContext& ctx)
    {
        PictureView* v = ctx.view();
        if (!v || quad_.size() != 4) {
            return false;
        }
        double corners[8];
        for (int i = 0; i < 4; ++i) {
            corners[2 * i] = quad_.at(i).x();
            corners[2 * i + 1] = quad_.at(i).y();
        }
        // A degenerate quad is refused and stays up for the user to fix.
        if (!perspective_crop_commit(*v, ::rust::Slice<const double>(corners, 8))) {
            ctx.refused(QStringLiteral("Could not complete the Perspective Crop: the "
                                       "corners do not form a usable quadrilateral."));
            return false;
        }
        quad_.clear();
        showQuad(ctx);
        ctx.emitSelectionCommitted();
        return true;
    }

    // Corners grab within a fixed screen radius, so they stay reachable at any
    // zoom.
    int cornerAt(ToolContext& ctx, const QPointF& imagePos) const
    {
        if (quad_.size() != 4) {
            return -1;
        }
        for (int i = 0; i < 4; ++i) {
            if (screenDistance(ctx, quad_.at(i), imagePos) <= 8.0) {
                return i;
            }
        }
        return -1;
    }

    static double screenDistance(ToolContext& ctx, const QPointF& a, const QPointF& b)
    {
        const double zoom = ctx.canvas() ? std::max(ctx.canvas()->zoom(), 1e-6) : 1.0;
        const QPointF d = (a - b) * zoom;
        return std::hypot(d.x(), d.y());
    }

    void showQuad(ToolContext& ctx)
    {
        if (ImageView* canvas = ctx.canvas()) {
            canvas->setPerspectiveCropQuad(quad_);
        }
    }

    QPolygonF quad_;
    QPolygonF startQuad_;
    QPointF start_;
    QPointF lastPress_;
    int corner_ = -1;
    Gesture gesture_ = Gesture::None;
    QElapsedTimer clock_;
    ToolContext* ctx_ = nullptr;
};

} // namespace

std::unique_ptr<ToolHandler> makePerspectiveCropToolHandler()
{
    return std::make_unique<PerspectiveCropToolHandler>();
}

} // namespace pictura
