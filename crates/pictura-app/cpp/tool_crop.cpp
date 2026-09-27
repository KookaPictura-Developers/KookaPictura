#include "tool_handler.h"

#include "crop_grip.h"
#include "image_view.h"
#include "tools.h"

#include "pictura_app/src/cxxqt_object.cxxqt.h"
#include "pictura_app/src/cxxqt_object/crop_group.cxxqt.h"

#include <QtCore/QElapsedTimer>
#include <QtCore/QPointF>
#include <QtCore/QRect>
#include <QtWidgets/QApplication>

#include <cmath>
#include <memory>

namespace pictura {

namespace {

// Crop (CS6's redesigned crop): choosing the tool places a crop box over the
// whole canvas with eight handles, a shield over what will be cut, and
// rule-of-thirds guides. Drag a handle to resize (ratio-locked when an aspect
// ratio is set), drag inside to move, drag outside to draw a new box. Enter, a
// double-click inside, or the options bar's Apply commits (discarding cropped
// pixels when Delete Cropped Pixels is on); Escape or Cancel resets the box.
// Ported from photorust's CanvasView resetCrop/dragCrop/applyCropRatio.
class CropToolHandler : public ToolHandler {
public:
    void onActivate(ToolContext& ctx) override
    {
        ctx_ = &ctx;
        active_ = true;
        reset(ctx);
    }

    void onDeactivate(ToolContext& ctx) override
    {
        active_ = false;
        grip_ = BoxGrip::None;
        box_ = QRectF();
        if (ImageView* canvas = ctx.canvas()) {
            canvas->clearCropBox();
        }
    }

    // A commit, undo, or tab switch changes the canvas under the box; start
    // over from the new canvas.
    void onDocumentRefreshed(ToolContext& ctx) override
    {
        if (active_ && canvasRect(ctx) != canvas_) {
            reset(ctx);
        }
    }

    void onOptionsChanged(ToolContext& ctx) override
    {
        if (active_ && !box_.isNull()) {
            box_ = fitRatio(box_, ctx.cropRatio(), BoxGrip::BottomRight);
            show(ctx);
        }
    }

    bool onPress(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return true;
        }
        ctx_ = &ctx;
        grip_ = boxGripAt(box_, imagePos, zoom(ctx));
        // A double-click is a second press on the same spot inside the box,
        // never on a handle.
        const QPointF d = (imagePos - lastPress_) * zoom(ctx);
        const bool doubleClick = grip_ == BoxGrip::Move && clock_.isValid()
            && clock_.elapsed() <= QApplication::doubleClickInterval()
            && std::hypot(d.x(), d.y()) <= 4.0;
        lastPress_ = imagePos;
        if (doubleClick) {
            clock_.invalidate();
            grip_ = BoxGrip::None;
            commitCrop();
            return true;
        }
        clock_.restart();
        if (grip_ == BoxGrip::None) {
            box_ = QRectF(imagePos, imagePos);
            grip_ = BoxGrip::BottomRight;
        }
        start_ = imagePos;
        startBox_ = box_;
        return true;
    }

    void onMove(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers) override
    {
        if (grip_ == BoxGrip::None) {
            if (ImageView* canvas = ctx.canvas(); canvas && active_) {
                canvas->setCursor(
                    boxGripCursor(boxGripAt(box_, imagePos, zoom(ctx)), Qt::CrossCursor));
            }
            return;
        }
        box_ = dragBox(startBox_, grip_, imagePos - start_);
        if (grip_ != BoxGrip::Move) {
            box_ = fitRatio(box_, ctx.cropRatio(), grip_);
        }
        show(ctx);
    }

    void onRelease(ToolContext& ctx, const QPointF& imagePos, Qt::KeyboardModifiers mods) override
    {
        if (grip_ == BoxGrip::None) {
            return;
        }
        onMove(ctx, imagePos, mods);
        grip_ = BoxGrip::None;
        // A click that drew no box puts the default box back.
        if (box_.width() < 1.0 || box_.height() < 1.0) {
            reset(ctx);
        }
    }

    bool commitCrop() override
    {
        if (!ctx_ || !hasPendingCrop()) {
            return false;
        }
        PictureView* v = ctx_->view();
        const QRect rect = pendingCropRect();
        if (!v || rect.width() < 1 || rect.height() < 1) {
            return false;
        }
        const bool ok = crop_to(*v, rect.x(), rect.y(), rect.width(), rect.height(),
                                ctx_->cropDeletePixels());
        if (ok) {
            ctx_->emitSelectionCommitted();
        }
        return ok;
    }

    // Escape / Cancel: back to the canvas-sized box.
    bool cancelPolygonLasso() override
    {
        if (!ctx_ || !hasPendingCrop()) {
            return false;
        }
        reset(*ctx_);
        return true;
    }

    // Pending only while the tool is active and the box differs from the
    // canvas, so Image > Crop is not captured by the default box.
    bool hasPendingCrop() const override
    {
        return active_ && !box_.isNull() && pendingCropRect() != canvas_;
    }

    QRect pendingCropRect() const override
    {
        const QRectF r = box_.normalized();
        const int x = int(std::floor(r.x()));
        const int y = int(std::floor(r.y()));
        return QRect(x, y, int(std::lround(r.right())) - x, int(std::lround(r.bottom())) - y)
            .intersected(canvas_);
    }

private:
    static double zoom(ToolContext& ctx) { return ctx.canvas() ? ctx.canvas()->zoom() : 1.0; }

    static QRect canvasRect(ToolContext& ctx)
    {
        PictureView* v = ctx.view();
        if (!v || !v->has_document()) {
            return QRect();
        }
        return QRect(0, 0, v->document_width(), v->document_height());
    }

    void reset(ToolContext& ctx)
    {
        grip_ = BoxGrip::None;
        canvas_ = canvasRect(ctx);
        box_ = canvas_.isEmpty() ? QRectF()
                                 : fitRatio(QRectF(canvas_), ctx.cropRatio(), BoxGrip::BottomRight);
        show(ctx);
    }

    void show(ToolContext& ctx)
    {
        if (ImageView* canvas = ctx.canvas()) {
            if (box_.isNull()) {
                canvas->clearCropBox();
            } else {
                canvas->setCropBox(box_);
            }
        }
    }

    bool active_ = false;
    QRectF box_;
    QRectF startBox_;
    QRect canvas_;
    QPointF start_;
    QPointF lastPress_;
    BoxGrip grip_ = BoxGrip::None;
    QElapsedTimer clock_;
    ToolContext* ctx_ = nullptr;
};

} // namespace

std::unique_ptr<ToolHandler> makeCropToolHandler()
{
    return std::make_unique<CropToolHandler>();
}

} // namespace pictura
